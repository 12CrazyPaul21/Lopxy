extern crate dirs;

use std::path;

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {

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
    } else if #[cfg(target_os = "macos")] {

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