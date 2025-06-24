//! Plugin communication
use crate::server::plugins::PluginResult;

pub struct PluginCommunication;

impl PluginCommunication {
    pub fn new() -> Self {
        Self
    }
    pub async fn send_message(&self, _msg: &str) -> PluginResult<()> {
        Ok(())
    }
}

impl Default for PluginCommunication {
    fn default() -> Self {
        Self::new()
    }
}
