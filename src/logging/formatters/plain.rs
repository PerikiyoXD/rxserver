//! Plain text log formatter.
//!
//! This module provides a simple plain text formatter for log entries.

use crate::logging::formatters::{FormatterConfig, LogFormatter, utils};
use crate::logging::types::*;
use crate::types::Result;

/// Plain text log formatter.
#[derive(Debug)]
pub struct PlainFormatter {
    config: FormatterConfig,
}

impl PlainFormatter {
    /// Creates a new plain formatter with default configuration.
    pub fn new() -> Self {
        Self {
            config: FormatterConfig::default(),
        }
    }

    /// Creates a new plain formatter with custom configuration.
    pub fn with_config(config: FormatterConfig) -> Self {
        Self { config }
    }
}

impl LogFormatter for PlainFormatter {
    fn format(&self, entry: &LogEntry) -> Result<String> {
        let mut parts = Vec::new();

        // Timestamp
        if self.config.include_timestamp {
            let timestamp = if let Some(ref format) = self.config.timestamp_format {
                utils::format_timestamp_with_format(entry.timestamp, format)
            } else {
                utils::format_timestamp(entry.timestamp)
            };
            parts.push(timestamp);
        }

        // Log level
        if self.config.include_level {
            parts.push(utils::format_level(entry.level));
        }

        // Thread ID
        if self.config.include_thread_id {
            if let Some(ref thread_id) = entry.thread_id {
                parts.push(format!("[{}]", thread_id));
            }
        }

        // Source location
        if self.config.include_source_location {
            if let Some(ref location) = entry.source_location {
                parts.push(format!("({})", utils::format_source_location(location)));
            }
        }

        // Context
        if self.config.include_context {
            if let Some(ref context) = entry.context {
                parts.push(format!("[{}]", utils::format_context(context)));
            }
        }

        // Message
        let message = utils::truncate_message(&entry.message, self.config.max_message_length);
        parts.push(message);

        // Fields
        if !entry.fields.is_empty() {
            let fields: Vec<String> = entry
                .fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            parts.push(format!("fields=[{}]", fields.join(", ")));
        }

        Ok(parts.join(" "))
    }

    fn name(&self) -> &str {
        "plain"
    }

    fn format_type(&self) -> FormatType {
        FormatType::Plain
    }
}

impl Default for PlainFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_plain_formatting() {
        let formatter = PlainFormatter::new();
        let entry = LogEntry {
            level: LogLevel::Info,
            message: "Test message".to_string(),
            timestamp: SystemTime::now(),
            thread_id: Some("main".to_string()),
            source_location: None,
            context: Some(LogContext::new("test".to_string())),
            fields: std::collections::HashMap::new(),
        };

        let formatted = formatter.format(&entry).unwrap();
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("component=test"));
    }

    #[test]
    fn test_plain_formatting_with_fields() {
        let formatter = PlainFormatter::new();
        let mut entry = LogEntry::new(LogLevel::Debug, "Debug message".to_string(), None);
        entry
            .fields
            .insert("key1".to_string(), LogValue::String("value1".to_string()));
        entry
            .fields
            .insert("key2".to_string(), LogValue::Integer(42));

        let formatted = formatter.format(&entry).unwrap();
        assert!(formatted.contains("DEBUG"));
        assert!(formatted.contains("Debug message"));
        assert!(formatted.contains("fields="));
    }

    #[test]
    fn test_message_truncation() {
        let mut config = FormatterConfig::default();
        config.max_message_length = Some(10);
        let formatter = PlainFormatter::with_config(config);

        let entry = LogEntry::new(
            LogLevel::Info,
            "This is a very long message that should be truncated".to_string(),
            None,
        );

        let formatted = formatter.format(&entry).unwrap();
        assert!(formatted.contains("This is..."));
    }
}
