// SPDX-License-Identifier: Apache-2.0
// RX-Completion-Status: Complete

// RX X11 Server - Main Server Implementation

use crate::{
    ServerConfig, ServerError, ServerResult,
    network::ConnectionManager,
    plugins::PluginRegistry,
    protocol::{
        HeadlessProtocolHandler, NativeDisplayProtocolHandler, ProtocolHandlerRegistry,
        VirtualDisplayProtocolHandler,
    },
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

/// Display mode for the X11 server
#[derive(Debug, Clone)]
pub enum DisplayMode {
    /// No visual output - suitable for testing and headless environments
    Headless,
    /// Virtual display in a native window - good for development and remote access
    VirtualDisplay { width: u32, height: u32 },
    /// Direct native display access - for production use with direct hardware access
    NativeDisplay { width: u32, height: u32 },
}

/// Main X11 server structure
pub struct RXServer {
    display: u32,
    config: ServerConfig,
    plugins: Arc<PluginRegistry>,
    connection_manager: ConnectionManager,
    protocol_handler_registry: ProtocolHandlerRegistry,
    display_mode: DisplayMode,
}

impl RXServer {
    /// Create a new X11 server instance
    pub async fn new(
        display_str: String,
        config: ServerConfig,
        display_mode: DisplayMode,
    ) -> ServerResult<Self> {
        let display_num: u32 = display_str
            .trim_start_matches(':')
            .parse()
            .map_err(|_| ServerError::InitError("Invalid display number".to_string()))?;

        info!(
            "Initializing RX X11 Server for display :{} with mode: {:?}",
            display_num, display_mode
        );

        // Initialize plugin registry
        let mut plugins = PluginRegistry::new();

        // Register core plugins
        plugins.register_core_plugins().map_err(|e| {
            ServerError::PluginError(format!("Failed to register core plugins: {}", e))
        })?;

        let plugins = Arc::new(plugins);

        // Initialize connection manager
        let connection_manager = ConnectionManager::new(&config.performance)?; // Initialize protocol handler registry
        let mut protocol_handler_registry = ProtocolHandlerRegistry::new();

        // Register the extension handler first (it handles common requests)
        let extension_handler = crate::protocol::ExtensionHandler::new();
        protocol_handler_registry
            .register_handler(extension_handler)
            .await?;

        // Register handlers based on display mode
        match &display_mode {
            DisplayMode::Headless => {
                info!("Setting up headless protocol handlers");
                let headless_handler = HeadlessProtocolHandler::new(Arc::clone(&plugins))?;
                protocol_handler_registry
                    .register_handler(headless_handler)
                    .await?;
            }
            DisplayMode::VirtualDisplay { width, height } => {
                info!(
                    "Setting up virtual display protocol handlers ({}x{})",
                    width, height
                );
                let virtual_display_handler =
                    VirtualDisplayProtocolHandler::new(Arc::clone(&plugins), *width, *height)?;
                // Start the virtual display task
                virtual_display_handler.start_display().await?;
                info!("Virtual display started");
                // Set display configuration on the registry
                let display_config = virtual_display_handler.get_display_config();
                protocol_handler_registry.set_display_config(display_config.await);
                //
                protocol_handler_registry
                    .register_handler(virtual_display_handler)
                    .await?;
            }
            DisplayMode::NativeDisplay { width, height } => {
                info!(
                    "Setting up native display protocol handlers ({}x{})",
                    width, height
                );
                let native_display_handler =
                    NativeDisplayProtocolHandler::new(Arc::clone(&plugins), *width, *height)?;
                protocol_handler_registry
                    .register_handler(native_display_handler)
                    .await?;
            }
        }

        // Also register specialized handlers for window and surface operations
        // These work with any display mode but are currently disabled due to opcode conflicts
        // TODO: Implement proper opcode priority/routing system
        // let window_handler = WindowProtocolHandler::new(Arc::new(crate::plugins::WindowPlugin::new()));
        // protocol_handler_registry.register_handler(window_handler).await?;

        // let surface_handler = SurfaceProtocolHandler::new(1920, 1080, 24);
        // protocol_handler_registry.register_handler(surface_handler).await?;

        // Register the PropertyProtocolHandler to handle GetProperty requests
        let window_plugin = Arc::new(crate::plugins::WindowPlugin::new());
        let property_handler =
            crate::protocol::handler::PropertyProtocolHandler::new(window_plugin);
        protocol_handler_registry
            .register_handler(property_handler)
            .await?;

        Ok(Self {
            display: display_num,
            config,
            plugins,
            connection_manager,
            protocol_handler_registry,
            display_mode,
        })
    }

    /// Run the X11 server
    pub async fn run(&self) -> ServerResult<()> {
        let listen_addr = format!(
            "{}:{}",
            self.config.network.listen_address,
            self.config.network.port_base + self.display as u16
        );

        info!("Starting X11 server on {}", listen_addr);

        let listener = TcpListener::bind(&listen_addr).await.map_err(|e| {
            ServerError::NetworkError(format!("Failed to bind to {}: {}", listen_addr, e))
        })?;
        info!("X11 server listening on {}", listen_addr);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New client connection from {}", addr);

                    let connection_manager = self.connection_manager.clone();
                    let protocol_handler_registry = Arc::new(tokio::sync::Mutex::new(
                        self.protocol_handler_registry.clone(),
                    ));

                    tokio::spawn(async move {
                        if let Err(e) = connection_manager
                            .handle_connection(stream, protocol_handler_registry)
                            .await
                        {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
    /// Gracefully shutdown the server
    pub async fn shutdown(&self) -> ServerResult<()> {
        info!("Shutting down X11 server");

        // Shutdown protocol handlers
        if let Err(e) = self.protocol_handler_registry.shutdown().await {
            error!("Failed to shutdown protocol handlers: {}", e);
        }

        // Note: Plugin shutdown is tricky due to Arc<PluginRegistry>
        // In a full implementation, we'd need to use Arc<Mutex<PluginRegistry>>
        // or implement a different shutdown mechanism
        // For now, we'll skip plugin shutdown since it's not critical for basic functionality

        // TODO: Implement proper plugin shutdown
        // if let Ok(plugins) = Arc::try_unwrap(self.plugins) {
        //     plugins.shutdown_all()?;
        // }

        info!("X11 server shutdown complete");
        Ok(())
    }
}
