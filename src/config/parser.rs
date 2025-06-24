//! Configuration parser utilities

use crate::config::formats::{ConfigFormat, FormatDetector};
use crate::config::types::ServerConfig;
use crate::types::{ConfigurationError, Result};
use std::path::Path;

/// Configuration parser that can handle multiple formats
pub struct ConfigParser {
    auto_detect_format: bool,
    default_format: ConfigFormat,
}

impl ConfigParser {
    /// Create a new parser with auto-detection enabled
    pub fn new() -> Self {
        Self {
            auto_detect_format: true,
            default_format: ConfigFormat::Toml,
        }
    }

    /// Create a parser with a specific default format
    pub fn with_default_format(format: ConfigFormat) -> Self {
        Self {
            auto_detect_format: true,
            default_format: format,
        }
    }

    /// Disable format auto-detection
    pub fn no_auto_detect(mut self) -> Self {
        self.auto_detect_format = false;
        self
    }

    /// Parse configuration from string with optional format hint
    pub fn parse_string(
        &self,
        content: &str,
        format: Option<ConfigFormat>,
    ) -> Result<ServerConfig> {
        let format = format.unwrap_or_else(|| {
            if self.auto_detect_format {
                FormatDetector::detect_from_content(content).unwrap_or(self.default_format)
            } else {
                self.default_format
            }
        });

        format.parse(content)
    }

    /// Parse configuration from file
    pub async fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<ServerConfig> {
        let path = path.as_ref();
        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            crate::types::Error::Configuration(ConfigurationError::FileError {
                path: path.to_path_buf(),
                message: e.to_string(),
            })
        })?;

        let format = if self.auto_detect_format {
            FormatDetector::detect(path, Some(&content))
        } else {
            self.default_format
        };

        format.parse(&content)
    }

    /// Validate configuration syntax without parsing
    pub fn validate_syntax(&self, content: &str, format: Option<ConfigFormat>) -> Result<()> {
        let format = format.unwrap_or_else(|| {
            if self.auto_detect_format {
                FormatDetector::detect_from_content(content).unwrap_or(self.default_format)
            } else {
                self.default_format
            }
        });

        // For now, just try to parse and return success if no error
        format.parse(content).map(|_| ())
    }
}

impl Default for ConfigParser {
    fn default() -> Self {
        Self::new()
    }
}
