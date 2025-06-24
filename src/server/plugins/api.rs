//! Plugin API
use crate::server::plugins::PluginResult;

#[derive(Debug)]
pub struct PluginApi;

impl PluginApi {
    pub fn new() -> Self {
        Self
    }
    pub async fn call_api(&self, _method: &str) -> PluginResult<()> {
        Ok(())
    }
}

impl Default for PluginApi {
    fn default() -> Self {
        Self::new()
    }
}
