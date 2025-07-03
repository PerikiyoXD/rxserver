use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use tracing::{debug, error, trace};

use crate::protocol::{
    RequestParser, RequestValidator, X11RequestParser, X11RequestValidator,
};

pub async fn handle_connection(mut stream: TcpStream) {
    let mut buf = vec![0u8; 4096];
    let mut sequence_number = 1u16;

    debug!("Starting connection handler");
    trace!("Buffer size: {} bytes", buf.len());

    loop {
        let n = match stream.read(&mut buf).await {
            Ok(0) => {
                debug!("Connection closed by client (EOF)");
                break;
            }
            Ok(n) => {
                trace!("Read {} bytes from client", n);
                n
            }
            Err(e) => {
                error!("IO error reading from client: {:?}", e);
                break;
            }
        };

        // 1. Parse request
        let mut request = match X11RequestParser::parse(&buf[..n]) {
            Ok(req) => {
                trace!("Successfully parsed request: {:?}", req.kind);
                req
            }
            Err(e) => {
                error!("Request parse error: {:?}", e);
                trace!("Failed to parse {} bytes of data", n);
                continue;
            }
        };

        // Set sequence number for the request
        request.sequence_number = sequence_number;
        debug!(
            "Processing request with sequence number {}",
            sequence_number
        );
        sequence_number = sequence_number.wrapping_add(1);

        // 2. Validate
        if let Err(e) = X11RequestValidator::validate(&request) {
            error!("Request validation failed: {:?}", e);
            continue;
        }
        trace!("Request validation passed");

        // 3. Execute request and create response
        trace!("Executing request: {:?}", request.kind);
        match &request.kind {
            _ => {
                trace!("Request type not yet implemented");
                continue; // Skip to next iteration instead of panic
            }
        }

    }

    debug!("Connection handler finished");
}
