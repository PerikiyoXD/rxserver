//! RX X11 Server - A modern X11 server implementation in Rust
//!
//! This crate provides a complete X11 server implementation with plugin-based
//! architecture, async networking, and comprehensive protocol support.

pub mod core;
pub mod graphics;
pub mod network;
pub mod plugins;
pub mod protocol;
pub mod server;
pub mod window;

// Re-export commonly used types
pub use core::{CommandlineArgs, ServerConfig, ServerError, ServerResult};
pub use server::{DisplayMode, RXServer};
