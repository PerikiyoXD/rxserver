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
use tracing::{error, info};

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

    async fn handle_connection(
        &self,
        socket: tokio::net::TcpStream,
        client_addr: std::net::SocketAddr,
    ) {
        let server = Arc::clone(&self.server);

        tokio::spawn(async move {
            match ConnectionHandler::new(server, socket, client_addr).await {
                Ok(handler) => {
                    if let Err(e) = handler.handle().await {
                        error!("Connection handler error for {}: {}", client_addr, e);
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to create connection handler for {}: {}",
                        client_addr, e
                    );
                }
            }
        });
    }
}

impl TransportContract for TcpTransport {
    async fn start(&self) -> Result<()> {
        info!("Starting TCP transport");

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
