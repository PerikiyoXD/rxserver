//! Health status management.
//!
//! This module manages the overall health status of the system based on individual health check results.

use super::{CheckResult, CheckStatus, HealthSeverity, OverallHealth};
use crate::{server::types::HealthStatus, types::Result};
use std::{collections::HashMap, time::SystemTime};

/// Manages health status aggregation and history.
#[derive(Debug, Clone)]
pub struct HealthStatusManager {
    current_status: Option<OverallHealth>,
    status_history: Vec<OverallHealth>,
    max_history_size: usize,
}

impl HealthStatusManager {
    /// Creates a new health status manager.
    pub fn new() -> Self {
        Self {
            current_status: None,
            status_history: Vec::new(),
            max_history_size: 100,
        }
    }

    /// Updates the health status based on check results.
    pub async fn update_status(
        &mut self,
        check_results: HashMap<String, CheckResult>,
    ) -> Result<OverallHealth> {
        let overall_health = self.calculate_overall_health(check_results)?;

        // Update current status
        self.current_status = Some(overall_health.clone());

        // Add to history
        self.add_to_history(overall_health.clone());

        Ok(overall_health)
    }

    /// Gets the current health status.
    pub fn get_current_status(&self) -> Option<&OverallHealth> {
        self.current_status.as_ref()
    }

    /// Gets the health status history.
    pub fn get_status_history(&self) -> &[OverallHealth] {
        &self.status_history
    }
    /// Gets health trends over time.
    pub fn get_health_trends(&self, duration: std::time::Duration) -> HealthTrend {
        let cutoff_time = SystemTime::now() - duration;
        let recent_statuses: Vec<_> = self
            .status_history
            .iter()
            .filter(|status| status.timestamp >= cutoff_time)
            .collect();

        if recent_statuses.is_empty() {
            return HealthTrend::Unknown;
        }

        let severity_trend = self.calculate_severity_trend(&recent_statuses);
        severity_trend
    }

    /// Clears the status history.
    pub fn clear_history(&mut self) {
        self.status_history.clear();
    }

    /// Sets the maximum history size.
    pub fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size;
        if self.status_history.len() > size {
            self.status_history.truncate(size);
        }
    }

    fn calculate_overall_health(
        &self,
        check_results: HashMap<String, CheckResult>,
    ) -> Result<OverallHealth> {
        if check_results.is_empty() {
            return Ok(OverallHealth {
                status: HealthStatus::Unknown,
                severity: HealthSeverity::Warning,
                check_results: HashMap::new(),
                message: "No health checks performed".to_string(),
                timestamp: SystemTime::now(),
                details: HashMap::new(),
            });
        }

        // Determine overall severity based on worst check result
        let mut overall_severity = HealthSeverity::Healthy;
        let mut failed_checks = Vec::new();
        let mut warning_checks = Vec::new();

        for (name, result) in &check_results {
            match result.status {
                CheckStatus::Pass => {}
                CheckStatus::Warning => {
                    if overall_severity < HealthSeverity::Warning {
                        overall_severity = HealthSeverity::Warning;
                    }
                    warning_checks.push(name.clone());
                }
                CheckStatus::Fail => {
                    if overall_severity < HealthSeverity::Critical {
                        overall_severity = HealthSeverity::Critical;
                    }
                    failed_checks.push(name.clone());
                }
                CheckStatus::Error => {
                    overall_severity = HealthSeverity::Fatal;
                    failed_checks.push(name.clone());
                }
            }
        }

        // Generate status message
        let message =
            self.generate_status_message(overall_severity, &failed_checks, &warning_checks);
        let status = match overall_severity {
            HealthSeverity::Healthy => HealthStatus::Healthy,
            HealthSeverity::Warning => HealthStatus::Degraded,
            HealthSeverity::Critical | HealthSeverity::Fatal => HealthStatus::Unhealthy,
        };

        Ok(OverallHealth {
            status,
            severity: overall_severity,
            check_results,
            message,
            timestamp: SystemTime::now(),
            details: HashMap::new(),
        })
    }

    fn generate_status_message(
        &self,
        severity: HealthSeverity,
        failed_checks: &[String],
        warning_checks: &[String],
    ) -> String {
        match severity {
            HealthSeverity::Healthy => "All systems healthy".to_string(),
            HealthSeverity::Warning => {
                if warning_checks.is_empty() {
                    "System warnings detected".to_string()
                } else {
                    format!("Warnings in: {}", warning_checks.join(", "))
                }
            }
            HealthSeverity::Critical => {
                if failed_checks.is_empty() {
                    "Critical system issues detected".to_string()
                } else {
                    format!("Critical failures in: {}", failed_checks.join(", "))
                }
            }
            HealthSeverity::Fatal => {
                if failed_checks.is_empty() {
                    "Fatal system errors detected".to_string()
                } else {
                    format!("Fatal errors in: {}", failed_checks.join(", "))
                }
            }
        }
    }

    fn add_to_history(&mut self, status: OverallHealth) {
        self.status_history.push(status);

        // Maintain maximum history size
        if self.status_history.len() > self.max_history_size {
            self.status_history.remove(0);
        }
    }

    fn calculate_severity_trend(&self, statuses: &[&OverallHealth]) -> HealthTrend {
        if statuses.len() < 2 {
            return HealthTrend::Stable;
        }

        let _first_severity = statuses[0].severity as u8;
        let _last_severity = statuses[statuses.len() - 1].severity as u8;

        // Calculate average trend
        let mut trend_sum = 0i32;
        for window in statuses.windows(2) {
            let current = window[1].severity as u8;
            let previous = window[0].severity as u8;
            trend_sum += current as i32 - previous as i32;
        }

        let average_trend = trend_sum as f64 / (statuses.len() - 1) as f64;

        if average_trend > 0.1 {
            HealthTrend::Deteriorating
        } else if average_trend < -0.1 {
            HealthTrend::Improving
        } else {
            HealthTrend::Stable
        }
    }
}

