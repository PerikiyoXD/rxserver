use serde::{Deserialize, Serialize};
use std::fs;
use tracing::warn;

use crate::display::config::DisplayConfig;
use crate::logging::LoggingConfig;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default = "default_displays")]
    pub displays: Vec<DisplayConfig>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig::default(),
            displays: default_displays(),
        }
    }
}

fn default_displays() -> Vec<DisplayConfig> {
    vec![DisplayConfig::default()]
}

pub fn load_config(path: Option<&str>) -> anyhow::Result<ServerConfig> {
    let config_path = path.unwrap_or("rxserver.toml");

    let contents = match fs::read_to_string(config_path) {
        Ok(contents) => contents,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            warn!("Config file '{}' not found, using defaults", config_path);
            return Ok(ServerConfig::default());
        }
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Cannot read config file '{}': {}",
                config_path,
                e
            ));
        }
    };

    toml::from_str(&contents)
        .map_err(|e| anyhow::anyhow!("Invalid config file '{}': {}", config_path, e))
}
