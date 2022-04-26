extern crate dirs;

use std::path;

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {

        use winapi::um::wininet;
        use winreg::enums as RegEnums;
        use winreg::types as RegTypes;
        use winreg::RegKey;

        ///
        /// Internet Settings Register's Sub Key
        ///
        /// Register Absolute Path: HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Internet Settings
        ///
        pub const INTERNET_SETTINGS_SUB_KEY: &'static str= r"SOFTWARE\Microsoft\Windows\CurrentVersion\Internet Settings";

        ///
        /// Get HKEY_CURRENT_USER Register Value
        ///
        /// # Tests
        ///
        /// ```
        /// let proxy_enable: u32 = util::config::get_register_hkcu_value(util::config::INTERNET_SETTINGS_SUB_KEY, r"ProxyEnable").unwrap();
        /// assert_eq!(proxy_enable == 0 || proxy_enable == 1, true);
        ///
        /// let _proxy_server: String = util::config::get_register_hkcu_value(util::config::INTERNET_SETTINGS_SUB_KEY, r"ProxyServer").unwrap();
        /// ```
        pub fn get_register_hkcu_value<T: RegTypes::FromRegValue>(sub_key: &'static str, value_name: &'static str) -> std::io::Result<T> {
            RegKey::predef(RegEnums::HKEY_CURRENT_USER).open_subkey(sub_key)?.get_value(value_name)
        }

        ///
        /// Set HKEY_CURRENT_USER Register Value
        ///
        /// # Tests
        ///
        /// ```ignore
        /// util::config::set_register_hkcu_value(util::config::INTERNET_SETTINGS_SUB_KEY, r"ProxyEnable", &0u32).unwrap();
        /// util::config::set_register_hkcu_value(util::config::INTERNET_SETTINGS_SUB_KEY, r"ProxyServer", &"".to_string()).unwrap();
        /// ```
        pub fn set_register_hkcu_value<T: RegTypes::ToRegValue>(sub_key: &'static str, value_name: &'static str, value: &T) -> std::io::Result<()> {
            RegKey::predef(RegEnums::HKEY_CURRENT_USER).create_subkey(sub_key)?.0.set_value(value_name, value)
        }

        ///
        /// Update Internet Settings
        ///
        pub fn update_internet_settings() {
            unsafe {
                wininet::InternetSetOptionA(std::ptr::null_mut(), wininet::INTERNET_OPTION_SETTINGS_CHANGED, std::ptr::null_mut(), 0);
            }
        }
    } else if #[cfg(target_os = "macos")] {

        use std::process::Command;

        fn normalize_server(server: &str) -> String {
            match server.find(":") {
                Some(_) => server.to_string(),
                None => format!("http://{}", server)
            }
        }

        fn build_address(server: &str) -> Option<networksetup::Address> {
            let server = normalize_server(server);
            let url = match url::Url::parse(&server) {
                Ok(url) => url,
                Err(_) => return None
            }

            let host = format!("{}", url.host()?);
            let port = format!("{}", url.port_or_known_default()?);

            Some(networksetup::new(host, port))
        }

        fn get_network_proxy_config(network: &'static str) -> Option<String> {
            let output = match Command::new("networksetup").args(["-getwebproxy", network]).output() {
                Ok(v) => v,
                Err(_) => return None
            };
        
            if !output.status.success() {
                return None;
            }
        
            let match String::from_utf8(output.stdout) {
                Ok(v) => v,
                Err(_) => return None
            };

            let captures = match Regex::new(&format!("(?:{0}: )(?P<{0}>[^\r\n]+)", item)) {
                Ok(r) => r,
                Err(_) => return None
            }.captures(&config)?;
        
            Some(captures[item].to_string())
        }

        ///
        /// Get Current Network Interface
        /// 
        /// # Panics
        /// 
        /// not implemented
        pub fn get_current_network() -> Option<String> {
            panic!("<mac> get_current_network stub")
        }

        ///
        /// Get Network Proxy Server With networksetup Command
        ///
        /// # Shell Command
        /// 
        /// ```bash
        /// networksetup -getwebproxy {network interface} | awk {'print $2'} | awk {'getline l2; getline l3; print l2":"l3'} | head -n 1
        /// # output like:
        /// #   Server: server_address
        /// #   Port: 8001
        /// #   ...
        /// ```
        /// 
        pub fn get_network_proxy_server(network: &'static str) -> Option<String> {
            format!("{0}:{1}", get_network_proxy_config("network", "Server")?, get_network_proxy_config("network", "Port")?)
        }

        ///
        /// Set Network Proxy Server
        ///
        pub fn set_network_proxy_server(network: &'static str, server: &str) -> bool {
            let addr = match build_address(server) {
                Some(v) => v,
                None => return false
            };

            return networksetup::web_proxy(networksetup::Network::Name(network), networksetup::Config::Value(&addr)).is_ok();
        }

        ///
        /// Is Network Proxy Enabled With networksetup Command
        ///
        /// # Shell Command
        /// ```bash
        /// networksetup -getwebproxy {network interface} | awk {'print $2'} | head -n 1
        /// # output like:
        /// #   Enabled: Yes
        /// #   ...
        /// ```
        pub fn is_network_proxy_enabled(network: &'static str) -> bool {
            match get_network_proxy_config(network, "Enabled") {
                Some(enabled) => enabled.find("Yes").is_some(),
                None => false
            }
        }

        ///
        /// Set Network Proxy Enabled
        /// 
        pub fn set_network_proxy_enabled(network: &'static str, enabled: bool) -> bool {
            use networksetup::Config;
            return networksetup::web_proxy(networksetup::Network::Name(network), if enabled { Config::On } else { Config::Off }).is_ok();
        }

    } else if #[cfg(target_os = "linux")] {

    }
}

///
/// Get program's user config folder, and attempt to recursively create the directory when it doesn't exist.
///
/// # Tests
///
/// ```ignore
/// let config_dir = util::config::program_config_dir("lopxy").unwrap();
/// let config_dir = config_dir.to_str().unwrap();
/// assert_eq!(config_dir.len() != 0, true);
/// ```
pub fn program_config_dir(app_name: &str) -> Option<path::PathBuf> {
    let mut home_dir = dirs::home_dir()?;
    home_dir.push(format!(".{}", app_name));

    if !home_dir.exists() {
        if let Err(_) = std::fs::create_dir_all(&home_dir) {
            return None;
        }
    }

    if !home_dir.is_dir() {
        return None;
    }

    Some(home_dir)
}
