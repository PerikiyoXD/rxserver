//! Environment variable configuration source

use crate::config::sources::ConfigSource;
use crate::config::types::*;
use crate::types::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::env;

/// Environment variable configuration source
pub struct EnvironmentSource {
    prefix: String,
    mapping: HashMap<String, ConfigPath>,
    priority: u32,
}

impl EnvironmentSource {
    /// Create a new environment source with default prefix
    pub fn new() -> Self {
        Self::with_prefix("RXSERVER_")
    }

    /// Create a new environment source with custom prefix
    pub fn with_prefix(prefix: &str) -> Self {
        let mut source = Self {
            prefix: prefix.to_string(),
            mapping: HashMap::new(),
            priority: 200, // Higher than file sources
        };

        source.setup_default_mappings();
        source
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Add custom environment variable mapping
    pub fn add_mapping(&mut self, env_var: &str, config_path: ConfigPath) {
        self.mapping.insert(env_var.to_string(), config_path);
    }

    /// Setup default environment variable mappings
    fn setup_default_mappings(&mut self) {
        // Server configuration
        self.add_mapping("NAME", ConfigPath::new("server.name"));
        self.add_mapping("VERSION", ConfigPath::new("server.version"));
        self.add_mapping("VENDOR", ConfigPath::new("server.vendor"));
        self.add_mapping("DISPLAY_NUMBER", ConfigPath::new("server.display_number"));
        self.add_mapping("SCREEN_COUNT", ConfigPath::new("server.screen_count"));

        // Network configuration
        self.add_mapping("BIND_ADDRESS", ConfigPath::new("network.tcp_addresses.0"));
        self.add_mapping("BIND_PORT", ConfigPath::new("network.tcp_addresses.0"));
        self.add_mapping("UNIX_SOCKET", ConfigPath::new("network.unix_sockets.0"));
        self.add_mapping(
            "MAX_CONNECTIONS",
            ConfigPath::new("network.max_connections"),
        );
        self.add_mapping(
            "CONNECTION_TIMEOUT",
            ConfigPath::new("network.connection_timeout"),
        );
        self.add_mapping(
            "AUTH_ENABLED",
            ConfigPath::new("network.authentication_enabled"),
        );

        // Display configuration
        self.add_mapping(
            "SCREEN_WIDTH",
            ConfigPath::new("display.default_resolution.width"),
        );
        self.add_mapping(
            "SCREEN_HEIGHT",
            ConfigPath::new("display.default_resolution.height"),
        );
        self.add_mapping("COLOR_DEPTH", ConfigPath::new("display.color_depth"));
        self.add_mapping("DPI", ConfigPath::new("display.dpi"));
        self.add_mapping("REFRESH_RATE", ConfigPath::new("display.refresh_rate"));
        self.add_mapping("DISPLAY_BACKEND", ConfigPath::new("display.backend"));

        // Security configuration
        self.add_mapping(
            "ACCESS_CONTROL",
            ConfigPath::new("security.access_control_enabled"),
        );
        self.add_mapping("AUDIT_ENABLED", ConfigPath::new("security.audit_enabled"));
        self.add_mapping("AUDIT_LOG_PATH", ConfigPath::new("security.audit_log_path"));

        // Logging configuration
        self.add_mapping("LOG_LEVEL", ConfigPath::new("logging.level"));
        self.add_mapping("STRUCTURED_LOGS", ConfigPath::new("logging.structured"));

        // Performance configuration
        self.add_mapping(
            "THREAD_POOL_SIZE",
            ConfigPath::new("performance.thread_pool_size"),
        );
        self.add_mapping(
            "REQUEST_QUEUE_SIZE",
            ConfigPath::new("performance.request_queue_size"),
        );
        self.add_mapping(
            "EVENT_QUEUE_SIZE",
            ConfigPath::new("performance.event_queue_size"),
        );

        // Feature toggles
        self.add_mapping(
            "EXTENSIONS_ENABLED",
            ConfigPath::new("features.extensions_enabled"),
        );
        self.add_mapping(
            "COMPOSITING_ENABLED",
            ConfigPath::new("features.compositing_enabled"),
        );
        self.add_mapping(
            "DAMAGE_TRACKING_ENABLED",
            ConfigPath::new("features.damage_tracking_enabled"),
        );
        self.add_mapping(
            "PERFORMANCE_MONITORING_ENABLED",
            ConfigPath::new("features.performance_monitoring_enabled"),
        );
        self.add_mapping(
            "DEBUG_FEATURES_ENABLED",
            ConfigPath::new("features.debug_features_enabled"),
        );
    }

    /// Read environment variables and build configuration
    fn read_environment(&self) -> ServerConfig {
        let mut config = ServerConfig::default();

        for (env_suffix, config_path) in &self.mapping {
            let env_var = format!("{}{}", self.prefix, env_suffix);

            if let Ok(value) = env::var(&env_var) {
                if let Err(e) = self.set_config_value(&mut config, config_path, &value) {
                    log::warn!("Failed to set config value from {}: {}", env_var, e);
                }
            }
        }

        // Handle special cases for arrays/complex types
        self.handle_special_env_vars(&mut config);

        config
    }

    /// Handle special environment variables that don't map directly
    fn handle_special_env_vars(&self, config: &mut ServerConfig) {
        // Handle bind address/port combination
        if let (Ok(addr), Ok(port)) = (
            env::var(&format!("{}BIND_ADDRESS", self.prefix)),
            env::var(&format!("{}BIND_PORT", self.prefix)),
        ) {
            if let Ok(socket_addr) = format!("{}:{}", addr, port).parse() {
                config.network.tcp_addresses = vec![socket_addr];
            }
        }

        // Handle comma-separated lists
        if let Ok(hosts) = env::var(&format!("{}ALLOWED_HOSTS", self.prefix)) {
            config.security.allowed_hosts =
                hosts.split(',').map(|s| s.trim().to_string()).collect();
        }

        if let Ok(auth_methods) = env::var(&format!("{}AUTH_METHODS", self.prefix)) {
            config.network.auth_methods = auth_methods
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }
    }

    /// Set a configuration value using a config path
    fn set_config_value(
        &self,
        config: &mut ServerConfig,
        path: &ConfigPath,
        value: &str,
    ) -> Result<()> {
        match path.as_str() {
            // Server configuration
            "server.name" => config.server.name = value.to_string(),
            "server.version" => config.server.version = value.to_string(),
            "server.vendor" => config.server.vendor = value.to_string(),
            "server.display_number" => config.server.display_number = value.parse().unwrap_or(0),
            "server.screen_count" => config.server.screen_count = value.parse().unwrap_or(1),

            // Network configuration
            "network.max_connections" => {
                config.network.max_connections = value.parse().unwrap_or(100)
            }
            "network.connection_timeout" => {
                config.network.connection_timeout = value.parse().unwrap_or(30)
            }
            "network.authentication_enabled" => {
                config.network.authentication_enabled = value.parse().unwrap_or(true)
            }

            // Display configuration
            "display.default_resolution.width" => {
                config.display.default_resolution.width = value.parse().unwrap_or(1920)
            }
            "display.default_resolution.height" => {
                config.display.default_resolution.height = value.parse().unwrap_or(1080)
            }
            "display.color_depth" => config.display.color_depth = value.parse().unwrap_or(24),
            "display.dpi" => config.display.dpi = value.parse().unwrap_or(96.0),
            "display.refresh_rate" => config.display.refresh_rate = value.parse().unwrap_or(60),
            "display.backend" => config.display.backend = value.to_string(),

            // Security configuration
            "security.access_control_enabled" => {
                config.security.access_control_enabled = value.parse().unwrap_or(true)
            }
            "security.audit_enabled" => {
                config.security.audit_enabled = value.parse().unwrap_or(false)
            }
            "security.audit_log_path" => {
                if !value.is_empty() {
                    config.security.audit_log_path = Some(value.into());
                }
            }

            // Logging configuration
            "logging.level" => config.logging.level = value.to_string(),
            "logging.structured" => config.logging.structured = value.parse().unwrap_or(false),

            // Performance configuration
            "performance.thread_pool_size" => {
                if let Ok(size) = value.parse::<usize>() {
                    config.performance.thread_pool_size = Some(size);
                }
            }
            "performance.request_queue_size" => {
                config.performance.request_queue_size = value.parse().unwrap_or(1000)
            }
            "performance.event_queue_size" => {
                config.performance.event_queue_size = value.parse().unwrap_or(5000)
            }

            // Feature toggles
            "features.extensions_enabled" => {
                config.features.extensions_enabled = value.parse().unwrap_or(true)
            }
            "features.compositing_enabled" => {
                config.features.compositing_enabled = value.parse().unwrap_or(false)
            }
            "features.damage_tracking_enabled" => {
                config.features.damage_tracking_enabled = value.parse().unwrap_or(true)
            }
            "features.performance_monitoring_enabled" => {
                config.features.performance_monitoring_enabled = value.parse().unwrap_or(false)
            }
            "features.debug_features_enabled" => {
                config.features.debug_features_enabled = value.parse().unwrap_or(false)
            }

            _ => {
                log::warn!("Unknown config path: {}", path.as_str());
            }
        }

        Ok(())
    }

    /// Get all environment variables with the configured prefix
    pub fn get_relevant_env_vars(&self) -> HashMap<String, String> {
        env::vars()
            .filter(|(key, _)| key.starts_with(&self.prefix))
            .collect()
    }

    /// List all supported environment variables
    pub fn list_supported_vars(&self) -> Vec<String> {
        self.mapping
            .keys()
            .map(|suffix| format!("{}{}", self.prefix, suffix))
            .collect()
    }
}

impl Default for EnvironmentSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConfigSource for EnvironmentSource {
    async fn load(&self) -> Result<ServerConfig> {
        Ok(self.read_environment())
    }

