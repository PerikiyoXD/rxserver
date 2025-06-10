//! Core X11 Server Implementation
//!
//! This module provides a clean, modular X11 server architecture that is easier to
//! understand and maintain than the original X server.

use std::sync::Arc;
#[cfg(unix)]
use tokio::net::UnixListener;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::config::ServerConfig;
use crate::{Error, Result};

// Import from sibling modules
use super::{
    client::{ClientManager, ClientInfo},
    state::ServerState,
};

/// The main X11 server
///
/// This is the central component that coordinates all X11 functionality.
/// Unlike the original X server, this is designed to be modular, testable,
/// and memory-safe.
#[derive(Clone)]
pub struct XServer {
    /// Server configuration
    config: ServerConfig,
    /// Shared server state
    state: Arc<ServerState>,
    /// Client manager for handling multiple clients
    client_manager: Arc<ClientManager>,
    /// Connection manager for network handling
    connection_manager: Arc<ConnectionManager>,
    /// Display manager for screen/visual management
    display_manager: Arc<DisplayManager>,
    /// Resource manager for X11 resources
    resource_manager: Arc<ResourceManager>,
    /// Request handler for processing X11 requests
    request_handler: Arc<RequestHandler>,
    /// Event broadcaster for server events
    event_sender: broadcast::Sender<ServerEvent>,
}

/// Server events that can be broadcast to components
#[derive(Debug, Clone)]
pub enum ServerEvent {
    ClientConnected { client_id: u32 },
    ClientDisconnected { client_id: u32 },
    WindowCreated { window_id: u32, client_id: u32 },
    WindowDestroyed { window_id: u32 },
    ServerShuttingDown,
}

impl XServer {
    /// Create a new X server instance
    pub async fn new(display_name: String, config: ServerConfig) -> Result<Self> {
        info!("Creating X server for display: {}", display_name);

        // Create event channel for server-wide communication
        let (event_sender, _) = broadcast::channel(1000);

        // Initialize shared state
        let state = Arc::new(ServerState::new(display_name.clone()));

        // Initialize managers
        let client_manager = Arc::new(ClientManager::new(event_sender.clone()));
        let connection_manager = Arc::new(ConnectionManager::new(&config)?);
        let display_manager = Arc::new(DisplayManager::new(&config.display)?);
        let resource_manager = Arc::new(ResourceManager::new());
        let request_handler = Arc::new(RequestHandler::new(
            state.clone(),
            resource_manager.clone(),
            display_manager.clone(),
            event_sender.clone(),
        ));

        Ok(XServer {
            config,
            state,
            client_manager,
            connection_manager,
            display_manager,
            resource_manager,
            request_handler,
            event_sender,
        })
    }

    /// Start the X server
    pub async fn run(&self) -> Result<()> {
        info!("Starting X server on display {}", self.state.display_name());

        // Mark server as running
        self.state.set_running(true).await;

        // Start the main event loop
        let event_loop = EventLoop::new(
            self.connection_manager.clone(),
            self.client_manager.clone(),
            self.request_handler.clone(),
            self.event_sender.clone(),
        );

        // Handle shutdown gracefully
        let result = event_loop.run().await;

        // Mark server as stopped
        self.state.set_running(false).await;
        
        // Broadcast shutdown event
        let _ = self.event_sender.send(ServerEvent::ServerShuttingDown);

        result
    }

    /// Stop the X server gracefully
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping X server");
        self.state.set_running(false).await;
        Ok(())
    }

    /// Get server statistics
    pub async fn stats(&self) -> ServerStats {
        let client_count = self.client_manager.client_count().await;
        let window_count = self.resource_manager.window_count().await;
        let pixmap_count = self.resource_manager.pixmap_count().await;

        ServerStats {
            uptime: self.state.uptime().await,
            client_count,
            window_count,
            pixmap_count,
            memory_usage: self.estimate_memory_usage().await,
        }
    }

    /// Check if server is running
    pub async fn is_running(&self) -> bool {
        self.state.is_running().await
    }

    /// Get display name
    pub fn display_name(&self) -> &str {
        self.state.display_name()
    }

    async fn estimate_memory_usage(&self) -> usize {
        // Simple memory usage estimation
        // In a real implementation, you'd want more sophisticated tracking
        let client_memory = self.client_manager.memory_usage().await;
        let resource_memory = self.resource_manager.memory_usage().await;
        client_memory + resource_memory
    }
}

/// Server statistics
#[derive(Debug, Clone)]
pub struct ServerStats {
    pub uptime: std::time::Duration,
    pub client_count: usize,
    pub window_count: usize,
    pub pixmap_count: usize,
    pub memory_usage: usize,
}

impl std::fmt::Display for ServerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Server Stats: uptime={:.2}s, clients={}, windows={}, pixmaps={}, memory={}KB",
            self.uptime.as_secs_f64(),
            self.client_count,
            self.window_count,
            self.pixmap_count,
            self.memory_usage / 1024
        )
    }
}

/// Builder for creating X server with custom configuration
pub struct XServerBuilder {
    display_name: String,
    config: ServerConfig,
}

impl XServerBuilder {
    pub fn new(display_name: String) -> Self {
        Self {
            display_name,
            config: ServerConfig::default(),
        }
    }

    pub fn with_config(mut self, config: ServerConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_max_clients(mut self, max_clients: usize) -> Self {
        self.config.server.max_clients = max_clients;
        self
    }

    pub async fn build(self) -> Result<XServer> {
        XServer::new(self.display_name, self.config).await
    }
}
