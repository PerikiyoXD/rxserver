//! Log manager implementation.
//!
//! This module provides the central LogManager that coordinates all logging functionality.

use crate::logging::analysis::LogAnalyzer;
use crate::logging::compression::LogCompressor;
use crate::logging::filtering::LogFilter;
use crate::logging::formatters::LogFormatter;
use crate::logging::outputs::LogOutput;
use crate::logging::rotation::LogRotator;
use crate::logging::types::*;
use crate::types::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Central logging manager that coordinates all logging functionality.
pub struct LogManager {
    config: LoggingConfig,
    outputs: Vec<Box<dyn LogOutput>>,
    filters: Vec<Box<dyn LogFilter>>,
    formatters: HashMap<String, Box<dyn LogFormatter>>,
    analyzer: Option<LogAnalyzer>,
    compressor: Option<LogCompressor>,
    rotator: Option<LogRotator>,
    enabled: bool,
    entry_count: u64,
}

impl LogManager {
    /// Creates a new log manager with default configuration.
    pub fn new() -> Result<Arc<Mutex<Self>>> {
        let config = LoggingConfig::default();
        let manager = Self {
            config,
            outputs: Vec::new(),
            filters: Vec::new(),
            formatters: HashMap::new(),
            analyzer: None,
            compressor: None,
            rotator: None,
            enabled: true,
            entry_count: 0,
        };
        Ok(Arc::new(Mutex::new(manager)))
    }

    /// Creates a new log manager with the specified configuration.
    pub fn with_config(config: LoggingConfig) -> Result<Arc<Mutex<Self>>> {
        let manager = Self {
            config,
            outputs: Vec::new(),
            filters: Vec::new(),
            formatters: HashMap::new(),
            analyzer: None,
            compressor: None,
            rotator: None,
            enabled: true,
            entry_count: 0,
        };
        Ok(Arc::new(Mutex::new(manager)))
    }

    /// Adds a log output.
    pub fn add_output(&mut self, output: Box<dyn LogOutput>) {
        self.outputs.push(output);
    }

    /// Adds a log filter.
    pub fn add_filter(&mut self, filter: Box<dyn LogFilter>) {
        self.filters.push(filter);
    }

    /// Adds a log formatter.
    pub fn add_formatter(&mut self, name: String, formatter: Box<dyn LogFormatter>) {
        self.formatters.insert(name, formatter);
    }

    /// Sets the log analyzer.
    pub fn set_analyzer(&mut self, analyzer: LogAnalyzer) {
        self.analyzer = Some(analyzer);
    }

    /// Sets the log compressor.
    pub fn set_compressor(&mut self, compressor: LogCompressor) {
        self.compressor = Some(compressor);
    }

    /// Sets the log rotator.
    pub fn set_rotator(&mut self, rotator: LogRotator) {
        self.rotator = Some(rotator);
    }

    /// Logs a message with the specified level and context.
    pub async fn log(
        &mut self,
        level: LogLevel,
        message: &str,
        context: Option<LogContext>,
    ) -> Result<()> {
        if !self.enabled || !self.should_log(level) {
            return Ok(());
        }

        let entry = LogEntry::new(level, message.to_string(), context);

        // Apply filters
        if !self.passes_filters(&entry) {
            return Ok(());
        }

        // Send to outputs
        for output in &mut self.outputs {
            if let Some(formatter) = self.formatters.get(&output.formatter_name()) {
                let formatted = formatter.format(&entry)?;
                output.write(&formatted).await?;
            }
        }

        // Send to analyzer
        if let Some(ref mut analyzer) = self.analyzer {
            analyzer.analyze(&entry).await?;
        }

        self.entry_count += 1;

        // Check if rotation is needed
        if let Some(ref mut rotator) = self.rotator {
            rotator.check_rotation(self.entry_count, &entry).await?;
        }

        Ok(())
    }

    /// Logs an error message.
    pub async fn error(&mut self, message: &str) -> Result<()> {
        self.log(LogLevel::Error, message, None).await
    }

    /// Logs a warning message.
    pub async fn warn(&mut self, message: &str) -> Result<()> {
        self.log(LogLevel::Warning, message, None).await
    }

