//! Logging system for the X11 server
//!
//! This module provides a comprehensive logging system with support for multiple
//! formatters, outputs, filtering, rotation, and analysis.

use crate::types::Result;
use std::sync::{Arc, Mutex};

pub mod analysis;
pub mod compression;
pub mod filtering;
pub mod formatters;
pub mod manager;
pub mod outputs;
pub mod rotation;
pub mod types;

pub use manager::LogManager;
pub use types::*;

/// Initialize the logging system with default configuration
pub fn init() -> Result<Arc<Mutex<LogManager>>> {
    LogManager::new()
}

/// Initialize the logging system with custom configuration
pub fn init_with_config(config: LoggingConfig) -> Result<Arc<Mutex<LogManager>>> {
    LogManager::with_config(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_init() {
        let manager = init().expect("Failed to initialize logging");
        // Assert that the Arc contains a value by checking strong_count > 0
        assert!(Arc::strong_count(&manager) > 0);
    }
}
