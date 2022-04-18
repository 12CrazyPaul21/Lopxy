mod args;

use clap::Parser;

#[tokio::main]
async fn main() {
    let args = args::LopxyArgs::parse();
    let lopxy_config = args::LopxyEnv::collect(args).expect("collect lopxy env failed");

    run(lopxy_config).await;
}

async fn run(lopxy_config: args::LopxyEnv) {
    match lopxy_config.command_args {
        args::LopxyCommand::Start(_) => {
            start_server(lopxy_config).await;
        },
        args::LopxyCommand::Stop(_) => {
            stop_server(lopxy_config);
        }
    }
}

async fn start_server(start_args: args::LopxyEnv) {
    println!("start args : {:?}", start_args);

    // start manager server
    let mut server = manager::LopxyManagerServer::build();
    server.start();

    loop {
        
    }
}

fn stop_server(stop_args: args::LopxyEnv) {
    println!("start args : {:?}", stop_args);
}