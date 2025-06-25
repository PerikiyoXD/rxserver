//! Logging types and structures.
//!
//! This module defines the core types used throughout the logging system.

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Log levels in order of severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    /// Most verbose logging level.
    Trace = 0,
    /// Debug information.
    Debug = 1,
    /// General information.
    Info = 2,
    /// Warnings about potential issues.
    Warning = 3,
    /// Error conditions.
    Error = 4,
    /// Critical errors.
    Critical = 5,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl From<&str> for LogLevel {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "TRACE" => LogLevel::Trace,
            "DEBUG" => LogLevel::Debug,
            "INFO" => LogLevel::Info,
            "WARN" | "WARNING" => LogLevel::Warning,
            "ERROR" => LogLevel::Error,
            "CRITICAL" => LogLevel::Critical,
            _ => LogLevel::Info, // Default
        }
    }
}

/// A log entry containing all relevant information.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Log level.
    pub level: LogLevel,
    /// Log message.
    pub message: String,
    /// Timestamp when the log was created.
    pub timestamp: SystemTime,
    /// Thread ID that created the log.
    pub thread_id: Option<String>,
    /// Source location information.
    pub source_location: Option<SourceLocation>,
    /// Additional context information.
    pub context: Option<LogContext>,
    /// Structured fields.
    pub fields: HashMap<String, LogValue>,
}

impl LogEntry {
    /// Creates a new log entry.
    pub fn new(level: LogLevel, message: String, context: Option<LogContext>) -> Self {
        Self {
            level,
            message,
            timestamp: SystemTime::now(),
            thread_id: std::thread::current().name().map(|s| s.to_string()),
            source_location: None,
            context,
            fields: HashMap::new(),
        }
    }

    /// Creates a new log entry with source location.
    pub fn with_location(
        level: LogLevel,
        message: String,
        context: Option<LogContext>,
        location: SourceLocation,
    ) -> Self {
        let mut entry = Self::new(level, message, context);
        entry.source_location = Some(location);
        entry
    }

    /// Adds a field to the log entry.
    pub fn with_field(mut self, key: String, value: LogValue) -> Self {
        self.fields.insert(key, value);
        self
    }

    /// Gets the timestamp as milliseconds since epoch.
    pub fn timestamp_millis(&self) -> u64 {
        self.timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

/// Source location information.
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// File name.
    pub file: String,
    /// Line number.
    pub line: u32,
    /// Column number.
    pub column: Option<u32>,
    /// Function name.
    pub function: Option<String>,
}

/// Additional context for log entries.
#[derive(Debug, Clone)]
pub struct LogContext {
    /// Component that generated the log.
    pub component: String,
    /// Request ID or correlation ID.
    pub request_id: Option<String>,
    /// User ID.
    pub user_id: Option<String>,
    /// Session ID.
    pub session_id: Option<String>,
    /// Client information.
    pub client_info: Option<ClientInfo>,
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

impl LogContext {
    /// Creates a new log context with the specified component.
    pub fn new(component: String) -> Self {
        Self {
            component,
            request_id: None,
            user_id: None,
            session_id: None,
            client_info: None,
            metadata: HashMap::new(),
        }
    }

    /// Adds a metadata field.
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Client information for logging context.
#[derive(Debug, Clone)]
pub struct ClientInfo {
    /// Client IP address.
    pub ip_address: String,
    /// Client user agent or application name.
    pub user_agent: Option<String>,
    /// Client protocol version.
    pub protocol_version: Option<String>,
}

/// Values that can be stored in log fields.
#[derive(Debug, Clone)]
pub enum LogValue {
    /// String value.
    String(String),
    /// Integer value.
    Integer(i64),
    /// Floating point value.
    Float(f64),
    /// Boolean value.
    Boolean(bool),
    /// Null value.
    Null,
}

impl fmt::Display for LogValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogValue::String(s) => write!(f, "{}", s),
            LogValue::Integer(i) => write!(f, "{}", i),
            LogValue::Float(fl) => write!(f, "{}", fl),
            LogValue::Boolean(b) => write!(f, "{}", b),
            LogValue::Null => write!(f, "null"),
        }
    }
}

impl From<String> for LogValue {
    fn from(s: String) -> Self {
        LogValue::String(s)
    }
}

