//! X11 Connection Handler
//!
//! This module handles the X11 protocol-specific connection logic,
//! including handshake, request parsing, and response handling with
//! comprehensive logging and performance monitoring.

use crate::protocol::{ConnectionHandler, RequestParser, ResponseBuilder};
use crate::server::{ClientManager, RequestHandler};
use crate::logging::{with_timing_async, ProtocolLogger};
use crate::Result;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error, info, warn};

/// Handles X11 protocol communication for a single client connection
pub struct X11ConnectionHandler {
    client_manager: Arc<ClientManager>,
    request_handler: Arc<RequestHandler>,
}

impl X11ConnectionHandler {
    pub fn new(
        client_manager: Arc<ClientManager>,
        request_handler: Arc<RequestHandler>,
    ) -> Self {
        Self {
            client_manager,
            request_handler,
        }
    }    /// Handle a complete client connection from handshake to disconnect
    pub async fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        info!("Handling new X11 client connection");
        
        let start_time = std::time::Instant::now();
        let mut client_id = 0;        // Step 1: Handle X11 connection setup handshake
        let handshake_result = with_timing_async("client_handshake", || async {
            self.handle_handshake(&mut stream).await
        }).await;
        
        match handshake_result {
            Ok(id) => {
                client_id = id;
                info!("Client {} handshake completed successfully", client_id);
                ProtocolLogger::connection_established(client_id);
            },
            Err(e) => {
                error!("Handshake failed: {}", e);
                ProtocolLogger::connection_failed(&e);
                return Ok(()); // Don't propagate handshake errors
            }
        };

        // Step 2: Handle regular X11 requests
        let request_result = with_timing_async("client_request_handling", || async {
            self.handle_requests(&mut stream, client_id).await
        }).await;
        
        if let Err(e) = request_result {
            warn!("Request handling error for client {}: {}", client_id, e);
            ProtocolLogger::request_error(client_id, &e);
        }

        // Step 3: Clean up client registration
        if client_id != 0 {
            let _ = self.client_manager.unregister_client(client_id).await;
            let total_duration = start_time.elapsed();
            info!("Client {} connection closed after {:?}", client_id, total_duration);
            ProtocolLogger::connection_closed(client_id, total_duration);
        }

        Ok(())
    }

    /// Handle the X11 connection setup handshake
    async fn handle_handshake(&self, stream: &mut TcpStream) -> Result<u32> {
        let connection_handler = ConnectionHandler::new();
        let mut setup_buffer = Vec::new();

        // Read and process the X11 connection setup request
        loop {
            let mut buffer = [0u8; 1024];
            
            match stream.read(&mut buffer).await {
                Ok(0) => {
                    warn!("Client disconnected during handshake");
                    return Ok(0);
                }
                Ok(n) => {
                    setup_buffer.extend_from_slice(&buffer[..n]);
                    
                    // Try to parse connection setup request (minimum 12 bytes)
                    if setup_buffer.len() >= 12 {
                        // Calculate expected total size based on auth lengths
                        if setup_buffer.len() >= 8 {
                            let auth_name_len = u16::from_le_bytes([setup_buffer[6], setup_buffer[7]]) as usize;
                            let auth_data_len = u16::from_le_bytes([setup_buffer[8], setup_buffer[9]]) as usize;
                            
                            // Calculate padded lengths (X11 protocol pads to 4-byte boundaries)
                            let padded_name_len = (auth_name_len + 3) & !3;
                            let padded_data_len = (auth_data_len + 3) & !3;
                            let total_expected = 12 + padded_name_len + padded_data_len;
                            
                            if setup_buffer.len() >= total_expected {
                                // We have complete setup request, parse it
                                match ConnectionHandler::parse_setup_request(&setup_buffer) {
                                    Ok(setup_request) => {
                                        info!("Processing X11 connection setup from client");
                                        debug!("Client requests protocol {}.{}", 
                                               setup_request.protocol_major_version,
                                               setup_request.protocol_minor_version);
                                        
                                        // Handle the connection setup
                                        match connection_handler.handle_connection_setup(setup_request).await {
                                            Ok(setup_response) => {
                                                // Serialize and send response
                                                match connection_handler.serialize_setup_response(&setup_response) {
                                                    Ok(response_bytes) => {
                                                        if let Err(e) = stream.write_all(&response_bytes).await {
                                                            error!("Failed to send connection setup response: {}", e);
                                                            return Ok(0);
                                                        }
                                                        
                                                        if setup_response.status == crate::protocol::ConnectionSetupStatus::Success {
                                                            info!("X11 connection setup successful");
                                                            
                                                            // Register the client after successful handshake
                                                            let client_id = self.client_manager
                                                                .register_client("authenticated".to_string(), None)
                                                                .await?;
                                                            debug!("Registered authenticated client with ID: {}", client_id);
                                                            return Ok(client_id);
                                                        } else {
                                                            warn!("X11 connection setup failed: {:?}", setup_response.status);
                                                            return Ok(0);
                                                        }
                                                    }
                                                    Err(e) => {
                                                        error!("Failed to serialize connection setup response: {}", e);
                                                        return Ok(0);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                error!("Failed to handle connection setup: {}", e);
                                                return Ok(0);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to parse connection setup request: {}", e);
                                        return Ok(0);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read during handshake: {}", e);
                    return Ok(0);
                }
            }
        }
    }

    /// Handle regular X11 requests after successful handshake
    async fn handle_requests(&self, stream: &mut TcpStream, client_id: u32) -> Result<()> {
        info!("Starting X11 request processing for client {}", client_id);
        
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
                                        debug!("Parsed request from client {:?}: {:?}",
                                                client_id, request
                                            );

                                        // Handle the request using existing handler
                                        match self.request_handler.handle_request(client_id, request).await {
                                            Ok(Some(response)) => {
                                                // Serialize response using existing serializer
                                                let response_bytes = ResponseBuilder::serialize(&response)?;

                                                if let Err(e) = stream.write_all(&response_bytes).await {
                                                    error!("Failed to write response to client {}: {}", client_id, e);
                                                    return Ok(());
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
