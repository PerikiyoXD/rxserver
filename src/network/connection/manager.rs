//! Connection manager implementation
//!
//! Manages active connections, their lifecycle, and provides connection-related services.

use crate::network::ConnectionId;
use crate::network::transport::{TransportEvent, TransportType};
use crate::types::NetworkError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};

/// Connection manager configuration
#[derive(Debug, Clone)]
pub struct ConnectionManagerConfig {
    /// Maximum number of simultaneous connections
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Idle timeout in seconds
    pub idle_timeout: u64,
    /// Enable connection pooling
    pub enable_pooling: bool,
    /// Authentication required
    pub require_authentication: bool,
}

impl Default for ConnectionManagerConfig {
    fn default() -> Self {
        Self {
            max_connections: 256,
            connection_timeout: 300,
            idle_timeout: 600,
            enable_pooling: true,
            require_authentication: true,
        }
    }
}

/// Connection state information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Connection ID
    pub id: ConnectionId,
    /// Transport type
    pub transport_type: TransportType,
    /// Connection state
    pub state: ConnectionState,
    /// Authenticated flag
    pub authenticated: bool,
    /// Client information (if available)
    pub client_info: Option<ClientInfo>,
    /// Connection statistics
    pub stats: ConnectionStats,
    /// Connection established timestamp
    pub established_at: std::time::SystemTime,
    /// Last activity timestamp
    pub last_activity: std::time::SystemTime,
}

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// Connection is being established
    Connecting,
    /// Connection established, waiting for authentication
    PendingAuth,
    /// Connection authenticated and active
    Active,
    /// Connection is idle
    Idle,
    /// Connection is being closed
    Closing,
    /// Connection closed
    Closed,
}

/// Client information
#[derive(Debug, Clone)]
pub struct ClientInfo {
    /// Client name/identifier
    pub name: String,
    /// Client version
    pub version: Option<String>,
    /// Protocol version requested
    pub protocol_version: (u16, u16),
    /// Client capabilities
    pub capabilities: Vec<String>,
}

/// Connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    /// Bytes received
    pub bytes_received: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Requests processed
    pub requests_processed: u64,
    /// Errors encountered
    pub errors: u64,
    /// Average response time in microseconds
    pub avg_response_time_us: u64,
}

impl Default for ConnectionStats {
    fn default() -> Self {
        Self {
            bytes_received: 0,
            bytes_sent: 0,
            requests_processed: 0,
            errors: 0,
            avg_response_time_us: 0,
        }
    }
}

/// Connection manager
#[derive(Debug)]
pub struct ConnectionManager {
    /// Configuration
    config: ConnectionManagerConfig,
    /// Active connections
    connections: Arc<RwLock<HashMap<ConnectionId, ConnectionInfo>>>,
    /// Transport event receiver
    transport_rx: mpsc::UnboundedReceiver<TransportEvent>,
    /// Connection event sender
    connection_tx: mpsc::UnboundedSender<ConnectionEvent>,
    /// Running state
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

/// Connection events
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    /// New connection established
    ConnectionEstablished {
        connection_id: ConnectionId,
        transport_type: TransportType,
    },
    /// Connection authenticated
    ConnectionAuthenticated {
        connection_id: ConnectionId,
        client_info: ClientInfo,
    },
    /// Connection state changed
    StateChanged {
        connection_id: ConnectionId,
        old_state: ConnectionState,
        new_state: ConnectionState,
    },
    /// Data received from connection
    DataReceived {
        connection_id: ConnectionId,
        data: Vec<u8>,
    },
    /// Connection error occurred
    ConnectionError {
        connection_id: ConnectionId,
        error: NetworkError,
    },
    /// Connection closed
    ConnectionClosed {
        connection_id: ConnectionId,
        reason: String,
    },
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(
        config: ConnectionManagerConfig,
        transport_rx: mpsc::UnboundedReceiver<TransportEvent>,
    ) -> (Self, mpsc::UnboundedReceiver<ConnectionEvent>) {
        let (connection_tx, connection_rx) = mpsc::unbounded_channel();

        let manager = Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            transport_rx,
            connection_tx,
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };

