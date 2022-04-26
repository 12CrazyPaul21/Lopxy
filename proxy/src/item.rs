use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProxyItem {
    resource_url: String,
    proxy_resource_url: String,
    content_type: String,
}

impl ProxyItem {
    pub fn new(resource_url: &str, proxy_resource_url: &str, content_type: &str) -> ProxyItem {
        ProxyItem {
            resource_url: resource_url.to_string(),
            proxy_resource_url: proxy_resource_url.to_string(),
            content_type: content_type.to_string()
        }
    }

    pub fn resource_url<'a>(&'a self) -> &'a str {
        &self.resource_url
    }

    pub fn proxy_resource_url<'a>(&'a self) -> &'a str {
        &self.proxy_resource_url
    }

    pub fn content_type<'a>(&'a self) -> &'a str {
        &self.content_type
    }

    pub fn update_proxy_resource_url(&mut self, proxy_resource_url: &str) {
        self.proxy_resource_url = proxy_resource_url.to_string()
    }

    pub fn update_resource_content_type(&mut self, content_type: &str) {
        self.content_type = content_type.to_string()
    }
}