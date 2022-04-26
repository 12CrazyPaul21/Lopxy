#![allow(dead_code)]

pub mod item;
pub mod controller;
pub mod request;
pub mod response;
pub mod stream;

pub use async_shutdown;

use async_std::io::WriteExt;
use async_std::net::{TcpListener, TcpStream, SocketAddr};

use super::util::config;
use controller::*;

/// Default Proxy Server
static DEFAULT_PROXY_SERVER: &str = "";

/// Default Proxy Override
static DEFAULT_PROXY_OVERRIDE: &str = "";

#[derive(Clone)]
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

    #[cfg(target_os = "windows")]
    pub fn is_system_proxy_enabled() -> bool {
        config::get_register_hkcu_value::<u32>(
            config::INTERNET_SETTINGS_SUB_KEY,
            r"ProxyEnable",
        )
        .unwrap_or(0u32)
            > 0
    }

    #[cfg(target_os = "mac")]
    pub fn is_system_proxy_enabled() -> bool {
        panic!(r"<mac> is_system_proxy_enabled stub");
    }

    #[cfg(target_os = "linux")]
    pub fn is_system_proxy_enabled() -> bool {
        panic!(r"<linux> is_system_proxy_enabled stub");
    }

    #[cfg(target_os = "windows")]
    pub fn set_system_proxy_enabled(enabled: bool) {
        match config::set_register_hkcu_value::<u32>(
            config::INTERNET_SETTINGS_SUB_KEY,
            r"ProxyEnable",
            &if enabled { 1 } else { 0 },
        ) { _ => {} }

        config::update_internet_settings();
    }

    #[cfg(target_os = "mac")]
    pub fn set_system_proxy_enabled(enabled: bool) {
        panic!(r"<mac> set_system_proxy_enabled stub");
    }

    #[cfg(target_os = "linux")]
    pub fn set_system_proxy_enabled(enabled: bool) {
        panic!(r"<linux> set_system_proxy_enabled stub");
    }

    ///
    /// Set System Internet Proxy Config
    ///
    /// # Panics
    ///
    /// The `update_system_proxy` function will panic if the target os isn't windows,
    /// because the `update_system_proxy` of other system are not implemented now.
    #[cfg(target_os = "windows")]
    pub fn update_system_proxy(&self) -> std::io::Result<()> {
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

        Ok(())
    }

    #[cfg(target_os = "mac")]
    pub fn update_system_proxy(&self) -> std::io::Result<()> {
        panic!(r"<macos> update_system_proxy stub");
    }

    #[cfg(target_os = "linux")]
    pub fn update_system_proxy(&self) -> std::io::Result<()> {
        panic!(r"<linux> update_system_proxy stub");
    }
}

pub struct ProxyClient {
    pub stream: TcpStream,
    pub addr: SocketAddr,
    pub shutdown: async_shutdown::Shutdown,
    pub system_proxy_config: ProxyConfig,
    pub controller: LopxyProxyServerControllerArc
}

impl ProxyClient {
    pub fn proxy_redirect(&self, url: &str) -> Option<item::ProxyItem> {
        self.controller.lock().unwrap().proxy_redirect(url)
    }

    pub fn use_system_proxy(&self) -> bool {
        self.system_proxy_config.proxy_server.is_some() && self.system_proxy_config.enabled()
    }

    pub async fn reply(&mut self, raw_response_bytes: &[u8]) -> std::io::Result<()> {
        match self.stream.write_all(raw_response_bytes).await {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!("send response to proxy client failed : {}", err);
                Err(err)
            }
        }
    }

    pub async fn reply_404(&mut self) {
        let raw_response_bytes = response::build_404_response();
        match self.reply(&raw_response_bytes).await { _ => {} };
    }

    pub async fn reply_502(&mut self) {
        let raw_response_bytes = response::build_502_response();
        match self.reply(&raw_response_bytes).await { _ => {} };
    }
}

pub struct Proxy {
    pub system_proxy_config: ProxyConfig,
    pub proxy_config: ProxyConfig,
    pub shutdown: async_shutdown::Shutdown,
    pub controller: LopxyProxyServerControllerArc
}

impl Proxy {
    fn build(system_proxy_config: ProxyConfig, proxy_config: ProxyConfig, shutdown: async_shutdown::Shutdown, controller: LopxyProxyServerControllerArc) -> Proxy {
        Proxy {
            system_proxy_config,
            proxy_config,
            shutdown: shutdown,
            controller
        }
    }

    async fn launch(mut proxy: Proxy) -> std::io::Result<()> {
        let proxy_server_addr = proxy.proxy_config.proxy_server.as_ref().expect("invalid proxy server address");
        let server = TcpListener::bind(proxy_server_addr).await.expect("bind proxy server failed");
    
        println!("lopxy proxy server binding in {}", proxy_server_addr);
    
        let proxy_server_shutdown = proxy.shutdown.clone();
        let server_accept_loop = std::thread::spawn(move || async move {
            while let Some(connection) = proxy.shutdown.wrap_cancel(server.accept()).await {
                match connection {
                    Ok((stream, addr)) => {
                        let client = proxy.build_client(stream, addr);
                        tokio::task::spawn(async move {
                            controller::handle_lopxy_proxy_client(client).await;
                        });
                    }
    
                    Err(err) => {
                        eprintln!("lopxy proxy server encountered IO error : {}", err);
                    }
                }
            }
        });
    
        proxy_server_shutdown.wait_shutdown_complete().await;
        server_accept_loop.join().expect("lopxy server accept loop join exception").await;
        Ok(())
    }

    pub async fn start(system_proxy_config: ProxyConfig, proxy_config: ProxyConfig, shutdown: async_shutdown::Shutdown, controller: LopxyProxyServerControllerArc) -> tokio::task::JoinHandle<()> {
        tokio::task::spawn(async move {
            Proxy::launch(Proxy::build(
                system_proxy_config,
                proxy_config,
                shutdown,
                controller
            )).await.expect("proxy server launch failed");
        })
    }

    pub fn build_client(&mut self, stream: TcpStream, addr: SocketAddr) -> ProxyClient {
        ProxyClient {
            stream,
            addr,
            shutdown: self.shutdown.clone(),
            system_proxy_config: self.system_proxy_config.clone(),
            controller: self.controller.clone()
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
