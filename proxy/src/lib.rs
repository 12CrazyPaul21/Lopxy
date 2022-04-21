extern crate util;

use util::config;

/// Default Proxy Server
static DEFAULT_PROXY_SERVER: &str = "";

/// Default Proxy Override
static DEFAULT_PROXY_OVERRIDE: &str = "";

pub struct ProxyConfig {
    enabled: bool,
    proxy_server: Option<String>,
    proxy_override: Option<String>,
}

impl<'a> ProxyConfig {
    pub fn new(
        enabled: bool,
        proxy_server: Option<String>,
        proxy_override: Option<String>,
    ) -> ProxyConfig {
        ProxyConfig {
            enabled,
            proxy_server,
            proxy_override,
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn proxy_server(&'a self) -> Option<&'a String> {
        self.proxy_server.as_ref()
    }

    pub fn proxy_override(&'a self) -> Option<&'a String> {
        self.proxy_override.as_ref()
    }
}

impl ProxyConfig {
    ///
    /// Get System Internet Proxy Config
    ///
    /// # Panics
    ///
    /// The `system_proxy` function will panic if the target os neighter windows nor mac,
    /// because the `system_proxy` of other system are not implemented now.
    #[cfg(target_os = "windows")]
    pub fn system_proxy() -> Option<ProxyConfig> {
        let proxy_enabled = config::get_register_hkcu_value::<u32>(
            config::INTERNET_SETTINGS_SUB_KEY,
            r"ProxyEnable",
        )
        .unwrap_or(0u32)
            > 0;

        let proxy_server = match config::get_register_hkcu_value::<String>(
            config::INTERNET_SETTINGS_SUB_KEY,
            "ProxyServer",
        ) {
            Ok(v) => Some(v),
            Err(_) => None,
        };

        let proxy_override = match config::get_register_hkcu_value::<String>(
            config::INTERNET_SETTINGS_SUB_KEY,
            "ProxyOverride",
        ) {
            Ok(v) => Some(v),
            Err(_) => None,
        };

        Some(ProxyConfig {
            enabled: proxy_enabled,
            proxy_server,
            proxy_override,
        })
    }

    #[cfg(target_os = "mac")]
    pub fn system_proxy() -> Option<ProxyConfig> {
        let network = config::get_current_network()?;
        Some(ProxyConfig {
            enabled: config::is_network_proxy_enabled(network),
            proxy_server: config::get_network_proxy_server(network),
            proxy_override: None,
        })
    }

    #[cfg(target_os = "linux")]
    pub fn system_proxy() -> Option<ProxyConfig> {
        panic!(r"<linux> system_proxy stub");
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
            let proxy_override: &str = match &self.proxy_override {
                Some(v) => &v[..],
                None => DEFAULT_PROXY_OVERRIDE,
            };

            config::set_register_hkcu_value::<u32>(
                config::INTERNET_SETTINGS_SUB_KEY,
                r"ProxyEnable",
                &if self.enabled { 1 } else { 0 },
            )?;
            config::set_register_hkcu_value::<&str>(
                config::INTERNET_SETTINGS_SUB_KEY,
                r"ProxyServer",
                &proxy_server,
            )?;
            config::set_register_hkcu_value::<&str>(
                config::INTERNET_SETTINGS_SUB_KEY,
                r"ProxyOverride",
                &proxy_override,
            )?;

            config::update_internet_settings();
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
            system_proxy_config: ProxyConfig {
                enabled: false,
                proxy_server: None,
                proxy_override: None,
            },
            proxy_config: ProxyConfig {
                enabled: false,
                proxy_server: None,
                proxy_override: None,
            },
        }
    }
}

impl Drop for Proxy {
    fn drop(&mut self) {}
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
        let new_proxy_config = ProxyConfig::new(
            true,
            Some("127.0.0.1:7878".to_string()),
            Some("<local>".to_string()),
        );

        //
        // update system proxy config
        //

        new_proxy_config.update_system_proxy().unwrap();
        let new_system_proxy_config = ProxyConfig::system_proxy().unwrap();
        assert_eq!(
            new_system_proxy_config.enabled(),
            new_proxy_config.enabled()
        );
        assert_eq!(
            new_system_proxy_config.proxy_server(),
            new_proxy_config.proxy_server()
        );
        assert_eq!(
            new_system_proxy_config.proxy_override(),
            new_proxy_config.proxy_override()
        );

        //
        // restore system proxy config
        //

        old_proxy_config.update_system_proxy().unwrap();
        let new_system_proxy_config = ProxyConfig::system_proxy().unwrap();
        assert_eq!(
            new_system_proxy_config.enabled(),
            old_proxy_config.enabled()
        );
        assert_eq!(
            new_system_proxy_config.proxy_server(),
            old_proxy_config.proxy_server()
        );
        assert_eq!(
            new_system_proxy_config.proxy_override(),
            new_proxy_config.proxy_override()
        );
    }
}
