//! Server Infrastructure
//!
//! This module provides the core server infrastructure including lifecycle management,
//! configuration, monitoring, and plugin systems.

pub mod api;
pub mod configuration;
pub mod coordination;
pub mod lifecycle;
pub mod monitoring;
pub mod plugins;
pub mod runtime;
pub mod services;
pub mod types;

use crate::diagnostics::health::{HealthCommand, HealthService};
use crate::display::DisplayManager;
use crate::fonts::FontSystem;
use crate::input::InputSystem;
use crate::network::{ConnectionManager, NetworkServer};
use crate::platform::Platform;
use crate::x11::protocol::ProtocolHandler;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main server state
#[derive(Debug)]
pub struct ServerState {
    /// Server configuration
    pub config: Arc<configuration::ServerConfig>,
    /// Running services
    pub services: Arc<RwLock<services::ServiceRegistry>>,
    /// Plugin manager
    pub plugins: Arc<RwLock<plugins::PluginManager>>,
    /// Runtime managers
    pub scheduler: Arc<runtime::TaskScheduler>,
    pub executor: Arc<runtime::TaskExecutor>,
    /// System components
    pub display_manager: Option<DisplayManager>,
    pub font_system: Option<FontSystem>,
    pub input_system: Option<InputSystem>,
    pub network_server: Option<NetworkServer>,
    pub connection_manager: Option<ConnectionManager>,
    pub protocol_handler: Arc<tokio::sync::Mutex<ProtocolHandler>>,
    pub platform: Platform,
}

impl ServerState {
    /// Create new server state
    pub fn new(config: configuration::ServerConfig) -> Self {
        let config_arc = Arc::new(config);

        Self {
            config: config_arc.clone(),
            services: Arc::new(RwLock::new(services::ServiceRegistry::new())),
            plugins: Arc::new(RwLock::new(plugins::PluginManager::new())),
            scheduler: Arc::new(runtime::TaskScheduler::new()),
            executor: Arc::new(runtime::TaskExecutor::new()),
            display_manager: None,
            font_system: None,
            input_system: None,
            network_server: None,
            connection_manager: None,
            protocol_handler: Arc::new(tokio::sync::Mutex::new(ProtocolHandler::with_config(
                config_arc,
            ))),
            platform: Platform::current(),
        }
    }
}

/// Server initialization result
pub type ServerResult<T> = Result<T, types::ServerError>;

/// Server main structure
#[derive(Debug)]
pub struct Server {
    state: ServerState,
    health_service: Option<HealthService>,
}

impl Server {
    /// Create a new server instance
    pub fn new(config: configuration::ServerConfig) -> Self {
        Self {
            state: ServerState::new(config),
            health_service: None,
        }
    }

    /// Initialize the server with all services
    pub async fn initialize(&mut self) -> ServerResult<()> {
        tracing::info!("Initializing X11 server");

        // Initialize platform detection
        tracing::info!("Detected platform: {:?}", self.state.platform);

        // Initialize health monitoring first
        let mut health_service = HealthService::new();
        health_service.start().await.map_err(|e| {
            types::ServerError::Initialization(format!("Health service failed: {}", e))
        })?;

        self.health_service = Some(health_service);

        // Initialize core subsystems
        self.initialize_subsystems().await?;

        // Initialize other services
        self.initialize_services().await?;

        // Register server health checks
        if let Some(ref health_service) = self.health_service {
            let command_tx = health_service.get_command_sender();

            // Add server-specific health checks
            let _ = command_tx.send(HealthCommand::AddCheck {
                name: "server_initialization".to_string(),
            });
            let _ = command_tx.send(HealthCommand::AddCheck {
                name: "service_registry".to_string(),
            });
            let _ = command_tx.send(HealthCommand::AddCheck {
                name: "plugin_system".to_string(),
            });
            let _ = command_tx.send(HealthCommand::AddCheck {
                name: "display_system".to_string(),
            });
            let _ = command_tx.send(HealthCommand::AddCheck {
                name: "font_system".to_string(),
            });
            let _ = command_tx.send(HealthCommand::AddCheck {
                name: "input_system".to_string(),
            });
            let _ = command_tx.send(HealthCommand::AddCheck {
                name: "network_server".to_string(),
            });
            let _ = command_tx.send(HealthCommand::AddCheck {
                name: "connection_manager".to_string(),
            });
        }

        tracing::info!("Server initialization completed successfully");
        Ok(())
    }

