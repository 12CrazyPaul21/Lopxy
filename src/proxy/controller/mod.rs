pub mod http;
pub mod https;

use std::sync::Arc;
use std::sync::Mutex;

use super::ProxyClient;
use super::item::*;

use super::super::util::netstat;

pub type LopxyProxyServerControllerArc = Arc<Mutex<dyn LopxyProxyServerController + Send>>;

pub trait LopxyProxyServerController {
    fn proxy_redirect(&mut self, resource_url: &str) -> Option<ProxyItem>;

    fn report_proxy_request_status(&mut self, request_url: &str, status: u16, pid: u32);
    fn report_connection_error(&mut self, host: &str, request_url: Option<String>, err: &dyn std::error::Error, pid: u32);
}

pub async fn handle_lopxy_proxy_client(mut client: ProxyClient) {
    // collect request raw buffer
    let request_buffer = match super::stream::collect_tcp_stream_buffer(&mut client.stream).await {
        Ok(buf) => buf,
        Err(err) => {
            eprintln!("collect proxy client request stream failed : {}", err);
            return;
        }
    };

    // parse request
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    match req.parse(&request_buffer[..]) {
        Ok(status) => {
            if status.is_partial() {
                eprintln!("the request is partial");
                // return;
            }
        }
        Err(err) => {
            eprintln!("parse proxy request failed : {}", err);
            return;
        }
    }

    // get host from request
    let host = match super::request::get_host_from_request(&mut req) {
        Some(h) => h,
        None => {
            eprintln!("get proxy request host failed");
            return;
        }
    };

    // get request method
    let method = match req.method {
        Some(method) => method,
        None => {
            eprintln!("get proxy request method failed");
            return;
        }
    };

    // client port to process id
    let client_port = client.addr.port();
    let pid = match netstat::tcp_port_to_pid(client_port) {
        Some(pid) => pid, 
        None => {
            eprintln!("get proxy request client pid failed");
            0
        }
    };

    // build lopxy proxy request parameter
    let proxy_request = super::request::LopxyProxyRequest {
        https: method.eq("CONNECT"),
        host: host,
        method: method.to_string(),
        client_port: client_port,
        pid,
        request_bytes: &request_buffer,
        request_header: req,
        client: client
    };

    // dispatch proxy request
    match proxy_request.https {
        false => self::http::handle_proxy_request(proxy_request).await,
        true => self::https::handle_proxy_request(proxy_request).await
    }
}