#![allow(dead_code)]

use std::sync::Mutex;
use std::collections::VecDeque;

use chrono::prelude::*;
use sysinfo::SystemExt;
use sysinfo::ProcessExt;
use sysinfo::{Pid, PidExt};
use serde_derive::{Serialize, Deserialize};

use super::proxy;
use super::args::*;
use super::config::*;
use super::util;

use util::config;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LopxyProxyRequestStatus {
    pub timestamp: i64,
    pub pid: u32,
    pub bin_name: String,
    pub path: String,
    pub status: String,
}

impl LopxyProxyRequestStatus {
    pub fn quota() -> usize {
        500
    }

    pub fn report(&self) -> LopxyProxyRequestStatus {
        LopxyProxyRequestStatus {
            timestamp: self.timestamp,
            pid: self.pid,
            bin_name: self.bin_name.clone(),
            path: self.path.clone(),
            status: self.status.clone()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LopxyStatusReport {
    pub success: bool,
    pub web_manager_port: u32,
    pub proxy_port: u32,
    pub proxy_enabled: bool,
    pub updated: bool,
    pub status_log_timestamp: i64,
    pub request_status_logs: Vec<LopxyProxyRequestStatus>,
    pub config_timestamp: i64,
    pub proxy_items: Vec<proxy::item::ProxyItem>
}

pub struct LopxyEnv {
    pub config_dir: std::path::PathBuf,
    pub static_assets_dir: std::path::PathBuf,
    pub config: Option<LopxyConfig>,
    pub command_args: LopxyCommand,
    pub proxy_shutdown: proxy::async_shutdown::Shutdown,
    pub request_status_logs: Mutex<VecDeque<LopxyProxyRequestStatus>>,
    pub status_refresh_timestamp: i64
}

impl LopxyEnv {
    pub fn collect(args: LopxyArgs) -> Option<LopxyEnv> {
        let config_dir = config::program_config_dir(env!("CARGO_PKG_NAME"))?;

        let mut static_assets_dir = config_dir.clone();
        static_assets_dir.push("static");

        Some(LopxyEnv {
            config_dir,
            static_assets_dir,
            config: None,
            command_args: args.command,
            proxy_shutdown: proxy::async_shutdown::Shutdown::new(),
            request_status_logs: Mutex::new(VecDeque::new()),
            status_refresh_timestamp: 0
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
    /// Release static assets
    /// 
    pub fn release_static_assets(&self) {
        super::assets::force_release(&self.static_assets_dir).expect("release static assets failed");
    }

    ///
    /// Get static folder path
    /// 
    pub fn static_assets_dir<'a>(&'a self) -> &'a std::path::PathBuf {
        &self.static_assets_dir
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

        self.status_refresh_timestamp = Local::now().timestamp_millis();

        // get process bin name
        let bin_name = {
            let mut system = sysinfo::System::default();
            system.refresh_all();

            match system.process(Pid::from_u32(pid)) {
                Some(p) => {
                    p.name().to_string()
                },
                None => {
                    String::from("")
                }
            }
        };

        record.push_back(LopxyProxyRequestStatus {
            timestamp: self.status_refresh_timestamp,
            pid,
            bin_name: bin_name,
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

    ///
    /// Get lopxy status
    /// 
    pub fn lopxy_status(&mut self, config_timestamp: i64, status_log_timestamp: i64) -> String {
        let start_args = self.start_args().unwrap();
        
        let mut report = LopxyStatusReport {
            success: true,
            web_manager_port: start_args.web_manager_port,
            proxy_port: start_args.proxy_port,
            proxy_enabled: proxy::ProxyConfig::is_system_proxy_enabled(),
            updated: false,
            status_log_timestamp: status_log_timestamp,
            request_status_logs: vec![],
            config_timestamp: config_timestamp,
            proxy_items: vec![]
        };

        if self.status_refresh_timestamp > status_log_timestamp {
            report.updated = true;
            report.status_log_timestamp = self.status_refresh_timestamp;

            let records = &*self.request_status_logs.lock().unwrap();
            for item in records {
                if item.timestamp > status_log_timestamp {
                    report.request_status_logs.push(item.report());
                }
            }
        }

        let config = self.load_config();
        if config.timestamp() > config_timestamp {
            report.updated = true;
            report.config_timestamp = config.timestamp();

            for item in config.proxy_item_list() {
                report.proxy_items.push(item.clone())
            }
        }

        serde_json::to_string(&report).unwrap_or("{\"success\": false}".to_string())
    }
}

impl Drop for LopxyEnv {
    fn drop(&mut self) {
        self.save_config();
    }
}