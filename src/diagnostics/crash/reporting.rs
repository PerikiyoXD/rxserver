//! Crash reporting and notification system

use crate::diagnostics::crash::CrashRecord;
use crate::types::Result;

/// Crash reporter for sending crash information
#[derive(Debug, Clone)]
pub struct CrashReporter {
    enabled: bool,
    report_endpoint: Option<String>,
}

impl CrashReporter {
    /// Create new crash reporter
    pub fn new() -> Result<Self> {
        Ok(Self {
            enabled: false, // Disabled by default for privacy
            report_endpoint: None,
        })
    }

    /// Report a crash event
    pub async fn report_crash(&self, crash: &CrashRecord) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Log crash locally
        self.log_crash_locally(crash).await?;

        // Send to external endpoint if configured
        if let Some(ref endpoint) = self.report_endpoint {
            self.send_crash_report(endpoint, crash).await?;
        }

        Ok(())
    }

    /// Log crash information locally
    async fn log_crash_locally(&self, crash: &CrashRecord) -> Result<()> {
        let log_entry = format!(
            "[CRASH] {} - {} in component '{}': {:?}",
            crash
                .timestamp
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            crash.id,
            crash.context.component,
            crash.crash_type
        );

        // In a real implementation, this would use the logging system
        eprintln!("{}", log_entry);
        Ok(())
    }

    /// Send crash report to external endpoint
    async fn send_crash_report(&self, endpoint: &str, crash: &CrashRecord) -> Result<()> {
        // Serialize crash data (excluding sensitive information)
        let report = CrashReport {
            crash_id: crash.id.clone(),
            timestamp: crash.timestamp,
            crash_type: format!("{:?}", crash.crash_type),
            component: crash.context.component.clone(),
            recovery_attempted: crash.recovery_attempted,
            recovery_successful: crash.recovery_successful,
        };

        // In a real implementation, this would make an HTTP request
        println!("Would send crash report to {}: {:?}", endpoint, report);
        Ok(())
    }

    /// Enable crash reporting
    pub fn enable_reporting(&mut self, endpoint: Option<String>) {
        self.enabled = true;
        self.report_endpoint = endpoint;
    }

    /// Disable crash reporting
    pub fn disable_reporting(&mut self) {
        self.enabled = false;
        self.report_endpoint = None;
    }
}

/// Sanitized crash report for external reporting
#[derive(Debug, serde::Serialize)]
struct CrashReport {
    crash_id: String,
    timestamp: std::time::SystemTime,
    crash_type: String,
    component: String,
    recovery_attempted: bool,
    recovery_successful: bool,
}
