//! Alert management
use crate::server::monitoring::MonitoringResult;

pub struct AlertManager;

impl AlertManager {
    pub fn new() -> Self {
        Self
    }
    pub async fn send_alert(&self, _message: &str) -> MonitoringResult<()> {
        Ok(())
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}
