//! RxServer - A Modern X11 Server Implementation
//!
//! This crate provides a complete X11 server implementation written in Rust,
//! following clean architecture principles and designed for cross-platform deployment.
//!
//! # Architecture Overview
//!
//! The server is organized into distinct domains with clear boundaries:
//! - **Protocol Layer**: X11 wire protocol handling, parsing, and serialization
//! - **Resource Management**: XId allocation, resource lifecycle, and cleanup
//! - **Request Handling**: Request dispatch, validation, and response generation
//! - **Event System**: Event generation, queuing, and delivery
//! - **State Management**: Server state, client tracking, and synchronization
//! - **Geometry**: Geometric calculations and transformations
//! - **Security**: Access control, authentication, and authorization
//! - **Extensions**: X11 extensions framework and built-in extensions
//!
//! # Quick Start
//!
//! ## Simple Usage (Convenience Functions)
//!
//! ```rust,no_run
//! use rxserver;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Start a server with default configuration
//!     let mut server = rxserver::start_server().await?;
//!     
//!     // Server runs until shutdown
//!     server.run().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Usage (Builder Pattern)
//!
//! ```rust,no_run
//! use rxserver::{ServerConfig, ServerBuilder, diagnostics::health::HealthMonitor};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create custom configuration
//!     let config = ServerConfig::default();
//!     let health_monitor = HealthMonitor::new();
//!     
//!     // Build and start server with custom settings
//!     let mut server = ServerBuilder::new()
//!         .with_config(config)
//!         .with_health_config(health_monitor)
//!         .build_and_start()
//!         .await?;
//!     
//!     // Server runs until shutdown
//!     server.run().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Manual Initialization
//!
//! ```rust,no_run
//! use rxserver::{ServerConfig, init_server_with_config};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = ServerConfig::default();
//!     
//!     // Create server but don't start it yet
//!     let mut server = init_server_with_config(config).await?;
//!     
//!     // Perform custom initialization here...
//!     
//!     // Start when ready
//!     server.start().await?;
//!     server.run().await?;
//!     
//!     Ok(())
//! }
//! ```

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
///
/// Creates a new server instance but does not start it. Use this when you need
/// to perform custom initialization before starting the server.
///
/// # Example
///
/// ```rust,no_run
/// use rxserver::init_server;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut server = init_server().await?;
///     // Perform custom setup...
///     server.start().await?;
///     Ok(())
/// }
/// ```
pub async fn init_server() -> ServerResult<Server> {
    ServerBuilder::new()
        .with_config(ServerConfig::default())
        .build()
}

/// Initialize the RxServer with custom configuration
///
/// Creates a new server instance with the provided configuration but does not start it.
/// Use this when you need custom configuration and manual startup control.
///
/// # Example
///
/// ```rust,no_run
/// use rxserver::{init_server_with_config, ServerConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = ServerConfig::default();
///     let mut server = init_server_with_config(config).await?;
///     // Perform custom setup...
///     server.start().await?;
///     Ok(())
/// }
/// ```
pub async fn init_server_with_config(config: ServerConfig) -> ServerResult<Server> {
    ServerBuilder::new().with_config(config).build()
}

/// Create and start a server with default configuration
///
/// This is the simplest way to start an X11 server. The server will be created
/// with default settings and started immediately.
///
/// # Example
///
/// ```rust,no_run
/// use rxserver::start_server;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut server = start_server().await?;
///     server.run().await?;
///     Ok(())
/// }
/// ```
pub async fn start_server() -> ServerResult<Server> {
    ServerBuilder::new()
        .with_config(ServerConfig::default())
        .build_and_start()
        .await
}

/// Create and start a server with custom configuration
///
/// Creates and immediately starts a server with the provided configuration.
/// This is ideal when you have a custom configuration but don't need manual
/// startup control.
///
/// # Example
///
/// ```rust,no_run
/// use rxserver::{start_server_with_config, ServerConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = ServerConfig::default();
///     let mut server = start_server_with_config(config).await?;
///     server.run().await?;
///     Ok(())
/// }
/// ```
pub async fn start_server_with_config(config: ServerConfig) -> ServerResult<Server> {
    ServerBuilder::new()
        .with_config(config)
        .build_and_start()
        .await
}

/// Initialize global health monitoring
///
/// Sets up the global health monitoring service. This should be called before
/// starting the server if you want to use global health monitoring functions.
///
/// # Example
///
/// ```rust,no_run
/// use rxserver::init_health_monitoring;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init_health_monitoring().await?;
///     // Now you can use global health functions
///     Ok(())
/// }
/// ```
pub async fn init_health_monitoring() -> types::Result<()> {
    diagnostics::health::init_global_health().await
}

/// Get global health status
///
/// Returns the current health status of the global health monitoring service.
/// Returns `None` if global health monitoring has not been initialized.
///
/// # Example
///
/// ```rust,no_run
/// use rxserver::{init_health_monitoring, get_global_health_status};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init_health_monitoring().await?;
///     
///     if let Some(health_result) = get_global_health_status().await {
///         let health = health_result?;
///         println!("Health status: {:?}", health.severity());
///     }
///     Ok(())
/// }
/// ```
pub async fn get_global_health_status() -> Option<types::Result<OverallHealth>> {
    if let Some(service) = diagnostics::health::global_health_service() {
        Some(service.get_health_status().await)
    } else {
        None
    }
}

/// Send command to global health service
///
/// Sends a command to the global health monitoring service. Returns an error
/// if the global health service has not been initialized.
///
/// # Example
///
/// ```rust,no_run
/// use rxserver::{init_health_monitoring, send_health_command, HealthCommand};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init_health_monitoring().await?;
///     
///     let interval = Duration::from_secs(30);
///     send_health_command(HealthCommand::SetInterval { interval })?;
///     Ok(())
/// }
/// ```
pub fn send_health_command(
    command: HealthCommand,
) -> Result<(), tokio::sync::mpsc::error::SendError<HealthCommand>> {
    if let Some(sender) = diagnostics::health::global_health_command_sender() {
        sender.send(command)
    } else {
        Err(tokio::sync::mpsc::error::SendError(command))
    }
}
