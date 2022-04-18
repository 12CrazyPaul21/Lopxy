use clap::{Parser, Args, Subcommand};

use util::config;

#[derive(Parser, Debug)]
#[clap(author, version, about = "lopxy is a local proxy tool.", long_about = None, subcommand_required = false)]
pub struct LopxyArgs {
    #[clap(subcommand)]
    pub command: LopxyCommand,
}

#[derive(Subcommand, Debug)]
pub enum LopxyCommand {
    /// Start lopxy
    #[clap(arg_required_else_help = false)]
    Start(StartArgs),

    /// Stop lopxy
    Stop(StopArgs),
}

#[derive(Args, Debug)]
pub struct StartArgs {
    #[clap(short, long, help = "Web Manager server port", default_value_t = 8283)]
    web_manager_port: u32,

    #[clap(short, long, help = "Proxy port", default_value_t = 7237)]
    proxy_port: u32,

    #[clap(short, long, help = "Running in background", takes_value(false), parse(from_flag))]
    daemon: bool,
}

#[derive(Args, Debug)]
pub struct StopArgs {

}

#[derive(Debug)]
pub struct LopxyEnv {
    pub config_dir: std::path::PathBuf,
    pub command_args: LopxyCommand
}

impl LopxyEnv {
    pub fn collect(args: LopxyArgs) -> Option<LopxyEnv> {
        let config_dir = config::program_config_dir(env!("CARGO_PKG_NAME"))?;

        Some(LopxyEnv {
            config_dir,
            command_args: args.command,
        })
    }
}