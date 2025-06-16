//! Plugin registry
//!
//! This module manages the registration and lifecycle of plugins
//! for the RX X11 Server.

use crate::core::error::{ServerError, ServerResult};
use std::collections::HashMap;

/// Plugin registry for managing server extensions
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

/// Plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get the plugin name
    fn name(&self) -> &str;

    /// Initialize the plugin
    fn initialize(&mut self) -> ServerResult<()>;

    /// Shutdown the plugin
    fn shutdown(&mut self) -> ServerResult<()>;
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, mut plugin: Box<dyn Plugin>) -> ServerResult<()> {
        let name = plugin.name().to_string();

        // Check for duplicate registration
        if self.plugins.contains_key(&name) {
            return Err(ServerError::PluginError(format!(
                "Plugin '{}' is already registered",
                name
            )));
        }

        // Initialize the plugin immediately upon registration
        plugin.initialize()?;
        self.plugins.insert(name, plugin);
        Ok(())
    }
    /// Register core plugins required for basic functionality
    pub fn register_core_plugins(&mut self) -> ServerResult<()> {
        // Register essential plugins
        self.register(Box::new(crate::plugins::WindowPlugin::new()))?;

        // TODO: Register other core plugins when implemented
        // self.register(Box::new(AtomRegistry::new()))?;
        // self.register(Box::new(FontManager::new()))?;
        // self.register(Box::new(CursorManager::new()))?;

        Ok(())
    }

    pub fn get_plugin(&self, name: &str) -> Option<&Box<dyn Plugin>> {
        self.plugins.get(name)
    }

    /// Initialize all registered plugins
    pub fn initialize_all(&mut self) -> ServerResult<()> {
        for plugin in self.plugins.values_mut() {
            plugin.initialize()?;
        }
        Ok(())
    }

    /// Shutdown all registered plugins
    pub fn shutdown_all(&mut self) -> ServerResult<()> {
        for plugin in self.plugins.values_mut() {
            plugin.shutdown()?;
        }
        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
