#![allow(dead_code)]

use serde_derive::{Serialize, Deserialize};

use super::proxy::item::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct LopxyConfig {
    proxy_items: Vec<ProxyItem>
}

impl LopxyConfig {
    pub fn new() -> LopxyConfig {
        LopxyConfig {
            proxy_items: vec![]
        }
    }

    pub fn load(path: &str) -> LopxyConfig {
        let config = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                return LopxyConfig::new();
            }
        };

        match toml::from_str(&config) {
            Ok(c) => c,
            Err(_) => {
                LopxyConfig::new()
            }
        }
    }

    pub fn save(&mut self, path: &str) -> std::io::Result<()> {
        let desc: String = match toml::to_string(self) {
            Ok(desc) => desc,
            Err(_) => {
                return Err(std::io::Error::from(std::io::ErrorKind::InvalidData))
            }
        };

        std::fs::write(path, &desc)
    }

    pub fn proxy_item_list<'a>(&'a self) -> &'a Vec<ProxyItem> {
        &self.proxy_items
    }

    pub fn proxy_item_count(&self) -> usize {
        self.proxy_items.len()
    }

    pub fn proxy_redirect(&self, resource_url: &str) -> Option<ProxyItem> {
        let mut result: Option<ProxyItem> = None;

        self.proxy_items.iter().any(|item: &ProxyItem| {
            if item.resource_url().ne(resource_url) {
                return false;
            }

            result = Some(item.clone());

            true
        });

        result
    }

    pub fn proxy_item_exists(&self, resource_url: &str) -> bool {
        self.proxy_items.iter().any(|item: &ProxyItem| item.resource_url().eq(resource_url))
    }

    pub fn add_proxy_item(&mut self, resource_url: &str, proxy_resource_url: &str, content_type: &str) -> bool {
        if self.proxy_item_exists(resource_url) {
            return false;
        }

        let verify_url = |url: &str| -> bool {
            url::Url::parse(url).is_ok()
        };

        if !verify_url(resource_url) || !verify_url(proxy_resource_url) {
            return false;
        }

        self.proxy_items.push(ProxyItem::new(resource_url, proxy_resource_url, content_type));

        true
    }

    pub fn remove_proxy_item(&mut self, resource_url: &str) -> bool {
        if !self.proxy_item_exists(resource_url) {
            return false;
        }

        self.proxy_items.retain(|item: &ProxyItem| item.resource_url().ne(resource_url));

        true
    }

    pub fn modify_proxy_item(&mut self, resource_url: &str, proxy_resource_url: &str, content_type: &str) -> bool {
        if url::Url::parse(proxy_resource_url).is_err() {
            return false;
        }

        if !self.proxy_items.iter_mut().any(|item: &mut ProxyItem| {
            if !item.resource_url().eq(resource_url) {
                return false;
            }

            item.update_proxy_resource_url(proxy_resource_url);
            item.update_resource_content_type(content_type);

            true
        }) {
            return self.add_proxy_item(resource_url, proxy_resource_url, content_type);
        }

        true
    }
}