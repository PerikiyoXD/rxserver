//! Plugin system interface
//!
//! This module provides the plugin system interface for loading, managing,
//! and coordinating plugins within the server.

pub mod api;
pub mod communication;
pub mod lifecycle;
pub mod loader;
pub mod manager;
pub mod registry;
pub mod sandboxing;

pub use self::api::*;
pub use self::communication::*;
pub use self::lifecycle::*;
pub use self::loader::*;
pub use self::manager::*;
pub use self::registry::*;
pub use self::sandboxing::*;

/// Plugin error types
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    #[error("Plugin loading failed: {0}")]
    Loading(String),
    #[error("Plugin lifecycle error: {0}")]
    Lifecycle(String),
    #[error("Plugin communication error: {0}")]
    Communication(String),
    #[error("Plugin sandboxing error: {0}")]
    Sandboxing(String),
    #[error("Plugin API error: {0}")]
    Api(String),
}

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;
