extern crate util;

use util::config;

/// Default Proxy Server
static DEFAULT_PROXY_SERVER: &str = "";

pub struct ProxyConfig {
    enabled: bool,
    proxy_server: Option<String>,
}

impl<'a> ProxyConfig {
    pub fn new(enabled: bool, proxy_server: Option<String>) -> ProxyConfig {
        ProxyConfig {
            enabled,
            proxy_server
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn proxy_server(&'a self) -> Option<&'a String> {
        self.proxy_server.as_ref()
    }
}

impl ProxyConfig {
    ///
    /// Get System Internet Proxy Config
    /// 
    /// # Panics
    /// 
    /// The `system_proxy` function will panic if the target os isn't windows, 
    /// because the `system_proxy` of other system are not implemented now.
    pub fn system_proxy() -> Option<ProxyConfig> {
        let mut proxy_enabled = false;
        let mut proxy_server: Option<String> = None;

        if cfg!(windows) {
            proxy_enabled = config::get_register_hkcu_value::<u32>(config::INTERNET_SETTINGS_SUB_KEY, r"ProxyEnable").unwrap_or(0u32) > 0;
            proxy_server = match config::get_register_hkcu_value::<String>(config::INTERNET_SETTINGS_SUB_KEY, "ProxyServer") {
                Ok(v) => Some(v),
                Err(_) => None
            };
        } else if cfg!(macos) {
            panic!(r"<macos> system_proxy stub");
        } else if cfg!(linux) {
            panic!(r"<linux> system_proxy stub");
        }

        Some(ProxyConfig {
            enabled: proxy_enabled,
            proxy_server: proxy_server,
        })
    }

    ///
    /// Set System Internet Proxy Config
    /// 
    /// # Panics
    /// 
    /// The `update_system_proxy` function will panic if the target os isn't windows, 
    /// because the `update_system_proxy` of other system are not implemented now.
    pub fn update_system_proxy(&self) -> std::io::Result<()> {
        if cfg!(windows) {
            let proxy_server: &str = match &self.proxy_server {
                Some(v) => &v[..],
                None => DEFAULT_PROXY_SERVER,
            };
            config::set_register_hkcu_value::<u32>(config::INTERNET_SETTINGS_SUB_KEY, r"ProxyEnable", &if self.enabled {1} else {0})?;
            config::set_register_hkcu_value::<&str>(config::INTERNET_SETTINGS_SUB_KEY, r"ProxyServer", &proxy_server)?;
        } else if cfg!(macos) {
            panic!(r"<macos> update_system_proxy stub");
        } else if cfg!(linux) {
            panic!(r"<linux> update_system_proxy stub");
        }

        Ok(())
    }
}

pub struct Proxy {
    system_proxy_config: ProxyConfig,
    proxy_config: ProxyConfig,
}

impl Proxy {
    pub fn new() -> Proxy {
        Proxy {
            system_proxy_config: ProxyConfig{
                enabled: false,
                proxy_server: None,
            },
            proxy_config: ProxyConfig{
                enabled: false,
                proxy_server: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_system_proxy_config() {
        ProxyConfig::system_proxy().unwrap();
    }

    #[test]
    #[ignore]
    fn set_system_proxy_config() {
        let old_proxy_config = ProxyConfig::system_proxy().unwrap();
        let new_proxy_config = ProxyConfig::new(true, Some("127.0.0.1:7878".to_string()));

        //
        // update system proxy config
        //

        new_proxy_config.update_system_proxy().unwrap();
        let new_system_proxy_config = ProxyConfig::system_proxy().unwrap();
        assert_eq!(new_system_proxy_config.enabled(), new_proxy_config.enabled());
        assert_eq!(new_system_proxy_config.proxy_server(), new_proxy_config.proxy_server());

        //
        // restore system proxy config
        //

        old_proxy_config.update_system_proxy().unwrap();
        let new_system_proxy_config = ProxyConfig::system_proxy().unwrap();
        assert_eq!(new_system_proxy_config.enabled(), old_proxy_config.enabled());
        assert_eq!(new_system_proxy_config.proxy_server(), old_proxy_config.proxy_server());
    }
}
