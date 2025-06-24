//! Server Types
//!
//! Common types used throughout the server infrastructure.

use std::time::Duration;
use uuid::Uuid;

/// Unique identifier for server components
pub type ComponentId = Uuid;

/// Health status of a component
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub enum HealthStatus {
    /// Component is healthy and operational
    Healthy,
    /// Component is degraded but still functional
    Degraded,
    /// Component is unhealthy and may not be functional
    Unhealthy,
    /// Component status is unknown
    Unknown,
}

/// Server metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct ServerMetrics {
    /// Server uptime
    pub uptime: Duration,
    /// Number of active connections
    pub active_connections: u64,
    /// Total requests processed
    pub total_requests: u64,
    /// Requests per second
    pub requests_per_second: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// CPU usage percentage
    pub cpu_usage: f64,
}

impl Default for ServerMetrics {
    fn default() -> Self {
        Self {
            uptime: Duration::ZERO,
            active_connections: 0,
            total_requests: 0,
            requests_per_second: 0.0,
            memory_usage: 0,
            cpu_usage: 0.0,
        }
    }
}

/// Priority levels for various operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Low priority operations
    Low = 1,
    /// Normal priority operations
    Normal = 2,
    /// High priority operations
    High = 3,
    /// Critical priority operations
    Critical = 4,
}

/// Server error types
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    /// Initialization error
    #[error("Initialization error: {0}")]
    Initialization(String),
    /// Shutdown error
    #[error("Shutdown error: {0}")]
    Shutdown(String),
    /// Health check error
    #[error("Health check error: {0}")]
    HealthCheck(String),
    /// Lifecycle error
    #[error("Lifecycle error: {0}")]
    Lifecycle(String),
    /// Service error
    #[error("Service error: {0}")]
    Service(String),
    /// Plugin error
    #[error("Plugin error: {0}")]
    Plugin(String),
    /// Runtime error
    #[error("Runtime error: {0}")]
    Runtime(String),
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Generic error
    #[error("Server error: {0}")]
    Generic(String),
}

/// Result type for server operations
pub type ServerResult<T> = Result<T, ServerError>;
