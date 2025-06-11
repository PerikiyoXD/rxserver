//! # RX - Rust X Window System
//!
//! RX is a modern, safe, and efficient implementation of the X11 protocol written in Rust.
//! This library provides the core functionality for running an X Window System server.
//!
//! ## Architecture
//!
//! The library is organized into several key modules:
//!
//! - [`protocol`] - X11 protocol implementation (requests, responses, events)
//! - [`server`] - Core server implementation and client connection management
//! - [`window`] - Window management and properties
//! - [`graphics`] - Graphics rendering and context management
//! - [`input`] - Input handling (keyboard and mouse events)
//! - [`config`] - Configuration management
//! - [`utils`] - Utility modules and helper functions
//!
//! ## Example
//!
//! ```rust,no_run
//! use rxserver::{config::ServerConfig, server::XServer};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = ServerConfig::default();
//!     let mut server = XServer::new(":0".to_string(), config).await?;
//!     server.run().await?;
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod core;
pub mod graphics;
pub mod input;
pub mod logging;
pub mod protocol;
pub mod server;
pub mod utils;
pub mod window;

// Re-export commonly used types
pub use config::ServerConfig;
pub use server::XServer;

/// Common result type used throughout the library
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the RX library
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Graphics error: {0}")]
    Graphics(String),

    #[error("Window management error: {0}")]
    Window(String),

    #[error("Input error: {0}")]
    Input(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}
