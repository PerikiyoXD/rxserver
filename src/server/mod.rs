//! Core X11 Server Implementation
//!
//! This module provides a clean, modular X11 server architecture that is easier to
//! understand and maintain than the original X server. The design focuses on:
//!
//! - **Type Safety**: Using Rust's type system to prevent common X server bugs
//! - **Modularity**: Clear separation of concerns between different components
//! - **Performance**: Async/await for handling many concurrent clients
//! - **Maintainability**: Well-documented, tested code with clear APIs
//! - **Memory Safety**: No buffer overflows, use-after-free, or memory leaks

pub mod client;
pub mod connection;
pub mod connection_handler;
pub mod connection_manager;
pub mod core;
pub mod display;
pub mod display_manager;
pub mod event_loop;
pub mod events;
pub mod request_handler;
pub mod resources;
pub mod state;

// Re-export the main types for easy access
pub use client::{ClientInfo, ClientManager};
pub use connection_handler::X11ConnectionHandler;
pub use connection_manager::ConnectionManager;
pub use core::{ServerStats, XServer, XServerBuilder};
pub use display_manager::DisplayManager;
pub use crate::display::{ScreenInfo, VisualInfo};
pub use events::ServerEvent;
pub use request_handler::RequestHandler;
pub use resources::ResourceManager;
pub use state::ServerState;
