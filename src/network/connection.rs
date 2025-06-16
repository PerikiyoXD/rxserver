//! Connection management for the RX X11 Server
//!
//! This module handles individual client connections and their lifecycle.

use crate::core::{PerformanceConfig, ServerError, ServerResult};
use crate::protocol::ProtocolHandler;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

/// Manages client connections to the X11 server
#[derive(Clone)]
pub struct ConnectionManager {
    performance_config: PerformanceConfig,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(performance_config: &PerformanceConfig) -> ServerResult<Self> {
        Ok(Self {
            performance_config: performance_config.clone(),
        })
    }
    /// Handle a new client connection
    pub async fn handle_connection(
        &self,
        mut stream: TcpStream,
        protocol_handler: Arc<Mutex<dyn ProtocolHandler + Send + Sync + 'static>>,
    ) -> ServerResult<()> {
        let peer_addr = stream
            .peer_addr()
            .map_err(|e| ServerError::NetworkError(format!("Failed to get peer address: {}", e)))?;

        info!("Handling connection from {}", peer_addr);

        // Set TCP options based on performance configuration
        if let Err(e) = self.configure_tcp_stream(&stream) {
            error!("Failed to configure TCP stream: {}", e);
        }

        // Handle the X11 protocol handshake and messages
        let mut handler = protocol_handler.lock().await;
        match handler.handle_client(&mut stream).await {
            Ok(_) => {
                info!("Client {} disconnected gracefully", peer_addr);
            }
            Err(e) => {
                error!("Error handling client {}: {}", peer_addr, e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Configure TCP stream with performance settings
    fn configure_tcp_stream(&self, stream: &TcpStream) -> ServerResult<()> {
        // Set TCP_NODELAY to reduce latency for X11 protocol
        stream
            .set_nodelay(true)
            .map_err(|e| ServerError::NetworkError(format!("Failed to set TCP_NODELAY: {}", e)))?;

        debug!("TCP stream configured with optimal settings");
        Ok(())
    }
}
