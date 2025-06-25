//! Telemetry collection
use crate::server::monitoring::MonitoringResult;

pub struct TelemetryCollector;

impl TelemetryCollector {
    pub fn new() -> Self {
        Self
    }
    pub async fn collect_metrics(&self) -> MonitoringResult<()> {
        Ok(())
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}
