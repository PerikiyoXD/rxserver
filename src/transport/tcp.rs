use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::{net::TcpListener, sync::Mutex};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::transport::TransportTrait;

pub struct TcpSocketTransport {
    listener: Arc<Mutex<TcpListener>>,
    cancel_token: CancellationToken,
}

impl TcpSocketTransport {
    pub fn stop(&self) {
        self.cancel_token.cancel();
    }
}

impl TransportTrait for TcpSocketTransport {
    async fn new(addr: &str) -> Result<Self> {
        Ok(Self {
            listener: Arc::new(Mutex::new(
                TcpListener::bind(addr)
                    .await
                    .context(format!("Failed to bind TCP socket at {}", addr))?,
            )),
            cancel_token: CancellationToken::new(),
        })
    }

    async fn run(&self) -> Result<()> {
        let cancel_token = self.cancel_token.clone();
        let listener = self.listener.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        info!("Cancellation requested, shutting down listener.");
                        break;
                    }
                    accept_result = async {
                        let guard = listener.lock().await;
                        guard.accept().await
                    } => {
                        match accept_result {
                            Ok((_socket, client_addr)) => {
                                info!("Accepted connection from {}", client_addr);
                                // TODO: Handle the socket connection here
                            }
                            Err(e) => {
                                info!("Failed to accept connection: {:?}", e);
                                break;
                            }
                        }
                    }
                }
            }
            info!("TCP listener stopped.");
        });
        Ok(())
    }
}
