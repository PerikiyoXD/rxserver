//! TCP transport implementation
//!
//! Provides TCP socket transport for X11 connections.

use super::traits::{
    ConnectionMetadata, Endpoint, Transport, TransportConfig, TransportError, TransportEvent,
    TransportStatistics, TransportType,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock as StdRwLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, error, info, warn};

/// TCP transport implementation
#[derive(Debug)]
pub struct TcpTransport {
    /// Transport configuration
    config: TransportConfig,
    /// Event sender for transport events
    event_sender: mpsc::UnboundedSender<TransportEvent>,
    /// TCP listener
    listener: Option<TcpListener>,
    /// Active connections (async access for operations)
    connections: Arc<tokio::sync::RwLock<HashMap<crate::network::ConnectionId, TcpConnection>>>,
    /// Connection metadata cache (sync access for trait methods)
    connection_metadata: Arc<StdRwLock<HashMap<crate::network::ConnectionId, ConnectionMetadata>>>,
    /// Transport statistics (async for updates)
    statistics: Arc<Mutex<TransportStatistics>>,
    /// Cached statistics for sync access
    cached_statistics: Arc<StdRwLock<TransportStatistics>>,
    /// Running state
    is_running: Arc<std::sync::atomic::AtomicBool>,
    /// Connection ID counter
    next_connection_id: Arc<std::sync::atomic::AtomicU32>,
}

/// TCP connection wrapper
#[derive(Debug)]
struct TcpConnection {
    /// Connection ID
    id: crate::network::ConnectionId,
    /// TCP stream
    stream: Arc<Mutex<TcpStream>>,
    /// Connection metadata
    metadata: ConnectionMetadata,
    /// Local address
    local_addr: SocketAddr,
    /// Remote address
    remote_addr: SocketAddr,
}

impl TcpTransport {
    /// Create a new TCP transport
    pub async fn new(
        config: TransportConfig,
        event_sender: mpsc::UnboundedSender<TransportEvent>,
    ) -> Result<Self, TransportError> {
        debug!("Creating TCP transport with address: {}", config.address);
        Ok(Self {
            config,
            event_sender,
            listener: None,
            connections: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            connection_metadata: Arc::new(StdRwLock::new(HashMap::new())),
            statistics: Arc::new(Mutex::new(TransportStatistics::default())),
            cached_statistics: Arc::new(StdRwLock::new(TransportStatistics::default())),
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            next_connection_id: Arc::new(std::sync::atomic::AtomicU32::new(1)),
        })
    }