    /// Start the server
    pub async fn start(&mut self) -> ServerResult<()> {
        tracing::info!("Starting X11 server");

        // Ensure server is initialized
        if self.health_service.is_none() {
            self.initialize().await?;
        }

        // Start lifecycle management
        self.start_lifecycle().await?;

        // Start monitoring and health checks
        self.start_monitoring().await?;

        tracing::info!("X11 server started successfully");
        Ok(())
    }

    /// Stop the server gracefully
    pub async fn stop(&mut self) -> ServerResult<()> {
        tracing::info!("Stopping X11 server");

        // Stop health monitoring
        if let Some(ref mut health_service) = self.health_service {
            health_service.stop().await.map_err(|e| {
                types::ServerError::Shutdown(format!("Health service stop failed: {}", e))
            })?;
        }

        // Stop other services
        self.stop_services().await?;

        tracing::info!("X11 server stopped successfully");
        Ok(())
    }

    /// Get current server health status
    pub async fn get_health_status(
        &self,
    ) -> ServerResult<crate::diagnostics::health::OverallHealth> {
        if let Some(ref health_service) = self.health_service {
            health_service.get_health_status().await.map_err(|e| {
                types::ServerError::HealthCheck(format!("Health status failed: {}", e))
            })
        } else {
            Err(types::ServerError::HealthCheck(
                "Health service not initialized".to_string(),
            ))
        }
    }

    /// Get health command sender for external control
    pub fn get_health_command_sender(
        &self,
    ) -> Option<tokio::sync::mpsc::UnboundedSender<HealthCommand>> {
        self.health_service
            .as_ref()
            .map(|service| service.get_command_sender())
    }

    /// Check if server is healthy
    pub async fn is_healthy(&self) -> bool {
        if let Ok(health) = self.get_health_status().await {
            health.severity() < crate::diagnostics::health::HealthSeverity::Critical
        } else {
            false
        }
    }

    /// Initialize core subsystems
    async fn initialize_subsystems(&mut self) -> ServerResult<()> {
        tracing::info!("Initializing core subsystems"); // Initialize display manager
        let display_config = self
            .state
            .config
            .display
            .clone()
            .unwrap_or_else(|| crate::display::DisplayConfig::default());
        let mut display_manager = DisplayManager::new(display_config);
        display_manager.initialize().await.map_err(|e| {
            types::ServerError::Initialization(format!("Display manager init failed: {}", e))
        })?;
        self.state.display_manager = Some(display_manager);
        tracing::info!("Display manager initialized");

        // Initialize font system
        let mut font_system = FontSystem::new().map_err(|e| {
            types::ServerError::Initialization(format!("Font system creation failed: {}", e))
        })?;
        font_system.initialize().await.map_err(|e| {
            types::ServerError::Initialization(format!("Font system init failed: {}", e))
        })?;
        self.state.font_system = Some(font_system);
        tracing::info!("Font system initialized"); // Initialize input system
        let input_config = self
            .state
            .config
            .input
            .clone()
            .unwrap_or_else(|| crate::input::InputConfiguration::default());
        let mut input_system = InputSystem::new(input_config).map_err(|e| {
            types::ServerError::Initialization(format!("Input system creation failed: {}", e))
        })?;
        input_system.initialize().await.map_err(|e| {
            types::ServerError::Initialization(format!("Input system init failed: {}", e))
        })?;
        self.state.input_system = Some(input_system);
        tracing::info!("Input system initialized"); // Initialize network system
        let mut network_server = NetworkServer::new();

        // Bind to TCP address
        let tcp_addr = self.state.config.network.bind_address;
        network_server
            .bind_tcp(&tcp_addr.to_string())
            .await
            .map_err(|e| {
                types::ServerError::Initialization(format!("Network TCP bind failed: {}", e))
            })?;
        // Bind to Unix socket if configured (Unix only)
        #[cfg(unix)]
        {
            use std::path::PathBuf;
            let unix_path = PathBuf::from("/tmp/.X11-unix/X0");
            network_server.bind_unix(&unix_path).await.map_err(|e| {
                types::ServerError::Initialization(format!("Network Unix bind failed: {}", e))
            })?;
        }

        // Initialize connection manager
        let (transport_tx, transport_rx) = tokio::sync::mpsc::unbounded_channel();

        // Set the transport sender on the network server
        network_server.set_transport_sender(transport_tx);

        // Store the network server
        self.state.network_server = Some(network_server);
        tracing::info!("Network server initialized");

        let (connection_manager, _connection_rx) = ConnectionManager::new(
            crate::network::connection::ConnectionManagerConfig::default(),
            transport_rx,
        );
        self.state.connection_manager = Some(connection_manager);
        tracing::info!("Connection manager initialized");

        tracing::info!("Core subsystems initialization completed");
        Ok(())
    }