    fn identifier(&self) -> String {
        format!("environment:{}", self.prefix)
    }

    fn priority(&self) -> u32 {
        self.priority
    }
}

/// Configuration path helper for nested configuration access
#[derive(Debug, Clone)]
pub struct ConfigPath {
    path: String,
}

impl ConfigPath {
    /// Create a new configuration path
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    /// Get the path as a string
    pub fn as_str(&self) -> &str {
        &self.path
    }

    /// Split the path into components
    pub fn components(&self) -> Vec<&str> {
        self.path.split('.').collect()
    }
}

/// System environment detector
pub struct SystemEnvironment;

impl SystemEnvironment {
    /// Detect if running in development environment
    pub fn is_development() -> bool {
        env::var("RXSERVER_ENV").unwrap_or_default() == "development"
            || env::var("NODE_ENV").unwrap_or_default() == "development"
            || cfg!(debug_assertions)
    }

    /// Detect if running in production environment
    pub fn is_production() -> bool {
        env::var("RXSERVER_ENV").unwrap_or_default() == "production"
            || env::var("NODE_ENV").unwrap_or_default() == "production"
    }

    /// Detect if running in testing environment
    pub fn is_testing() -> bool {
        env::var("RXSERVER_ENV").unwrap_or_default() == "test"
            || env::var("NODE_ENV").unwrap_or_default() == "test"
    }

