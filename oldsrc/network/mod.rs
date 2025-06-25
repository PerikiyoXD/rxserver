//! Network infrastructure for X11 server
//!
//! This module provides comprehensive network capabilities including transport layers,
//! connection management, protocol handling, service discovery, proxying, and monitoring.

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

/// Connection identifier
pub type ConnectionId = u32;

// Core network modules
pub mod connection;
pub mod discovery;
pub mod monitoring;
pub mod protocol;
pub mod proxy;
pub mod transport;

// Re-export commonly used types
pub use connection::{AuthenticationManager, ConnectionManager, ConnectionPool, Session};
pub use discovery::{
    BroadcastDiscovery, DnsSdDiscovery, MdnsDiscovery, ServiceInfo, ServiceRegistry,
    StaticConfigDiscovery,
};
pub use monitoring::{
    BandwidthMonitor, HealthMonitor, LatencyMonitor, NetworkMetrics, NetworkMonitor,
};
pub use protocol::{CompressionManager, EncryptionManager, FrameProcessor, ProtocolNegotiator};
pub use proxy::{LoadBalancer, Proxy, ProxyConnection, SshProxy, StunnelProxy, TcpProxy};
pub use transport::tcp::TcpTransport;
#[cfg(unix)]
pub use transport::unix_socket::UnixSocketTransport;
pub use transport::{Transport, TransportConfig, TransportError, TransportType};

/// Client connection abstraction  
#[derive(Debug)]
pub struct Connection {
    id: ConnectionId,
    transport: SimpleTransport,
    buffer: Vec<u8>,
    authenticated: bool,
}

/// Simple transport abstraction for basic connection
#[derive(Debug)]
pub enum SimpleTransport {
    Tcp(TcpStream),
    #[cfg(unix)]
    Unix(tokio::net::UnixStream),
}

impl Connection {
    /// Create a new connection from TCP stream
    pub fn from_tcp(id: ConnectionId, stream: TcpStream) -> Self {
        Self {
            id,
            transport: SimpleTransport::Tcp(stream),
            buffer: Vec::with_capacity(8192),
            authenticated: false,
        }
    }

    /// Create a new connection from Unix stream
    #[cfg(unix)]
    pub fn from_unix(id: ConnectionId, stream: tokio::net::UnixStream) -> Self {
        Self {
            id,
            transport: SimpleTransport::Unix(stream),
            buffer: Vec::with_capacity(8192),
            authenticated: false,
        }
    }

    /// Get connection ID
    pub fn id(&self) -> ConnectionId {
        self.id
    }

    /// Check if connection is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    /// Set authentication status
    pub fn set_authenticated(&mut self, authenticated: bool) {
        self.authenticated = authenticated;
    }

    /// Read data from connection
    pub async fn read(&mut self) -> io::Result<&[u8]> {
        let mut buf = [0; 8192];
        let n = match &mut self.transport {
            SimpleTransport::Tcp(stream) => stream.read(&mut buf).await?,
            #[cfg(unix)]
            SimpleTransport::Unix(stream) => stream.read(&mut buf).await?,
        };

        tracing::trace!(
            "Connection {}: read {} bytes from {:?}",
            self.id,
            n,
            match &self.transport {
                SimpleTransport::Tcp(_) => "TCP",
                #[cfg(unix)]
                SimpleTransport::Unix(_) => "Unix",
            }
        );

        if n == 0 {
            tracing::trace!("Connection {}: EOF (connection closed)", self.id);
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Connection closed",
            ));
        }

        self.buffer.clear();
        self.buffer.extend_from_slice(&buf[..n]);
        Ok(&self.buffer)
    }

    /// Write data to connection
    pub async fn write(&mut self, data: &[u8]) -> io::Result<()> {
        match &mut self.transport {
            SimpleTransport::Tcp(stream) => stream.write_all(data).await?,
            #[cfg(unix)]
            SimpleTransport::Unix(stream) => stream.write_all(data).await?,
        }
        Ok(())
    }

    /// Flush the connection
    pub async fn flush(&mut self) -> io::Result<()> {
        match &mut self.transport {
            SimpleTransport::Tcp(stream) => stream.flush().await?,
            #[cfg(unix)]
            SimpleTransport::Unix(stream) => stream.flush().await?,
        }
        Ok(())
    }

    /// Get peer address (for TCP connections)
    pub fn peer_addr(&self) -> Option<SocketAddr> {
        match &self.transport {
            SimpleTransport::Tcp(stream) => stream.peer_addr().ok(),
            #[cfg(unix)]
            SimpleTransport::Unix(_) => None, // Unix sockets don't have peer addresses
        }
    }
}