impl From<&str> for LogValue {
    fn from(s: &str) -> Self {
        LogValue::String(s.to_string())
    }
}

impl From<i64> for LogValue {
    fn from(i: i64) -> Self {
        LogValue::Integer(i)
    }
}

impl From<f64> for LogValue {
    fn from(f: f64) -> Self {
        LogValue::Float(f)
    }
}

impl From<bool> for LogValue {
    fn from(b: bool) -> Self {
        LogValue::Boolean(b)
    }
}

impl From<i8> for LogValue {
    fn from(i: i8) -> Self {
        LogValue::Integer(i as i64)
    }
}

impl From<usize> for LogValue {
    fn from(i: usize) -> Self {
        LogValue::Integer(i as i64)
    }
}

impl From<f32> for LogValue {
    fn from(f: f32) -> Self {
        LogValue::Float(f as f64)
    }
}

/// Configuration for the logging system.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Default log level.
    pub level: LogLevel,
    /// Whether to enable asynchronous logging.
    pub async_logging: bool,
    /// Buffer size for async logging.
    pub buffer_size: usize,
    /// Maximum number of log files to keep.
    pub max_files: usize,
    /// Maximum size of each log file.
    pub max_file_size: u64,
    /// Log file directory.
    pub log_directory: PathBuf,
    /// Log file name pattern.
    pub file_name_pattern: String,
    /// Whether to enable console output.
    pub console_output: bool,
    /// Whether to enable structured logging.
    pub structured_logging: bool,
    /// Whether to enable compression.
    pub compression: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            async_logging: true,
            buffer_size: 1024,
            max_files: 10,
            max_file_size: 100 * 1024 * 1024, // 100MB
            log_directory: PathBuf::from("logs"),
            file_name_pattern: "rxserver.log".to_string(),
            console_output: true,
            structured_logging: false,
            compression: true,
        }
    }
}

/// Log output destination types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    /// Console/terminal output.
    Console,
    /// File output.
    File,
    /// Syslog output.
    Syslog,
    /// Journald output (Linux).
    Journald,
    /// Network output.
    Network,
    /// Database output.
    Database,
}

/// Log format types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatType {
    /// Plain text format.
    Plain,
    /// JSON format.
    Json,
    /// Structured format.
    Structured,
    /// Custom format.
    Custom,
}

/// Rotation policy for log files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationPolicy {
    /// Rotate by file size.
    Size,
    /// Rotate by time interval.
    Time,
    /// Rotate by number of entries.
    Count,
    /// Combined rotation policy.
    Combined,
}

/// Compression algorithm for log files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    /// No compression.
    None,
    /// Gzip compression.
    Gzip,
    /// LZ4 compression.
    Lz4,
    /// Zstandard compression.
    Zstd,
}

/// Configuration for log rotation.
#[derive(Debug, Clone)]
pub struct RotationConfig {
    /// Whether rotation is enabled.
    pub enabled: bool,
    /// Rotation policy.
    pub policy: RotationPolicy,
    /// Maximum file size before rotation.
    pub max_size: u64,
    /// Time interval for rotation.
    pub time_interval: std::time::Duration,
    /// Maximum number of entries before rotation.
    pub max_entries: u64,
    /// Maximum number of archived files to keep.
    pub max_archives: usize,
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            policy: RotationPolicy::Size,
            max_size: 100 * 1024 * 1024, // 100MB
            time_interval: std::time::Duration::from_secs(24 * 60 * 60), // 24 hours
            max_entries: 1_000_000,
            max_archives: 10,
        }
    }
}

/// Configuration for log compression.
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Whether compression is enabled.
    pub enabled: bool,
    /// Compression algorithm.
    pub algorithm: CompressionAlgorithm,
    /// Compression level (algorithm-specific).
    pub level: u32,
    /// Whether to compress archived files.
    pub compress_archives: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: CompressionAlgorithm::Gzip,
            level: 6,
            compress_archives: true,
        }
    }
}

/// Configuration for log analysis.
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Whether analysis is enabled.
    pub enabled: bool,
    /// Analysis interval.
    pub interval: std::time::Duration,
    /// Whether to generate reports.
    pub generate_reports: bool,
    /// Whether to detect anomalies.
    pub anomaly_detection: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval: std::time::Duration::from_secs(60 * 60), // 1 hour
            generate_reports: false,
            anomaly_detection: false,
        }
    }
}
