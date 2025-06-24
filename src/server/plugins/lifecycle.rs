//! Plugin lifecycle
use crate::server::plugins::PluginResult;

pub struct PluginLifecycle;

impl PluginLifecycle {
    pub fn new() -> Self {
        Self
    }
    pub async fn initialize(&self) -> PluginResult<()> {
        Ok(())
    }
}

impl Default for PluginLifecycle {
    fn default() -> Self {
        Self::new()
    }
}
