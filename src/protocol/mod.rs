//! X11 protocol implementation
//!
//! This module contains the core X11 protocol handling, including request parsing,
//! response generation, event handling, and data type definitions.

pub mod events;
pub mod opcodes;
pub mod requests;
pub mod responses;
pub mod response_builder;
pub mod types;

// Export specific types to avoid naming conflicts
pub use events::Event as ProtocolEvent;
pub use requests::{Request, RequestParser};
pub use responses::{Response, Reply, Event as ResponseEvent};
pub use response_builder::ResponseBuilder;
pub use types::*;
