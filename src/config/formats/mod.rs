//! Configuration format support
//!
//! This module provides support for various configuration file formats
//! including TOML, JSON, YAML, XML, and INI.

pub mod ini;
pub mod json;
pub mod toml;
pub mod xml;
pub mod yaml;

use crate::config::types::ServerConfig;
use crate::types::{ConfigurationError, Result};
use std::path::Path;
use std::str::FromStr;

/// Supported configuration formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// TOML format (.toml)
    Toml,
    /// JSON format (.json)
    Json,
    /// YAML format (.yaml, .yml)
    Yaml,
    /// XML format (.xml)
    Xml,
    /// INI format (.ini, .conf)
    Ini,
}

impl ConfigFormat {
    /// Detect format from file extension
    pub fn from_extension(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "toml" => Some(Self::Toml),
            "json" => Some(Self::Json),
            "yaml" | "yml" => Some(Self::Yaml),
            "xml" => Some(Self::Xml),
            "ini" | "conf" => Some(Self::Ini),
            _ => None,
        }
    }

    /// Get the primary file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Toml => "toml",
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Xml => "xml",
            Self::Ini => "ini",
        }
    }

    /// Get MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Toml => "application/toml",
            Self::Json => "application/json",
            Self::Yaml => "application/yaml",
            Self::Xml => "application/xml",
            Self::Ini => "text/plain",
        }
    }

    /// Parse configuration from string
    pub fn parse(&self, content: &str) -> Result<ServerConfig> {
        match self {
            Self::Toml => toml::parse(content),
            Self::Json => json::parse(content),
            Self::Yaml => yaml::parse(content),
            Self::Xml => xml::parse(content),
            Self::Ini => ini::parse(content),
        }
    }

    /// Serialize configuration to string
    pub fn serialize(&self, config: &ServerConfig) -> Result<String> {
        match self {
            Self::Toml => toml::serialize(config),
            Self::Json => json::serialize(config),
            Self::Yaml => yaml::serialize(config),
            Self::Xml => xml::serialize(config),
            Self::Ini => ini::serialize(config),
        }
    }

    /// Check if this format supports comments
    pub fn supports_comments(&self) -> bool {
        matches!(self, Self::Toml | Self::Yaml | Self::Ini)
    }

    /// Check if this format supports environment variable substitution
    pub fn supports_env_substitution(&self) -> bool {
        // All formats can support env substitution via preprocessing
        true
    }
}

impl FromStr for ConfigFormat {
    type Err = ConfigurationError;

    fn from_str(s: &str) -> std::result::Result<Self, ConfigurationError> {
        match s.to_lowercase().as_str() {
            "toml" => Ok(Self::Toml),
            "json" => Ok(Self::Json),
            "yaml" | "yml" => Ok(Self::Yaml),
            "xml" => Ok(Self::Xml),
            "ini" | "conf" => Ok(Self::Ini),
            _ => Err(ConfigurationError::UnsupportedFormat(s.to_string())),
        }
    }
}

impl std::fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Toml => write!(f, "TOML"),
            Self::Json => write!(f, "JSON"),
            Self::Yaml => write!(f, "YAML"),
            Self::Xml => write!(f, "XML"),
            Self::Ini => write!(f, "INI"),
        }
    }
}

/// Format detection utility
pub struct FormatDetector;

impl FormatDetector {
    /// Detect format from content
    pub fn detect_from_content(content: &str) -> Option<ConfigFormat> {
        let trimmed = content.trim_start();

        // JSON detection
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            return Some(ConfigFormat::Json);
        }

        // XML detection
        if trimmed.starts_with("<?xml") || trimmed.starts_with('<') {
            return Some(ConfigFormat::Xml);
        }

        // YAML detection (starts with ---, has key: value, or starts with -)
        if trimmed.starts_with("---")
            || trimmed.lines().any(|line| line.contains(": "))
            || trimmed.starts_with('-')
        {
            return Some(ConfigFormat::Yaml);
        }

        // INI detection (has [section] headers)
        if trimmed.lines().any(|line| {
            let line = line.trim();
            line.starts_with('[') && line.ends_with(']')
        }) {
            return Some(ConfigFormat::Ini);
        }

        // Default to TOML if none of the above match
        Some(ConfigFormat::Toml)
    }

    /// Detect format from file path and optionally content
    pub fn detect(path: &Path, content: Option<&str>) -> ConfigFormat {
        // First try to detect from file extension
        if let Some(format) = ConfigFormat::from_extension(path) {
            return format;
        }

        // If no extension or unknown extension, try content detection
        if let Some(content) = content {
            if let Some(format) = Self::detect_from_content(content) {
                return format;
            }
        }

        // Default to TOML
        ConfigFormat::Toml
    }
}

/// Configuration format registry for custom formats
pub struct FormatRegistry {
    parsers:
        std::collections::HashMap<String, Box<dyn Fn(&str) -> Result<ServerConfig> + Send + Sync>>,
    serializers: std::collections::HashMap<
        String,
        Box<dyn Fn(&ServerConfig) -> Result<String> + Send + Sync>,
    >,
}

impl FormatRegistry {
    /// Create a new format registry
    pub fn new() -> Self {
        Self {
            parsers: std::collections::HashMap::new(),
            serializers: std::collections::HashMap::new(),
        }
    }

    /// Register a custom format parser
    pub fn register_parser<F>(&mut self, format: &str, parser: F)
    where
        F: Fn(&str) -> Result<ServerConfig> + Send + Sync + 'static,
    {
        self.parsers.insert(format.to_string(), Box::new(parser));
    }

    /// Register a custom format serializer
    pub fn register_serializer<F>(&mut self, format: &str, serializer: F)
    where
        F: Fn(&ServerConfig) -> Result<String> + Send + Sync + 'static,
    {
        self.serializers
            .insert(format.to_string(), Box::new(serializer));
    }

    /// Parse using a custom format
    pub fn parse(&self, format: &str, content: &str) -> Result<ServerConfig> {
        self.parsers
            .get(format)
            .ok_or_else(|| ConfigurationError::UnsupportedFormat(format.to_string()))?(
            content
        )
    }

    /// Serialize using a custom format
    pub fn serialize(&self, format: &str, config: &ServerConfig) -> Result<String> {
        self.serializers
            .get(format)
            .ok_or_else(|| ConfigurationError::UnsupportedFormat(format.to_string()))?(
            config
        )
    }

    /// Check if a format is registered
    pub fn supports(&self, format: &str) -> bool {
        self.parsers.contains_key(format)
    }

    /// List all registered formats
    pub fn formats(&self) -> Vec<String> {
        self.parsers.keys().cloned().collect()
    }
}

impl Default for FormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}
