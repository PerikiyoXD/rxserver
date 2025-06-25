//! Transport trait definitions and common functionality
//!
//! This module defines the core transport traits and types used throughout
//! the network layer.

use async_trait::async_trait;
use std::fmt::Debug;
use tokio::sync::mpsc;

/// Transport layer error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum TransportError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Transport not supported: {0}")]
    NotSupported(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Address in use: {0}")]
    AddressInUse(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

impl From<std::io::Error> for TransportError {
    fn from(err: std::io::Error) -> Self {
        TransportError::Io(err.to_string())
    }
}

/// Transport types supported by the server
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportType {
    /// TCP/IP transport
    Tcp,
    /// Unix domain socket transport
    UnixSocket,
    /// Named pipe transport (Windows)
    NamedPipe,
    /// Shared memory transport
    SharedMemory,
    /// Abstract transport (for testing/mocking)
    Abstract,
}

impl std::fmt::Display for TransportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportType::Tcp => write!(f, "TCP"),
            TransportType::UnixSocket => write!(f, "Unix Socket"),
            TransportType::NamedPipe => write!(f, "Named Pipe"),
            TransportType::SharedMemory => write!(f, "Shared Memory"),
            TransportType::Abstract => write!(f, "Abstract"),
        }
    }
}

/// Transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Type of transport
    pub transport_type: TransportType,
    /// Address to bind/connect to
    pub address: String,
    /// Whether this transport is enabled
    pub enabled: bool,
    /// Transport-specific options
    pub options: std::collections::HashMap<String, String>,
}

/// Transport endpoint information
#[derive(Debug, Clone)]
pub struct Endpoint {
    /// Transport type
    pub transport_type: TransportType,
    /// Address string
    pub address: String,
    /// Whether this is a server (listening) endpoint
    pub is_server: bool,
}

/// Transport connection handle
#[derive(Debug)]
pub struct TransportConnection {
    /// Unique connection identifier
    pub id: crate::network::ConnectionId,
    /// Transport type
    pub transport_type: TransportType,
    /// Local endpoint
    pub local_endpoint: Endpoint,
    /// Remote endpoint (if applicable)
    pub remote_endpoint: Option<Endpoint>,
    /// Connection metadata
    pub metadata: ConnectionMetadata,
}

/// Connection metadata
#[derive(Debug, Clone)]
pub struct ConnectionMetadata {
    /// Connection establishment timestamp
    pub established_at: std::time::SystemTime,
    /// Last activity timestamp
    pub last_activity: std::time::SystemTime,
    /// Bytes received
    pub bytes_received: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Number of messages received
    pub messages_received: u64,
    /// Number of messages sent
    pub messages_sent: u64,
}

impl Default for ConnectionMetadata {
    fn default() -> Self {
        let now = std::time::SystemTime::now();
        Self {
            established_at: now,
            last_activity: now,
            bytes_received: 0,
            bytes_sent: 0,
            messages_received: 0,
            messages_sent: 0,
        }
    }
}

/// Transport events
#[derive(Debug, Clone)]
pub enum TransportEvent {
    /// New connection accepted
    ConnectionAccepted {
        connection_id: crate::network::ConnectionId,
        transport_type: TransportType,
        remote_endpoint: Option<Endpoint>,
    },
    /// Connection closed
    ConnectionClosed {
        connection_id: crate::network::ConnectionId,
        reason: String,
    },
    /// Data received on connection
    DataReceived {
        connection_id: crate::network::ConnectionId,
        data: Vec<u8>,
    },
    /// Error occurred
    Error {
        connection_id: Option<crate::network::ConnectionId>,
        error: TransportError,
    },
}

/// Main transport trait
#[async_trait]
pub trait Transport: Send + Sync + Debug {
    /// Get the transport type
    fn transport_type(&self) -> TransportType;

    /// Start the transport (begin listening for connections)
    async fn start(&mut self) -> Result<(), TransportError>;

    /// Stop the transport
    async fn stop(&mut self) -> Result<(), TransportError>;

    /// Send data on a connection
    async fn send_data(
        &mut self,
        connection_id: crate::network::ConnectionId,
        data: &[u8],
    ) -> Result<usize, TransportError>;

    /// Close a specific connection
    async fn close_connection(
        &mut self,
        connection_id: crate::network::ConnectionId,
    ) -> Result<(), TransportError>;

    /// Get connection metadata
    fn get_connection_metadata(
        &self,
        connection_id: crate::network::ConnectionId,
    ) -> Option<&ConnectionMetadata>;

    /// Get all active connections
    fn get_active_connections(&self) -> Vec<crate::network::ConnectionId>;

    /// Check if transport is running
    fn is_running(&self) -> bool;

    /// Get transport statistics
    fn get_statistics(&self) -> TransportStatistics;
}

/// Transport statistics
#[derive(Debug, Clone)]
pub struct TransportStatistics {
    /// Total connections accepted
    pub connections_accepted: u64,
    /// Currently active connections
    pub active_connections: u32,
    /// Total bytes received
    pub total_bytes_received: u64,
    /// Total bytes sent
    pub total_bytes_sent: u64,
    /// Total messages received
    pub total_messages_received: u64,
    /// Total messages sent
    pub total_messages_sent: u64,
    /// Transport uptime
    pub uptime: std::time::Duration,
    /// Last error (if any)
    pub last_error: Option<String>,
}

impl Default for TransportStatistics {
    fn default() -> Self {
        Self {
            connections_accepted: 0,
            active_connections: 0,
            total_bytes_received: 0,
            total_bytes_sent: 0,
            total_messages_received: 0,
            total_messages_sent: 0,
            uptime: std::time::Duration::default(),
            last_error: None,
        }
    }
}

/// Transport factory for creating transport instances
pub struct TransportFactory;

impl TransportFactory {
    /// Create a transport instance based on configuration
    pub async fn create_transport(
        config: TransportConfig,
        event_sender: mpsc::UnboundedSender<TransportEvent>,
    ) -> Result<Box<dyn Transport>, TransportError> {
        match config.transport_type {
            TransportType::Tcp => {
                let transport = super::tcp::TcpTransport::new(config, event_sender).await?;
                Ok(Box::new(transport))
            }
            TransportType::UnixSocket => {
                #[cfg(unix)]
                {
                    let transport =
                        super::unix_socket::UnixSocketTransport::new(config, event_sender).await?;
                    Ok(Box::new(transport))
                }
                #[cfg(not(unix))]
                {
                    Err(TransportError::NotSupported(
                        "Unix sockets not supported on this platform".to_string(),
                    ))
                }
            }
            #[cfg(windows)]
            TransportType::NamedPipe => {
                let transport =
                    super::named_pipe::NamedPipeTransport::new(config, event_sender).await?;
                Ok(Box::new(transport))
            }
            TransportType::SharedMemory => {
                let transport =
                    super::shared_memory::SharedMemoryTransport::new(config, event_sender).await?;
                Ok(Box::new(transport))
            }
            _ => Err(TransportError::NotSupported(format!(
                "{}",
                config.transport_type
            ))),
        }
    }
}
