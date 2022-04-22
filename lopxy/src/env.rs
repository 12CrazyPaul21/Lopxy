#![allow(dead_code)]

use sysinfo::SystemExt;
use util::config;

use super::args::*;

struct LopxyInstance {
    pid: u32,
    web_manager_port: u32,
    proxy_port: u32,
}

impl LopxyInstance {
    fn lopxy_pid_path(config_dir: &std::path::PathBuf) -> std::path::PathBuf {
        let mut config_path = std::path::PathBuf::clone(config_dir);
        config_path.push("lopxy.pid");
        config_path
    }

    ///
    /// Check Lopxy has a instance
    ///
    pub fn instance(config_dir: &std::path::PathBuf) -> Option<LopxyInstance> {
        let config_path = LopxyInstance::lopxy_pid_path(config_dir);
        if !config_path.exists() {
            return None;
        }

        // read lopxy.pid
        let lopxy_pid_content = match std::fs::read_to_string(config_path) {
            Ok(c) => c,
            Err(_) => return None,
        };

        //
        // verify instance
        //

        let lopxy_instance_config: Vec<&str> = lopxy_pid_content.split("\r\n").collect();
        if lopxy_instance_config.len() < 3 {
            return None;
        }

        let pid: u32 = match lopxy_instance_config[0].parse() {
            Ok(v) => v,
            Err(_) => return None,
        };

        // check process exists
        use sysinfo::{Pid, PidExt};
        let mut system = sysinfo::System::default();
        system.refresh_all();
        system.process(Pid::from_u32(pid))?;

        let web_manager_port: u32 = match lopxy_instance_config[1].parse() {
            Ok(v) => v,
            Err(_) => return None,
        };

        let proxy_port: u32 = match lopxy_instance_config[2].parse() {
            Ok(v) => v,
            Err(_) => return None,
        };

        Some(LopxyInstance {
            pid,
            web_manager_port,
            proxy_port,
        })
    }

    ///
    /// Record Lopxy instance
    ///
    pub fn record(
        config_dir: &std::path::PathBuf,
        web_manager_port: u32,
        proxy_port: u32,
    ) -> std::io::Result<()> {
        let config_path = LopxyInstance::lopxy_pid_path(config_dir);
        let contents = format!(
            "{}\r\n{}\r\n{}",
            std::process::id(),
            web_manager_port,
            proxy_port
        );
        std::fs::write(config_path, contents)
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn web_manager_port(&self) -> u32 {
        self.web_manager_port
    }

    pub fn proxy_port(&self) -> u32 {
        self.proxy_port
    }

    pub fn web_manager_url(&self) -> String {
        format!("127.0.0.1:{}", self.web_manager_port)
    }

    pub fn proxy_url(&self) -> String {
        format!("127.0.0.1:{}", self.proxy_port)
    }
}

#[derive(Debug)]
pub struct LopxyEnv {
    pub config_dir: std::path::PathBuf,
    pub command_args: LopxyCommand,
}

impl LopxyEnv {
    pub fn collect(args: LopxyArgs) -> Option<LopxyEnv> {
        let config_dir = config::program_config_dir(env!("CARGO_PKG_NAME"))?;

        Some(LopxyEnv {
            config_dir,
            command_args: args.command,
        })
    }

    pub fn start_args<'a>(&'a self) -> Option<&'a StartArgs> {
        match &self.command_args {
            LopxyCommand::Start(arg) => Some(arg),
            _ => None,
        }
    }

    pub fn add_args<'a>(&'a self) -> Option<&'a AddArgs> {
        match &self.command_args {
            LopxyCommand::Add(arg) => Some(arg),
            _ => None,
        }
    }

    pub fn remove_args<'a>(&'a self) -> Option<&'a RemoveArgs> {
        match &self.command_args {
            LopxyCommand::Remove(arg) => Some(arg),
            _ => None,
        }
    }

    ///
    /// Check lopxy instance and decide whether to switch to background
    ///
    pub fn guard_instance(&self) {
        // check instance
        if LopxyInstance::instance(&self.config_dir).is_some() {
            println!("lopxy is already running...");
            std::process::exit(1);
        }

        // switch to background
        let start_args = self.start_args().expect("start args collect failed");
        if start_args.daemon {
            util::daemon::daemon();
        }

        // record instance
        LopxyInstance::record(
            &self.config_dir,
            start_args.web_manager_port,
            start_args.proxy_port,
        )
        .expect("record lopxy instance failed");
    }

    ///
    /// Get Web Manager Server Instance URL
    ///
    pub fn web_manager_instance(&self) -> Option<String> {
        let instance = LopxyInstance::instance(&self.config_dir)?;
        Some(format!("http://{}", instance.web_manager_url()))
    }
}
