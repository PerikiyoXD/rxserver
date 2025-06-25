//! Configuration Profiles
//!
//! This module provides predefined configuration profiles for different deployment scenarios.

use super::ServerConfig;
use std::time::Duration;

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
    tracing::debug!("Using development configuration profile");
    let mut config = ServerConfig::default();
    config.logging.level = "debug".to_string();
    config.performance.worker_threads = 2;
    config.security.enable_host_access_control = false;

    // Development discovery settings
    config.discovery.health_check_interval = Duration::from_secs(10); // More frequent checks
    config.discovery.enable_metrics = true;
    config.discovery.enable_mdns = true;
    config.discovery.enable_dns_sd = false; // Simpler setup for dev
    config.discovery.persistence_file = Some("dev_services.json".into());

    config
}

fn production_config() -> ServerConfig {
    tracing::debug!("Using production configuration profile");
    let mut config = ServerConfig::default();
    config.logging.level = "info".to_string();
    config.security.enable_host_access_control = true;
    config.performance.enable_batching = true;

    // Production discovery settings - more conservative
    config.discovery.health_check_interval = Duration::from_secs(60);
    config.discovery.health_check_retries = 5;
    config.discovery.enable_mdns = true;
    config.discovery.enable_dns_sd = true; // Full discovery in production
    config.discovery.enable_broadcast = false; // Security concern
    config.discovery.cache_ttl = Duration::from_secs(600); // Longer cache
    config.discovery.persistence_file = Some("/var/lib/rxserver/services.json".into());

    config
}

fn testing_config() -> ServerConfig {
    tracing::debug!("Using testing configuration profile");
    let mut config = ServerConfig::default();
    config.logging.level = "warn".to_string();
    config.network.bind_address = "127.0.0.1:0".parse().unwrap(); // Random port
    config.performance.worker_threads = 1;

    // Testing discovery settings - minimal overhead
    config.discovery.enabled = false; // Disable discovery for tests by default
    config.discovery.health_check_interval = Duration::from_secs(5); // Fast for tests
    config.discovery.enable_mdns = false;
    config.discovery.enable_dns_sd = false;
    config.discovery.enable_broadcast = false;
    config.discovery.persistence_file = None; // No persistence in tests
    config.discovery.enable_metrics = false;

    config
}
