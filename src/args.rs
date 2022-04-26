#![allow(dead_code)]

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about = "lopxy is a local proxy tool.", long_about = None, subcommand_required = false)]
pub struct LopxyArgs {
    #[clap(subcommand)]
    pub command: LopxyCommand,
}

#[derive(Subcommand, Debug)]
pub enum LopxyCommand {
    /// Start All lopxy Services
    #[clap(arg_required_else_help = false)]
    Start(StartArgs),

    /// Stop All lopxy Services
    Stop(StopArgs),

    /// List All Proxy Item
    List(ListArgs),

    /// Add Proxy Item
    Add(AddArgs),

    /// Remove Proxy Item
    Remove(RemoveArgs),

    /// Modify Proxy Item
    Modify(ModifyArgs),

    /// Enable Lopxy Proxy
    Enable,

    /// Disable Lopxy Proxy
    Disable,

    /// Get Lopxy Server Status
    Status,

    /// Open Web Manager Page in Browser
    Manager,
}

#[derive(Args, Debug)]
pub struct StartArgs {
    #[clap(short, long, help = "Web Manager server port", default_value_t = 8283)]
    pub web_manager_port: u32,

    #[clap(short, long, help = "Proxy port", default_value_t = 7237)]
    pub proxy_port: u32,

    #[clap(
        short,
        long,
        help = "Running in background",
        takes_value(false),
        parse(from_flag)
    )]
    pub daemon: bool,
}

#[derive(Args, Debug)]
pub struct StopArgs {}

#[derive(Args, Debug)]
pub struct ListArgs {}

#[derive(Args, Debug)]
pub struct AddArgs {
    #[clap(short, long, help = "resource url")]
    pub resource_url: String,

    #[clap(short, long, help = "proxy resource url")]
    pub proxy_resource_url: String,

    #[clap(short, long, help = "proxy resource content type MIME, only used for local scheme", default_value = "application/octet-stream")]
    pub content_type: String,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    #[clap(short, long, help = "resource url")]
    pub resource_url: String,
}

#[derive(Args, Debug)]
pub struct ModifyArgs {
    #[clap(short, long, help = "resource url")]
    pub resource_url: String,

    #[clap(short, long, help = "proxy resource url")]
    pub proxy_resource_url: String,

    #[clap(short, long, help = "proxy resource content type MIME, only used for local scheme")]
    pub content_type: String,
}