use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use tracing::{debug, error, trace};

use crate::protocol::{
    Request, RequestValidator, ResponseSerializer, X11RequestValidator, X11ResponseSerializer,
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
        let mut request = match Request::parse(&buf[..n]) {
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
        let _response = match &request.kind {
            _ => {
                trace!("Request type not yet implemented");
                todo!("Handle other request types")
            }
        };

        // 4. Serialize response
        let _serializer: X11ResponseSerializer = X11ResponseSerializer;
        let reply = match _serializer.serialize(&_response) {
            Ok(Some(data)) => {
                trace!("Serialized response: {} bytes", data.len());
                data
            }
            Ok(None) => {
                trace!("No response needed for this request");
                continue;
            }
            Err(e) => {
                error!("Response serialization error: {:?}", e);
                continue;
            }
        };

        // 5. Write response
        if let Err(e) = stream.write_all(&reply).await {
            error!("Failed to write response to client: {:?}", e);
            break;
        }
        debug!("Successfully sent response to client");
    }

    debug!("Connection handler finished");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::opcodes;
    use tokio::io::AsyncWriteExt;
    use tokio::net::{TcpListener, TcpStream};
    use tracing::info;

    #[tokio::test]
    async fn test_get_geometry_pipeline() {
        // Start a test server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn server handler
        tokio::spawn(async move {
            if let Ok((socket, _)) = listener.accept().await {
                handle_connection(socket).await;
            }
        });

        // Connect as client
        let mut client = TcpStream::connect(addr).await.unwrap();

        // Send a GetGeometry request
        let mut request = vec![0u8; 8];
        request[0] = opcodes::GET_GEOMETRY; // opcode
        request[1] = 0; // unused
        request[2..4].copy_from_slice(&2u16.to_le_bytes()); // length
        request[4..8].copy_from_slice(&0x12345678u32.to_le_bytes()); // drawable

        client.write_all(&request).await.unwrap();

        // Read response
        let mut response = vec![0u8; 32];
        let n = client.read_exact(&mut response).await.unwrap();

        assert_eq!(n, 32); // GetGeometry response is 32 bytes
        assert_eq!(response[0], 1); // response_type (reply)
        assert_eq!(response[1], 24); // depth

        // Verify sequence number
        let seq = u16::from_le_bytes([response[2], response[3]]);
        assert_eq!(seq, 1);

        info!("GetGeometry test passed! Response: {:?}", &response[..8]);
    }
}
