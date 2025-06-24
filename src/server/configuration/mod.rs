//! Server Configuration Management
//!
//! This module handles server configuration loading, validation, and hot reloading.

pub mod hot_reload;
pub mod profiles;
pub mod schema;
pub mod validation;

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

/// Main server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Network configuration
    pub network: NetworkConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Plugin configuration
    pub plugins: PluginConfig,
    /// Display configuration
    pub display: Option<crate::display::DisplayConfig>,
    /// Input configuration
    pub input: Option<crate::input::InputConfiguration>,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Address to bind to
    pub bind_address: SocketAddr,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Enable TCP keepalive
    pub tcp_keepalive: bool,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable host-based access control
    pub enable_host_access_control: bool,
    /// Allowed hosts
    pub allowed_hosts: Vec<String>,
    /// Enable authentication
    pub enable_authentication: bool,
    /// Maximum failed authentication attempts
    pub max_auth_attempts: u32,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Number of worker threads
    pub worker_threads: usize,
    /// Request queue size
    pub request_queue_size: usize,
    /// Enable request batching
    pub enable_batching: bool,
    /// Batch timeout
    pub batch_timeout: Duration,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Log file path
    pub file: Option<PathBuf>,
    /// Enable structured logging
    pub structured: bool,
    /// Log rotation size
    pub rotation_size: Option<u64>,
    /// Enable block-style protocol logging
    #[serde(default)]
    pub block_style_protocol_logging: bool,
    /// Minimum log level for protocol operations
    #[serde(default = "default_protocol_log_level")]
    pub protocol_log_level: String,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Enable plugins
    pub enabled: bool,
    /// Plugin directory
    pub plugin_dir: PathBuf,
    /// Auto-load plugins
    pub auto_load: bool,
}

/// Default protocol log level
fn default_protocol_log_level() -> String {
    "debug".to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            security: SecurityConfig::default(),
            performance: PerformanceConfig::default(),
            logging: LoggingConfig::default(),
            plugins: PluginConfig::default(),
            display: Some(crate::display::DisplayConfig::default()),
            input: Some(crate::input::InputConfiguration::default()),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:6000".parse().unwrap(),
            max_connections: 1000,
            connection_timeout: Duration::from_secs(30),
            tcp_keepalive: true,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_host_access_control: false,
            allowed_hosts: vec!["localhost".to_string(), "127.0.0.1".to_string()],
            enable_authentication: false,
            max_auth_attempts: 3,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            request_queue_size: 10000,
            enable_batching: true,
            batch_timeout: Duration::from_millis(10),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "trace".to_string(),
            file: None,
            structured: false,
            rotation_size: Some(100 * 1024 * 1024), // 100MB
            block_style_protocol_logging: false,
            protocol_log_level: "debug".to_string(),
        }
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            plugin_dir: PathBuf::from("plugins"),
            auto_load: true,
        }
    }
}

impl ServerConfig {
    /// Load configuration from a file
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;

        // Determine format based on file extension
        let config = match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => toml::from_str(&content)?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
            Some("json") => serde_json::from_str(&content)?,
            _ => {
                // Try to parse as TOML by default
                toml::from_str(&content)?
            }
        };

        Ok(config)
    }

    /// Save configuration to a file
    pub fn save<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();

        let content = match path.extension().and_then(|ext| ext.to_str()) {
            Some("yaml") | Some("yml") => serde_yaml::to_string(self)?,
            Some("json") => serde_json::to_string_pretty(self)?,
            _ => toml::to_string_pretty(self)?,
        };

        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate network configuration
        if self.network.max_connections == 0 {
            return Err("max_connections must be greater than 0".to_string());
        }

        // Validate performance configuration
        if self.performance.worker_threads == 0 {
            return Err("worker_threads must be greater than 0".to_string());
        }

        if self.performance.request_queue_size == 0 {
            return Err("request_queue_size must be greater than 0".to_string());
        }

        // Validate logging configuration
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(format!("invalid log level: {}", self.logging.level));
        }

        Ok(())
    }
}
