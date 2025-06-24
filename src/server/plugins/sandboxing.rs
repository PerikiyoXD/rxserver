//! Plugin sandboxing
use crate::server::plugins::PluginResult;

pub struct PluginSandbox;

impl PluginSandbox {
    pub fn new() -> Self {
        Self
    }
    pub async fn create_sandbox(&self) -> PluginResult<()> {
        Ok(())
    }
}

impl Default for PluginSandbox {
    fn default() -> Self {
        Self::new()
    }
}