        (manager, connection_rx)
    }

    /// Start the connection manager
    pub async fn start(&mut self) -> Result<(), NetworkError> {
        if self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        info!("Starting connection manager");
        self.is_running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        // Start event processing loop
        self.run_event_loop().await;

        Ok(())
    }

    /// Stop the connection manager
    pub async fn stop(&mut self) -> Result<(), NetworkError> {
        if !self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        info!("Stopping connection manager");
        self.is_running
            .store(false, std::sync::atomic::Ordering::SeqCst);

        // Close all connections
        let connection_ids: Vec<ConnectionId> = {
            let connections = self.connections.read().await;
            connections.keys().cloned().collect()
        };

        for connection_id in connection_ids {
            self.close_connection(connection_id, "Server shutdown")
                .await?;
        }

        info!("Connection manager stopped");
        Ok(())
    }
    /// Process transport events
    async fn run_event_loop(&mut self) {
        let connections = self.connections.clone();
        let connection_tx = self.connection_tx.clone();
        let is_running = self.is_running.clone();
        let config = self.config.clone();

        // Move transport_rx out of self to pass to the spawned task
        let mut transport_rx = std::mem::replace(
            &mut self.transport_rx,
            mpsc::unbounded_channel().1, // Replace with dummy receiver
        );

        tokio::spawn(async move {
            while is_running.load(std::sync::atomic::Ordering::SeqCst) {
                // Process transport events with a timeout to periodically check running state
                match tokio::time::timeout(
                    std::time::Duration::from_millis(100),
                    transport_rx.recv(),
                )
                .await
                {
                    Ok(Some(event)) => {
                        Self::handle_transport_event(event, &connections, &connection_tx, &config)
                            .await;
                    }
                    Ok(None) => {
                        // Transport channel closed
                        debug!("Transport event channel closed");
                        break;
                    }
                    Err(_) => {
                        // Timeout, continue loop to check running state
                        continue;
                    }
                }
            }
            debug!("Connection manager event loop stopped");
        });
    }
    /// Handle a single transport event
    async fn handle_transport_event(
        event: TransportEvent,
        connections: &Arc<RwLock<HashMap<ConnectionId, ConnectionInfo>>>,
        connection_tx: &mpsc::UnboundedSender<ConnectionEvent>,
        config: &ConnectionManagerConfig,
    ) {
        match event {
            TransportEvent::ConnectionAccepted {
                connection_id,
                transport_type,
                remote_endpoint: _,
            } => {
                debug!(
                    "New connection accepted: {} via {}",
                    connection_id, transport_type
                );

                // Check connection limit
                {
                    let connections_guard = connections.read().await;
                    if connections_guard.len() >= config.max_connections {
                        warn!(
                            "Connection limit reached, rejecting connection {}",
                            connection_id
                        );
                        // Send connection error event
                        let error_event = ConnectionEvent::ConnectionError {
                            connection_id,
                            error: crate::types::NetworkError::TooManyConnections,
                        };
                        if let Err(e) = connection_tx.send(error_event) {
                            error!("Failed to send connection error event: {}", e);
                        }
                        return;
                    }
                }

                // Create new connection info
                let now = std::time::SystemTime::now();
                let connection_info = ConnectionInfo {
                    id: connection_id,
                    transport_type,
                    state: ConnectionState::Connecting,
                    authenticated: false,
                    client_info: None,
                    stats: ConnectionStats::default(),
                    established_at: now,
                    last_activity: now,
                };

                // Add to connections map
                {
                    let mut connections_guard = connections.write().await;
                    connections_guard.insert(connection_id, connection_info);
                }

                // Send connection established event
                let event = ConnectionEvent::ConnectionEstablished {
                    connection_id,
                    transport_type,
                };
                if let Err(e) = connection_tx.send(event) {
                    warn!("Failed to send connection established event: {}", e);
                }
            }

            TransportEvent::DataReceived {
                connection_id,
                data,
            } => {
                debug!(
                    "Data received on connection {}: {} bytes",
                    connection_id,
                    data.len()
                );

                // Update connection activity and stats
                {
                    let mut connections_guard = connections.write().await;
                    if let Some(connection) = connections_guard.get_mut(&connection_id) {
                        connection.last_activity = std::time::SystemTime::now();
                        connection.stats.bytes_received += data.len() as u64;
                    } else {
                        warn!("Received data for unknown connection: {}", connection_id);
                        return;
                    }
                }

                // Forward data to protocol handler
                let event = ConnectionEvent::DataReceived {
                    connection_id,
                    data,
                };
                if let Err(e) = connection_tx.send(event) {
                    warn!("Failed to send data received event: {}", e);
                }
            }

            TransportEvent::ConnectionClosed {
                connection_id,
                reason,
            } => {
                debug!("Connection {} closed: {}", connection_id, reason);

                // Remove from connections map
                {
                    let mut connections_guard = connections.write().await;
                    if let Some(mut connection) = connections_guard.remove(&connection_id) {
                        connection.state = ConnectionState::Closed;
                    } else {
                        warn!("Tried to close unknown connection: {}", connection_id);
                        return;
                    }
                }

                // Send connection closed event
                let event = ConnectionEvent::ConnectionClosed {
                    connection_id,
                    reason,
                };
                if let Err(e) = connection_tx.send(event) {
                    warn!("Failed to send connection closed event: {}", e);
                }
            }

            TransportEvent::Error {
                connection_id,
                error,
            } => {
                if let Some(connection_id) = connection_id {
                    error!("Transport error on connection {}: {}", connection_id, error);

                    // Update connection error count
                    {
                        let mut connections_guard = connections.write().await;
                        if let Some(connection) = connections_guard.get_mut(&connection_id) {
                            connection.stats.errors += 1;
                            connection.last_activity = std::time::SystemTime::now();
                        }
                    }

                    // Send connection error event
                    let error_event = ConnectionEvent::ConnectionError {
                        connection_id,
                        error: crate::types::NetworkError::Transport(error.to_string()),
                    };
                    if let Err(e) = connection_tx.send(error_event) {
                        error!("Failed to send connection error event: {}", e);
                    }
                } else {
                    error!("General transport error: {}", error);
                }
            }
        }
    }

    /// Handle new connection
    async fn handle_new_connection(
        &self,
        connection_id: ConnectionId,
        transport_type: TransportType,
    ) -> Result<(), NetworkError> {
        // Check connection limit
        {
            let connections = self.connections.read().await;
            if connections.len() >= self.config.max_connections {
                return Err(NetworkError::TooManyConnections);
            }
        }

        let now = std::time::SystemTime::now();
        let connection_info = ConnectionInfo {
            id: connection_id,
            transport_type,
            state: ConnectionState::Connecting,
            authenticated: false,
            client_info: None,
            stats: ConnectionStats::default(),
            established_at: now,
            last_activity: now,
        };

        // Add to connections map
        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id, connection_info);
        }

        debug!("New connection established: {}", connection_id);

        // Send connection event
        let event = ConnectionEvent::ConnectionEstablished {
            connection_id,
            transport_type,
        };

        if let Err(e) = self.connection_tx.send(event) {
            warn!("Failed to send connection established event: {}", e);
        }

        Ok(())
    }

    /// Update connection state
    pub async fn update_connection_state(
        &self,
        connection_id: ConnectionId,
        new_state: ConnectionState,
    ) -> Result<(), NetworkError> {
        let old_state = {
            let mut connections = self.connections.write().await;
            if let Some(connection) = connections.get_mut(&connection_id) {
                let old_state = connection.state.clone();
                connection.state = new_state.clone();
                connection.last_activity = std::time::SystemTime::now();
                old_state
            } else {
                return Err(NetworkError::ConnectionNotFound(connection_id));
            }
        };

        // Send state change event
        let event = ConnectionEvent::StateChanged {
            connection_id,
            old_state,
            new_state,
        };

        if let Err(e) = self.connection_tx.send(event) {
            warn!("Failed to send state changed event: {}", e);
        }

        Ok(())
    }

    /// Mark connection as authenticated
    pub async fn authenticate_connection(
        &self,
        connection_id: ConnectionId,
        client_info: ClientInfo,
    ) -> Result<(), NetworkError> {
        {
            let mut connections = self.connections.write().await;
            if let Some(connection) = connections.get_mut(&connection_id) {
                connection.authenticated = true;
                connection.client_info = Some(client_info.clone());
                connection.state = ConnectionState::Active;
                connection.last_activity = std::time::SystemTime::now();
            } else {
                return Err(NetworkError::ConnectionNotFound(connection_id));
            }
        }

        debug!(
            "Connection {} authenticated: {}",
            connection_id, client_info.name
        );

        // Send authentication event
        let event = ConnectionEvent::ConnectionAuthenticated {
            connection_id,
            client_info,
        };

        if let Err(e) = self.connection_tx.send(event) {
            warn!("Failed to send connection authenticated event: {}", e);
        }

        Ok(())
    }

    /// Close a connection
    pub async fn close_connection(
        &self,
        connection_id: ConnectionId,
        reason: &str,
    ) -> Result<(), NetworkError> {
        {
            let mut connections = self.connections.write().await;
            if let Some(mut connection) = connections.remove(&connection_id) {
                connection.state = ConnectionState::Closed;
            } else {
                return Err(NetworkError::ConnectionNotFound(connection_id));
            }
        }

        debug!("Connection {} closed: {}", connection_id, reason);

        // Send connection closed event
        let event = ConnectionEvent::ConnectionClosed {
            connection_id,
            reason: reason.to_string(),
        };

        if let Err(e) = self.connection_tx.send(event) {
            warn!("Failed to send connection closed event: {}", e);
        }

        Ok(())
    }

    /// Get connection information
    pub async fn get_connection_info(&self, connection_id: ConnectionId) -> Option<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections.get(&connection_id).cloned()
    }

    /// Get all active connection IDs
    pub async fn get_active_connections(&self) -> Vec<ConnectionId> {
        let connections = self.connections.read().await;
        connections.keys().cloned().collect()
    }

    /// Get connection count
    pub async fn get_connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    /// Update connection statistics
    pub async fn update_connection_stats(
        &self,
        connection_id: ConnectionId,
        bytes_received: u64,
        bytes_sent: u64,
        requests_processed: u64,
    ) -> Result<(), NetworkError> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(&connection_id) {
            connection.stats.bytes_received += bytes_received;
            connection.stats.bytes_sent += bytes_sent;
            connection.stats.requests_processed += requests_processed;
            connection.last_activity = std::time::SystemTime::now();
            Ok(())
        } else {
            Err(NetworkError::ConnectionNotFound(connection_id))
        }
    }
}
