use anyhow::{Context, Result};
use tokio::net::UnixListener;
use tracing::{error, info};

use crate::transport::TransportTrait;

pub struct UnixSocketTransport {
    listener: Arc<Mutex<UnixListener>>,
    cancel_token: CancellationToken,
}

impl UnixSocketTransport {
    pub fn stop(&self) {
        self.cancel_token.cancel();
    }
}

impl TransportTrait for UnixSocketTransport {
    async fn new(addr: &str) -> Self {
        Self {
            listener: Arc::new(Mutex::new(
                UnixListener::bind(addr)
                    .await
                    .context(format!("Failed to bind Unix socket at {}", addr))?,
            )),
            cancel_token: CancellationToken::new(),
        }
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
            info!("Unix listener stopped.");
        });
        Ok(())
    }
}
