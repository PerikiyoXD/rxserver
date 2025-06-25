use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::protocol::{Request, RequestValidator, Response, X11RequestValidator};

pub async fn handle_connection(mut stream: TcpStream) {
    let mut buf = vec![0u8; 4096];

    loop {
        let n = match stream.read(&mut buf).await {
            Ok(0) => break, // EOF
            Ok(n) => n,
            Err(e) => {
                eprintln!("IO error: {:?}", e);
                break;
            }
        };

        // 1. Parse request
        let request = match Request::parse(&buf[..n]) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Request parse error: {:?}", e);
                continue;
            }
        };

        // 2. Validate
        if let Err(e) = X11RequestValidator::validate(&request) {
            eprintln!("Validation failed: {:?}", e);
            continue;
        }

        // 3. Execute request (stub)
        let _response = Response::default();

        // 4. Serialize response (stub)
        // Note: You'd match on response.kind and call the right serializer
        let reply = vec![]; // Replace with actual serialization

        // 5. Write response
        if let Err(e) = stream.write_all(&reply).await {
            eprintln!("Write error: {:?}", e);
            break;
        }
    }
}
