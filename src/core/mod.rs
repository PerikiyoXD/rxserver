//! Core functionality for the RX X11 Server
//!
//! This module contains fundamental components like configuration,
//! command line arguments, logging setup, and error handling.

pub mod args;
pub mod config;
pub mod error;
pub mod logging;

// Re-export commonly used types from core modules
pub use args::CommandlineArgs;
pub use config::{
    LoggingConfig, NetworkConfig, PerformanceConfig, PluginConfig, SecurityConfig, ServerConfig,
};
pub use error::{ServerError, ServerResult};
pub use logging::init_logging;
