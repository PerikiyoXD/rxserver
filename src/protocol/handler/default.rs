//! Default Protocol Handler Implementation
//!
//! This module provides a default implementation of the ProtocolHandler trait.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, error, info};

use crate::{
    plugins::PluginRegistry,
    protocol::{ClientId, Opcode, ProtocolHandler, Request, Response},
    ServerError, ServerResult,
};

/// Default implementation of ProtocolHandler
#[derive(Clone)]
pub struct DefaultProtocolHandler {
    plugins: Arc<PluginRegistry>,
}

impl DefaultProtocolHandler {
    /// Create a new DefaultProtocolHandler
    pub fn new(plugins: Arc<PluginRegistry>) -> ServerResult<Self> {
        Ok(Self { plugins })
    }
}

#[async_trait]
impl ProtocolHandler for DefaultProtocolHandler {
    async fn handle_client(&mut self, stream: &mut tokio::net::TcpStream) -> ServerResult<()> {
        debug!("Starting X11 protocol handshake");
        let mut buffer = [0u8; 1024];
        match stream.read(&mut buffer).await {
            Ok(bytes_read) => {
                debug!("Received {} bytes from client", bytes_read);
                let response = b"Connection accepted";
                if let Err(e) = stream.write_all(response).await {
                    error!("Failed to send response to client: {}", e);
                    return Err(ServerError::NetworkError(format!("Write failed: {}", e)));
                }
                info!("Client handshake completed successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to read from client: {}", e);
                Err(ServerError::NetworkError(format!("Read failed: {}", e)))
            }
        }
    }

    async fn handle_request(
        &mut self,
        client_id: ClientId,
        request: Request,
    ) -> ServerResult<Option<Response>> {
        debug!(
            "Handling request from client {}: {:?}",
            client_id,
            request.opcode()
        );
        // Route request based on opcode
        match request.opcode() {
            Opcode::CreateWindow => {
                // Handle CreateWindow request
                // TODO: Parse the request properly to extract window parameters
                // For now, create a basic window with default parameters
                // Get the window plugin to handle window creation
                if let Some(_plugin) = self.plugins.get_plugin("WindowManager") {
                    // Since we can't downcast the plugin easily, we'll implement basic functionality here
                    // In a full implementation, this would parse the request data and call the appropriate window methods

                    // Placeholder implementation
                    Ok(Some(Response::new(1, vec![0; 32]))) // Success response
                } else {
                    Err(ServerError::PluginError(
                        "WindowManager plugin not found".to_string(),
                    ))
                }
            }
            Opcode::DestroyWindow => {
                // Handle DestroyWindow request
                // TODO: Parse the request to extract window ID

                if let Some(_plugin) = self.plugins.get_plugin("WindowManager") {
                    // Placeholder implementation
                    Ok(Some(Response::new(1, vec![0; 32]))) // Success response
                } else {
                    Err(ServerError::PluginError(
                        "WindowManager plugin not found".to_string(),
                    ))
                }
            }
            _ => {
                // Unsupported opcode
                Err(ServerError::ProtocolError(
                    crate::protocol::ProtocolError::UnimplementedOpcode(request.opcode()),
                ))
            }
        }
    }

    fn supported_opcodes(&self) -> &[Opcode] {
        &[
            Opcode::CreateWindow,
            Opcode::DestroyWindow,
            Opcode::MapWindow,
            Opcode::UnmapWindow,
        ]
    }
}
