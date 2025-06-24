//! Monitoring dashboards
use crate::server::monitoring::MonitoringResult;

pub struct Dashboard;

impl Dashboard {
    pub fn new() -> Self {
        Self
    }
    pub async fn render(&self) -> MonitoringResult<String> {
        Ok("Dashboard".to_string())
    }
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}
