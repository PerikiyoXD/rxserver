// tcp.rs
use super::{ConnectionEvent, TransportContract, TransportKind, TransportMessage};
use crate::server::{ConnectionHandler, state::Server};
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    sync::{Mutex, mpsc},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, Instrument};

pub struct TcpTransport {
    cancel_token: CancellationToken,
    listener: TcpListener,
    message_sender: mpsc::UnboundedSender<TransportMessage>,
    server: Arc<Mutex<Server>>,
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

        Ok(Self {
            listener,
            cancel_token: CancellationToken::new(),
            message_sender: tx,
            server: server_state,
        })
    }

    fn handle_connection(&self, socket: tokio::net::TcpStream, client_addr: std::net::SocketAddr) {
        let server = Arc::clone(&self.server);
        let message_sender = self.message_sender.clone();

        let connection_task = async move {
            let span = tracing::info_span!("tcp-connection", client = %client_addr);
            let _enter = span.enter();

            info!("Connection handler task started for {}", client_addr);

            // Configure TCP socket options
            if let Err(e) = socket.set_nodelay(true) {
                error!("Failed to set TCP nodelay for {}: {}", client_addr, e);
            }

            let result = match ConnectionHandler::new(server, socket, client_addr).await {
                Ok(handler) => {
                    let handler =
                        handler.with_transport_info(message_sender.clone(), TransportKind::Tcp);
                    handler.handle().await
                }
                Err(e) => {
                    error!(
                        "Failed to create connection handler for {}: {}",
                        client_addr, e
                    );

                    // Send connection closed event since we failed to create the handler
                    let event = ConnectionEvent {
                        client_addr: client_addr.to_string(),
                        transport_kind: TransportKind::Tcp,
                    };
                    let _ = message_sender.send(TransportMessage::ConnectionClosed(event));
                    return;
                }
            };

            // Log the result and ensure cleanup happens
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

        // Spawn the connection task with its own isolated span
        tokio::spawn(connection_task.instrument(tracing::info_span!("connection-task", client = %client_addr)));

        info!("Connection handler spawned for {}", client_addr);
    }
}

impl TransportContract for TcpTransport {
    async fn start(&self) -> Result<()> {
        let span = tracing::info_span!("tcp-transport", addr = %self.listener.local_addr().unwrap_or_else(|_| "unknown".parse().unwrap()));
        let _enter = span.enter();

        info!("Starting TCP transport");
        info!("Server ready to accept connections on all interfaces");

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

                            self.handle_connection(socket, client_addr);
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to accept TCP connection: {}", e);
                            error!("{}", error_msg);
                            let _ = self.message_sender.send(TransportMessage::Error(error_msg));
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn stop(&self) {
        info!("Stopping TCP transport");
        self.cancel_token.cancel();
    }

    fn transport_kind(&self) -> TransportKind {
        TransportKind::Tcp
    }
}
