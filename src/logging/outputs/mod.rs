//! Log output destinations.
//!
//! This module provides various output destinations for log entries.

use crate::logging::types::*;
use crate::types::Result;
use async_trait::async_trait;

pub mod console;
pub mod database;
pub mod file;
pub mod journald;
pub mod network;
pub mod syslog;

pub use console::*;
pub use database::*;
pub use file::*;
pub use journald::*;
pub use network::*;
pub use syslog::*;

/// Trait for log output destinations.
#[async_trait]
pub trait LogOutput: Send + Sync {
    /// Writes a formatted log message to the output.
    async fn write(&mut self, message: &str) -> Result<()>;

    /// Flushes any buffered output.
    async fn flush(&mut self) -> Result<()>;

    /// Closes the output and cleans up resources.
    async fn close(&mut self) -> Result<()>;

    /// Gets the name of the formatter this output expects.
    fn formatter_name(&self) -> String;

    /// Gets the output type.
    fn output_type(&self) -> OutputType;

    /// Checks if the output is healthy and operational.
    async fn health_check(&self) -> Result<bool>;
}

/// Creates an output by type.
pub fn create_output(output_type: OutputType) -> Result<Box<dyn LogOutput>> {
    match output_type {
        OutputType::Console => Ok(Box::new(ConsoleOutput::new())),
        OutputType::File => Ok(Box::new(FileOutput::new("logs/rxserver.log")?)),
        OutputType::Syslog => Ok(Box::new(SyslogOutput::new())),
        OutputType::Journald => Ok(Box::new(JournaldOutput::new())),
        OutputType::Network => Ok(Box::new(NetworkOutput::new("localhost:514")?)),
        OutputType::Database => Ok(Box::new(DatabaseOutput::new("sqlite:logs.db")?)),
    }
}

/// Configuration for output destinations.
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// Output type.
    pub output_type: OutputType,
    /// Associated formatter name.
    pub formatter: String,
    /// Whether the output is enabled.
    pub enabled: bool,
    /// Buffer size for the output.
    pub buffer_size: usize,
    /// Timeout for write operations.
    pub write_timeout: std::time::Duration,
    /// Maximum retries for failed writes.
    pub max_retries: u32,
    /// Whether to enable compression.
    pub compression: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            output_type: OutputType::Console,
            formatter: "plain".to_string(),
            enabled: true,
            buffer_size: 1024,
            write_timeout: std::time::Duration::from_secs(5),
            max_retries: 3,
            compression: false,
        }
    }
}
