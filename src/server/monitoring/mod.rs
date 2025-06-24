//! Server monitoring interface
//!
//! This module provides monitoring capabilities including telemetry collection,
//! alerting, dashboards, and status reporting.

pub mod alerting;
pub mod dashboards;
pub mod reporting;
pub mod telemetry;

pub use self::alerting::*;
pub use self::dashboards::*;
pub use self::reporting::*;
pub use self::telemetry::*;

/// Monitoring error types
#[derive(Debug, thiserror::Error)]
pub enum MonitoringError {
    #[error("Telemetry error: {0}")]
    Telemetry(String),
    #[error("Alerting error: {0}")]
    Alerting(String),
    #[error("Dashboard error: {0}")]
    Dashboard(String),
    #[error("Reporting error: {0}")]
    Reporting(String),
}

/// Result type for monitoring operations
pub type MonitoringResult<T> = Result<T, MonitoringError>;
