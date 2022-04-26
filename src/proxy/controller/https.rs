use async_std::io::WriteExt;
use async_std::net::TcpStream;

use futures::FutureExt;

use super::super::request::LopxyProxyRequest;

///
/// Handle https proxy request
/// 
/// # Notes
/// The local proxy adapter function is not implemented temporarily, forward directly at present
/// 
pub async fn handle_proxy_request(mut proxy_request: LopxyProxyRequest<'_>) {
    let client_stream = &mut proxy_request.client.stream;

    // connect remote host
    let server_stream = match TcpStream::connect(&proxy_request.host).await {
        Ok(stream) => stream,
        Err(err) => {
            eprintln!("connect remote https server failed : {}", err);
            proxy_request.report_connection_error(&proxy_request.host, proxy_request.try_request_url(), &err);
            return;
        }
    };

    // response connection established
    match client_stream.write_all(b"HTTP/1.0 200 Connection Established\r\n\r\n").await {
        Ok(_) => {},
        Err(err) => {
            eprintln!("reply https connection established failed : {}", err);
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
        Some(Err(err)) => eprintln!("https tunnel failed : {}", err),
        None => eprintln!("proxy server shutdown triggered, closing connection"),
    }
}