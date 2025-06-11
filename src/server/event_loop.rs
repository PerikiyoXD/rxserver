//! Event Loop
//!
//! This module contains the main server event loop.

use crate::server::ServerEvent;
use crate::server::{ClientManager, ConnectionManager, RequestHandler};
use crate::{todo_critical, todo_high, todo_medium, Result};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Main server event loop
pub struct EventLoop {
    connection_manager: Arc<ConnectionManager>,
    client_manager: Arc<ClientManager>,
    request_handler: Arc<RequestHandler>,
    event_sender: broadcast::Sender<ServerEvent>,
}

impl EventLoop {
    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        client_manager: Arc<ClientManager>,
        request_handler: Arc<RequestHandler>,
        event_sender: broadcast::Sender<ServerEvent>,
    ) -> Self {
        Self {
            connection_manager,
            client_manager,
            request_handler,
            event_sender,
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting main server event loop");

        // Set up Unix domain socket for X11 connections
        let socket_path = format!("/tmp/.X11-unix/X0"); // TODO: Make configurable

        // For now, create a simple TCP listener as a placeholder
        let listener = tokio::net::TcpListener::bind("127.0.0.1:6000")
            .await
            .map_err(|e| crate::Error::Io(e))?;

        info!("X server listening on 127.0.0.1:6000 (TCP fallback)");
        info!("X server would normally listen on {}", socket_path);

        loop {
            tokio::select! {
                // Accept new connections
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            info!("New client connection from: {}", addr);

                            // Handle connection in background task
                            let connection_manager = self.connection_manager.clone();
                            let client_manager = self.client_manager.clone();
                            let request_handler = self.request_handler.clone();
                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_client_connection(
                                    stream,
                                    connection_manager,
                                    client_manager,
                                    request_handler,
                                ).await {
                                    warn!("Client connection error: {}", e);
                                }
                            });                        }
                        Err(e) => {
                            warn!("Failed to accept connection: {}", e);
                        }
                    }
                }

                // Handle shutdown signal
                _ = tokio::signal::ctrl_c() => {
                    info!("Received shutdown signal");
                    break;
                }
            }
        }

        info!("Event loop shutting down");
        Ok(())
    }
    /// Handle a single client connection
    async fn handle_client_connection(
        mut stream: tokio::net::TcpStream,
        _connection_manager: Arc<ConnectionManager>,
        client_manager: Arc<ClientManager>,
        request_handler: Arc<RequestHandler>,
    ) -> Result<()> {
        use crate::protocol::{RequestParser, ResponseBuilder};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tracing::{debug, error, info, warn};

        info!("Handling new client connection");

        // Register the client
        let client_id = client_manager
            .register_client("unknown".to_string(), None)
            .await?;

        debug!("Registered client with ID: {}", client_id);

        let mut buffer = [0u8; 4096];
        let mut connection_buffer = Vec::new();

        loop {
            tokio::select! {
                // Read from client
                result = stream.read(&mut buffer) => {
                    match result {
                        Ok(0) => {
                            info!("Client {} disconnected", client_id);
                            break;
                        }
                        Ok(n) => {
                            debug!("Received {} bytes from client {}", n, client_id);

                            // Add to connection buffer
                            connection_buffer.extend_from_slice(&buffer[..n]);

                            // Try to parse complete requests from buffer
                            while let Some(request_data) = Self::extract_complete_request(&mut connection_buffer)? {
                                // Parse the X11 request using existing parser
                                match RequestParser::parse(&request_data) {
                                    Ok(request) => {
                                        debug!("Parsed request from client {}: {:?}", client_id, request);

                                        // Handle the request using existing handler
                                        match request_handler.handle_request(client_id, request).await {
                                            Ok(Some(response)) => {
                                                // Serialize response using existing serializer
                                                let response_bytes = ResponseBuilder::serialize(&response)?;

                                                if let Err(e) = stream.write_all(&response_bytes).await {
                                                    error!("Failed to write response to client {}: {}", client_id, e);
                                                    break;
                                                }
                                                debug!("Sent {} byte response to client {}", response_bytes.len(), client_id);
                                            }
                                            Ok(None) => {
                                                debug!("No response needed for request from client {}", client_id);
                                            }
                                            Err(e) => {
                                                error!("Failed to handle request from client {}: {}", client_id, e);
                                                // Continue processing other requests
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse request from client {}: {}", client_id, e);
                                        // Send error response or continue
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to read from client {}: {}", client_id, e);
                            break;
                        }
                    }
                }

                // Handle timeout
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(300)) => {
                    info!("Client {} connection timeout", client_id);
                    break;
                }
            }
        }

        // Unregister client on disconnect
        let _ = client_manager.unregister_client(client_id).await;
        info!("Client {} connection handler terminating", client_id);
        Ok(())
    }

    /// Extract a complete X11 request from the buffer
    fn extract_complete_request(buffer: &mut Vec<u8>) -> Result<Option<Vec<u8>>> {
        if buffer.len() < 4 {
            return Ok(None); // Not enough data for header
        }

        // Read the length field from the request header
        let length = u16::from_ne_bytes([buffer[2], buffer[3]]) as usize * 4;

        if length < 4 {
            return Err(crate::Error::Protocol("Invalid request length".to_string()));
        }

        if buffer.len() >= length {
            // We have a complete request
            let request_data = buffer.drain(..length).collect();
            Ok(Some(request_data))
        } else {
            // Need more data
            Ok(None)
        }
    }
}
