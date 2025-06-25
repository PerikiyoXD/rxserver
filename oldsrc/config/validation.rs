//! Configuration validation

use crate::config::types::ServerConfig;
use crate::types::{ConfigurationError, Result};

/// Configuration validator
#[derive(Debug, Clone)]
pub struct ConfigValidator {
    strict_mode: bool,
}

impl ConfigValidator {
    /// Create a new validator
    pub fn new() -> Result<Self> {
        Ok(Self { strict_mode: false })
    }

    /// Enable strict validation mode
    pub fn strict(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    /// Validate configuration
    pub fn validate(&self, config: &ServerConfig) -> Result<()> {
        self.validate_server_config(&config.server)?;
        self.validate_network_config(&config.network)?;
        self.validate_display_config(&config.display)?;
        self.validate_security_config(&config.security)?;
        self.validate_logging_config(&config.logging)?;
        self.validate_performance_config(&config.performance)?;
        self.validate_features_config(&config.features)?;
        Ok(())
    }

    fn validate_server_config(&self, config: &crate::config::types::ServerInfo) -> Result<()> {
        if config.name.is_empty() {
            return Err(crate::types::Error::Configuration(
                ConfigurationError::InvalidValue {
                    key: "server.name".to_string(),
                    value: config.name.clone(),
                    reason: "Server name cannot be empty".to_string(),
                },
            ));
        }

        if config.screen_count == 0 {
            return Err(crate::types::Error::Configuration(
                ConfigurationError::InvalidValue {
                    key: "server.screen_count".to_string(),
                    value: config.screen_count.to_string(),
                    reason: "Screen count must be at least 1".to_string(),
                },
            ));
        }

        Ok(())
    }

    fn validate_network_config(&self, config: &crate::config::types::NetworkConfig) -> Result<()> {
        if config.tcp_addresses.is_empty() && config.unix_sockets.is_empty() {
            return Err(crate::types::Error::Configuration(
                ConfigurationError::InvalidValue {
                    key: "network".to_string(),
                    value: "no addresses".to_string(),
                    reason: "At least one TCP address or Unix socket must be configured"
                        .to_string(),
                },
            ));
        }

        if config.max_connections == 0 {
            return Err(crate::types::Error::Configuration(
                ConfigurationError::InvalidValue {
                    key: "network.max_connections".to_string(),
                    value: config.max_connections.to_string(),
                    reason: "Max connections must be greater than 0".to_string(),
                },
            ));
        }

        Ok(())
    }

    fn validate_display_config(&self, config: &crate::config::types::DisplayConfig) -> Result<()> {
        if config.default_resolution.width == 0 || config.default_resolution.height == 0 {
            return Err(crate::types::Error::Configuration(
                ConfigurationError::InvalidValue {
                    key: "display.default_resolution".to_string(),
                    value: format!(
                        "{}x{}",
                        config.default_resolution.width, config.default_resolution.height
                    ),
                    reason: "Resolution dimensions must be greater than 0".to_string(),
                },
            ));
        }

        if config.color_depth != 8
            && config.color_depth != 16
            && config.color_depth != 24
            && config.color_depth != 32
        {
            return Err(crate::types::Error::Configuration(
                ConfigurationError::InvalidValue {
                    key: "display.color_depth".to_string(),
                    value: config.color_depth.to_string(),
                    reason: "Color depth must be 8, 16, 24, or 32 bits".to_string(),
                },
            ));
        }

        if config.dpi <= 0.0 {
            return Err(crate::types::Error::Configuration(
                ConfigurationError::InvalidValue {
                    key: "display.dpi".to_string(),
                    value: config.dpi.to_string(),
                    reason: "DPI must be greater than 0".to_string(),
                },
            ));
        }

        Ok(())
    }

    fn validate_security_config(
        &self,
        _config: &crate::config::types::SecurityConfig,
    ) -> Result<()> {
        // Basic security validation
        Ok(())
    }

    fn validate_logging_config(&self, config: &crate::config::types::LoggingConfig) -> Result<()> {
        let valid_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_levels.contains(&config.level.as_str()) {
            return Err(crate::types::Error::Configuration(
                ConfigurationError::InvalidValue {
                    key: "logging.level".to_string(),
                    value: config.level.clone(),
                    reason: format!("Log level must be one of: {}", valid_levels.join(", ")),
                },
            ));
        }

        Ok(())
    }

    fn validate_performance_config(
        &self,
        config: &crate::config::types::PerformanceConfig,
    ) -> Result<()> {
        if config.request_queue_size == 0 {
            return Err(crate::types::Error::Configuration(
                ConfigurationError::InvalidValue {
                    key: "performance.request_queue_size".to_string(),
                    value: config.request_queue_size.to_string(),
                    reason: "Request queue size must be greater than 0".to_string(),
                },
            ));
        }

        if config.event_queue_size == 0 {
            return Err(crate::types::Error::Configuration(
                ConfigurationError::InvalidValue {
                    key: "performance.event_queue_size".to_string(),
                    value: config.event_queue_size.to_string(),
                    reason: "Event queue size must be greater than 0".to_string(),
                },
            ));
        }

        Ok(())
    }

    fn validate_features_config(
        &self,
        _config: &crate::config::types::FeatureConfig,
    ) -> Result<()> {
        // Feature validation would check for conflicting features, etc.
        Ok(())
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new().expect("Failed to create default ConfigValidator")
    }
}
