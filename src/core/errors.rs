//! Enhanced error types for X11 server operations
//!
//! This module provides more specific error types than the basic Error enum
//! in lib.rs, allowing for better error handling and debugging.

use std::fmt;

/// Specific errors for X11 protocol operations
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolError {
    /// Invalid request opcode
    InvalidOpcode(u8),

    /// Request too short for the specified opcode
    RequestTooShort { expected: usize, actual: usize },

    /// Invalid parameter value
    InvalidParameter { parameter: String, value: String },

    /// Resource not found
    ResourceNotFound { resource_type: String, id: u32 },

    /// Access denied
    AccessDenied { operation: String, resource_id: u32 },

    /// Invalid resource type for operation
    InvalidResourceType { expected: String, actual: String },

    /// Unsupported operation
    UnsupportedOperation(String),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::InvalidOpcode(op) => write!(f, "Invalid opcode: {}", op),
            ProtocolError::RequestTooShort { expected, actual } => {
                write!(
                    f,
                    "Request too short: expected {} bytes, got {}",
                    expected, actual
                )
            }
            ProtocolError::InvalidParameter { parameter, value } => {
                write!(f, "Invalid parameter '{}': {}", parameter, value)
            }
            ProtocolError::ResourceNotFound { resource_type, id } => {
                write!(f, "{} with ID {} not found", resource_type, id)
            }
            ProtocolError::AccessDenied {
                operation,
                resource_id,
            } => {
                write!(
                    f,
                    "Access denied for operation '{}' on resource {}",
                    operation, resource_id
                )
            }
            ProtocolError::InvalidResourceType { expected, actual } => {
                write!(
                    f,
                    "Invalid resource type: expected {}, got {}",
                    expected, actual
                )
            }
            ProtocolError::UnsupportedOperation(op) => {
                write!(f, "Unsupported operation: {}", op)
            }
        }
    }
}

impl std::error::Error for ProtocolError {}

/// Errors specific to connection handling
#[derive(Debug, Clone)]
pub enum ConnectionError {
    /// Connection authentication failed
    AuthenticationFailed(String),

    /// Connection closed unexpectedly
    UnexpectedClose,

    /// Protocol version mismatch
    VersionMismatch { client: u16, server: u16 },

    /// Too many connections
    TooManyConnections { current: usize, max: usize },

    /// Invalid connection state for operation
    InvalidState { expected: String, actual: String },
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::AuthenticationFailed(reason) => {
                write!(f, "Authentication failed: {}", reason)
            }
            ConnectionError::UnexpectedClose => write!(f, "Connection closed unexpectedly"),
            ConnectionError::VersionMismatch { client, server } => {
                write!(
                    f,
                    "Protocol version mismatch: client {}, server {}",
                    client, server
                )
            }
            ConnectionError::TooManyConnections { current, max } => {
                write!(f, "Too many connections: {} current, {} max", current, max)
            }
            ConnectionError::InvalidState { expected, actual } => {
                write!(
                    f,
                    "Invalid connection state: expected {}, got {}",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for ConnectionError {}

/// Errors specific to resource management
#[derive(Debug, Clone)]
pub enum ResourceError {
    /// Resource already exists
    AlreadyExists { resource_type: String, id: u32 },

    /// Resource pool exhausted
    PoolExhausted(String),

    /// Invalid resource operation
    InvalidOperation { operation: String, reason: String },

    /// Resource in use and cannot be freed
    InUse { resource_type: String, id: u32 },
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceError::AlreadyExists { resource_type, id } => {
                write!(f, "{} with ID {} already exists", resource_type, id)
            }
            ResourceError::PoolExhausted(pool) => {
                write!(f, "Resource pool '{}' exhausted", pool)
            }
            ResourceError::InvalidOperation { operation, reason } => {
                write!(f, "Invalid operation '{}': {}", operation, reason)
            }
            ResourceError::InUse { resource_type, id } => {
                write!(
                    f,
                    "{} with ID {} is in use and cannot be freed",
                    resource_type, id
                )
            }
        }
    }
}

impl std::error::Error for ResourceError {}
