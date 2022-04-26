pub mod controller;
pub mod request;
pub mod response;

use std::sync::mpsc;
use std::sync::Arc;
use std::net::{IpAddr, Ipv4Addr};

use urlencoding::encode;

use controller::*;

async fn launch(port: u32, controller: LopxyManagerServerControllerArc, shutdown_sign: mpsc::Receiver<bool>) -> Result<(), rocket::Error> {
    let config = rocket::Config {
        address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        port: port as u16,
        shutdown: rocket::config::Shutdown {
            force: false,
            ..rocket::config::Shutdown::default()
        },
        ..rocket::Config::default()
    };

    let server = rocket::custom(&config).
        attach(rocket::fairing::AdHoc::on_ignite("Lopxy Server Manage State", |rocket| async move {
            rocket.manage(LopxyManagerServerStatus::new(controller))
        })).
        mount("/", lopxy_web_manager_routes()).
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
    port: u32,
    controller: LopxyManagerServerControllerArc,
    shutdown_sign_trigger: Option<mpsc::Sender<bool>>
}

impl LopxyManagerServer {
    pub fn build(port: u32, controller: LopxyManagerServerControllerArc) -> LopxyManagerServer {
        LopxyManagerServer {
            port,
            controller,
            shutdown_sign_trigger: None
        }
    }

    ///
    /// Start lopxy manager server
    /// 
    /// # Panics
    /// 
    /// The `start` method will panic if the server launch failed or server crash in future
    pub fn start(&mut self) -> rocket::tokio::task::JoinHandle<()> {
        let (trigger, shutdown_sign): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel();
        self.shutdown_sign_trigger = Some(trigger);

        let server_port = self.port;
        let server_controller = Arc::clone(&self.controller);
        
        rocket::tokio::spawn(async move {
            launch(server_port, server_controller, shutdown_sign).await.expect("web manager server launch failed");
        })
    }

    #[deprecated]
    #[allow(dead_code)]
    pub fn shutdown(&mut self) {
        if let Some(trigger) = &self.shutdown_sign_trigger {
            match trigger.send(true) { _ => {} };
            self.shutdown_sign_trigger = None;
        }
    }

    pub async fn stop_lopxy_server(web_manager_url: &str) -> reqwest::Result<String> {
        reqwest::Client::builder().
            no_proxy().
            build()?.
            get(format!("{}/shutdown", web_manager_url)).
            send().
            await?.
            text().
            await
    }

    pub async fn list_all_proxy_item(web_manager_url: &str) -> reqwest::Result<String> {
        reqwest::Client::builder().
            no_proxy().
            build()?.
            get(format!("{}/list", web_manager_url)).
            send().
            await?.
            text().
            await
    }

    pub async fn add_proxy_item(web_manager_url: &str, resource_url: &str, proxy_resource_url: &str, content_type: &str) -> reqwest::Result<String> {
        let mut params = std::collections::HashMap::new();
        params.insert("resource", encode(resource_url));
        params.insert("resource_proxy", encode(proxy_resource_url));
        params.insert("resource_content_type", encode(content_type));
        
        let response = reqwest::Client::builder().
            no_proxy().
            build()?.
            post(format!("{}/add", web_manager_url)).
            form(&params).
            send().
            await?.
            json::<response::AddResponse>().
            await.
            expect("add proxy item expect");

        Ok(if response.result {
            "".to_string()
        } else {
            "add proxy item failed\r\n".to_string()
        })
    }

    pub async fn remove_proxy_item(web_manager_url: &str, resource_url: &str) -> reqwest::Result<String> {
        let mut params = std::collections::HashMap::new();
        params.insert("resource", encode(resource_url));

        let response = reqwest::Client::builder().
            no_proxy().
            build()?.
            delete(format!("{}/remove", web_manager_url)).
            form(&params).
            send().
            await?.
            json::<response::RemoveResponse>().
            await.
            expect("remove proxy item expect");

        Ok(if response.result {
            "".to_string()
        } else {
            "remove proxy item failed\r\n".to_string()
        })
    }

    pub async fn modify_proxy_item(web_manager_url: &str, resource_url: &str, proxy_resource_url: &str, content_type: &str) -> reqwest::Result<String> {
        let mut params = std::collections::HashMap::new();
        params.insert("resource", encode(resource_url));
        params.insert("resource_proxy", encode(proxy_resource_url));
        params.insert("resource_content_type", encode(content_type));

        let response = reqwest::Client::builder().
            no_proxy().
            build()?.
            post(format!("{}/modify", web_manager_url)).
            form(&params).
            send().
            await?.
            json::<response::ModifyResponse>().
            await.
            expect("modify proxy item expect");

        Ok(if response.result {
            "".to_string()
        } else {
            "modify proxy item failed\r\n".to_string()
        })
    }

    pub async fn is_lopxy_proxy_enabled(web_manager_url: &str) -> reqwest::Result<bool> {
        Ok(reqwest::Client::builder().
            no_proxy().
            build()?.
            get(format!("{}/is_proxy_enabled", web_manager_url)).
            send().
            await?.
            json::<response::IsProxyEnabledResponse>().
            await.
            expect("get lopxy proxy enabled status failed").
            result
        )
    }

    pub async fn set_lopxy_proxy_enabled(web_manager_url: &str, enabled: bool) -> reqwest::Result<String> {
        let mut params = std::collections::HashMap::new();
        params.insert("enabled", enabled);

        let method = if enabled { "enable" } else { "disable" };

        let response = reqwest::Client::builder().
            no_proxy().
            build()?.
            post(format!("{}/enable_proxy", web_manager_url)).
            form(&params).
            send().
            await?.
            json::<response::SetProxyEnabledResponse>().
            await.
            expect(&format!("{} lopxy proxy expect", method));

        Ok(if response.result {
            "".to_string()
        } else {
            format!("{} lopxy proxy failed\r\n", method)
        })
    }
}