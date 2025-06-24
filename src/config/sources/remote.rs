//! Remote configuration source

use crate::config::sources::ConfigSource;
use crate::config::types::ServerConfig;
use crate::types::{ConfigurationError, Result};
use async_trait::async_trait;

/// Remote configuration source (HTTP, consul, etcd, etc.)
pub struct RemoteSource {
    url: String,
    priority: u32,
}

impl RemoteSource {
    /// Create a new remote source
    pub fn new(url: String) -> Self {
        Self { url, priority: 150 }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

#[async_trait]
impl ConfigSource for RemoteSource {
    async fn load(&self) -> Result<ServerConfig> {
        // Remote configuration support not yet implemented
        Err(ConfigurationError::UnsupportedSource("remote".to_string()).into())
    }

    fn identifier(&self) -> String {
        format!("remote:{}", self.url)
    }

    fn priority(&self) -> u32 {
        self.priority
    }
}
