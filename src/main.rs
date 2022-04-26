//!
//! # Lopxy
//! 
//! lopxy is a local proxy server tool for some unreachable remote tiny file.
//! 

#[macro_use]
extern crate rocket;

mod args;
mod env;
mod config;
mod controller;
mod manager;
mod proxy;
mod util;
mod assets;

use clap::Parser;
use std::sync::Arc;
use std::sync::Mutex;

use manager::controller::LopxyManagerServerController;
use proxy::item::*;

#[tokio::main]
async fn main() {
    run(controller::LopxyController::build(
        env::LopxyEnv::collect(
            args::LopxyArgs::parse()
        ).expect("collect lopxy env failed")
    )).await;
}

async fn run(mut controller: controller::LopxyController) {
    match controller.env().command_args {
        args::LopxyCommand::Start(_) => {
            start_server(controller).await;
        },
        args::LopxyCommand::Stop(_) => {
            stop_server(controller).await;
        },
        args::LopxyCommand::List(_) => {
            list_all_proxy_item(controller).await;
        },
        args::LopxyCommand::Add(_) => {
            add_proxy_item(controller).await;
        },
        args::LopxyCommand::Remove(_) => {
            remove_proxy_item(controller).await;
        },
        args::LopxyCommand::Modify(_) => {
            modify_proxy_item(controller).await;
        },
        args::LopxyCommand::Enable => {
            set_lopxy_proxy_enabled(controller, true).await;
        },
        args::LopxyCommand::Disable => {
            set_lopxy_proxy_enabled(controller, false).await;
        },
        args::LopxyCommand::Status => {
            lopxy_status(controller).await;
        }
    }
}

async fn start_server(mut controller: controller::LopxyController) {
    let lopxy_env = controller.env();

    // check lopxy instance and decide whether to switch to background
    lopxy_env.guard_instance();

    // release static assets
    lopxy_env.release_static_assets();

    // get start args ref
    let start_args = lopxy_env.start_args().unwrap();

    // wrap controller
    let web_manager_port = start_args.web_manager_port;
    let proxy_addr = format!("127.0.0.1:{}", start_args.proxy_port);
    let proxy_shutdown = lopxy_env.clone_proxy_shutdown();
    let controller = Arc::new(Mutex::new(controller));

    // start manager server
    let mut web_manager_server = manager::LopxyManagerServer::build(web_manager_port, controller.clone());
    let web_manager_server_future = web_manager_server.start();

    // build proxy config
    let system_proxy_config = proxy::ProxyConfig::system_proxy().expect("get system proxy config failed");
    let proxy_config = proxy::ProxyConfig::new(true, Some(proxy_addr), Some("<local>".to_string()));

    // set proxy config
    proxy_config.update_system_proxy().expect("set proxy config failed");

    // start proxy server
    let proxy_server_future = proxy::Proxy::start(system_proxy_config.clone(), proxy_config, proxy_shutdown, controller.clone());

    // wait for all server
    match futures::join!(web_manager_server_future, proxy_server_future) { _ => {} }

    // restore system proxy config
    system_proxy_config.update_system_proxy().expect("restore system proxy config failed");
}

async fn stop_server(mut controller: controller::LopxyController) {
    let lopxy_env = controller.env();
    let web_manager_instance = match lopxy_env.web_manager_instance() {
        Some(s) => s,
        None => {
            println!("lopxy not running...");
            std::process::exit(1);
        }
    };

    println!("web manager server url : {}", web_manager_instance);
    println!("stop lopxy all services...");

    manager::LopxyManagerServer::stop_lopxy_server(&web_manager_instance).await.expect("stop lopxy server failed");

    println!("all lopxy servcies already stop...");
}

async fn list_all_proxy_item(mut controller: controller::LopxyController) {
    let lopxy_env = controller.env();

    let show_proxy_item_list = |proxy_items: &Vec<ProxyItem>| {
        for item in proxy_items {
            println!("{} => {} [{}]", item.resource_url(), item.proxy_resource_url(), item.content_type());
        }
    };

    let web_manager_instance = match lopxy_env.web_manager_instance() {
        Some(s) => s,
        None => {
            show_proxy_item_list(controller.list_all_proxy_item());
            return;
        }
    };

    let result = manager::LopxyManagerServer::list_all_proxy_item(&web_manager_instance).
        await.expect("list all proxy item failed");
    
    let proxy_items: Vec<ProxyItem> = serde_json::from_str(&result).expect("parse list all proxy item response failed");
    show_proxy_item_list(&proxy_items);
}

