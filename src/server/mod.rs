//! Core X11 server implementation
//!
//! This module contains the main server logic, including client connection
//! management, request processing, and server lifecycle management.

pub mod connection;
pub mod display;
pub mod resources;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crate::{config::ServerConfig, Error, Result};
use crate::protocol::{Event, Request, Response};
use connection::ClientConnection;
use display::Display;
use resources::ResourceManager;

/// Main X11 server structure
pub struct XServer {
    /// Display identifier (e.g., ":0")
    display_name: String,
    /// Server configuration
    config: ServerConfig,
    /// Display manager
    display: Arc<RwLock<Display>>,
    /// Resource manager
    resources: Arc<RwLock<ResourceManager>>,
    /// Active client connections
    clients: Arc<RwLock<HashMap<u32, ClientConnection>>>,
    /// Next client ID
    next_client_id: Arc<RwLock<u32>>,
    /// Server running state
    running: Arc<RwLock<bool>>,
}

impl XServer {
    /// Create a new X server instance
    pub async fn new(display_name: String, config: ServerConfig) -> Result<Self> {
        log::info!("Initializing X server for display {}", display_name);

        let display = Arc::new(RwLock::new(Display::new(&config.display)?));
        let resources = Arc::new(RwLock::new(ResourceManager::new()));
        let clients = Arc::new(RwLock::new(HashMap::new()));
        let next_client_id = Arc::new(RwLock::new(1));
        let running = Arc::new(RwLock::new(false));

        Ok(Self {
            display_name,
            config,
            display,
            resources,
            clients,
            next_client_id,
            running,
        })
    }

    /// Start the X server
    pub async fn run(&mut self) -> Result<()> {
        log::info!("Starting X server on display {}", self.display_name);
        
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        // Start listening for connections
        if self.config.server.enable_tcp {
            self.start_tcp_listener().await?;
        }
        
        self.start_unix_socket_listener().await?;

        // Main server loop
        while *self.running.read().await {
            // TODO: Process events, handle client requests, etc.
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        log::info!("X server stopped");
        Ok(())
    }

    /// Shutdown the server gracefully
    pub async fn shutdown(&mut self) -> Result<()> {
        log::info!("Shutting down X server");
        
        {
            let mut running = self.running.write().await;
            *running = false;
        }

        // Close all client connections
        let mut clients = self.clients.write().await;
        for (id, mut client) in clients.drain() {
            log::debug!("Closing client connection {}", id);
            if let Err(e) = client.close().await {
                log::warn!("Error closing client {}: {}", id, e);
            }
        }

        Ok(())
    }

    /// Start TCP listener for remote connections
    async fn start_tcp_listener(&self) -> Result<()> {
        let display_num: u16 = self.display_name
            .trim_start_matches(':')
            .parse()
            .map_err(|_| Error::Server("Invalid display number".to_string()))?;
        
        let port = self.config.server.tcp_port_base + display_num;
        let addr = format!("0.0.0.0:{}", port);
        
        log::info!("Starting TCP listener on {}", addr);
        
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| Error::Server(format!("Failed to bind TCP socket: {}", e)))?;

        let clients = Arc::clone(&self.clients);
        let next_client_id = Arc::clone(&self.next_client_id);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            while *running.read().await {
                match listener.accept().await {
                    Ok((socket, addr)) => {
                        log::info!("New TCP client connection from {}", addr);
                        let client_id = {
                            let mut next_id = next_client_id.write().await;
                            let id = *next_id;
                            *next_id += 1;
                            id
                        };
                        
                        // TODO: Create and manage client connection
                        // let client = ClientConnection::new(client_id, socket);
                        // clients.write().await.insert(client_id, client);
                    }
                    Err(e) => {
                        log::error!("Failed to accept TCP connection: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Start Unix socket listener for local connections
    async fn start_unix_socket_listener(&self) -> Result<()> {
        // TODO: Implement Unix socket listener
        // This is more complex on Windows, might need named pipes
        log::info!("Unix socket listener not yet implemented");
        Ok(())
    }

    /// Process a client request
    pub async fn process_request(&self, client_id: u32, request: Request) -> Result<Option<Response>> {
        log::debug!("Processing request from client {}: {:?}", client_id, request);

        match request {
            Request::CreateWindow { .. } => {
                // TODO: Implement window creation
                Ok(Some(Response::Success))
            }
            Request::DestroyWindow { .. } => {
                // TODO: Implement window destruction
                Ok(Some(Response::Success))
            }
            Request::MapWindow { .. } => {
                // TODO: Implement window mapping
                Ok(Some(Response::Success))
            }
            Request::UnmapWindow { .. } => {
                // TODO: Implement window unmapping
                Ok(Some(Response::Success))
            }
            Request::GetWindowAttributes { .. } => {
                // TODO: Implement get window attributes
                Ok(Some(Response::Reply {
                    data: 0,
                    sequence_number: 0,
                    body: vec![0; 44], // Placeholder
                }))
            }
            Request::Unknown { opcode, .. } => {
                log::warn!("Unknown request opcode: {}", opcode);
                Ok(Some(Response::Error {
                    error_code: 1, // BadRequest
                    sequence_number: 0,
                    bad_value: opcode as u32,
                    minor_opcode: 0,
                    major_opcode: opcode,
                }))
            }
            _ => {
                log::warn!("Unhandled request type");
                Ok(None)
            }
        }
    }

    /// Send an event to all interested clients
    pub async fn send_event(&self, event: Event) -> Result<()> {
        log::debug!("Broadcasting event: {:?}", event);
        
        // TODO: Determine which clients should receive this event
        // and send it to them
        
        Ok(())
    }
}
