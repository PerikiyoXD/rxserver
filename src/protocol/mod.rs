//! X11 protocol implementation
//!
//! This module contains the core X11 protocol handling, including request parsing,
//! response generation, event handling, and data type definitions.

pub mod events;
pub mod opcodes;
pub mod requests;
pub mod response_builder;
pub mod responses;
pub mod types;
pub mod wire;

// Export specific types to avoid naming conflicts
pub use events::Event as ProtocolEvent;
pub use requests::{Request, RequestParser};
pub use response_builder::ResponseBuilder;
pub use responses::{Event as ResponseEvent, Reply, Response};
pub use types::*;
pub use wire::{Deserialize, Serialize, X11BufExt, X11BufMutExt};
