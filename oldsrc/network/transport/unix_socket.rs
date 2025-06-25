//! Unix socket transport implementation
//!
//! Provides Unix domain socket transport for X11 connections.

#[cfg(unix)]
pub use unix_impl::*;

#[cfg(unix)]
mod unix_impl {
    use super::traits::{
        ConnectionMetadata, Endpoint, Transport, TransportConfig, TransportError, TransportEvent,
        TransportStatistics, TransportType,
    };
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{UnixListener, UnixStream};
    use tokio::sync::{Mutex, RwLock, mpsc};
    use tracing::{debug, error, info, warn};

    /// Unix socket transport implementation
    #[derive(Debug)]
    pub struct UnixSocketTransport {
        /// Transport configuration
        config: TransportConfig,
        /// Event sender for transport events
        event_sender: mpsc::UnboundedSender<TransportEvent>,
        /// Unix socket listener
        listener: Option<UnixListener>,
        /// Socket path
        socket_path: PathBuf,
        /// Active connections
        connections: Arc<RwLock<HashMap<crate::network::ConnectionId, UnixConnection>>>,
        /// Transport statistics
        statistics: Arc<Mutex<TransportStatistics>>,
        /// Running state
        is_running: Arc<std::sync::atomic::AtomicBool>,
        /// Connection ID counter
        next_connection_id: Arc<std::sync::atomic::AtomicU32>,
    }

    /// Unix socket connection wrapper
    #[derive(Debug)]
    struct UnixConnection {
        /// Connection ID
        id: crate::network::ConnectionId,
        /// Unix stream
        stream: Arc<Mutex<UnixStream>>,
        /// Connection metadata
        metadata: ConnectionMetadata,
    }

    impl UnixSocketTransport {
        /// Create a new Unix socket transport
        pub async fn new(
            config: TransportConfig,
            event_sender: mpsc::UnboundedSender<TransportEvent>,
        ) -> Result<Self, TransportError> {
            debug!(
                "Creating Unix socket transport with path: {}",
                config.address
            );

            let socket_path = PathBuf::from(&config.address);

            Ok(Self {
                config,
                event_sender,
                listener: None,
                socket_path,
                connections: Arc::new(RwLock::new(HashMap::new())),
                statistics: Arc::new(Mutex::new(TransportStatistics::default())),
                is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                next_connection_id: Arc::new(std::sync::atomic::AtomicU32::new(1)),
            })
        }
    }

    #[async_trait]
    impl Transport for UnixSocketTransport {
        fn transport_type(&self) -> TransportType {
            TransportType::UnixSocket
        }

        async fn start(&mut self) -> Result<(), TransportError> {
            if self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
                return Ok(());
            }

            info!(
                "Starting Unix socket transport on {}",
                self.socket_path.display()
            );

            // Remove existing socket file if it exists
            if self.socket_path.exists() {
                std::fs::remove_file(&self.socket_path).map_err(|e| {
                    TransportError::ConnectionFailed(format!(
                        "Failed to remove existing socket: {}",
                        e
                    ))
                })?;
            }

            // Create parent directory if it doesn't exist
            if let Some(parent) = self.socket_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    TransportError::ConnectionFailed(format!(
                        "Failed to create socket directory: {}",
                        e
                    ))
                })?;
            }

            let listener = UnixListener::bind(&self.socket_path).map_err(|e| {
                TransportError::ConnectionFailed(format!("Failed to bind Unix socket: {}", e))
            })?;

            info!(
                "Unix socket transport listening on {}",
                self.socket_path.display()
            );

            self.listener = Some(listener);
            self.is_running
                .store(true, std::sync::atomic::Ordering::SeqCst);

            // Spawn accept loop
            let listener = self
                .listener
                .as_ref()
                .unwrap()
                .try_clone()
                .map_err(TransportError::Io)?;
            let connections = self.connections.clone();
            let event_sender = self.event_sender.clone();
            let statistics = self.statistics.clone();
            let is_running = self.is_running.clone();
            let next_connection_id = self.next_connection_id.clone();

            tokio::spawn(async move {
                while is_running.load(std::sync::atomic::Ordering::SeqCst) {
                    match listener.accept().await {
                        Ok((stream, _addr)) => {
                            debug!("Accepted Unix socket connection");

                            let connection_id = next_connection_id
                                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                            let connection = UnixConnection {
                                id: connection_id,
                                stream: Arc::new(Mutex::new(stream)),
                                metadata: ConnectionMetadata::default(),
                            };

                            // Add to connections map
                            {
                                let mut connections_guard = connections.write().await;
                                connections_guard.insert(connection_id, connection);
                            }

                            // Update statistics
                            {
                                let mut stats = statistics.lock().await;
                                stats.connections_accepted += 1;
                                stats.active_connections += 1;
                            }

                            // Send event
                            let event = TransportEvent::ConnectionAccepted {
                                connection_id,
                                transport_type: TransportType::UnixSocket,
                                remote_endpoint: Some(Endpoint {
                                    transport_type: TransportType::UnixSocket,
                                    address: "unix".to_string(),
                                    is_server: false,
                                }),
                            };

                            if let Err(e) = event_sender.send(event) {
                                warn!("Failed to send connection accepted event: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to accept Unix socket connection: {}", e);

                            let event = TransportEvent::Error {
                                connection_id: None,
                                error: TransportError::Io(e),
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

            info!("Stopping Unix socket transport");

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

            // Remove socket file
            if self.socket_path.exists() {
                if let Err(e) = std::fs::remove_file(&self.socket_path) {
                    warn!("Failed to remove socket file: {}", e);
                }
            }

            info!("Unix socket transport stopped");
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

                        // Update statistics
                        let mut stats = self.statistics.lock().await;
                        stats.total_bytes_sent += data.len() as u64;
                        stats.total_messages_sent += 1;

                        Ok(data.len())
                    }
                    Err(e) => {
                        error!("Failed to send data to connection {}: {}", connection_id, e);
                        Err(TransportError::Io(e))
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
                debug!("Closed Unix socket connection {}", connection_id);

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
            // Note: This would need to be async in practice to access the RwLock
            None
        }

        fn get_active_connections(&self) -> Vec<crate::network::ConnectionId> {
            // Note: This would need to be async in practice to access the RwLock
            Vec::new()
        }

        fn is_running(&self) -> bool {
            self.is_running.load(std::sync::atomic::Ordering::SeqCst)
        }
        fn get_statistics(&self) -> TransportStatistics {
            // Note: This would need to be async in practice to access the Mutex
            TransportStatistics::default()
        }
    }
} // End of unix_impl module
