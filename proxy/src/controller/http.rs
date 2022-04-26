use async_std::io::WriteExt;
use async_std::net::TcpStream;

use futures::FutureExt;

use super::super::request::*;
use super::super::response::*;
use super::super::stream::*;
use super::super::item::*;

pub async fn handle_proxy_request(proxy_request: LopxyProxyRequest<'_>) {
    // fetch request url
    let request_url = match proxy_request.request_header.path {
        Some(url) => url,
        None => {
            eprintln!("invalid request url");
            return;
        }
    };

    // lopxy proxy redirect
    let proxy_redirect = proxy_request.client.proxy_redirect(request_url);

    // direct request
    if proxy_redirect.is_none() && !proxy_request.client.use_system_proxy() {
        handle_direct_request(proxy_request, request_url).await;
        return;
    }

    // local file
    if proxy_redirect.is_some() {
        let proxy_item = proxy_redirect.as_ref().unwrap();
        let scheme = match super::super::request::get_uri_scheme(&proxy_item.proxy_resource_url()) {
            Some(s) => s,
            None => {
                eprintln!("invalid request scheme");
                return;
            }
        };

        if scheme.eq_ignore_ascii_case("file") {
            handle_local_file_request(proxy_request, proxy_item.clone()).await;
            return;
        }
    }

    // do redirect request
    handle_redirect_request(proxy_request, request_url, proxy_redirect).await;
}

async fn direct_tunnel_transmit(proxy_request: &mut LopxyProxyRequest<'_>, server_stream: &mut TcpStream) -> std::io::Result<Option<u16>>
{
    server_stream.write_all(proxy_request.request_bytes).await?;
    let raw_response_bytes = collect_tcp_stream_buffer(server_stream).await?;
    let status_code = try_response_status(&raw_response_bytes);
    proxy_request.client.stream.write_all(&raw_response_bytes).await?;
    Ok(status_code)
}

///
/// Handle direct request
/// 
/// # Notes
/// At present, for the `Keep-Alive Session`, except for the first request, 
/// the subsequent request will not take over the processing temporarily
async fn handle_direct_request(mut proxy_request: LopxyProxyRequest<'_>, request_url: &str) {
    // connect remote host
    let mut server_stream = match TcpStream::connect(&proxy_request.host).await {
        Ok(stream) => stream,
        Err(err) => {
            eprintln!("connect remote http server failed : {}", err);
            proxy_request.report_connection_error(&proxy_request.host, proxy_request.try_request_url(), &err);
            return;
        }
    };

    match direct_tunnel_transmit(&mut proxy_request, &mut server_stream).await {
        Ok(Some(status_code)) => {
            proxy_request.report_proxy_request_status(request_url, status_code);
        },
        Ok(None) => {

        },
        Err(err) => {
            eprintln!("direct tunnel transmit failed : {}", err);
            proxy_request.report_connection_error(&proxy_request.host, proxy_request.try_request_url(), &err);
            return;
        }
    }

    //
    // tunnel
    //

    let (client_receiver, client_sender) = &mut (&proxy_request.client.stream, &proxy_request.client.stream);
    let (server_receiver, server_sender) = &mut (&server_stream, &server_stream);
 
    let cf1 = async_std::io::copy(client_receiver, server_sender);
    let cf2 = async_std::io::copy(server_receiver, client_sender);

    let waits = move || async move {
        futures::select! {
           r1 = cf1.fuse() => r1,
           r2 = cf2.fuse() => r2,
        }
    };

    match proxy_request.client.shutdown.wrap_cancel(waits()).await {
        Some(Ok(_)) => {},
        Some(Err(err)) => eprintln!("http tunnel failed : {}", err),
        None => eprintln!("proxy server shutdown triggered, closing connection"),
    }
}

async fn handle_local_file_request(mut proxy_request: LopxyProxyRequest<'_>, proxy_redirect: ProxyItem) {
    let raw_response_bytes = build_local_file_response(proxy_redirect.proxy_resource_url(), proxy_redirect.content_type());
    match proxy_request.client.reply(&raw_response_bytes).await { _ => {} };
}

async fn handle_redirect_request(mut proxy_request: LopxyProxyRequest<'_>, request_url: &str, proxy_redirect: Option<ProxyItem>) {
    let mut client_builder = reqwest::Client::builder();

    // config proxy
    if proxy_request.client.use_system_proxy() {
        let system_proxy_addr = format!("http://{}", proxy_request.client.system_proxy_config.proxy_server.as_ref().unwrap());
        match reqwest::Proxy::http(system_proxy_addr) {
            Ok(proxy) => {
                client_builder = client_builder.proxy(proxy);
            }

            Err(_) => {

            }
        }
    }

    // collect headers
    let mut headers = proxy_request.headers();

    // build request
    let request = match client_builder.build() {
        Ok(request) => request,
        Err(err) => {
            eprintln!("build request failed : {}", err);
            proxy_request.client.reply_502().await;
            return;
        }
    };

    // request url
    let request_url = match proxy_redirect {
        Some(ref item) => item.proxy_resource_url(),
        None => request_url
    };

    //
    // request host
    //

    let host = match get_host_from_url(request_url) {
        Some(host) => host,
        None => "".to_string()
    };

    match reqwest::header::HeaderValue::from_bytes(&host[..].as_bytes()) {
        Ok(value) => {
            headers.insert(reqwest::header::HOST, value);
        }

        Err(_) => {

        }
    };

    // execute request
    let response = match {
            match proxy_request.method().to_uppercase().as_ref() {
                "GET" => request.get(request_url),
                "POST" => request.post(request_url),
                "PUT" => request.put(request_url),
                "DELETE" => request.delete(request_url),
                "HEAD" => request.head(request_url),
                "PATCH" => request.patch(request_url),
                _ => request.get(request_url),
            }
            .headers(headers)
            .body(proxy_request.body())
            .send()
            .await
    } {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("execute proxy redirect request failed : {}", err);
            proxy_request.report_connection_error(&proxy_request.host, Some(request_url.to_string()), &err);
            return;
        }
    };

    // report response status
    proxy_request.report_proxy_request_status(request_url, response.status().as_u16());

    // build response bytes
    let raw_response_bytes: Vec<u8> = match build_raw_response_bytes(response).await {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("build raw response bytes failed : {}", err);
            return;
        }
    };

    // send response to lopxy proxy client
    match proxy_request.client.reply(&raw_response_bytes).await { _ => {} };
}