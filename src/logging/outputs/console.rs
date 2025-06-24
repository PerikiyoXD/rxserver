//! Console log output.
//!
//! This module provides console/terminal output for log entries.

use crate::logging::outputs::{LogOutput, OutputConfig};
use crate::logging::types::*;
use crate::types::Result;
use async_trait::async_trait;

/// Console log output that writes to stdout/stderr.
#[derive(Debug)]
pub struct ConsoleOutput {
    config: OutputConfig,
    use_stderr_for_errors: bool,
}

impl ConsoleOutput {
    /// Creates a new console output with default configuration.
    pub fn new() -> Self {
        Self {
            config: OutputConfig::default(),
            use_stderr_for_errors: true,
        }
    }

    /// Creates a new console output with custom configuration.
    pub fn with_config(config: OutputConfig) -> Self {
        Self {
            config,
            use_stderr_for_errors: true,
        }
    }

    /// Sets whether to use stderr for error messages.
    pub fn use_stderr_for_errors(mut self, use_stderr: bool) -> Self {
        self.use_stderr_for_errors = use_stderr;
        self
    }
}

#[async_trait]
impl LogOutput for ConsoleOutput {
    async fn write(&mut self, message: &str) -> Result<()> {
        // For now, just use println! - in a real implementation you'd
        // parse the log level from the message and route appropriately
        if self.use_stderr_for_errors && message.contains("ERROR") {
            eprintln!("{}", message);
        } else {
            println!("{}", message);
        }
        Ok(())
    }
    async fn flush(&mut self) -> Result<()> {
        use std::io::{self, Write};
        io::stdout()
            .flush()
            .map_err(|e| crate::types::ServerError::Io(e.to_string()))?;
        io::stderr()
            .flush()
            .map_err(|e| crate::types::ServerError::Io(e.to_string()))?;
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        self.flush().await
    }

    fn formatter_name(&self) -> String {
        self.config.formatter.clone()
    }

    fn output_type(&self) -> OutputType {
        OutputType::Console
    }

    async fn health_check(&self) -> Result<bool> {
        // Console is always available
        Ok(true)
    }
}

impl Default for ConsoleOutput {
    fn default() -> Self {
        Self::new()
    }
}
