//! Configuration Hot Reloading
//!
//! This module provides hot reloading capabilities for server configuration.

use super::ServerConfig;
use tokio::sync::broadcast;

/// Configuration change event
#[derive(Debug, Clone)]
pub struct ConfigChangeEvent {
    /// Old configuration
    pub old_config: ServerConfig,
    /// New configuration
    pub new_config: ServerConfig,
}

/// Configuration watcher
pub struct ConfigWatcher {
    _sender: broadcast::Sender<ConfigChangeEvent>,
}

impl ConfigWatcher {
    /// Create a new configuration watcher
    pub fn new() -> (Self, broadcast::Receiver<ConfigChangeEvent>) {
        let (sender, receiver) = broadcast::channel(100);
        (Self { _sender: sender }, receiver)
    }
}

impl Default for ConfigWatcher {
    fn default() -> Self {
        Self::new().0
    }
}
