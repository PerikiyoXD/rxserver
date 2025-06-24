//! Structured log formatter.
//!
//! This module provides a structured formatter for log entries.

use crate::logging::formatters::{FormatterConfig, LogFormatter};
use crate::logging::types::*;
use crate::types::Result;

/// Structured log formatter.
#[derive(Debug)]
pub struct StructuredFormatter {
    config: FormatterConfig,
}

impl StructuredFormatter {
    /// Creates a new structured formatter with default configuration.
    pub fn new() -> Self {
        Self {
            config: FormatterConfig::default(),
        }
    }

    /// Creates a new structured formatter with custom configuration.
    pub fn with_config(config: FormatterConfig) -> Self {
        Self { config }
    }
}

impl LogFormatter for StructuredFormatter {
    fn format(&self, entry: &LogEntry) -> Result<String> {
        todo!("Implement structured log formatting")
    }

    fn name(&self) -> &str {
        "structured"
    }

    fn format_type(&self) -> FormatType {
        FormatType::Structured
    }
}

impl Default for StructuredFormatter {
    fn default() -> Self {
        Self::new()
    }
}
