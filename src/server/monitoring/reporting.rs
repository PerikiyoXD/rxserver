//! Status reporting
use crate::server::monitoring::MonitoringResult;

pub struct StatusReporter;

impl StatusReporter {
    pub fn new() -> Self {
        Self
    }
    pub async fn generate_report(&self) -> MonitoringResult<String> {
        Ok("Status OK".to_string())
    }
}

impl Default for StatusReporter {
    fn default() -> Self {
        Self::new()
    }
}
