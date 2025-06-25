//! Configuration sources
//!
//! This module provides various sources for loading configuration data,
//! including files, environment variables, command line arguments, and remote sources.

pub mod command_line;
pub mod database;
pub mod environment;
pub mod file;
pub mod registry;
pub mod remote;

use crate::config::types::ServerConfig;
use crate::types::Result;
use async_trait::async_trait;

/// Trait for configuration sources
#[async_trait]
pub trait ConfigSource: Send + Sync {
    /// Load configuration from this source
    async fn load(&self) -> Result<ServerConfig>;

    /// Get the source identifier
    fn identifier(&self) -> String;

    /// Get the source priority (higher numbers = higher priority)
    fn priority(&self) -> u32;

    /// Check if this source supports watching for changes
    fn supports_watch(&self) -> bool {
        false
    }

    /// Start watching for changes (if supported)
    async fn start_watch(&self) -> Result<()> {
        Ok(())
    }

    /// Stop watching for changes
    async fn stop_watch(&self) -> Result<()> {
        Ok(())
    }
}

/// Configuration source registry
pub struct SourceRegistry {
    sources: Vec<Box<dyn ConfigSource>>,
}

impl SourceRegistry {
    /// Create a new source registry
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// Add a configuration source
    pub fn add_source(&mut self, source: Box<dyn ConfigSource>) {
        self.sources.push(source);
        // Sort by priority (highest first)
        self.sources.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Remove a source by identifier
    pub fn remove_source(&mut self, identifier: &str) {
        self.sources
            .retain(|source| source.identifier() != identifier);
    }

    /// Get all sources
    pub fn sources(&self) -> &[Box<dyn ConfigSource>] {
        &self.sources
    }

    /// Load configuration from all sources in priority order
    pub async fn load_all(&self) -> Result<ServerConfig> {
        let mut config = ServerConfig::default();

        // Load from sources in reverse priority order (lowest first)
        // so higher priority sources override lower priority ones
        for source in self.sources.iter().rev() {
            match source.load().await {
                Ok(source_config) => {
                    config = config.merge(source_config)?;
                }
                Err(e) => {
                    log::warn!(
                        "Failed to load config from source '{}': {}",
                        source.identifier(),
                        e
                    );
                }
            }
        }

        Ok(config)
    }

    /// Start watching all sources that support it
    pub async fn start_watching(&self) -> Result<()> {
        for source in &self.sources {
            if source.supports_watch() {
                if let Err(e) = source.start_watch().await {
                    log::warn!(
                        "Failed to start watching source '{}': {}",
                        source.identifier(),
                        e
                    );
                }
            }
        }
        Ok(())
    }

    /// Stop watching all sources
    pub async fn stop_watching(&self) -> Result<()> {
        for source in &self.sources {
            if source.supports_watch() {
                if let Err(e) = source.stop_watch().await {
                    log::warn!(
                        "Failed to stop watching source '{}': {}",
                        source.identifier(),
                        e
                    );
                }
            }
        }
        Ok(())
    }

    /// Get sources that support watching
    pub fn watchable_sources(&self) -> Vec<&Box<dyn ConfigSource>> {
        self.sources.iter().filter(|s| s.supports_watch()).collect()
    }

    /// Clear all sources
    pub fn clear(&mut self) {
        self.sources.clear();
    }

    /// Get source count
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
}

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Default source configuration builder
pub struct DefaultSources;

impl DefaultSources {
    /// Create a registry with default sources
    pub fn create_default_registry() -> SourceRegistry {
        let mut registry = SourceRegistry::new();

        // Add default sources in priority order (lowest to highest)

        // 1. Default configuration (lowest priority)
        registry.add_source(Box::new(DefaultConfigSource));

        // 2. Configuration files
        if let Ok(file_source) = file::FileSource::new("rxserver.toml") {
            registry.add_source(Box::new(file_source));
        }

        // 3. Environment variables
        registry.add_source(Box::new(environment::EnvironmentSource::new()));

        // 4. Command line arguments (highest priority)
        registry.add_source(Box::new(command_line::CommandLineSource::new()));

        registry
    }

    /// Create registry for development
    pub fn create_development_registry() -> SourceRegistry {
        let mut registry = SourceRegistry::new();

        // Development sources
        registry.add_source(Box::new(DefaultConfigSource));

        // Look for development config files
        for filename in &[
            "rxserver.dev.toml",
            "rxserver.development.toml",
            "rxserver.toml",
        ] {
            if let Ok(file_source) = file::FileSource::new(filename) {
                registry.add_source(Box::new(file_source));
                break;
            }
        }

        registry.add_source(Box::new(environment::EnvironmentSource::new()));
        registry.add_source(Box::new(command_line::CommandLineSource::new()));

        registry
    }

    /// Create registry for production
    pub fn create_production_registry() -> SourceRegistry {
        let mut registry = SourceRegistry::new();

        // Production sources with specific paths
        registry.add_source(Box::new(DefaultConfigSource));

        // System configuration files
        for config_path in &[
            "/etc/rxserver/rxserver.toml",
            "/usr/local/etc/rxserver.toml",
            "./rxserver.toml",
        ] {
            if let Ok(file_source) = file::FileSource::new(config_path) {
                registry.add_source(Box::new(file_source));
            }
        }

        registry.add_source(Box::new(environment::EnvironmentSource::new()));
        registry.add_source(Box::new(command_line::CommandLineSource::new()));

        registry
    }
}

/// Default configuration source
struct DefaultConfigSource;

#[async_trait]
impl ConfigSource for DefaultConfigSource {
    async fn load(&self) -> Result<ServerConfig> {
        Ok(ServerConfig::default())
    }

    fn identifier(&self) -> String {
        "default".to_string()
    }

    fn priority(&self) -> u32 {
        0 // Lowest priority
    }
}
