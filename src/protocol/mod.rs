//! X11 Protocol Implementation
//!
//! This module provides a clean, well-organized implementation of the X11 protocol.
//! It separates concerns into logical modules for better maintainability and understanding.

// Core protocol modules
pub mod types;           // Basic X11 data types and constants
pub mod opcodes;         // Request and response opcodes
pub mod requests;        // Client request definitions and parsing
pub mod message;         // Response/event/error message definitions
pub mod serialization;   // Wire format serialization
pub mod builder;         // High-level response construction utilities
pub mod connection;      // Connection setup and authentication
pub mod wire;           // Low-level wire format utilities

// Re-export commonly used types for convenience
pub use types::*;
pub use requests::{Request, RequestParser};
pub use message::{Response, Reply, Event, ErrorResponse};
pub use builder::ResponseBuilder;
pub use serialization::ResponseSerializer;
pub use connection::{
    ConnectionHandler, ConnectionSetupRequest, ConnectionSetupResponse, ConnectionSetupStatus,
};
pub use wire::{Deserialize, Serialize, X11BufExt, X11BufMutExt};
