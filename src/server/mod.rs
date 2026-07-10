// mod.rs - Final clean version
pub mod atom_system;
pub mod client_system;
pub mod config;
pub mod connection;
pub mod display_system;
pub mod gcontext_system;
pub mod graphics;
pub mod pixmap_system;
pub mod resource_system;
pub mod state;
pub mod window_system;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tracing::info;

use crate::transport::{Transport, TransportMessage};

pub use config::*;
pub use connection::*;
pub use state::*;

pub struct RX11Server {
    state: Arc<Mutex<Server>>,
    config: ServerConfig,
}

impl RX11Server {
    pub fn new(config: ServerConfig) -> Result<Self> {
        let state = Server::new(config.displays.clone())?;
        info!("Server initialized successfully");

        // DEBUG: Print full config parsed.
        info!("Server configuration: {:?}", config);
        Ok(Self { state, config })
    }

    pub fn state(&self) -> &Arc<Mutex<Server>> {
        &self.state
    }

    pub async fn run(self) -> Result<()> {
        info!("Starting X11 RxServer...");

        let display_count = {
            let server = self.state.lock().await;
            server.display_count()
        };
        info!("Server running with {} display(s)", display_count);

        // Sync initial window state to displays
        {
            let server = self.state.lock().await;
            server.sync_windows_to_displays().await;
        }

        // Create transport message channel
        let (tx, mut rx) = mpsc::unbounded_channel::<TransportMessage>();

        // Start all transports
        let mut transports = Vec::new();
        for (index, transport_config) in self.config.transports.iter().enumerate() {
            let transport = Transport::new(
                transport_config.kind,
                &transport_config.bind_address,
                Arc::clone(&self.state),
                tx.clone(),
            )
            .await?;

            let transport_kind = transport.transport_kind();
            let bind_address = transport_config.bind_address.clone();

            info!("Started {:?} transport on {}", transport_kind, bind_address);

            // Start transport in background
            let transport_handle = {
                let transport = transport;
                tokio::spawn(async move {
                    // Add span for better tracing
                    let span = tracing::info_span!("transport", kind = ?transport_kind, index = index);
                    let _enter = span.enter();
                    
                    if let Err(e) = transport.start().await {
                        tracing::error!(
                            "Transport {:?} at {} failed: {}",
                            transport_kind,
                            bind_address,
                            e
                        );
                    }
                })
            };

            transports.push((transport_handle, transport_config.clone()));
        }

        // Handle transport messages
        let message_handler = tokio::spawn(async move {
            let span = tracing::info_span!("transport-msg-handler");
            let _enter = span.enter();
            while let Some(message) = rx.recv().await {
                match message {
                    TransportMessage::ConnectionAccepted(event) => {
                        info!(
                            "Connection accepted from {} via {:?}",
                            event.client_addr, event.transport_kind
                        );
                    }
                    TransportMessage::ConnectionClosed(event) => {
                        info!(
                            "Connection closed from {} via {:?}",
                            event.client_addr, event.transport_kind
                        );
                    }
                    TransportMessage::Error(error) => {
                        tracing::error!("Transport error: {}", error);
                    }
                    TransportMessage::Shutdown => {
                        info!("Transport shutdown message received");
                        break;
                    }
                }
            }
        });

        // Wait for shutdown signal
        tokio::signal::ctrl_c().await?;

        info!("Received shutdown signal triggered by Ctrl+C, stopping server...");

        // Stop all transports
        for (handle, config) in transports {
            info!(
                "Stopping {:?} transport on {}",
                config.kind, config.bind_address
            );
            handle.abort();
        }

        message_handler.abort();

        {
            let server = self.state.lock().await;
            server.shutdown().await;
        }
        info!("X11 RxServer stopped");
        Ok(())
    }
}
