//! Client connection management
//!
//! This module handles individual client connections, including authentication,
//! request processing, and connection lifecycle.

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::RwLock;

use crate::protocol::{Request, RequestParser, Response, ResponseBuilder};
use crate::{todo_critical, Error, Result};

/// Represents a client connection to the X server
pub struct ClientConnection {
    /// Unique client identifier
    pub id: u32,
    /// TCP stream for communication
    stream: Option<TcpStream>,
    /// Client authentication state
    authenticated: bool,
    /// Client protocol version
    protocol_major: u16,
    protocol_minor: u16,
    /// Connection state
    connected: Arc<RwLock<bool>>,
}

impl ClientConnection {
    /// Create a new client connection
    pub fn new(id: u32, stream: TcpStream) -> Self {
        Self {
            id,
            stream: Some(stream),
            authenticated: false,
            protocol_major: 0,
            protocol_minor: 0,
            connected: Arc::new(RwLock::new(true)),
        }
    }

    /// Handle the connection setup process
    pub async fn setup(&mut self) -> Result<()> {
        log::debug!("Setting up client connection {}", self.id);

        // TODO: Implement X11 connection setup protocol
        // 1. Read connection setup request
        // 2. Validate authentication
        // 3. Send connection setup response

        self.authenticated = true;
        log::info!("Client {} authenticated successfully", self.id);

        Ok(())
    }

    /// Read and process requests from the client
    pub async fn process_requests(&mut self) -> Result<()> {
        log::debug!("Starting request processing for client {}", self.id);

        while *self.connected.read().await {
            match self.read_request().await {
                Ok(Some(request)) => {
                    log::debug!("Received request from client {}: {:?}", self.id, request);

                    // TODO: Process request and send response
                    let response = self.handle_request(request).await?;
                    if let Some(resp) = response {
                        self.send_response(resp).await?;
                    }
                }
                Ok(None) => {
                    // No more data, client disconnected
                    log::info!("Client {} disconnected", self.id);
                    break;
                }
                Err(e) => {
                    log::error!("Error reading request from client {}: {}", self.id, e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Read a single request from the client
    async fn read_request(&mut self) -> Result<Option<Request>> {
        if let Some(ref mut stream) = self.stream {
            let mut header = [0u8; 4];

            match stream.read_exact(&mut header).await {
                Ok(_) => {
                    let length = u16::from_ne_bytes([header[2], header[3]]) as usize * 4;

                    if length < 4 {
                        return Err(Error::Protocol("Invalid request length".to_string()));
                    }

                    let mut data = vec![0u8; length];
                    data[..4].copy_from_slice(&header);

                    if length > 4 {
                        stream.read_exact(&mut data[4..]).await?;
                    }

                    let request = RequestParser::parse(&data)?;
                    Ok(Some(request))
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(None),
                Err(e) => Err(Error::Io(e)),
            }
        } else {
            Ok(None)
        }
    }

    /// Handle a specific request
    async fn handle_request(&mut self, request: Request) -> Result<Option<Response>> {
        todo_critical!(
            "client_connection",
            "Request handling not implemented for request: {:?}",
            request
        );
    }

    /// Send a response to the client
    pub async fn send_response(&mut self, response: Response) -> Result<()> {
        if let Some(ref mut stream) = self.stream {
            let data = ResponseBuilder::serialize(&response)?;
            stream.write_all(&data).await?;
            stream.flush().await?;
            log::debug!("Sent response to client {}", self.id);
        }
        Ok(())
    }

    /// Send an event to the client
    pub async fn send_event(&mut self, event_data: &[u8]) -> Result<()> {
        if let Some(ref mut stream) = self.stream {
            stream.write_all(event_data).await?;
            stream.flush().await?;
            log::debug!("Sent event to client {}", self.id);
        }
        Ok(())
    }

    /// Check if the connection is still active
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Close the connection
    pub async fn close(&mut self) -> Result<()> {
        log::debug!("Closing connection for client {}", self.id);

        {
            let mut connected = self.connected.write().await;
            *connected = false;
        }

        if let Some(mut stream) = self.stream.take() {
            stream.shutdown().await?;
        }

        Ok(())
    }
}
