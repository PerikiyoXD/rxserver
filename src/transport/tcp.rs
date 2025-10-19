// tcp.rs
use super::{ConnectionEvent, TransportContract, TransportKind, TransportMessage};
use crate::server::{ConnectionHandler, state::Server};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    sync::{Mutex, mpsc},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub struct TcpTransport {
    cancel_token: CancellationToken,
    listener: TcpListener,
    message_sender: mpsc::UnboundedSender<TransportMessage>,
    server: Arc<Mutex<Server>>,
    // Track active connections for proper cleanup
    active_connections: Arc<Mutex<HashMap<SocketAddr, JoinHandle<()>>>>,
    max_connections: usize,
}

impl TcpTransport {
    pub async fn new(
        addr: &str,
        server_state: Arc<Mutex<Server>>,
        tx: mpsc::UnboundedSender<TransportMessage>,
    ) -> Result<Self> {
        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("Failed to bind TCP socket at {}", addr))?;

        info!("TCP transport bound to {}", addr);

        // Set reasonable connection limits to prevent resource exhaustion
        let max_connections = 100;

        Ok(Self {
            listener,
            cancel_token: CancellationToken::new(),
            message_sender: tx,
            server: server_state,
            active_connections: Arc::new(Mutex::new(HashMap::new())),
            max_connections,
        })
    }

    async fn handle_connection(
        &self,
        socket: tokio::net::TcpStream,
        client_addr: std::net::SocketAddr,
    ) {
        // Check connection limit
        if self.active_connections.lock().await.len() >= self.max_connections {
            error!(
                "Connection limit ({}) reached, rejecting connection from {}",
                self.max_connections, client_addr
            );

            // Send connection rejected message
            let event = ConnectionEvent {
                client_addr: client_addr.to_string(),
                transport_kind: TransportKind::Tcp,
            };
            let _ = self
                .message_sender
                .send(TransportMessage::ConnectionClosed(event));

            // Close the socket
            drop(socket);
            return;
        }

        let server = Arc::clone(&self.server);
        let message_sender = self.message_sender.clone();
        let active_connections = Arc::clone(&self.active_connections);
        let cancel_token = self.cancel_token.clone();

        let connection_task = async move {
            info!("Connection handler task started for {}", client_addr);

            // Configure TCP socket options
            if let Err(e) = socket.set_nodelay(true) {
                error!("Failed to set TCP nodelay for {}: {}", client_addr, e);
            }

            // Create and run the connection handler
            let result = match ConnectionHandler::new(server, socket, client_addr).await {
                Ok(handler) => {
                    let handler =
                        handler.with_transport_info(message_sender.clone(), TransportKind::Tcp);

                    // Run the connection handler with cancellation support
                    tokio::select! {
                        result = handler.handle() => result,
                        _ = cancel_token.cancelled() => {
                            info!("Connection handler for {} cancelled due to transport shutdown", client_addr);
                            Ok(())
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to create connection handler for {}: {}",
                        client_addr, e
                    );

                    // Only send connection closed event if we failed to create the handler
                    // The connection handler itself will send the event when it finishes normally
                    let event = ConnectionEvent {
                        client_addr: client_addr.to_string(),
                        transport_kind: TransportKind::Tcp,
                    };
                    let _ = message_sender.send(TransportMessage::ConnectionClosed(event));
                    Err(e)
                }
            };

            // Clean up this connection from the active connections registry
            {
                let mut connections = active_connections.lock().await;
                connections.remove(&client_addr);
            }

            match result {
                Ok(_) => {
                    info!(
                        "Connection handler completed successfully for {}",
                        client_addr
                    );
                }
                Err(e) => {
                    error!("Connection handler error for {}: {}", client_addr, e);
                }
            }

            info!("Connection handler task finished for {}", client_addr);
        };

        // Pre-register the connection to avoid race conditions
        {
            let mut connections = self.active_connections.lock().await;
            // Insert a placeholder handle that will be replaced
            connections.insert(client_addr, tokio::spawn(async {}));
        }

        // Spawn the connection task and track it
        let handle = tokio::spawn(connection_task);

        // Replace the placeholder with the actual handle
        {
            let mut connections = self.active_connections.lock().await;
            connections.insert(client_addr, handle);
        }

        info!("Connection handler spawned for {}", client_addr);
    }

    async fn cleanup_finished_connections(&self) {
        let mut connections = self.active_connections.lock().await;
        let mut finished_connections = Vec::new();

        for (addr, handle) in connections.iter() {
            if handle.is_finished() {
                finished_connections.push(*addr);
            }
        }

        for addr in finished_connections {
            if let Some(handle) = connections.remove(&addr) {
                info!("Cleaned up finished connection to {}", addr);
                // Handle will be dropped, cleaning up the task
                drop(handle);
            }
        }
    }
}

impl TransportContract for TcpTransport {
    async fn start(&self) -> Result<()> {
        let span = tracing::info_span!("tcp-transport", addr = %self.listener.local_addr().unwrap_or_else(|_| "unknown".parse().unwrap()));
        let _enter = span.enter();

        info!("Starting TCP transport");
        info!("Server ready to accept connections on all interfaces");

        // Set up periodic cleanup of finished connections
        let cleanup_interval = tokio::time::interval(std::time::Duration::from_secs(30));
        tokio::pin!(cleanup_interval);

        loop {
            tokio::select! {
                _ = self.cancel_token.cancelled() => {
                    info!("TCP transport shutdown requested");
                    let _ = self.message_sender.send(TransportMessage::Shutdown);
                    break;
                }
                accept_result = self.listener.accept() => {
                    match accept_result {
                        Ok((socket, client_addr)) => {
                            info!("TCP connection accepted from {}", client_addr);

                            let event = ConnectionEvent {
                                client_addr: client_addr.to_string(),
                                transport_kind: TransportKind::Tcp,
                            };

                            if self.message_sender.send(TransportMessage::ConnectionAccepted(event)).is_err() {
                                info!("Failed to send connection event, shutting down");
                                break;
                            }

                            self.handle_connection(socket, client_addr).await;
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to accept TCP connection: {}", e);
                            error!("{}", error_msg);
                            let _ = self.message_sender.send(TransportMessage::Error(error_msg));
                            break;
                        }
                    }
                }
                _ = cleanup_interval.tick() => {
                    self.cleanup_finished_connections().await;
                }
            }
        }

        Ok(())
    }

    fn stop(&self) {
        info!("Stopping TCP transport");
        self.cancel_token.cancel();

        // Cancel all active connections
        let active_connections = Arc::clone(&self.active_connections);
        tokio::spawn(async move {
            let mut connections = active_connections.lock().await;
            let connection_count = connections.len();
            if connection_count > 0 {
                info!("Cancelling {} active connections", connection_count);
                for (addr, handle) in connections.drain() {
                    info!("Cancelling connection to {}", addr);
                    handle.abort();
                }
            }
        });
    }

    fn transport_kind(&self) -> TransportKind {
        TransportKind::Tcp
    }
}
