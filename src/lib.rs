//! RxServer - A Modern X11 Server Implementation
//!
//! This crate provides a complete X11 server implementation written in Rust,
//! following clean architecture principles and designed for cross-platform deployment.
//!
//! # Architecture Overview
//!
//! The server is organized into distinct domains with clear boundaries:
//! - **Protocol Layer**: X11 wire protocol handling, parsing, and serialization
//! - **Resource Management**: XID allocation, resource lifecycle, and cleanup
//! - **Request Handling**: Request dispatch, validation, and response generation
//! - **Event System**: Event generation, queuing, and delivery
//! - **State Management**: Server state, client tracking, and synchronization
//! - **Geometry**: Geometric calculations and transformations
//! - **Security**: Access control, authentication, and authorization
//! - **Extensions**: X11 extensions framework and built-in extensions
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use rxserver::{RxServer, ServerConfig, ServerBuilder};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize logging
//!     rxserver::init_logging(None)?;
//!     
//!     // Create and start server
//!     let mut server = ServerBuilder::new()
//!         .with_config(ServerConfig::default())
//!         .build_and_start()
//!         .await?;
//!     
//!     // Server will run until shutdown
//!     server.run().await?;
//!     
//!     Ok(())
//! }
//! ```

#![warn(rust_2018_idioms, unreachable_pub, missing_debug_implementations)]

pub mod config;
pub mod diagnostics;
pub mod display;
pub mod fonts;
pub mod input;
pub mod logging;
pub mod network;
pub mod platform;
pub mod server;
pub mod types;
pub mod x11;

// Core re-exports for public API
pub use diagnostics::health::{HealthCommand, HealthService, HealthSeverity, OverallHealth};
pub use display::{DisplayConfig, DisplayManager};
pub use fonts::FontSystem;
pub use input::{InputConfiguration, InputSystem};
pub use network::{ConnectionManager, NetworkConfig, NetworkServer};
pub use platform::Platform;
pub use server::configuration::ServerConfig;
pub use server::{Server, ServerBuilder, ServerResult};

// Re-export commonly used types for convenience
pub use x11::geometry::types::{Point, Rectangle};
pub use x11::protocol::errors::ProtocolError;
pub use x11::protocol::types::*;

/// Error types used throughout the server
pub mod error {
    //! Centralized error handling for the X11 server

    // Re-export ServerError from server module
    pub use crate::server::types::ServerError;

    /// Protocol parsing and handling errors (re-exported from protocol module)
    pub use crate::x11::protocol::errors::ProtocolError;

    // Re-export ResourceError from types module
    pub use crate::types::ResourceError;
}

/// Initialize the RxServer with default configuration
pub async fn init_server() -> ServerResult<Server> {
    ServerBuilder::new()
        .with_config(ServerConfig::default())
        .build()
}

/// Initialize the RxServer with custom configuration
pub async fn init_server_with_config(config: ServerConfig) -> ServerResult<Server> {
    ServerBuilder::new().with_config(config).build()
}

/// Create and start a server with default configuration
pub async fn start_server() -> ServerResult<Server> {
    let mut server = init_server().await?;
    server.start().await?;
    Ok(server)
}

/// Create and start a server with custom configuration
pub async fn start_server_with_config(config: ServerConfig) -> ServerResult<Server> {
    let mut server = init_server_with_config(config).await?;
    server.start().await?;
    Ok(server)
}

/// Initialize global health monitoring
pub async fn init_health_monitoring() -> types::Result<()> {
    diagnostics::health::init_global_health().await
}

/// Initialize global configuration
pub async fn init_global_configuration() -> types::Result<()> {
    config::init_global_config()
}

/// Initialize global diagnostics
pub async fn init_global_diagnostics() -> types::Result<()> {
    diagnostics::init_global_diagnostics().await
}

/// Initialize global logging
pub fn init_global_logging(
    config: Option<&server::configuration::LoggingConfig>,
) -> types::Result<()> {
    // Convert server logging config to our logging config
    let log_config = if let Some(cfg) = config {
        logging::types::LoggingConfig {
            level: logging::types::LogLevel::from(cfg.level.as_str()),
            async_logging: true,
            buffer_size: 1024,
            max_files: 10,
            max_file_size: 100 * 1024 * 1024,
            log_directory: cfg
                .file
                .clone()
                .unwrap_or_else(|| "logs".into())
                .parent()
                .unwrap_or_else(|| std::path::Path::new("logs"))
                .to_path_buf(),
            file_name_pattern: cfg
                .file
                .clone()
                .unwrap_or_else(|| "rxserver.log".into())
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("rxserver.log"))
                .to_string_lossy()
                .to_string(),
            console_output: true,
            structured_logging: cfg.structured,
            compression: true,
        }
    } else {
        logging::types::LoggingConfig::default()
    };

    logging::init_with_config(log_config).map(|_| ())
}

/// Complete server initialization with all subsystems
pub async fn init_complete_server(config: ServerConfig) -> ServerResult<Server> {
    // Initialize global systems first
    init_global_configuration().await.map_err(|e| {
        server::types::ServerError::Initialization(format!("Global config init failed: {}", e))
    })?;

    init_global_diagnostics().await.map_err(|e| {
        server::types::ServerError::Initialization(format!("Global diagnostics init failed: {}", e))
    })?;

    init_health_monitoring().await.map_err(|e| {
        server::types::ServerError::Initialization(format!("Health monitoring init failed: {}", e))
    })?;

    // Create and initialize server
    let mut server = init_server_with_config(config).await?;
    server.initialize().await?;

    Ok(server)
}

/// Get global health status
pub async fn get_global_health_status() -> Option<types::Result<OverallHealth>> {
    if let Some(service) = diagnostics::health::global_health_service() {
        Some(service.get_health_status().await)
    } else {
        None
    }
}

/// Send command to global health service
pub fn send_health_command(
    command: HealthCommand,
) -> Result<(), tokio::sync::mpsc::error::SendError<HealthCommand>> {
    if let Some(sender) = diagnostics::health::global_health_command_sender() {
        sender.send(command)
    } else {
        Err(tokio::sync::mpsc::error::SendError(command))
    }
}
