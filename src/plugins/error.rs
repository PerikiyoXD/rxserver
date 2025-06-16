// PluginError

use std::fmt;

#[derive(Debug, Clone)]
pub enum PluginError {
    /// Plugin initialization error
    InitError(String),
    /// Plugin execution error
    ExecutionError(String),
    /// Plugin communication error
    CommunicationError(String),
    /// Plugin resource error
    ResourceError(String),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::InitError(msg) => {
                write!(f, "Plugin initialization error: {}", msg)
            }
            PluginError::ExecutionError(msg) => {
                write!(f, "Plugin execution error: {}", msg)
            }
            PluginError::CommunicationError(msg) => {
                write!(f, "Plugin communication error: {}", msg)
            }
            PluginError::ResourceError(msg) => {
                write!(f, "Plugin resource error: {}", msg)
            }
        }
    }
}

impl std::error::Error for PluginError {}
