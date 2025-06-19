// SPDX-License-Identifier: Apache-2.0
//! RX X11 Server - Configuration

use crate::{ServerError, ServerResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{info, warn};

/// Server configuration loaded from TOML file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfig {
    #[serde(default)]
    pub network: ServerNetworkConfig,
    #[serde(default)]
    pub logging: ServerLoggingConfig,
    #[serde(default)]
    pub plugins: ServerPluginConfig,
    #[serde(default)]
    pub security: ServerSecurityConfig,
    #[serde(default)]
    pub performance: ServerPerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerNetworkConfig {
    #[serde(default = "default_listen_address")]
    pub listen_address: String,
    #[serde(default = "default_port_base")]
    pub port_base: u16,
    #[serde(default)]
    pub tcp_keepalive: bool,
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerLoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    pub file: Option<String>,
    #[serde(default = "default_colored")]
    pub colored: bool,
    #[serde(default)]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPluginConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub search_paths: Vec<String>,
    #[serde(default)]
    pub disabled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSecurityConfig {
    #[serde(default = "default_access_control")]
    pub access_control: bool,
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
    #[serde(default = "default_max_auth_attempts")]
    pub max_auth_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPerformanceConfig {
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    #[serde(default = "default_request_batching")]
    pub request_batching: bool,
}

// Default value functions
fn default_listen_address() -> String {
    "127.0.0.1".to_string()
}
fn default_port_base() -> u16 {
    6000
}
fn default_connection_timeout() -> u64 {
    30
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_colored() -> bool {
    true
}
fn default_enabled() -> bool {
    true
}
fn default_access_control() -> bool {
    true
}
fn default_max_auth_attempts() -> u32 {
    3
}
fn default_max_connections() -> u32 {
    100
}
fn default_buffer_size() -> usize {
    8192
}
fn default_request_batching() -> bool {
    true
}

impl Default for ServerNetworkConfig {
    fn default() -> Self {
        Self {
            listen_address: default_listen_address(),
            port_base: default_port_base(),
            tcp_keepalive: false,
            connection_timeout: default_connection_timeout(),
        }
    }
}

impl Default for ServerLoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: None,
            colored: default_colored(),
            json: false,
        }
    }
}

impl Default for ServerPluginConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            search_paths: vec![],
            disabled: vec![],
        }
    }
}

impl Default for ServerSecurityConfig {
    fn default() -> Self {
        Self {
            access_control: default_access_control(),
            allowed_hosts: vec![],
            max_auth_attempts: default_max_auth_attempts(),
        }
    }
}

impl Default for ServerPerformanceConfig {
    fn default() -> Self {
        Self {
            max_connections: default_max_connections(),
            buffer_size: default_buffer_size(),
            request_batching: default_request_batching(),
        }
    }
}

impl ServerConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> ServerResult<Self> {
        let path = path.as_ref();
        info!("Loading server configuration from {:?}", path);

        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                info!("Config file {:?} not found, creating default", path);
                let config = Self::default();
                if let Err(e) = config.save(path) {
                    warn!("Failed to save default config: {}", e);
                }
                return Ok(config);
            }
            Err(e) => {
                return Err(ServerError::ConfigError(format!(
                    "Failed to read config: {}",
                    e
                )));
            }
        };

        toml::from_str(&content)
            .map_err(|e| ServerError::ConfigError(format!("Failed to parse config: {}", e)))
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> ServerResult<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ServerError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, content)
            .map_err(|e| ServerError::ConfigError(format!("Failed to write config: {}", e)))
    }

    pub fn validate(&self) -> ServerResult<()> {
        if self.logging.level.is_empty() {
            return Err(ServerError::ConfigError(
                "Log level cannot be empty".to_string(),
            ));
        }
        if self.performance.max_connections == 0 {
            return Err(ServerError::ConfigError(
                "Max connections must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}