    /// Logs an info message.
    pub async fn info(&mut self, message: &str) -> Result<()> {
        self.log(LogLevel::Info, message, None).await
    }

    /// Logs a debug message.
    pub async fn debug(&mut self, message: &str) -> Result<()> {
        self.log(LogLevel::Debug, message, None).await
    }

    /// Logs a trace message.
    pub async fn trace(&mut self, message: &str) -> Result<()> {
        self.log(LogLevel::Trace, message, None).await
    }

    /// Logs a critical message.
    pub async fn critical(&mut self, message: &str) -> Result<()> {
        self.log(LogLevel::Critical, message, None).await
    }

    /// Enables or disables logging.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Sets the minimum log level.
    pub fn set_level(&mut self, level: LogLevel) {
        self.config.level = level;
    }

    /// Gets the current configuration.
    pub fn config(&self) -> &LoggingConfig {
        &self.config
    }

    /// Updates the configuration.
    pub fn update_config(&mut self, config: LoggingConfig) {
        self.config = config;
    }

    /// Rotates logs if rotation is enabled.
    pub async fn rotate_logs(&mut self) -> Result<()> {
        if let Some(ref mut rotator) = self.rotator {
            rotator.force_rotation().await?;
        }
        Ok(())
    }

    /// Compresses old logs if compression is enabled.
    pub async fn compress_logs(&mut self) -> Result<()> {
        if let Some(ref mut compressor) = self.compressor {
            compressor.compress_archived_logs().await?;
        }
        Ok(())
    }

    /// Gets log analysis results.
    pub async fn get_analysis(&self) -> Result<Option<LogAnalysisResult>> {
        if let Some(ref analyzer) = self.analyzer {
            Ok(Some(analyzer.get_results().await?))
        } else {
            Ok(None)
        }
    }

    /// Gets statistics about logging.
    pub fn get_statistics(&self) -> LogStatistics {
        LogStatistics {
            total_entries: self.entry_count,
            outputs_count: self.outputs.len(),
            filters_count: self.filters.len(),
            formatters_count: self.formatters.len(),
            enabled: self.enabled,
            current_level: self.config.level,
        }
    }

    /// Flushes all outputs.
    pub async fn flush(&mut self) -> Result<()> {
        for output in &mut self.outputs {
            output.flush().await?;
        }
        Ok(())
    }

    /// Shuts down the log manager.
    pub async fn shutdown(&mut self) -> Result<()> {
        self.flush().await?;

        for output in &mut self.outputs {
            output.close().await?;
        }

        Ok(())
    }

    fn should_log(&self, level: LogLevel) -> bool {
        level >= self.config.level
    }

    fn passes_filters(&self, entry: &LogEntry) -> bool {
        for filter in &self.filters {
            if !filter.should_log(entry) {
                return false;
            }
        }
        true
    }
}

/// Statistics about the logging system.
#[derive(Debug, Clone)]
pub struct LogStatistics {
    /// Total number of log entries processed.
    pub total_entries: u64,
    /// Number of configured outputs.
    pub outputs_count: usize,
    /// Number of configured filters.
    pub filters_count: usize,
    /// Number of configured formatters.
    pub formatters_count: usize,
    /// Whether logging is enabled.
    pub enabled: bool,
    /// Current minimum log level.
    pub current_level: LogLevel,
}

/// Log analysis results.
#[derive(Debug, Clone)]
pub struct LogAnalysisResult {
    /// Analysis timestamp.
    pub timestamp: std::time::SystemTime,
    /// Total entries analyzed.
    pub total_entries: u64,
    /// Entries by level.
    pub entries_by_level: HashMap<LogLevel, u64>,
    /// Most frequent messages.
    pub frequent_messages: Vec<(String, u64)>,
    /// Error patterns detected.
    pub error_patterns: Vec<String>,
    /// Performance metrics.
    pub performance_metrics: PerformanceMetrics,
}

/// Performance metrics for logging.
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Average processing time per entry.
    pub avg_processing_time: std::time::Duration,
    /// Maximum processing time.
    pub max_processing_time: std::time::Duration,
    /// Entries processed per second.
    pub entries_per_second: f64,
    /// Memory usage.
    pub memory_usage: u64,
}
