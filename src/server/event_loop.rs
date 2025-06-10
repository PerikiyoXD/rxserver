//! Event Loop
//!
//! This module contains the main server event loop.

use crate::server::ServerEvent;
use crate::server::{ClientManager, ConnectionManager, RequestHandler};
use crate::{todo_critical, todo_high, todo_medium, Result};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Main server event loop
pub struct EventLoop {
    connection_manager: Arc<ConnectionManager>,
    client_manager: Arc<ClientManager>,
    request_handler: Arc<RequestHandler>,
    event_sender: broadcast::Sender<ServerEvent>,
}

impl EventLoop {
    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        client_manager: Arc<ClientManager>,
        request_handler: Arc<RequestHandler>,
        event_sender: broadcast::Sender<ServerEvent>,
    ) -> Self {
        Self {
            connection_manager,
            client_manager,
            request_handler,
            event_sender,
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting main server event loop");

        // Set up Unix domain socket for X11 connections
        let socket_path = format!("/tmp/.X11-unix/X0"); // TODO: Make configurable

        // For now, create a simple TCP listener as a placeholder
        let listener = tokio::net::TcpListener::bind("127.0.0.1:6000")
            .await
            .map_err(|e| crate::Error::Io(e))?;

        info!("X server listening on 127.0.0.1:6000 (TCP fallback)");
        info!("X server would normally listen on {}", socket_path);

        loop {
            tokio::select! {
                // Accept new connections
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            info!("New client connection from: {}", addr);

                            // Handle connection in background task
                            let connection_manager = self.connection_manager.clone();
                            let client_manager = self.client_manager.clone();
                            let request_handler = self.request_handler.clone();

                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_client_connection(
                                    stream,
                                    connection_manager,
                                    client_manager,
                                    request_handler,
                                ).await {
                                    eprintln!("Client connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("Failed to accept connection: {}", e);
                        }
                    }
                }

                // Handle shutdown signal
                _ = tokio::signal::ctrl_c() => {
                    info!("Received shutdown signal");
                    break;
                }
            }
        }

        info!("Event loop shutting down");
        Ok(())
    }

    /// Handle a single client connection
    async fn handle_client_connection(
        mut stream: tokio::net::TcpStream,
        _connection_manager: Arc<ConnectionManager>,
        _client_manager: Arc<ClientManager>,
        _request_handler: Arc<RequestHandler>,
    ) -> Result<()> {
        use crate::protocol::requests::Request;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        info!("Handling new client connection");

        // Simple connection handling - read some data and respond
        let mut buffer = [0u8; 1024];

        loop {
            tokio::select! {
                // Read from client
                result = stream.read(&mut buffer) => {
                    match result {
                        Ok(0) => {
                            info!("Client disconnected");
                            break;
                        }
                        Ok(n) => {
                            info!("Received {} bytes from client", n);

                            // TODO: Parse X11 protocol requests
                            // For now, just echo back
                            if let Err(e) = stream.write_all(&buffer[..n]).await {
                                eprintln!("Failed to write to client: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read from client: {}", e);
                            break;
                        }
                    }
                }

                // Handle timeout or other events
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                    info!("Client connection timeout");
                    break;
                }
            }
        }

        info!("Client connection handler terminating");
        Ok(())
    }
}