    /// Initialize server services
    async fn initialize_services(&mut self) -> ServerResult<()> {
        tracing::debug!("Initializing server services");

        // Initialize service registry
        {
            let mut services = self.state.services.write().await;
            services.initialize().await.map_err(|e| {
                types::ServerError::Service(format!("Service registry init failed: {}", e))
            })?;
        }

        // Initialize plugin manager
        {
            let mut plugins = self.state.plugins.write().await;
            plugins.initialize().await.map_err(|e| {
                types::ServerError::Plugin(format!("Plugin manager init failed: {}", e))
            })?;
        }

        // Initialize runtime components
        self.state
            .scheduler
            .initialize()
            .await
            .map_err(|e| types::ServerError::Runtime(format!("Scheduler init failed: {}", e)))?;

        self.state
            .executor
            .initialize()
            .await
            .map_err(|e| types::ServerError::Runtime(format!("Executor init failed: {}", e)))?;

        Ok(())
    }
    /// Start lifecycle management
    async fn start_lifecycle(&mut self) -> ServerResult<()> {
        tracing::debug!("Starting server lifecycle management");

        // Start the connection manager
        if let Some(ref mut connection_manager) = self.state.connection_manager {
            connection_manager.start().await.map_err(|e| {
                types::ServerError::Service(format!("Failed to start connection manager: {}", e))
            })?;
            tracing::info!("Connection manager started");
        }

        // Start the main connection acceptance loop
        self.start_connection_loop().await?;

        Ok(())
    }
    /// Start the main connection acceptance loop
    async fn start_connection_loop(&mut self) -> ServerResult<()> {
        if let Some(mut network_server) = self.state.network_server.take() {
            let is_running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
            let is_running_clone = is_running.clone();
            let protocol_handler = self.state.protocol_handler.clone();

            tokio::spawn(async move {
                tracing::info!("Starting main connection acceptance loop");

                while is_running_clone.load(std::sync::atomic::Ordering::SeqCst) {
                    match network_server.accept().await {
                        Ok(connection) => {
                            tracing::info!(
                                "New connection accepted: id={}, peer={:?}",
                                connection.id(),
                                connection.peer_addr()
                            );

                            // The connection is automatically registered via transport events
                            // sent from network_server.accept() to the ConnectionManager

                            // Spawn a task to handle this specific connection
                            Self::spawn_connection_handler(connection, protocol_handler.clone())
                                .await;
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // No connection available, sleep briefly and try again
                            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        }
                        Err(e) => {
                            tracing::error!("Error accepting connection: {}", e);
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        }
                    }
                }

                tracing::info!("Connection acceptance loop stopped");
            });

            // Store the flag for potential cleanup later
            // Note: we don't put the network_server back since it's moved into the task
            tracing::info!("Connection acceptance loop started");
        }

        Ok(())
    }

    /// Spawn a task to handle an individual X11 connection
    async fn spawn_connection_handler(
        mut connection: crate::network::Connection,
        protocol_handler: Arc<tokio::sync::Mutex<ProtocolHandler>>,
    ) {
        let connection_id = connection.id();

        tokio::spawn(async move {
            tracing::info!(
                "Starting connection handler for connection {}",
                connection_id
            );

            loop {
                // Read data from the connection
                match connection.read().await {
                    Ok(data) => {
                        if data.is_empty() {
                            tracing::debug!(
                                "Connection {} sent empty data, continuing",
                                connection_id
                            );
                            continue;
                        }

                        tracing::debug!(
                            "Received {} bytes from connection {}",
                            data.len(),
                            connection_id
                        );

                        // Process the data through the protocol handler
                        let response = {
                            let mut handler = protocol_handler.lock().await;
                            match handler.process_data(connection_id, data).await {
                                Ok(response) => response,
                                Err(e) => {
                                    tracing::error!(
                                        "Protocol error for connection {}: {}",
                                        connection_id,
                                        e
                                    );
                                    break; // Close connection on protocol error
                                }
                            }
                        };

                        // Send response if there is one
                        if !response.is_empty() {
                            tracing::trace!(
                                "Sending {} bytes response to connection {}",
                                response.len(),
                                connection_id
                            );

                            if let Err(e) = connection.write(&response).await {
                                tracing::error!(
                                    "Failed to write response to connection {}: {}",
                                    connection_id,
                                    e
                                );
                                break; // Close connection on write error
                            }

                            if let Err(e) = connection.flush().await {
                                tracing::error!(
                                    "Failed to flush connection {}: {}",
                                    connection_id,
                                    e
                                );
                                break; // Close connection on flush error
                            }

                            tracing::debug!(
                                "Sent {} bytes response to connection {}",
                                response.len(),
                                connection_id
                            );
                        }
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::UnexpectedEof {
                            tracing::info!("Connection {} closed by client", connection_id);
                        } else {
                            tracing::error!("Read error on connection {}: {}", connection_id, e);
                        }
                        break; // Close connection on read error
                    }
                }
            }

            tracing::info!(
                "Connection handler for connection {} terminated",
                connection_id
            );
        });
    }