/// Network server for accepting connections
#[derive(Debug)]
pub struct NetworkServer {
    tcp_listener: Option<TcpListener>,
    #[cfg(unix)]
    unix_listener: Option<tokio::net::UnixListener>,
    next_connection_id: ConnectionId,
    transport_tx: Option<mpsc::UnboundedSender<transport::TransportEvent>>,
}

impl NetworkServer {
    /// Create a new network server
    pub fn new() -> Self {
        Self {
            tcp_listener: None,
            #[cfg(unix)]
            unix_listener: None,
            next_connection_id: 1,
            transport_tx: None,
        }
    }

    /// Set the transport event sender
    pub fn set_transport_sender(
        &mut self,
        sender: mpsc::UnboundedSender<transport::TransportEvent>,
    ) {
        self.transport_tx = Some(sender);
    }

    /// Bind to TCP address
    pub async fn bind_tcp(&mut self, addr: &str) -> io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        tracing::info!("TCP server listening on {}", addr);
        self.tcp_listener = Some(listener);
        Ok(())
    }

    /// Bind to Unix socket
    #[cfg(unix)]
    pub async fn bind_unix(&mut self, path: &std::path::Path) -> io::Result<()> {
        use tokio::net::UnixListener;

        // Remove existing socket file if it exists
        if path.exists() {
            std::fs::remove_file(path)?;
        }

        let listener = UnixListener::bind(path)?;
        tracing::info!("Unix socket server listening on {}", path.display());
        self.unix_listener = Some(listener);
        Ok(())
    }
    /// Accept next connection
    pub async fn accept(&mut self) -> io::Result<Connection> {
        let connection_id = self.next_connection_id;
        self.next_connection_id += 1;

        // Try TCP first
        if let Some(ref listener) = self.tcp_listener {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    tracing::debug!("New TCP connection from {}: id={}", addr, connection_id);

                    // Send transport event to connection manager
                    if let Some(ref transport_tx) = self.transport_tx {
                        let event = transport::TransportEvent::ConnectionAccepted {
                            connection_id,
                            transport_type: transport::TransportType::Tcp,
                            remote_endpoint: Some(transport::Endpoint {
                                transport_type: transport::TransportType::Tcp,
                                address: addr.to_string(),
                                is_server: false,
                            }),
                        };
                        if let Err(e) = transport_tx.send(event) {
                            tracing::warn!("Failed to send transport event: {}", e);
                        }
                    }

                    return Ok(Connection::from_tcp(connection_id, stream));
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Continue to try Unix socket
                }
                Err(e) => return Err(e),
            }
        }

        // Try Unix socket
        #[cfg(unix)]
        if let Some(ref listener) = self.unix_listener {
            match listener.accept().await {
                Ok((stream, _)) => {
                    tracing::debug!("New Unix socket connection: id={}", connection_id);

                    // Send transport event to connection manager
                    if let Some(ref transport_tx) = self.transport_tx {
                        let event = transport::TransportEvent::ConnectionAccepted {
                            connection_id,
                            transport_type: transport::TransportType::UnixSocket,
                            remote_endpoint: None, // Unix sockets don't have remote addresses
                        };
                        if let Err(e) = transport_tx.send(event) {
                            tracing::warn!("Failed to send transport event: {}", e);
                        }
                    }

                    return Ok(Connection::from_unix(connection_id, stream));
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No connections available
                }
                Err(e) => return Err(e),
            }
        }

        Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "No connections available",
        ))
    }
}

impl Default for NetworkServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// TCP bind address
    pub tcp_address: Option<String>,
    /// Unix socket path
    pub unix_socket_path: Option<std::path::PathBuf>,
    /// Maximum number of connections
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            tcp_address: Some("127.0.0.1:6000".to_string()),
            #[cfg(unix)]
            unix_socket_path: Some(std::path::PathBuf::from("/tmp/.X11-unix/X0")),
            #[cfg(not(unix))]
            unix_socket_path: None,
            max_connections: 256,
            connection_timeout: 300,
        }
    }
}

/// Network errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum NetworkError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("Connection not found: {0}")]
    ConnectionNotFound(ConnectionId),

    #[error("Too many connections")]
    TooManyConnections,

    #[error("Connection timeout")]
    Timeout,

    #[error("Invalid address: {0}")]
    InvalidAddress(String),
}

impl From<io::Error> for NetworkError {
    fn from(err: io::Error) -> Self {
        NetworkError::Io(err.to_string())
    }
}
