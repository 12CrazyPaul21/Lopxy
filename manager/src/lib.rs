#[macro_use]
extern crate rocket;

use std::sync::mpsc;

#[get("/")]
fn index() -> &'static str {
    "hello lopxy manager server"
}

#[get("/shutdown")]
fn shutdown(shutdown: rocket::Shutdown) -> &'static str {
    shutdown.notify();
    "Shutting down..."
}

async fn launch(shutdown_sign: mpsc::Receiver<bool>) -> Result<(), rocket::Error> {
    let config = rocket::Config {
        shutdown: rocket::config::Shutdown {
            force: false,
            ..rocket::config::Shutdown::default()
        },
        ..rocket::Config::default()
    };

    let server = rocket::custom(&config).
        attach(rocket::fairing::AdHoc::on_liftoff("liftoff", |_| Box::pin(async move {
            println!("lift off middleware");
        }))).
        mount("/", routes![index, shutdown]).
        ignite().
        await?;
    let shutdown = server.shutdown();

    rocket::tokio::spawn(async move {
        if match shutdown_sign.recv() {
            Ok(v) => v,
            Err(_) => false
        } {
            shutdown.notify();
        }
    });

    server.launch().await.expect("web manager server panic");

    Ok(())        
}

pub struct LopxyManagerServer {
    shutdown_sign_trigger: Option<mpsc::Sender<bool>>
}

impl LopxyManagerServer {
    pub fn build() -> LopxyManagerServer {
        LopxyManagerServer {
            shutdown_sign_trigger: None
        }
    }

    ///
    /// Start lopxy manager server
    /// 
    /// # Panics
    /// 
    /// The `start` method will panic if the server launch failed or server crash in future
    pub fn start(&mut self) {
        let (trigger, shutdown_sign): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel();
        self.shutdown_sign_trigger = Some(trigger);

        rocket::tokio::spawn(async move {
            launch(shutdown_sign).await.expect("web manager server launch failed");
        });
    }

    pub fn shutdown(&mut self) {
        if let Some(trigger) = &self.shutdown_sign_trigger {
            match trigger.send(true) { _ => {} };
            self.shutdown_sign_trigger = None;
        }
    }
}