    /// Handle incoming connection
    async fn handle_connection(&self, stream: TcpStream) -> Result<(), TransportError> {
        let connection_id = self
            .next_connection_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let local_addr = stream
            .local_addr()
            .map_err(|e| TransportError::Io(e.to_string()))?;
        let remote_addr = stream
            .peer_addr()
            .map_err(|e| TransportError::Io(e.to_string()))?;

        debug!(
            "New TCP connection {}: {} -> {}",
            connection_id, remote_addr, local_addr
        );

        let connection = TcpConnection {
            id: connection_id,
            stream: Arc::new(Mutex::new(stream)),
            metadata: ConnectionMetadata::default(),
            local_addr,
            remote_addr,
        }; // Add to connections map
        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id, connection);
        }

        // Cache metadata for sync access
        {
            let mut metadata_cache = self.connection_metadata.write().unwrap();
            metadata_cache.insert(connection_id, ConnectionMetadata::default());
        }

        // Update statistics
        {
            let mut stats = self.statistics.lock().await;
            stats.connections_accepted += 1;
            stats.active_connections += 1;
        }

        // Update cached statistics for sync access
        {
            if let Ok(stats) = self.statistics.try_lock() {
                let mut cached = self.cached_statistics.write().unwrap();
                *cached = stats.clone();
            }
        }

        // Send event
        let event = TransportEvent::ConnectionAccepted {
            connection_id,
            transport_type: TransportType::Tcp,
            remote_endpoint: Some(Endpoint {
                transport_type: TransportType::Tcp,
                address: remote_addr.to_string(),
                is_server: false,
            }),
        };

        if let Err(e) = self.event_sender.send(event) {
            warn!("Failed to send connection accepted event: {}", e);
        }

        // Start reading from connection
        self.spawn_connection_reader(connection_id).await;

        Ok(())
    }
    /// Spawn a task to read from a connection
    async fn spawn_connection_reader(&self, connection_id: crate::network::ConnectionId) {
        let connections = self.connections.clone();
        let connection_metadata = self.connection_metadata.clone();
        let event_sender = self.event_sender.clone();
        let statistics = self.statistics.clone();

        tokio::spawn(async move {
            let mut buffer = vec![0u8; 8192];

            loop {
                let stream = {
                    let connections_guard = connections.read().await;
                    if let Some(connection) = connections_guard.get(&connection_id) {
                        connection.stream.clone()
                    } else {
                        debug!("Connection {} no longer exists", connection_id);
                        break;
                    }
                };

                let bytes_read = {
                    let mut stream_guard = stream.lock().await;
                    match stream_guard.read(&mut buffer).await {
                        Ok(0) => {
                            debug!("Connection {} closed by peer", connection_id);
                            break;
                        }
                        Ok(n) => n,
                        Err(e) => {
                            error!("Error reading from connection {}: {}", connection_id, e);
                            break;
                        }
                    }
                }; // Update metadata
                {
                    let mut connections_guard = connections.write().await;
                    if let Some(connection) = connections_guard.get_mut(&connection_id) {
                        connection.metadata.bytes_received += bytes_read as u64;
                        connection.metadata.messages_received += 1;
                        connection.metadata.last_activity = std::time::SystemTime::now();
                    }
                }

                // Update cached metadata
                {
                    if let Ok(mut metadata_cache) = connection_metadata.write() {
                        if let Some(cached_meta) = metadata_cache.get_mut(&connection_id) {
                            cached_meta.bytes_received += bytes_read as u64;
                            cached_meta.messages_received += 1;
                            cached_meta.last_activity = std::time::SystemTime::now();
                        }
                    }
                }

                // Update statistics
                {
                    let mut stats = statistics.lock().await;
                    stats.total_bytes_received += bytes_read as u64;
                    stats.total_messages_received += 1;
                }

                // Send data event
                let event = TransportEvent::DataReceived {
                    connection_id,
                    data: buffer[..bytes_read].to_vec(),
                };

                if let Err(e) = event_sender.send(event) {
                    warn!("Failed to send data received event: {}", e);
                    break;
                }
            } // Connection closed - clean up
            {
                let mut connections_guard = connections.write().await;
                connections_guard.remove(&connection_id);
            }

            // Clean up cached metadata
            {
                if let Ok(mut metadata_cache) = connection_metadata.write() {
                    metadata_cache.remove(&connection_id);
                }
            }

            {
                let mut stats = statistics.lock().await;
                stats.active_connections = stats.active_connections.saturating_sub(1);
            }

            let event = TransportEvent::ConnectionClosed {
                connection_id,
                reason: "Connection closed".to_string(),
            };

            let _ = event_sender.send(event);
        });
    }

    /// Update cached statistics from the async version
    fn update_cached_statistics(&self) {
        if let Ok(stats) = self.statistics.try_lock() {
            if let Ok(mut cached) = self.cached_statistics.write() {
                *cached = stats.clone();
            }
        }
    }

    /// Update both async and cached statistics
    async fn update_statistics<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut TransportStatistics),
    {
        {
            let mut stats = self.statistics.lock().await;
            update_fn(&mut stats);
        }
        self.update_cached_statistics();
    }
}

#[async_trait]
impl Transport for TcpTransport {
    fn transport_type(&self) -> TransportType {
        TransportType::Tcp
    }

