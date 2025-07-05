use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::display::config::DisplayConfig;
use crate::logging::LoggingConfig;

/// Configuration containing all displays
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub logging: LoggingConfig,
    pub displays: Vec<DisplayConfig>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            displays: vec![DisplayConfig::default()],
            logging: LoggingConfig::default(),
        }
    }
}

/// Load the server configuration from a file.
/// If the file does not exist or is invalid, return a default configuration.
pub fn load_config(path: Option<String>) -> anyhow::Result<ServerConfig> {
    let config_path = path.unwrap_or_else(|| "rxserver.toml".to_string());
    let mut contents = String::new();
    if let Ok(mut file) = File::open(&config_path) {
        if file.read_to_string(&mut contents).is_ok() {
            if let Ok(cfg) = toml::from_str(&contents) {
                return Ok(cfg);
            }
        }
    }
    Ok(ServerConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let config = load_config(None);
        assert!(config.is_ok());
    }
}
