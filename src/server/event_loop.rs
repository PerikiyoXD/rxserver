//! Event Loop
//!
//! This module contains the main server event loop that coordinates between
//! connection management, client management, and request handling.

use crate::server::ServerEvent;
use crate::server::{ClientManager, ConnectionManager, RequestHandler};
use crate::Result;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Main server event loop
pub struct EventLoop {
    client_manager: Arc<ClientManager>,
    request_handler: Arc<RequestHandler>,
}

impl EventLoop {    pub fn new(
        client_manager: Arc<ClientManager>,
        request_handler: Arc<RequestHandler>,
        _event_sender: broadcast::Sender<ServerEvent>,
    ) -> Self {
        Self {
            client_manager,
            request_handler,
        }
    }pub async fn run(&self, connection_manager: Arc<ConnectionManager>) -> Result<()> {
        info!("Starting main server event loop");

        loop {
            tokio::select! {                // Accept new connections through the connection manager
                result = connection_manager.accept_connection() => {
                    match result {
                        Ok(new_connection) => {
                            info!("New client connection from: {}", new_connection.addr);

                            // Handle connection in background task using dedicated handler
                            let client_manager = self.client_manager.clone();
                            let request_handler = self.request_handler.clone();
                            tokio::spawn(async move {
                                let connection_handler = crate::server::X11ConnectionHandler::new(
                                    client_manager,
                                    request_handler,
                                );
                                
                                if let Err(e) = connection_handler.handle_connection(new_connection.stream).await {
                                    warn!("Client connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            warn!("Failed to accept connection: {}", e);
                        }
                    }
                }

                // Handle shutdown signal
                _ = tokio::signal::ctrl_c() => {
                    info!("Received shutdown signal");
                    break;
                }
            }
        }        info!("Event loop shutting down");
        Ok(())
    }
}
