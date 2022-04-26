#[allow(dead_code)]
pub struct LopxyProxyRequest<'a> {
    pub https: bool,
    pub host: String,
    pub method: String,
    pub client_port: u16,
    pub pid: u32,
    pub request_bytes: &'a Vec<u8>,
    pub request_header: httparse::Request<'a, 'a>,
    pub client: super::ProxyClient,
}

impl LopxyProxyRequest<'_> {

    ///
    /// Try to get request url
    /// 
    pub fn try_request_url(&self) -> Option<String> {
        match self.request_header.path {
            Some(url) => Some(url.to_string()),
            None => None
        }
    }

    ///
    /// Get request url
    /// 
    pub fn request_url(&self) -> String {
        if let Some(url) = self.request_header.path {
            return url.to_string();
        }

        return format!("http{}://{}", if self.https { "s" } else { "" }, self.host)
    }

    ///
    /// Get request method
    /// 
    pub fn method<'a>(&'a self) -> &'a str {
        &self.method
    }

    ///
    /// Get request body length
    /// 
    pub fn content_length(&self) -> usize {
        for header in self.request_header.headers.iter() {
            if header.name.eq_ignore_ascii_case("content-length") {
                return String::from_utf8_lossy(header.value).parse().unwrap_or(0);
            }
        }

        0
    }

    ///
    /// Copy request body
    /// 
    pub fn body(&self) -> Vec<u8> {
        let mut body = vec![0; 0];
        let content_length = self.content_length();

        if content_length == 0 || content_length >= self.request_bytes.len() {
            return body;
        }

        body.extend_from_slice(&self.request_bytes.as_slice()[self.request_bytes.len() - content_length..]);
        body
    }
    
    ///
    /// Collect request headers, ignore `host` header
    /// 
    pub fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        let check_header = |name: &str| -> bool {
            !{
                name.eq_ignore_ascii_case("host") ||
                name.eq_ignore_ascii_case("postman-token") ||
                name.eq_ignore_ascii_case("user-agent") ||
                name.eq_ignore_ascii_case("accept") ||
                name.eq_ignore_ascii_case("accept-encoding") ||
                name.eq_ignore_ascii_case("cache-control") ||
                name.eq_ignore_ascii_case("connection")
            }
        };
        
        for header in self.request_header.headers.iter() {
            if !check_header(header.name) {
                continue;
            }

            let name = match reqwest::header::HeaderName::from_bytes(header.name.as_ref()) {
                Ok(name) => name,
                Err(_) => {
                    continue;
                }
            };

            let value = match reqwest::header::HeaderValue::from_bytes(header.value) {
                Ok(value) => value,
                Err(_) => {
                    continue;
                }
            };

            headers.insert(name, value);
        }

        headers
    }

    ///
    /// Report proxy request status
    /// 
    pub fn report_proxy_request_status(&self, request_url: &str, status: u16) {
        self.client.controller.lock().unwrap().report_proxy_request_status(request_url, status, self.pid);
    }
    
    ///
    /// Report proxy connection error
    /// 
    pub fn report_connection_error(&self, host: &str, request_url: Option<String>, err: &dyn std::error::Error) {
        self.client.controller.lock().unwrap().report_connection_error(host, request_url, err, self.pid);
    }
}

pub fn get_uri_scheme(url: &str) -> Option<String> {
    let url = match url::Url::parse(url) {
        Ok(url) => url,
        Err(_) => {
            return None;
        }
    };
    
    Some(url.scheme().to_string())
}

pub fn get_uri_path(url: &str) -> Option<String> {
    let url = match url::Url::parse(url) {
        Ok(url) => url,
        Err(_) => {
            return None;
        }
    };
    
    Some(url.path().to_string())
}

pub fn get_host_from_url(url: &str) -> Option<String> {
    match url::Url::parse(url) {
        Ok(url) => {
            let host = url.host()?;
            Some(host.to_string())
        }

        Err(_) => {
            None
        }
    }
}

pub fn get_host_from_request(request: &mut httparse::Request) -> Option<String> {
    let mut host: String = String::new();

    if !request.headers.iter().any(|h| {
        if !h.name.eq_ignore_ascii_case("host") {
            return false;
        }

        host = match std::str::from_utf8(h.value) {
            Ok(v) => v.to_string(),
            Err(_) => {
                return false;
            }
        };

        true
    }) {
        host = get_host_from_url(request.path?)?;
    }

    if !host.contains(":") {
        host.push_str(":80");
    }

    Some(host)
}