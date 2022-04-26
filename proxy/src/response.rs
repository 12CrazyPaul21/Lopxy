pub fn http_version_desc(version: &reqwest::Version) -> &'static str {
    match *version {
        reqwest::Version::HTTP_09 => "HTTP/0.9",
        reqwest::Version::HTTP_10 => "HTTP/1.0",
        reqwest::Version::HTTP_11 => "HTTP/1.1",
        reqwest::Version::HTTP_2 => "HTTP/2.0",
        reqwest::Version::HTTP_3 => "HTTP/3.0",
        _ => "HTTP/1.1"
    }
}

pub fn get_exception_request_status_desc(status: u16) -> Option<String> {
    Some(match reqwest::StatusCode::from_u16(status) {
        Ok(status) => {
            if status.is_success() {
                return None;
            }

            format!("{}", status)
        },
        Err(_) => "Unknown Status".to_string()
    })
}

pub fn try_response_status(raw_response_bytes: &[u8]) -> Option<u16> {
    let parse_len = std::cmp::min(20, raw_response_bytes.len());
    let response_line = String::from_utf8_lossy(&raw_response_bytes[0..parse_len]);
    let mut response_line = response_line.split_ascii_whitespace();

    if  response_line.next().is_some() {
        return match response_line.next()?.parse() {
            Ok(status_code) => Some(status_code),
            Err(_) => None
        };
    }

    None
}

pub async fn build_raw_response_bytes(resp: reqwest::Response) -> reqwest::Result<Vec<u8>> {
    let mut raw_response_bytes: Vec<u8> = vec![];

    // response status
    raw_response_bytes.extend_from_slice(format!("{} {}\r\n", http_version_desc(&resp.version()), resp.status()).as_bytes());

    // response headers
    for (name, value) in resp.headers() {
        raw_response_bytes.extend_from_slice(name.as_str().as_bytes());
        raw_response_bytes.extend_from_slice(b": ");
        raw_response_bytes.extend_from_slice(value.as_bytes());
        raw_response_bytes.extend_from_slice(b"\r\n");
    }
    raw_response_bytes.extend_from_slice(b"\r\n");

    // response body
    raw_response_bytes.extend_from_slice(resp.bytes().await?.as_ref());

    Ok(raw_response_bytes)
}

pub fn build_404_response() -> Vec<u8> {
    let mut raw_response_bytes: Vec<u8> = vec![];

    raw_response_bytes.extend_from_slice(b"HTTP/1.1 404 Not Found\r\ncontent-type: text/html; charset=utf-8\r\n\r\n");
    raw_response_bytes.extend_from_slice(b"<html><head><title>404 Not Found</title></head>");
    raw_response_bytes.extend_from_slice(b"<body><center><h1>404 Not Found</h1></center></body></html>");

    raw_response_bytes
}

pub fn build_502_response() -> Vec<u8> {
    let mut raw_response_bytes: Vec<u8> = vec![];

    raw_response_bytes.extend_from_slice(b"HTTP/1.1 502 Bad Gateway\r\ncontent-type: text/html; charset=utf-8\r\n\r\n");
    raw_response_bytes.extend_from_slice(b"<html><head><title>502 Bad Gateway</title></head>");
    raw_response_bytes.extend_from_slice(b"<body><center><h1>502 Bad Gateway</h1></center></body></html>");

    raw_response_bytes
}

///
/// Build local file response
/// 
/// # Notes
/// Just read the entire file directly
pub fn build_local_file_response(local_file_uri: &str, content_type: &str) -> Vec<u8> {
    if local_file_uri.len() == 0 {
        return build_404_response();
    }

    //
    // read entire file
    //

    let mut file_path = match super::request::get_uri_path(local_file_uri) {
        Some(fp) => fp,
        None => {
            return build_404_response();
        }
    };

    // decode urlencoding str
    file_path = match urlencoding::decode(&file_path) {
        Ok(c) => c.to_string(),
        Err(_) => {
            return build_404_response();
        }
    };

    if cfg!(target_os = "windows") {
        file_path = file_path[1..].to_string().replace("/", "\\")
    }

    let file = std::path::Path::new(&file_path);
    if !file.exists() {
        return build_404_response();
    }

    let file_buf = match std::fs::read(file) {
        Ok(b) => b,
        Err(_) => {
            return build_404_response();
        }
    };

    //
    // build response
    //

    let mut raw_response_bytes: Vec<u8> = vec![];
    raw_response_bytes.extend_from_slice(
        format!("HTTP/1.1 202 OK\r\ncontent-length: {}\r\ncontent-type: {}\r\n\r\n", file_buf.len(), content_type).
        as_bytes());
    raw_response_bytes.extend_from_slice(file_buf.as_ref());
    raw_response_bytes
}