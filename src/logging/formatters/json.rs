//! JSON log formatter.
//!
//! This module provides a JSON formatter for structured log entries.

use crate::logging::formatters::{FormatterConfig, LogFormatter};
use crate::logging::types::*;
use crate::types::Result;

/// JSON log formatter.
#[derive(Debug)]
pub struct JsonFormatter {
    config: FormatterConfig,
}

impl JsonFormatter {
    /// Creates a new JSON formatter with default configuration.
    pub fn new() -> Self {
        Self {
            config: FormatterConfig::default(),
        }
    }

    /// Creates a new JSON formatter with custom configuration.
    pub fn with_config(config: FormatterConfig) -> Self {
        Self { config }
    }
}

impl LogFormatter for JsonFormatter {
    fn format(&self, entry: &LogEntry) -> Result<String> {
        todo!("Implement JSON log formatting")
    }

    fn name(&self) -> &str {
        "json"
    }

    fn format_type(&self) -> FormatType {
        FormatType::Json
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}
