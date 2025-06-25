//! X11 extensions framework
//!
//! This module provides the framework for X11 extensions, including
//! registration, management, and request dispatching.

/// Extension identifier type
pub type ExtensionId = u8;

/// Extension manager
#[derive(Debug, Default)]
pub struct ExtensionManager {
    // TODO: Add extension management components
}

impl ExtensionManager {
    /// Create a new extension manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an extension
    pub fn register_extension(&mut self, _name: &str) -> Result<ExtensionId, ExtensionError> {
        // TODO: Implement extension registration
        Ok(0)
    }

    /// Check if an extension is supported
    pub fn is_extension_supported(&self, _name: &str) -> bool {
        // TODO: Implement extension lookup
        false
    }

    /// Get extension by name
    pub fn get_extension_id(&self, _name: &str) -> Option<ExtensionId> {
        // TODO: Implement extension ID lookup
        None
    }
}

/// Extension-related errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionError {
    /// Extension not found
    NotFound,
    /// Extension already registered
    AlreadyRegistered,
    /// Invalid extension
    Invalid,
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtensionError::NotFound => write!(f, "Extension not found"),
            ExtensionError::AlreadyRegistered => write!(f, "Extension already registered"),
            ExtensionError::Invalid => write!(f, "Invalid extension"),
        }
    }
}

impl std::error::Error for ExtensionError {}
