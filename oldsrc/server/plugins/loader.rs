//! Plugin loader
use crate::server::plugins::PluginResult;

pub struct PluginLoader;

impl PluginLoader {
    pub fn new() -> Self {
        Self
    }
    pub async fn load_plugin(&self, _path: &str) -> PluginResult<()> {
        Ok(())
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}
