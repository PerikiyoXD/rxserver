//! Syslog output.
//!
//! This module provides syslog output for log entries.

use crate::logging::outputs::{LogOutput, OutputConfig};
use crate::logging::types::*;
use crate::types::Result;
use async_trait::async_trait;

/// Syslog output.
#[derive(Debug)]
pub struct SyslogOutput {
    config: OutputConfig,
}

impl SyslogOutput {
    /// Creates a new syslog output.
    pub fn new() -> Self {
        Self {
            config: OutputConfig::default(),
        }
    }

    /// Creates a new syslog output with custom configuration.
    pub fn with_config(config: OutputConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl LogOutput for SyslogOutput {
    async fn write(&mut self, message: &str) -> Result<()> {
        todo!("Implement syslog output")
    }

    async fn flush(&mut self) -> Result<()> {
        Ok(()) // Syslog doesn't need explicit flushing
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    fn formatter_name(&self) -> String {
        self.config.formatter.clone()
    }

    fn output_type(&self) -> OutputType {
        OutputType::Syslog
    }

    async fn health_check(&self) -> Result<bool> {
        todo!("Implement syslog health check")
    }
}

impl Default for SyslogOutput {
    fn default() -> Self {
        Self::new()
    }
}
