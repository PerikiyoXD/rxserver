//! Named pipe transport implementation (Windows)
//!
//! Provides Windows named pipe transport for X11 connections.

#[cfg(windows)]
use super::traits::{
    ConnectionMetadata, Endpoint, Transport, TransportConfig, TransportError, TransportEvent,
    TransportStatistics, TransportType,
};
#[cfg(windows)]
use async_trait::async_trait;
#[cfg(windows)]
use std::collections::HashMap;
#[cfg(windows)]
use std::sync::Arc;
#[cfg(windows)]
use tokio::sync::{Mutex, RwLock, mpsc};
#[cfg(windows)]
use tracing::{debug, error, info, warn};

#[cfg(windows)]
/// Named pipe transport implementation
#[derive(Debug)]
pub struct NamedPipeTransport {
    /// Transport configuration
    config: TransportConfig,
    /// Event sender for transport events
    event_sender: mpsc::UnboundedSender<TransportEvent>,
    /// Pipe name
    pipe_name: String,
    /// Active connections
    connections: Arc<RwLock<HashMap<crate::network::ConnectionId, NamedPipeConnection>>>,
    /// Transport statistics
    statistics: Arc<Mutex<TransportStatistics>>,
    /// Running state
    is_running: Arc<std::sync::atomic::AtomicBool>,
    /// Connection ID counter
    next_connection_id: Arc<std::sync::atomic::AtomicU32>,
}

#[cfg(windows)]
/// Named pipe connection wrapper
#[derive(Debug)]
struct NamedPipeConnection {
    /// Connection ID
    id: crate::network::ConnectionId,
    /// Connection metadata
    metadata: ConnectionMetadata,
}

#[cfg(windows)]
impl NamedPipeTransport {
    /// Create a new named pipe transport
    pub async fn new(
        config: TransportConfig,
        event_sender: mpsc::UnboundedSender<TransportEvent>,
    ) -> Result<Self, TransportError> {
        debug!(
            "Creating named pipe transport with name: {}",
            config.address
        );

        // Ensure pipe name has correct format
        let pipe_name = if config.address.starts_with(r"\\.\pipe\") {
            config.address.clone()
        } else {
            format!(r"\\.\pipe\{}", config.address)
        };

        Ok(Self {
            config,
            event_sender,
            pipe_name,
            connections: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(Mutex::new(TransportStatistics::default())),
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            next_connection_id: Arc::new(std::sync::atomic::AtomicU32::new(1)),
        })
    }
}

#[cfg(windows)]
#[async_trait]
impl Transport for NamedPipeTransport {
    fn transport_type(&self) -> TransportType {
        TransportType::NamedPipe
    }

    async fn start(&mut self) -> Result<(), TransportError> {
        if self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        info!("Starting named pipe transport on {}", self.pipe_name);

        // TODO: Implement Windows named pipe server
        // This would use winapi or windows-rs crate to create named pipe server
        // For now, return an error indicating this is not yet implemented

        self.is_running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        warn!("Named pipe transport is not yet fully implemented");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), TransportError> {
        if !self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        info!("Stopping named pipe transport");

        self.is_running
            .store(false, std::sync::atomic::Ordering::SeqCst);

        // Close all connections
        let connection_ids: Vec<_> = {
            let connections = self.connections.read().await;
            connections.keys().cloned().collect()
        };

        for connection_id in connection_ids {
            let _ = self.close_connection(connection_id).await;
        }

        info!("Named pipe transport stopped");
        Ok(())
    }

    async fn send_data(
        &mut self,
        _connection_id: crate::network::ConnectionId,
        _data: &[u8],
    ) -> Result<usize, TransportError> {
        // TODO: Implement data sending for named pipes
        Err(TransportError::NotSupported(
            "Named pipe data sending not yet implemented".to_string(),
        ))
    }

    async fn close_connection(
        &mut self,
        connection_id: crate::network::ConnectionId,
    ) -> Result<(), TransportError> {
        let mut connections = self.connections.write().await;

        if let Some(_connection) = connections.remove(&connection_id) {
            debug!("Closed named pipe connection {}", connection_id);

            // Update statistics
            let mut stats = self.statistics.lock().await;
            stats.active_connections = stats.active_connections.saturating_sub(1);

            // Send event
            let event = TransportEvent::ConnectionClosed {
                connection_id,
                reason: "Connection closed by server".to_string(),
            };

            if let Err(e) = self.event_sender.send(event) {
                warn!("Failed to send connection closed event: {}", e);
            }

            Ok(())
        } else {
            Err(TransportError::ConnectionFailed(format!(
                "Connection {} not found",
                connection_id
            )))
        }
    }

    fn get_connection_metadata(
        &self,
        _connection_id: crate::network::ConnectionId,
    ) -> Option<&ConnectionMetadata> {
        None
    }

    fn get_active_connections(&self) -> Vec<crate::network::ConnectionId> {
        Vec::new()
    }

    fn is_running(&self) -> bool {
        self.is_running.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn get_statistics(&self) -> TransportStatistics {
        TransportStatistics::default()
    }
}

#[cfg(not(windows))]
/// Stub for non-Windows platforms
pub struct NamedPipeTransport;

#[cfg(not(windows))]
impl NamedPipeTransport {
    pub async fn new(
        _config: super::traits::TransportConfig,
        _event_sender: tokio::sync::mpsc::UnboundedSender<super::traits::TransportEvent>,
    ) -> Result<Self, super::traits::TransportError> {
        Err(super::traits::TransportError::NotSupported(
            "Named pipes are only available on Windows".to_string(),
        ))
    }
}
