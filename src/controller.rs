use super::env;

use super::manager;
use super::proxy;
use proxy::item::*;

pub struct LopxyController {
    env: env::LopxyEnv
}

impl LopxyController {
    pub fn build(lopxy_env: env::LopxyEnv) -> LopxyController {
        LopxyController {
            env: lopxy_env
        }
    }

    pub fn env(&mut self) -> &env::LopxyEnv {
        &self.env
    }

    #[allow(dead_code)]
    pub fn env_mut(&mut self) -> &mut env::LopxyEnv {
        &mut self.env
    }
}

impl manager::controller::LopxyManagerServerController for LopxyController {
    fn shutdown(&mut self) {
        self.env.proxy_shutdown.shutdown();
    }

    fn list_all_proxy_item<'a>(&'a mut self) -> &'a Vec<ProxyItem> {
        self.env.load_config().proxy_item_list()
    }

    fn add_proxy_item(&mut self, resource_url: &str, proxy_resource_url: &str, content_type: &str) -> bool {
        let result = self.env.load_config().add_proxy_item(resource_url, proxy_resource_url, content_type);
        self.env.save_config();
        result
    }

    fn remove_proxy_item(&mut self, resource_url: &str) -> bool {
        let result = self.env.load_config().remove_proxy_item(resource_url);
        self.env.save_config();
        result
    }

    fn modify_proxy_item(&mut self, resource_url: &str, proxy_resource_url: &str, content_type: &str) -> bool {
        let result = self.env.load_config().modify_proxy_item(resource_url, proxy_resource_url, content_type);
        self.env.save_config();
        result
    }

    fn is_system_proxy_enabled(&mut self) -> bool {
        proxy::ProxyConfig::is_system_proxy_enabled()
    }

    fn set_system_proxy_enabled(&mut self, enabled: bool) -> bool {
        proxy::ProxyConfig::set_system_proxy_enabled(enabled);
        true
    }

    fn proxy_request_logs(&mut self) -> String {
        self.env.proxy_request_status_logs()
    }

    fn lopxy_status(&mut self, config_timestamp: i64, status_log_timestamp: i64) -> String {
        self.env.lopxy_status(config_timestamp, status_log_timestamp)
    }
}

impl proxy::controller::LopxyProxyServerController for LopxyController {
    fn proxy_redirect(&mut self, resource_url: &str) -> Option<ProxyItem> {
        self.env.load_config().proxy_redirect(resource_url)
    }

    fn report_proxy_request_status(&mut self, request_url: &str, status: u16, pid: u32) {
        let exception_status_desc = proxy::response::get_exception_request_status_desc(status);
        if exception_status_desc.is_none() {
            return;
        }
        self.env.report_proxy_request_status(pid, request_url.to_string(), exception_status_desc.unwrap());
    }

    fn report_connection_error(&mut self, host: &str, request_url: Option<String>, err: &dyn std::error::Error, pid: u32) {
        let path = request_url.unwrap_or(host.to_string());
        self.env.report_proxy_request_status(pid, path, err.to_string());
    }
}