async fn add_proxy_item(mut controller: controller::LopxyController) {
    let lopxy_env = controller.env();
    let add_args = lopxy_env.add_args().expect("add args invalid");

    let web_manager_instance = match lopxy_env.web_manager_instance() {
        Some(s) => s,
        None => {
            let resource_url = add_args.resource_url.clone();
            let proxy_resource_url = add_args.proxy_resource_url.clone();
            let content_type = add_args.content_type.clone();
            if !controller.add_proxy_item(&resource_url, &proxy_resource_url, &content_type) {
                eprintln!("add proxy item failed");
                std::process::exit(1);
            }
            return;
        }
    };

    let result = manager::LopxyManagerServer::add_proxy_item(&web_manager_instance, &add_args.resource_url, &add_args.proxy_resource_url, &add_args.content_type).
        await.expect("add proxy item failed");
    print!("{}", result);
}

async fn remove_proxy_item(mut controller: controller::LopxyController) {
    let lopxy_env = controller.env();
    let remove_args = lopxy_env.remove_args().expect("remove args invalid");
    
    let web_manager_instance = match lopxy_env.web_manager_instance() {
        Some(s) => s,
        None => {
            let resource_url = remove_args.resource_url.clone();
            if !controller.remove_proxy_item(&resource_url) {
                eprintln!("remove proxy item failed");
                std::process::exit(1);
            }
            return;
        }
    };

    let result = manager::LopxyManagerServer::remove_proxy_item(&web_manager_instance, &remove_args.resource_url).
        await.expect("remove proxy item failed");
    print!("{}", result);
}

async fn modify_proxy_item(mut controller: controller::LopxyController) {
    let lopxy_env = controller.env();
    let modify_args = lopxy_env.modify_args().expect("modify args invalid");

    let web_manager_instance = match lopxy_env.web_manager_instance() {
        Some(s) => s,
        None => {
            let resource_url = modify_args.resource_url.clone();
            let proxy_resource_url = modify_args.proxy_resource_url.clone();
            let content_type = modify_args.content_type.clone();
            if !controller.modify_proxy_item(&resource_url, &proxy_resource_url, &content_type) {
                eprintln!("modify proxy item failed");
                std::process::exit(1);
            }
            return;
        }
    };

    let result = manager::LopxyManagerServer::modify_proxy_item(&web_manager_instance, &modify_args.resource_url, &modify_args.proxy_resource_url, &modify_args.content_type).
        await.expect("modify proxy item failed");
    print!("{}", result);
}

async fn set_lopxy_proxy_enabled(mut controller: controller::LopxyController, enabled: bool) {
    let lopxy_env = controller.env();
    let web_manager_instance = match lopxy_env.web_manager_instance() {
        Some(s) => s,
        None => {
            println!("lopxy not running...");
            std::process::exit(1);
        }
    };

    let result = manager::LopxyManagerServer::set_lopxy_proxy_enabled(&web_manager_instance, enabled)
        .await
        .expect(
            &format!("{} lopxy proxy failed", if enabled { "enable" } else { "disable" })
        );
    print!("{}", result);
}

fn print_lopxy_server_status(web_manager_running: bool, proxy_enabled: bool) {
    println!("lopxy web manager running : {}", web_manager_running);
    println!("lopxy proxy enabled : {}", proxy_enabled);
}

async fn lopxy_status(mut controller: controller::LopxyController) {
    let lopxy_env = controller.env();
    let web_manager_instance = match lopxy_env.web_manager_instance() {
        Some(s) => s,
        None => {
            print_lopxy_server_status(false, false);
            return;
        }
    };

    print_lopxy_server_status(true, 
        manager::LopxyManagerServer::is_lopxy_proxy_enabled(&web_manager_instance)
            .await
            .unwrap_or(false)
    );
}