    /// Get environment name
    pub fn environment_name() -> String {
        env::var("RXSERVER_ENV")
            .or_else(|_| env::var("NODE_ENV"))
            .unwrap_or_else(|_| {
                if cfg!(debug_assertions) {
                    "development".to_string()
                } else {
                    "production".to_string()
                }
            })
    }

    /// Get home directory
    pub fn home_directory() -> Option<std::path::PathBuf> {
        dirs::home_dir()
    }

    /// Get config directory
    pub fn config_directory() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|dir| dir.join("rxserver"))
    }

    /// Get data directory
    pub fn data_directory() -> Option<std::path::PathBuf> {
        dirs::data_dir().map(|dir| dir.join("rxserver"))
    }

    /// Get cache directory
    pub fn cache_directory() -> Option<std::path::PathBuf> {
        dirs::cache_dir().map(|dir| dir.join("rxserver"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_environment_source() {
        // Set test environment variables
        unsafe {
            env::set_var("RXSERVER_NAME", "TestServer");
            env::set_var("RXSERVER_MAX_CONNECTIONS", "42");
            env::set_var("RXSERVER_LOG_LEVEL", "debug");
        }
        let source = EnvironmentSource::new();
        let config = source.load().await.unwrap();

        assert_eq!(config.server.name, "TestServer");
        assert_eq!(config.network.max_connections, 42);
        assert_eq!(config.logging.level, "debug");

        // Clean up
        unsafe {
            env::remove_var("RXSERVER_NAME");
            env::remove_var("RXSERVER_MAX_CONNECTIONS");
            env::remove_var("RXSERVER_LOG_LEVEL");
        }
    }

    #[test]
    fn test_config_path() {
        let path = ConfigPath::new("server.network.port");
        assert_eq!(path.as_str(), "server.network.port");
        assert_eq!(path.components(), vec!["server", "network", "port"]);
    }

    #[test]
    fn test_system_environment() {
        // Test environment detection (these may vary based on actual environment)
        let env_name = SystemEnvironment::environment_name();
        assert!(!env_name.is_empty());

        // These should not panic
        let _ = SystemEnvironment::is_development();
        let _ = SystemEnvironment::is_production();
        let _ = SystemEnvironment::is_testing();
    }
}
