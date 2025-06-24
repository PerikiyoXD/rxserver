//! Configuration Validation
//!
//! This module provides validation logic for server configuration.

use super::ServerConfig;

/// Validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid network configuration: {0}")]
    Network(String),
    #[error("Invalid security configuration: {0}")]
    Security(String),
    #[error("Invalid performance configuration: {0}")]
    Performance(String),
    #[error("Invalid logging configuration: {0}")]
    Logging(String),
    #[error("Invalid plugin configuration: {0}")]
    Plugin(String),
}

/// Validate server configuration
pub fn validate_config(config: &ServerConfig) -> Result<(), ValidationError> {
    validate_network(&config.network)?;
    validate_security(&config.security)?;
    validate_performance(&config.performance)?;
    validate_logging(&config.logging)?;
    validate_plugins(&config.plugins)?;
    Ok(())
}

fn validate_network(config: &super::NetworkConfig) -> Result<(), ValidationError> {
    if config.max_connections == 0 {
        return Err(ValidationError::Network(
            "max_connections must be greater than 0".to_string(),
        ));
    }
    Ok(())
}

fn validate_security(config: &super::SecurityConfig) -> Result<(), ValidationError> {
    if config.max_auth_attempts == 0 {
        return Err(ValidationError::Security(
            "max_auth_attempts must be greater than 0".to_string(),
        ));
    }
    Ok(())
}

fn validate_performance(config: &super::PerformanceConfig) -> Result<(), ValidationError> {
    if config.worker_threads == 0 {
        return Err(ValidationError::Performance(
            "worker_threads must be greater than 0".to_string(),
        ));
    }
    Ok(())
}

fn validate_logging(_config: &super::LoggingConfig) -> Result<(), ValidationError> {
    // TODO: Validate log level string
    Ok(())
}

fn validate_plugins(_config: &super::PluginConfig) -> Result<(), ValidationError> {
    // TODO: Validate plugin directory exists
    Ok(())
}
