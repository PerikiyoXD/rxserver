// unix.rs
#[cfg(unix)]
use super::{ConnectionEvent, TransportContract, TransportKind, TransportMessage};
#[cfg(unix)]
use crate::server::{ConnectionHandler, state::Server};
#[cfg(unix)]
use anyhow::{Context, Result};
#[cfg(unix)]
use std::sync::Arc;
#[cfg(unix)]
use tokio::{
    net::UnixListener,
    sync::{Mutex, mpsc},
};
#[cfg(unix)]
use tokio_util::sync::CancellationToken;
#[cfg(unix)]
use tracing::{error, info};

#[cfg(unix)]
pub struct UnixTransport {
    listener: UnixListener,
    cancel_token: CancellationToken,
    message_sender: mpsc::UnboundedSender<TransportMessage>,
    server_state: Arc<Mutex<Server>>,
    socket_path: String,
}

#[cfg(unix)]
impl UnixTransport {
    pub async fn new(
        path: &str,
        server_state: Arc<Mutex<Server>>,
        tx: mpsc::UnboundedSender<TransportMessage>,
    ) -> Result<Self> {
        // Remove existing socket file if it exists
        let _ = std::fs::remove_file(path);

        let listener = UnixListener::bind(path)
            .with_context(|| format!("Failed to bind Unix socket at {}", path))?;

        info!("Unix transport bound to {}", path);

        Ok(Self {
            listener,
            cancel_token: CancellationToken::new(),
            message_sender: tx,
            server_state,
            socket_path: path.to_string(),
        })
    }

    async fn handle_connection(&self, socket: tokio::net::UnixStream) {
        let server_state = Arc::clone(&self.server_state);
        let socket_path = self.socket_path.clone();

        tokio::spawn(async move {
            match ConnectionHandler::new_unix(server_state, socket, socket_path.clone()).await {
                Ok(handler) => {
                    if let Err(e) = handler.handle().await {
                        error!("Unix connection handler error: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to create Unix connection handler: {}", e);
                }
            }
        });
    }
}

#[cfg(unix)]
impl TransportContract for UnixTransport {
    async fn start(&self) -> Result<()> {
        info!("Starting Unix transport");

        loop {
            tokio::select! {
                _ = self.cancel_token.cancelled() => {
                    info!("Unix transport shutdown requested");
                    let _ = self.message_sender.send(TransportMessage::Shutdown);
                    break;
                }
                accept_result = self.listener.accept() => {
                    match accept_result {
                        Ok((socket, _)) => {
                            info!("Unix connection accepted");

                            let event = ConnectionEvent {
                                client_addr: self.socket_path.clone(),
                                transport_kind: TransportKind::Unix,
                            };

                            if self.message_sender.send(TransportMessage::ConnectionAccepted(event)).is_err() {
                                info!("Failed to send connection event, shutting down");
                                break;
                            }

                            self.handle_connection(socket).await;
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to accept Unix connection: {}", e);
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
        info!("Stopping Unix transport");
        self.cancel_token.cancel();
        // Clean up socket file
        let _ = std::fs::remove_file(&self.socket_path);
    }

    fn transport_kind(&self) -> TransportKind {
        TransportKind::Unix
    }
}

#[cfg(unix)]
impl Drop for UnixTransport {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}
