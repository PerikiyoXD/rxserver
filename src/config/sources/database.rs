//! Database configuration source

use crate::config::sources::ConfigSource;
use crate::config::types::ServerConfig;
use crate::types::{ConfigurationError, Result};
use async_trait::async_trait;

/// Database configuration source (placeholder)
pub struct DatabaseSource {
    connection_string: String,
    priority: u32,
}

impl DatabaseSource {
    /// Create a new database source
    pub fn new(connection_string: String) -> Self {
        Self {
            connection_string,
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
impl ConfigSource for DatabaseSource {
    async fn load(&self) -> Result<ServerConfig> {
        // Database support not yet implemented
        Err(ConfigurationError::UnsupportedSource("database".to_string()).into())
    }

    fn identifier(&self) -> String {
        format!("database:{}", self.connection_string)
    }

    fn priority(&self) -> u32 {
        self.priority
    }
}
