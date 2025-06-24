//! Log formatters for different output formats.
//!
//! This module provides various log formatting implementations.

use crate::logging::types::*;
use crate::types::Result;
use std::collections::HashMap;

pub mod custom;
pub mod json;
pub mod plain;
pub mod structured;

pub use custom::*;
pub use json::*;
pub use plain::*;
pub use structured::*;

/// Trait for log formatters.
pub trait LogFormatter: Send + Sync {
    /// Formats a log entry into a string.
    fn format(&self, entry: &LogEntry) -> Result<String>;

    /// Gets the name of this formatter.
    fn name(&self) -> &str;

    /// Gets the format type.
    fn format_type(&self) -> FormatType;
}

/// Creates a formatter by type.
pub fn create_formatter(format_type: FormatType) -> Box<dyn LogFormatter> {
    match format_type {
        FormatType::Plain => Box::new(PlainFormatter::new()),
        FormatType::Json => Box::new(JsonFormatter::new()),
        FormatType::Structured => Box::new(StructuredFormatter::new()),
        FormatType::Custom => Box::new(CustomFormatter::new()),
    }
}

/// Creates a formatter with custom configuration.
pub fn create_formatter_with_config(
    format_type: FormatType,
    config: FormatterConfig,
) -> Box<dyn LogFormatter> {
    match format_type {
        FormatType::Plain => Box::new(PlainFormatter::with_config(config)),
        FormatType::Json => Box::new(JsonFormatter::with_config(config)),
        FormatType::Structured => Box::new(StructuredFormatter::with_config(config)),
        FormatType::Custom => Box::new(CustomFormatter::with_config(config)),
    }
}

/// Configuration for formatters.
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    /// Whether to include timestamps.
    pub include_timestamp: bool,
    /// Whether to include thread IDs.
    pub include_thread_id: bool,
    /// Whether to include source location.
    pub include_source_location: bool,
    /// Whether to include log level.
    pub include_level: bool,
    /// Whether to include context information.
    pub include_context: bool,
    /// Custom timestamp format.
    pub timestamp_format: Option<String>,
    /// Custom field mappings.
    pub field_mappings: HashMap<String, String>,
    /// Whether to pretty-print JSON.
    pub pretty_print: bool,
    /// Maximum message length before truncation.
    pub max_message_length: Option<usize>,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            include_timestamp: true,
            include_thread_id: false,
            include_source_location: false,
            include_level: true,
            include_context: true,
            timestamp_format: None,
            field_mappings: HashMap::new(),
            pretty_print: false,
            max_message_length: None,
        }
    }
}

/// Utility functions for formatting.
pub mod utils {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Formats a timestamp using the default format.
    pub fn format_timestamp(timestamp: SystemTime) -> String {
        format_timestamp_with_format(timestamp, "%Y-%m-%d %H:%M:%S%.3f")
    }

    /// Formats a timestamp with a custom format.
    pub fn format_timestamp_with_format(timestamp: SystemTime, format: &str) -> String {
        match timestamp.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let millis = duration.as_millis();
                let secs = millis / 1000;
                let subsec_millis = millis % 1000;

                // Simple formatting - in a real implementation you'd use chrono
                format!("{}.{:03}", secs, subsec_millis)
            }
            Err(_) => "invalid-time".to_string(),
        }
    }

    /// Truncates a message if it exceeds the maximum length.
    pub fn truncate_message(message: &str, max_length: Option<usize>) -> String {
        match max_length {
            Some(max) if message.len() > max => {
                format!("{}...", &message[..max.saturating_sub(3)])
            }
            _ => message.to_string(),
        }
    }

    /// Escapes special characters for JSON.
    pub fn escape_json_string(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '"' => "\\\"".to_string(),
                '\\' => "\\\\".to_string(),
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                c if c.is_control() => format!("\\u{:04x}", c as u32),
                c => c.to_string(),
            })
            .collect()
    }

    /// Formats log level with consistent width.
    pub fn format_level(level: LogLevel) -> String {
        format!("{:<8}", level.to_string())
    }

    /// Formats context information.
    pub fn format_context(context: &LogContext) -> String {
        let mut parts = vec![format!("component={}", context.component)];

        if let Some(ref request_id) = context.request_id {
            parts.push(format!("request_id={}", request_id));
        }

        if let Some(ref user_id) = context.user_id {
            parts.push(format!("user_id={}", user_id));
        }

        if let Some(ref session_id) = context.session_id {
            parts.push(format!("session_id={}", session_id));
        }

        for (key, value) in &context.metadata {
            parts.push(format!("{}={}", key, value));
        }

        parts.join(" ")
    }

    /// Formats source location information.
    pub fn format_source_location(location: &SourceLocation) -> String {
        match (&location.function, location.column) {
            (Some(function), Some(column)) => {
                format!(
                    "{}:{}:{}:{}",
                    location.file, location.line, column, function
                )
            }
            (Some(function), None) => {
                format!("{}:{}:{}", location.file, location.line, function)
            }
            (None, Some(column)) => {
                format!("{}:{}:{}", location.file, location.line, column)
            }
            (None, None) => {
                format!("{}:{}", location.file, location.line)
            }
        }
    }
}
