//! RX Server - Protocol error types for the X11 server.
//!
//! This module defines error types that can occur during X11 protocol handling,
//! particularly related to opcode registration and processing.
//!
//! # Examples
//!
//! ```rust
//! use crate::protocol::{ProtocolError, Opcode};
//!
//! // Handle an unregistered opcode
//! let error = ProtocolError::OpcodeNotRegistered(42);
//! println!("Error: {}", error);
//! ```

use crate::protocol::Opcode;

/// Errors that can occur during X11 protocol processing.
///
/// These errors are primarily related to opcode management and validation
/// within the protocol handler system.
#[derive(Debug, Clone)]
pub enum ProtocolError {
    /// An attempt was made to register an opcode that is already registered.
    ///
    /// This typically occurs when trying to register a protocol extension
    /// or handler for an opcode that already has a handler assigned.
    OpcodeAlreadyRegistered(Opcode),

    /// An attempt was made to use an opcode that has not been registered.
    ///
    /// This occurs when the server receives a request with an opcode
    /// that doesn't have a corresponding handler registered.
    OpcodeNotRegistered(Opcode),

    /// The specified opcode is not supported by this server implementation.
    ///
    /// This indicates that while the opcode may be valid in the X11 protocol,
    /// this particular server implementation does not support it.
    UnimplementedOpcode(Opcode),

    /// The requested operation is not implemented.
    Unimplemented,

    /// Invalid message format or content.
    InvalidMessage(String),

    /// Unsupported protocol version.
    UnsupportedProtocolVersion(u16, u16),
}

/// Implementation of Display trait for user-friendly error messages.
///
/// Provides human-readable error descriptions that can be used for logging
/// or client error responses.
impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::OpcodeAlreadyRegistered(opcode) => {
                write!(f, "Opcode {} is already registered", opcode)
            }
            ProtocolError::OpcodeNotRegistered(opcode) => {
                write!(f, "Opcode {} is not registered", opcode)
            }
            ProtocolError::UnimplementedOpcode(opcode) => {
                write!(f, "Unimplemented opcode: {}", opcode)
            }
            ProtocolError::Unimplemented => {
                write!(f, "The requested operation is not implemented")
            }
            ProtocolError::InvalidMessage(msg) => {
                write!(f, "Invalid message: {}", msg)
            }
            ProtocolError::UnsupportedProtocolVersion(major, minor) => {
                write!(f, "Unsupported protocol version: {}.{}", major, minor)
            }
        }
    }
}

/// Standard error trait implementation for integration with Rust's error handling.
impl std::error::Error for ProtocolError {}
