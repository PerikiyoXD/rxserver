//! Core X11 server implementation
//!
//! This module contains the main server logic, including client connection
//! management, request processing, and server lifecycle management.

pub mod connection;
pub mod display;
pub mod resources;
pub mod events;
pub mod handlers;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crate::{config::ServerConfig, Error, Result};
use crate::protocol::{Event, Request, Response};
use connection::ClientConnection;
use display::Display;
use resources::ResourceManager;
use events::{EventBus, ServerEvent, EventResponse};
use handlers::{WindowHandler, GraphicsHandler};

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
    /// Event bus
    event_bus: Arc<EventBus>,
}

impl XServer {    /// Create a new X server instance
    pub async fn new(display_name: String, config: ServerConfig) -> Result<Self> {
        log::info!("Initializing X server for display {}", display_name);

        let display = Arc::new(RwLock::new(Display::new(&config.display)?));
        let resources = Arc::new(RwLock::new(ResourceManager::new()));
        let clients = Arc::new(RwLock::new(HashMap::new()));
        let next_client_id = Arc::new(RwLock::new(1));
        let running = Arc::new(RwLock::new(false));
        let event_bus = Arc::new(EventBus::new());        // Register event handlers
        let window_handler = Arc::new(WindowHandler::new(Arc::clone(&resources)));
        event_bus.register_handler(window_handler).await;
        
        let graphics_handler = Arc::new(GraphicsHandler::new(Arc::clone(&resources)));
        event_bus.register_handler(graphics_handler).await;

        log::info!("Registered {} event handlers", event_bus.handler_count().await);

        Ok(Self {
            display_name,
            config,
            display,
            resources,
            clients,
            next_client_id,
            running,
            event_bus,
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

        let _clients = Arc::clone(&self.clients);
        let next_client_id = Arc::clone(&self.next_client_id);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            while *running.read().await {
                match listener.accept().await {                    Ok((socket, addr)) => {
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
                        
                        // For now, just close the socket to avoid resource leaks
                        drop(socket);
                        log::debug!("Client {} connection handling not yet implemented", client_id);
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
    }    /// Process a client request
    pub async fn process_request(&self, client_id: u32, sequence_number: u16, request: Request) -> Result<Vec<EventResponse>> {
        log::debug!("Processing request from client {}: {:?}", client_id, request);

        // Emit RequestReceived event
        let event = ServerEvent::RequestReceived {
            client_id,
            sequence_number,
            request,
        };

        let responses = self.event_bus.emit(event).await?;
          if responses.is_empty() {
            // No handler processed this request, return a generic error
            log::warn!("No handler processed request from client {}", client_id);
            return Ok(vec![EventResponse::Response(Response::Error {
                error_code: 1, // BadRequest
                sequence_number,
                bad_value: 0,
                minor_opcode: 0,
                major_opcode: 0,
            })]);
        }

        Ok(responses)
    }

    /// Send an event to all interested clients
    pub async fn send_event(&self, event: Event) -> Result<()> {
        log::debug!("Broadcasting event: {:?}", event);
        
        // TODO: Determine which clients should receive this event
        // and send it to them
        
        Ok(())
    }

    /// Emit a server event
    pub async fn emit_event(&self, event: ServerEvent) -> Result<Vec<EventResponse>> {
        self.event_bus.emit(event).await
    }

    /// Get the event bus (for external components)
    pub fn get_event_bus(&self) -> Arc<EventBus> {
        Arc::clone(&self.event_bus)
    }

    /// Handle event responses (send responses to clients, emit protocol events, etc.)
    pub async fn handle_event_responses(&self, responses: Vec<EventResponse>) -> Result<()> {
        for response in responses {
            match response {
                EventResponse::Response(resp) => {
                    // TODO: Send response to the appropriate client
                    log::debug!("Generated response: {:?}", resp);
                }
                EventResponse::ProtocolEvent { event, target_clients } => {
                    if target_clients.is_empty() {
                        // Broadcast to all clients
                        log::debug!("Broadcasting protocol event: {:?}", event);
                        self.broadcast_event(event).await?;
                    } else {
                        // Send to specific clients
                        for client_id in target_clients {
                            log::debug!("Sending protocol event to client {}: {:?}", client_id, event);
                            self.send_event_to_client(client_id, &event).await?;
                        }
                    }
                }                EventResponse::ServerEvent(server_event) => {
                    // Chain another server event
                    log::debug!("Chaining server event: {:?}", server_event);
                    // TODO: Emit chained event (requires careful handling to avoid infinite recursion)
                    // let chained_responses = self.event_bus.emit(server_event).await?;
                    // self.handle_event_responses(chained_responses).await?;
                    log::warn!("Chained server events not yet implemented to avoid recursion");
                }
            }
        }
        Ok(())
    }

    /// Send an event to a specific client
    async fn send_event_to_client(&self, client_id: u32, event: &Event) -> Result<()> {        let clients = self.clients.read().await;
        if let Some(_client) = clients.get(&client_id) {
            let event_data = event.serialize();
            // TODO: Send event_data to client
            log::debug!("Would send {} bytes to client {}", event_data.len(), client_id);
        } else {
            log::warn!("Client {} not found when trying to send event", client_id);
        }
        Ok(())
    }

    /// Broadcast an event to all clients
    async fn broadcast_event(&self, event: Event) -> Result<()> {
        let clients = self.clients.read().await;
        let event_data = event.serialize();
        
        for (client_id, _client) in clients.iter() {
            // TODO: Send event_data to each client
            log::debug!("Would broadcast {} bytes to client {}", event_data.len(), client_id);
        }
        
        Ok(())
    }
}