impl Default for HealthStatusManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Health trend over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthTrend {
    /// Health is improving.
    Improving,
    /// Health is stable.
    Stable,
    /// Health is deteriorating.
    Deteriorating,
    /// Not enough data to determine trend.
    Unknown,
}

/// Health status summary for reporting.
#[derive(Debug, Clone)]
pub struct HealthSummary {
    /// Current overall severity.
    pub current_severity: HealthSeverity,
    /// Health trend.
    pub trend: HealthTrend,
    /// Number of passing checks.
    pub passing_checks: usize,
    /// Number of warning checks.
    pub warning_checks: usize,
    /// Number of failing checks.
    pub failing_checks: usize,
    /// Number of error checks.
    pub error_checks: usize,
    /// Time since last check.
    pub time_since_last_check: Option<std::time::Duration>,
}

impl HealthStatusManager {
    /// Gets a health summary for reporting.
    pub fn get_health_summary(&self) -> HealthSummary {
        let current_status = self.current_status.as_ref();

        let (
            current_severity,
            passing_checks,
            warning_checks,
            failing_checks,
            error_checks,
            time_since_last_check,
        ) = match current_status {
            Some(status) => {
                let mut passing = 0;
                let mut warning = 0;
                let mut failing = 0;
                let mut error = 0;

                for result in status.check_results.values() {
                    match result.status {
                        CheckStatus::Pass => passing += 1,
                        CheckStatus::Warning => warning += 1,
                        CheckStatus::Fail => failing += 1,
                        CheckStatus::Error => error += 1,
                    }
                }
                (
                    status.severity,
                    passing,
                    warning,
                    failing,
                    error,
                    status.timestamp.elapsed().ok(),
                )
            }
            None => (HealthSeverity::Warning, 0, 0, 0, 0, None),
        };

        let trend = self.get_health_trends(std::time::Duration::from_secs(300)); // 5 minute trend

        HealthSummary {
            current_severity,
            trend,
            passing_checks,
            warning_checks,
            failing_checks,
            error_checks,
            time_since_last_check,
        }
    }
}
