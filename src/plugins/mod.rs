//! Plugin system for the RX X11 Server
//!
//! This module provides a plugin architecture for extending
//! the server with additional functionality.

pub mod atom_registry;
pub mod cursor_manager;
pub mod error;
pub mod font_manager;
pub mod registry;
pub mod window;

pub use atom_registry::*;
pub use cursor_manager::*;
pub use error::*;
pub use font_manager::*;
pub use registry::*;
pub use window::*;