    /// Start monitoring systems
    async fn start_monitoring(&mut self) -> ServerResult<()> {
        tracing::debug!("Starting server monitoring");

        // Start performance monitoring
        // Start metrics collection
        // Start alerting systems
        // For now, this is a placeholder

        Ok(())
    }
    /// Stop server services
    async fn stop_services(&mut self) -> ServerResult<()> {
        tracing::debug!("Stopping server services");

        // Stop subsystems first
        self.stop_subsystems().await?;

        // Stop services in reverse order of initialization
        // Implementation would gracefully shut down all services

        Ok(())
    }

    /// Stop core subsystems
    async fn stop_subsystems(&mut self) -> ServerResult<()> {
        tracing::info!("Stopping core subsystems");

        // Stop input system
        if let Some(mut input_system) = self.state.input_system.take() {
            input_system.shutdown().await.map_err(|e| {
                types::ServerError::Shutdown(format!("Input system shutdown failed: {}", e))
            })?;
            tracing::info!("Input system stopped");
        }

        // Stop font system (doesn't have async shutdown, so we just drop it)
        if let Some(_font_system) = self.state.font_system.take() {
            tracing::info!("Font system stopped");
        }

        // Stop display manager (doesn't have shutdown method yet, so we just drop it)
        if let Some(_display_manager) = self.state.display_manager.take() {
            tracing::info!("Display manager stopped");
        }

        tracing::info!("Core subsystems shutdown completed");
        Ok(())
    }

    /// Run the server main loop
    pub async fn run(&mut self) -> ServerResult<()> {
        tracing::info!("Starting server main loop");

        // Ensure server is started
        if self.health_service.is_none() {
            return Err(types::ServerError::Runtime(
                "Server not initialized".to_string(),
            ));
        }

        // Main server event loop
        loop {
            // Process X11 requests
            // Handle client connections
            // Manage resources
            // Process events

            // For now, this is a placeholder that runs indefinitely
            // In a real implementation, this would:
            // 1. Accept new client connections
            // 2. Process X11 protocol requests
            // 3. Generate and dispatch events
            // 4. Manage server state
            // 5. Handle cleanup and maintenance

            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            // Check if we should stop (this would be controlled by signals or commands)
            // For now, just continue running
        }
    }
}

/// Server builder for configuration and setup
#[derive(Debug)]
pub struct ServerBuilder {
    config: Option<configuration::ServerConfig>,
    health_config: Option<crate::diagnostics::health::HealthMonitor>,
}

impl ServerBuilder {
    /// Create a new server builder
    pub fn new() -> Self {
        Self {
            config: None,
            health_config: None,
        }
    }

    /// Set server configuration
    pub fn with_config(mut self, config: configuration::ServerConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set health monitoring configuration
    pub fn with_health_config(
        mut self,
        health_config: crate::diagnostics::health::HealthMonitor,
    ) -> Self {
        self.health_config = Some(health_config);
        self
    }

    /// Build the server
    pub fn build(self) -> ServerResult<Server> {
        let config = self
            .config
            .unwrap_or_else(|| configuration::ServerConfig::default());
        Ok(Server::new(config))
    }

    /// Build and start the server
    pub async fn build_and_start(self) -> ServerResult<Server> {
        let mut server = self.build()?;
        server.start().await?;
        Ok(server)
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
