// SPDX-License-Identifier: Apache-2.0

// RX X11 Server - Error Handling

use crate::protocol::ProtocolError;
use std::fmt;

/// Main error type for the RX X11 Server
#[derive(Debug, Clone)]
/// Configuration-related errors
pub enum ServerError {
    /// Configuration-related errors
    ConfigError(String),
    /// Network-related errors
    NetworkError(String),
    /// Plugin-related errors
    PluginError(String),
    /// Protocol-related errors
    ProtocolError(ProtocolError),
    /// Logging-related errors
    LoggingError(String),
    /// I/O errors
    IoError(String),
    /// Authentication/authorization errors
    AuthError(String),
    /// Generic server errors
    ServerError(String),
    /// Initialization errors
    InitError(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            ServerError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ServerError::PluginError(msg) => write!(f, "Plugin error: {}", msg),
            ServerError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            ServerError::LoggingError(msg) => write!(f, "Logging error: {}", msg),
            ServerError::IoError(msg) => write!(f, "I/O error: {}", msg),
            ServerError::AuthError(msg) => write!(f, "Authentication error: {}", msg),
            ServerError::ServerError(msg) => write!(f, "Server error: {}", msg),
            ServerError::InitError(msg) => write!(f, "Initialization error: {}", msg),
        }
    }
}

impl std::error::Error for ServerError {}

// Implement From conversions for common error types
impl From<std::io::Error> for ServerError {
    fn from(err: std::io::Error) -> Self {
        ServerError::IoError(err.to_string())
    }
}

impl From<toml::de::Error> for ServerError {
    fn from(err: toml::de::Error) -> Self {
        ServerError::ConfigError(err.to_string())
    }
}

impl From<ProtocolError> for ServerError {
    fn from(err: ProtocolError) -> Self {
        ServerError::ProtocolError(err)
    }
}
