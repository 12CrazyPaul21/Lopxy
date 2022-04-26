use async_std::io::ReadExt;
use async_std::net::TcpStream;

///
/// collect tcp stream in Vec<u8> buffer until read size < buffer size
/// 
pub async fn collect_tcp_stream_buffer(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    const BUFFER_SIZE: usize = 1024;

    let mut full_buffer: Vec<u8> = vec![];

    loop {
        let mut buf = vec![0; BUFFER_SIZE];

        match stream.read(&mut buf).await {
            Ok(0) => {
                break;
            }

            Ok(read_byte_count) => {
                full_buffer.extend_from_slice(&mut buf[..read_byte_count]);

                if read_byte_count < BUFFER_SIZE {
                    break;
                }

                continue;
            }

            Err(ref err) if err.kind() == std::io::ErrorKind::Interrupted => {
                continue;
            }

            Err(err) => {
                return Err(err);
            }
        }
    }

    Ok(full_buffer)
}