    async fn start(&mut self) -> Result<(), TransportError> {
        if self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        info!("Starting TCP transport on {}", self.config.address);

        let listener = TcpListener::bind(&self.config.address).await.map_err(|e| {
            TransportError::ConnectionFailed(format!("Failed to bind TCP listener: {}", e))
        })?;

        let local_addr = listener
            .local_addr()
            .map_err(|e| TransportError::Io(e.to_string()))?;

        info!("TCP transport listening on {}", local_addr);
        self.listener = Some(listener);
        self.is_running
            .store(true, std::sync::atomic::Ordering::SeqCst);
        // Take the listener to move it into the spawn task
        let listener = self.listener.take().unwrap();
        let event_sender = self.event_sender.clone();
        let is_running = self.is_running.clone();

        // Create a TcpTransport instance for the background task
        // Note: We need to reconstruct the transport for the spawned task
        // since we can't move self into the task
        let transport_for_task = Self {
            config: self.config.clone(),
            event_sender: self.event_sender.clone(),
            listener: None, // Will not be used in the spawned task
            connections: self.connections.clone(),
            connection_metadata: self.connection_metadata.clone(),
            statistics: self.statistics.clone(),
            cached_statistics: self.cached_statistics.clone(),
            is_running: self.is_running.clone(),
            next_connection_id: self.next_connection_id.clone(),
        };

        tokio::spawn(async move {
            while is_running.load(std::sync::atomic::Ordering::SeqCst) {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        debug!("Accepted TCP connection from {}", addr);

                        // Use the existing handle_connection method
                        if let Err(e) = transport_for_task.handle_connection(stream).await {
                            error!("Failed to handle connection from {}: {}", addr, e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to accept TCP connection: {}", e);
                        let event = TransportEvent::Error {
                            connection_id: None,
                            error: TransportError::Io(e.to_string()),
                        };

                        let _ = event_sender.send(event);
                    }
                }
            }
        });

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), TransportError> {
        if !self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        info!("Stopping TCP transport");

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

        self.listener = None;

        info!("TCP transport stopped");
        Ok(())
    }

    async fn send_data(
        &mut self,
        connection_id: crate::network::ConnectionId,
        data: &[u8],
    ) -> Result<usize, TransportError> {
        let connections = self.connections.read().await;

        if let Some(connection) = connections.get(&connection_id) {
            let mut stream = connection.stream.lock().await;

            match stream.write_all(data).await {
                Ok(()) => {
                    // Update metadata
                    drop(stream);
                    drop(connections);

                    let mut connections = self.connections.write().await;
                    if let Some(connection) = connections.get_mut(&connection_id) {
                        connection.metadata.bytes_sent += data.len() as u64;
                        connection.metadata.messages_sent += 1;
                        connection.metadata.last_activity = std::time::SystemTime::now();
                    }

                    // Update cached metadata
                    {
                        if let Ok(mut metadata_cache) = self.connection_metadata.write() {
                            if let Some(cached_meta) = metadata_cache.get_mut(&connection_id) {
                                cached_meta.bytes_sent += data.len() as u64;
                                cached_meta.messages_sent += 1;
                                cached_meta.last_activity = std::time::SystemTime::now();
                            }
                        }
                    }

                    // Update statistics
                    let mut stats = self.statistics.lock().await;
                    stats.total_bytes_sent += data.len() as u64;
                    stats.total_messages_sent += 1;

                    // Update cached statistics
                    self.update_cached_statistics();

                    Ok(data.len())
                }
                Err(e) => {
                    error!("Failed to send data to connection {}: {}", connection_id, e);
                    Err(TransportError::Io(e.to_string()))
                }
            }
        } else {
            Err(TransportError::ConnectionFailed(format!(
                "Connection {} not found",
                connection_id
            )))
        }
    }

    async fn close_connection(
        &mut self,
        connection_id: crate::network::ConnectionId,
    ) -> Result<(), TransportError> {
        let mut connections = self.connections.write().await;
        if let Some(_connection) = connections.remove(&connection_id) {
            debug!("Closed TCP connection {}", connection_id);

            // Clean up cached metadata
            {
                if let Ok(mut metadata_cache) = self.connection_metadata.write() {
                    metadata_cache.remove(&connection_id);
                }
            }

            // Update statistics
            let mut stats = self.statistics.lock().await;
            stats.active_connections = stats.active_connections.saturating_sub(1);

            // Update cached statistics
            self.update_cached_statistics();

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
        // Return a reference to cached metadata
        // Note: This is a limitation of the sync API - we can't hold a lock across the return
        // In practice, this method should probably return Option<ConnectionMetadata> instead
        None // Temporary limitation due to sync API constraints
    }

    fn get_active_connections(&self) -> Vec<crate::network::ConnectionId> {
        // Get active connections from cached metadata
        if let Ok(metadata) = self.connection_metadata.read() {
            metadata.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    fn is_running(&self) -> bool {
        self.is_running.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn get_statistics(&self) -> TransportStatistics {
        // Return cached statistics
        if let Ok(stats) = self.cached_statistics.read() {
            stats.clone()
        } else {
            TransportStatistics::default()
        }
    }
}
