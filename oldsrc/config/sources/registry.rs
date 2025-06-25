//! Registry configuration source (Windows)

use crate::config::sources::ConfigSource;
use crate::config::types::ServerConfig;
use crate::types::{ConfigurationError, Result};
use async_trait::async_trait;

/// Windows registry configuration source
pub struct RegistrySource {
    key_path: String,
    priority: u32,
}

impl RegistrySource {
    /// Create a new registry source
    pub fn new(key_path: String) -> Self {
        Self {
            key_path,
            priority: 150,
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

#[async_trait]
impl ConfigSource for RegistrySource {
    async fn load(&self) -> Result<ServerConfig> {
        // Registry support not yet implemented
        Err(ConfigurationError::UnsupportedSource("registry".to_string()).into())
    }

    fn identifier(&self) -> String {
        format!("registry:{}", self.key_path)
    }

    fn priority(&self) -> u32 {
        self.priority
    }
}
