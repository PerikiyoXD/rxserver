//! X11 Protocol Messages
//!
//! This module contains the core X11 protocol message types and structures.
//! It provides clean separation between message definitions and serialization logic.

pub mod events;
pub mod replies;
pub mod errors;

// Re-export message types
pub use events::Event;
pub use replies::Reply;
pub use errors::ErrorResponse;

use std::fmt::{Display, Formatter, Result};

/// Generic X11 response type
#[derive(Debug, Clone)]
pub enum Response {
    /// Reply to a request
    Reply(Reply),
    /// Event notification
    Event(Event),
    /// Error response
    Error(ErrorResponse),
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Response::Reply(reply) => write!(f, "Reply({:?})", reply),
            Response::Event(event) => write!(f, "Event({:?})", event),
            Response::Error(error) => {
                write!(f, "Error({:?}: {})", error.error_code, error.bad_value)
            }
        }
    }
}
