//! Plugin registry
use crate::server::plugins::PluginResult;

#[derive(Debug)]
pub struct PluginRegistry;

impl PluginRegistry {
    pub fn new() -> Self {
        Self
    }
    pub async fn register(&self, _name: &str) -> PluginResult<()> {
        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
