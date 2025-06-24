//! Configuration management for the X11 server
//!
//! This module provides comprehensive configuration management capabilities,
//! including multiple format support, hot reloading, validation, and profiling.

use crate::types::{ConfigurationError, Result};
use serde::{Deserialize, Serialize};

// Re-export configuration modules
pub mod defaults;
pub mod formats;
pub mod hot_reload;
pub mod migration;
pub mod parser;
pub mod profiles;
pub mod schema;
pub mod sources;
pub mod types;
pub mod validation;

// Re-export main types for convenience
pub use defaults::DefaultConfig;
pub use formats::ConfigFormat;
pub use parser::ConfigParser;
pub use profiles::ProfileManager;
pub use schema::ConfigSchema;
pub use sources::ConfigSource;
pub use types::*;
pub use validation::ConfigValidator;

/// Main configuration manager for the X11 server
pub struct ConfigManager {
    /// Current active configuration
    config: ServerConfig,
    /// Configuration sources in priority order
    sources: Vec<Box<dyn ConfigSource>>,
    /// Schema validator
    validator: ConfigValidator,
    /// Profile manager
    profiles: ProfileManager,
    /// Hot reload handler
    hot_reload: Option<hot_reload::HotReloadManager>,
}

impl ConfigManager {
    /// Create a new configuration manager with default settings
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: ServerConfig::default(),
            sources: Vec::new(),
            validator: ConfigValidator::new()?,
            profiles: ProfileManager::new(),
            hot_reload: None,
        })
    }

    /// Add a configuration source
    pub fn add_source(&mut self, source: Box<dyn ConfigSource>) -> Result<()> {
        self.sources.push(source);
        Ok(())
    }

    /// Load configuration from all sources
    pub async fn load(&mut self) -> Result<()> {
        let mut config = ServerConfig::default();

        // Load from sources in order (later sources override earlier ones)
        for source in &self.sources {
            let source_config = source.load().await?;
            config = config.merge(source_config)?;
        }

        // Validate the merged configuration
        self.validator.validate(&config)?;

        self.config = config;
        Ok(())
    }

    /// Get the current configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Enable hot reloading for file-based sources
    pub fn enable_hot_reload(&mut self) -> Result<()> {
        let manager = hot_reload::HotReloadManager::new()?;
        self.hot_reload = Some(manager);
        Ok(())
    }

    /// Reload configuration (for hot reload)
    pub async fn reload(&mut self) -> Result<bool> {
        let old_config = self.config.clone();
        self.load().await?;
        Ok(old_config != self.config)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default ConfigManager")
    }
}

// Custom Debug implementation for ConfigManager since it contains trait objects
impl std::fmt::Debug for ConfigManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigManager")
            .field("config", &self.config)
            .field("sources", &format!("{} sources", self.sources.len()))
            .field("validator", &self.validator)
            .field("profiles", &self.profiles)
            .field("hot_reload", &self.hot_reload)
            .finish()
    }
}

/// Global configuration instance for the server
static mut GLOBAL_CONFIG: Option<ConfigManager> = None;

/// Initialize the global configuration manager
#[allow(static_mut_refs)]
pub fn init_global_config() -> Result<()> {
    unsafe {
        if GLOBAL_CONFIG.is_some() {
            return Err(crate::types::Error::Configuration(
                ConfigurationError::AlreadyInitialized,
            ));
        }
        GLOBAL_CONFIG = Some(ConfigManager::new()?);
    }
    Ok(())
}

/// Get a reference to the global configuration
#[allow(static_mut_refs)]
pub fn global_config() -> Result<&'static ServerConfig> {
    unsafe {
        GLOBAL_CONFIG
            .as_ref()
            .map(|config| config.config())
            .ok_or_else(|| crate::types::Error::Configuration(ConfigurationError::NotInitialized))
    }
}

/// Update the global configuration
#[allow(static_mut_refs)]
pub async fn update_global_config() -> Result<bool> {
    unsafe {
        GLOBAL_CONFIG
            .as_mut()
            .ok_or_else(|| crate::types::Error::Configuration(ConfigurationError::NotInitialized))?
            .reload()
            .await
    }
}
