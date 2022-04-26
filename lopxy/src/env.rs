#![allow(dead_code)]

use std::sync::Mutex;
use std::collections::VecDeque;

use chrono::prelude::*;
use sysinfo::SystemExt;
use serde_derive::{Serialize, Deserialize};

use util::config;

use super::args::*;
use super::config::*;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct LopxyProxyRequestStatus {
    pub timestamp: String,
    pub pid: u32,
    pub path: String,
    pub status: String,
}

impl LopxyProxyRequestStatus {
    pub fn quota() -> usize {
        500
    }
}

pub struct LopxyEnv {
    pub config_dir: std::path::PathBuf,
    pub config: Option<LopxyConfig>,
    pub command_args: LopxyCommand,
    pub proxy_shutdown: proxy::async_shutdown::Shutdown,
    pub request_status_logs: Mutex<VecDeque<LopxyProxyRequestStatus>>
}

impl LopxyEnv {
    pub fn collect(args: LopxyArgs) -> Option<LopxyEnv> {
        let config_dir = config::program_config_dir(env!("CARGO_PKG_NAME"))?;

        Some(LopxyEnv {
            config_dir,
            config: None,
            command_args: args.command,
            proxy_shutdown: proxy::async_shutdown::Shutdown::new(),
            request_status_logs: Mutex::new(VecDeque::new())
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

    pub fn modify_args<'a>(&'a self) -> Option<&'a ModifyArgs> {
        match &self.command_args {
            LopxyCommand::Modify(arg) => Some(arg),
            _ => None,
        }
    }

    pub fn clone_proxy_shutdown(&self) -> proxy::async_shutdown::Shutdown {
        self.proxy_shutdown.clone()
    }

    ///
    /// Get Config.toml path
    /// 
    pub fn config_path(&mut self) -> String {
        let mut config_path = self.config_dir.clone();
        config_path.push("config.toml");
        config_path.to_str().expect("get lopxy config file path failed").to_string()
    }

    ///
    /// Load lopxy config
    /// 
    pub fn load_config<'a>(&'a mut self) -> &'a mut LopxyConfig {
        if self.config.is_some() {
            return self.config.as_mut().unwrap();
        }

        let config_path = self.config_path();
        self.config = Some(LopxyConfig::load(&config_path));

        self.config.as_mut().unwrap()
    }

    ///
    /// Save lopxy config
    /// 
    pub fn save_config(&mut self) -> bool {
        let config_path = self.config_path();

        if let Some(config) = self.config.as_mut() {
            return match config.save(&config_path) {
                Ok(_) => true,
                Err(_) => false
            }
        }

        false
    }

    ///
    /// Check lopxy instance and decide whether to switch to background
    ///
    pub fn guard_instance(&self) {
        // check instance
        if LopxyInstance::instance(&self.config_dir).is_some() {
            println!("lopxy is already running...");
            std::process::exit(0);
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

    ///
    /// Record proxy request status
    /// 
    /// # Notes
    /// Only record exception request
    pub fn report_proxy_request_status(&mut self, pid: u32, path: String, status: String) {
        let record = &mut *self.request_status_logs.lock().unwrap();

        if record.len() >= LopxyProxyRequestStatus::quota() {
            match record.pop_front() { _ => {} }
        }

        record.push_back(LopxyProxyRequestStatus {
            timestamp: Local::now().to_string(),
            pid,
            path,
            status
        });
    }

    ///
    /// Get proxy request status logs
    ///
    pub fn proxy_request_status_logs(&self) -> String {
        let record = &*self.request_status_logs.lock().unwrap();
        serde_json::to_string(&record).unwrap_or("[]".to_string())
    }
}

impl Drop for LopxyEnv {
    fn drop(&mut self) {
        self.save_config();
    }
}