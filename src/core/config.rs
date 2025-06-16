use crate::{ServerError, ServerResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{info, warn};

/// Server configuration loaded from TOML file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub plugins: PluginConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default = "NetworkConfig::default_listen_address")]
    pub listen_address: String,
    #[serde(default = "NetworkConfig::default_port_base")]
    pub port_base: u16,
    #[serde(default)]
    pub tcp_keepalive: bool,
    #[serde(default = "NetworkConfig::default_connection_timeout")]
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "LoggingConfig::default_level")]
    pub level: String,
    pub file: Option<String>,
    #[serde(default = "LoggingConfig::default_colored")]
    pub colored: bool,
    #[serde(default)]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    #[serde(default = "PluginConfig::default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub search_paths: Vec<String>,
    #[serde(default)]
    pub disabled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "SecurityConfig::default_access_control")]
    pub access_control: bool,
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
    #[serde(default = "SecurityConfig::default_max_auth_attempts")]
    pub max_auth_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    #[serde(default = "PerformanceConfig::default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "PerformanceConfig::default_buffer_size")]
    pub buffer_size: usize,
    #[serde(default = "PerformanceConfig::default_request_batching")]
    pub request_batching: bool,
}

impl ServerConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> ServerResult<Self> {
        log::info!("Loading server configuration from {:?}", path.as_ref());
        let path = path.as_ref();

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
                )))
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
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            logging: LoggingConfig::default(),
            plugins: PluginConfig::default(),
            security: SecurityConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl NetworkConfig {
    fn default_listen_address() -> String {
        "127.0.0.1".to_string()
    }
    fn default_port_base() -> u16 {
        6000
    }
    fn default_connection_timeout() -> u64 {
        30
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_address: Self::default_listen_address(),
            port_base: Self::default_port_base(),
            tcp_keepalive: false,
            connection_timeout: Self::default_connection_timeout(),
        }
    }
}

impl LoggingConfig {
    fn default_level() -> String {
        "info".to_string()
    }
    fn default_colored() -> bool {
        true
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Self::default_level(),
            file: None,
            colored: Self::default_colored(),
            json: false,
        }
    }
}

impl PluginConfig {
    fn default_enabled() -> bool {
        true
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
            search_paths: vec![],
            disabled: vec![],
        }
    }
}

impl SecurityConfig {
    fn default_access_control() -> bool {
        true
    }
    fn default_max_auth_attempts() -> u32 {
        3
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            access_control: Self::default_access_control(),
            allowed_hosts: vec![],
            max_auth_attempts: Self::default_max_auth_attempts(),
        }
    }
}

impl PerformanceConfig {
    fn default_max_connections() -> u32 {
        100
    }
    fn default_buffer_size() -> usize {
        8192
    }
    fn default_request_batching() -> bool {
        true
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_connections: Self::default_max_connections(),
            buffer_size: Self::default_buffer_size(),
            request_batching: Self::default_request_batching(),
        }
    }
}
