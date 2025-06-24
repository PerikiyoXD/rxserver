//! Configuration Profiles
//!
//! This module provides predefined configuration profiles for different deployment scenarios.

use super::ServerConfig;

/// Available configuration profiles
#[derive(Debug, Clone, Copy)]
pub enum ConfigProfile {
    /// Development profile with debug settings
    Development,
    /// Production profile with optimized settings
    Production,
    /// Testing profile for automated tests
    Testing,
}

impl ConfigProfile {
    /// Get configuration for this profile
    pub fn config(self) -> ServerConfig {
        match self {
            ConfigProfile::Development => development_config(),
            ConfigProfile::Production => production_config(),
            ConfigProfile::Testing => testing_config(),
        }
    }
}

fn development_config() -> ServerConfig {
    let mut config = ServerConfig::default();
    config.logging.level = "debug".to_string();
    config.performance.worker_threads = 2;
    config.security.enable_host_access_control = false;
    config
}

fn production_config() -> ServerConfig {
    let mut config = ServerConfig::default();
    config.logging.level = "info".to_string();
    config.security.enable_host_access_control = true;
    config.performance.enable_batching = true;
    config
}

fn testing_config() -> ServerConfig {
    let mut config = ServerConfig::default();
    config.logging.level = "warn".to_string();
    config.network.bind_address = "127.0.0.1:0".parse().unwrap(); // Random port
    config.performance.worker_threads = 1;
    config
}
