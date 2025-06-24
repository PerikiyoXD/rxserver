//! Diagnostic types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Severity levels for diagnostic events
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational messages
    Info,
    /// Warning conditions
    Warning,
    /// Error conditions
    Error,
    /// Critical system failures
    Critical,
    /// System is unusable
    Emergency,
}

/// Diagnostic event category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    /// Performance-related events
    Performance,
    /// Memory usage and allocation
    Memory,
    /// Network connectivity and protocol
    Network,
    /// Display and rendering subsystem
    Display,
    /// Input handling and devices
    Input,
    /// Security and authentication
    Security,
    /// Configuration and settings
    Configuration,
    /// Resource management
    Resources,
    /// X11 protocol compliance
    Protocol,
    /// Platform-specific issues
    Platform,
    /// Extension handling
    Extensions,
}

/// Diagnostic event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticEvent {
    /// Unique event identifier
    pub id: u64,
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Event severity
    pub severity: Severity,
    /// Event category
    pub category: Category,
    /// Component that generated the event
    pub component: String,
    /// Human-readable message
    pub message: String,
    /// Additional structured data
    pub metadata: HashMap<String, String>,
    /// Optional stack trace for errors
    pub stack_trace: Option<String>,
}

impl DiagnosticEvent {
    /// Create a new diagnostic event
    pub fn new(
        severity: Severity,
        category: Category,
        component: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);

        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            timestamp: SystemTime::now(),
            severity,
            category,
            component: component.into(),
            message: message.into(),
            metadata: HashMap::new(),
            stack_trace: None,
        }
    }

    /// Add metadata to the event
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add stack trace to the event
    pub fn with_stack_trace(mut self, trace: impl Into<String>) -> Self {
        self.stack_trace = Some(trace.into());
        self
    }
}

/// System component health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentHealth {
    /// Component is operating normally
    Healthy,
    /// Component has minor issues but is functional
    Warning,
    /// Component has errors but may still be partially functional
    Degraded,
    /// Component is not functioning
    Critical,
    /// Component status cannot be determined
    Unknown,
}

/// Health check result for a system component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Component name
    pub component: String,
    /// Health status
    pub status: ComponentHealth,
    /// Optional status message
    pub message: Option<String>,
    /// Check timestamp
    pub timestamp: SystemTime,
    /// Time taken to perform the check
    pub check_duration_ms: u64,
    /// Additional check metadata
    pub metadata: HashMap<String, String>,
}

/// Performance metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// Metric name/identifier
    pub name: String,
    /// Metric value
    pub value: f64,
    /// Metric unit (e.g., "bytes", "ms", "percent", "count")
    pub unit: String,
    /// Measurement timestamp
    pub timestamp: SystemTime,
    /// Optional labels for grouping metrics
    pub labels: HashMap<String, String>,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total allocated memory in bytes
    pub allocated_bytes: u64,
    /// Memory in use in bytes
    pub used_bytes: u64,
    /// Peak memory usage in bytes
    pub peak_bytes: u64,
    /// Number of allocations
    pub allocation_count: u64,
    /// Number of deallocations
    pub deallocation_count: u64,
    /// Memory fragmentation percentage
    pub fragmentation_percent: f32,
}

/// Performance timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingStats {
    /// Average operation time in microseconds
    pub avg_micros: f64,
    /// Minimum operation time in microseconds
    pub min_micros: u64,
    /// Maximum operation time in microseconds
    pub max_micros: u64,
    /// 95th percentile time in microseconds
    pub p95_micros: u64,
    /// 99th percentile time in microseconds
    pub p99_micros: u64,
    /// Total number of operations measured
    pub operation_count: u64,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// CPU usage percentage (0-100)
    pub cpu_percent: f32,
    /// Memory usage statistics
    pub memory: MemoryStats,
    /// Network I/O bytes per second
    pub network_io_bps: u64,
    /// Disk I/O operations per second
    pub disk_io_ops: u64,
    /// Number of active file descriptors
    pub file_descriptors: u32,
    /// Number of active threads
    pub thread_count: u32,
}

/// Diagnostic filter criteria
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticFilter {
    /// Filter by minimum severity level
    pub min_severity: Option<Severity>,
    /// Filter by event categories
    pub categories: Vec<Category>,
    /// Filter by component names
    pub components: Vec<String>,
    /// Filter by time range
    pub time_range: Option<(SystemTime, SystemTime)>,
    /// Filter by metadata key-value pairs
    pub metadata_filters: HashMap<String, String>,
}

impl DiagnosticFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by minimum severity
    pub fn with_min_severity(mut self, severity: Severity) -> Self {
        self.min_severity = Some(severity);
        self
    }

    /// Filter by categories
    pub fn with_categories(mut self, categories: Vec<Category>) -> Self {
        self.categories = categories;
        self
    }

    /// Filter by components
    pub fn with_components(mut self, components: Vec<String>) -> Self {
        self.components = components;
        self
    }

    /// Check if an event matches this filter
    pub fn matches(&self, event: &DiagnosticEvent) -> bool {
        // Check severity
        if let Some(min_severity) = self.min_severity {
            if event.severity < min_severity {
                return false;
            }
        }

        // Check categories
        if !self.categories.is_empty() && !self.categories.contains(&event.category) {
            return false;
        }

        // Check components
        if !self.components.is_empty() && !self.components.contains(&event.component) {
            return false;
        }

        // Check time range
        if let Some((start, end)) = self.time_range {
            if event.timestamp < start || event.timestamp > end {
                return false;
            }
        }

        // Check metadata filters
        for (key, expected_value) in &self.metadata_filters {
            if let Some(actual_value) = event.metadata.get(key) {
                if actual_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

/// Diagnostic configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticConfig {
    /// Enable crash detection
    pub enable_crash_detection: bool,
    /// Enable health monitoring
    pub enable_health_monitoring: bool,
    /// Enable metrics collection
    pub enable_metrics_collection: bool,
    /// Maximum number of events to keep in memory
    pub max_events_in_memory: usize,
    /// Automatic cleanup interval in seconds
    pub cleanup_interval_seconds: u64,
    /// Default log level for diagnostic events
    pub default_log_level: Severity,
    /// Components to monitor (empty = all)
    pub monitored_components: Vec<String>,
    /// Health check interval in seconds
    pub health_check_interval_seconds: u64,
    /// Metrics collection interval in seconds
    pub metrics_collection_interval_seconds: u64,
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        Self {
            enable_crash_detection: true,
            enable_health_monitoring: true,
            enable_metrics_collection: true,
            max_events_in_memory: 10000,
            cleanup_interval_seconds: 300, // 5 minutes
            default_log_level: Severity::Info,
            monitored_components: Vec::new(),
            health_check_interval_seconds: 30,
            metrics_collection_interval_seconds: 10,
        }
    }
}
