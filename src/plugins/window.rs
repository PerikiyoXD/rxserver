//! Window Manager Plugin
//!
//! This plugin provides window management functionality by wrapping
//! the core WindowManager and exposing it through the plugin interface.

use crate::{
    core::error::{ServerError, ServerResult},
    plugins::registry::Plugin as PluginTrait,
    window::{properties::WindowProperties, WindowClass, WindowId, WindowManager},
};
use std::sync::{Arc, Mutex};

/// Window manager plugin that handles all window operations
pub struct WindowPlugin {
    manager: Arc<Mutex<WindowManager>>,
    properties: Arc<Mutex<WindowProperties>>,
}

impl WindowPlugin {
    /// Create a new window plugin
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(WindowManager::new())),
            properties: Arc::new(Mutex::new(WindowProperties::default())),
        }
    }

    /// Get the window manager instance
    pub fn get_manager(&self) -> Arc<Mutex<WindowManager>> {
        Arc::clone(&self.manager)
    }

    /// Get the window properties instance
    pub fn get_properties(&self) -> Arc<Mutex<WindowProperties>> {
        Arc::clone(&self.properties)
    }

    /// Create a new window through the plugin
    pub fn create_window(
        &self,
        parent: WindowId,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        class: WindowClass,
    ) -> ServerResult<WindowId> {
        let mut manager = self
            .manager
            .lock()
            .map_err(|_| ServerError::PluginError("Failed to lock window manager".to_string()))?;
        manager.create_window(parent, x, y, width, height, border_width, class)
    }

    /// Map a window through the plugin
    pub fn map_window(&self, window_id: WindowId) -> ServerResult<()> {
        let mut manager = self
            .manager
            .lock()
            .map_err(|_| ServerError::PluginError("Failed to lock window manager".to_string()))?;
        manager.map_window(window_id)
    }

    /// Unmap a window through the plugin
    pub fn unmap_window(&self, window_id: WindowId) -> ServerResult<()> {
        let mut manager = self
            .manager
            .lock()
            .map_err(|_| ServerError::PluginError("Failed to lock window manager".to_string()))?;
        manager.unmap_window(window_id)
    }

    /// Destroy a window through the plugin
    pub fn destroy_window(&self, window_id: WindowId) -> ServerResult<()> {
        let mut manager = self
            .manager
            .lock()
            .map_err(|_| ServerError::PluginError("Failed to lock window manager".to_string()))?;
        manager.destroy_window(window_id)
    }

    /// Get root window ID
    pub fn get_root_window(&self) -> ServerResult<WindowId> {
        let manager = self
            .manager
            .lock()
            .map_err(|_| ServerError::PluginError("Failed to lock window manager".to_string()))?;
        Ok(manager.get_root_window())
    }
}

impl PluginTrait for WindowPlugin {
    fn name(&self) -> &str {
        "WindowManager"
    }

    fn initialize(&mut self) -> ServerResult<()> {
        // Window manager is already initialized in new()
        Ok(())
    }

    fn shutdown(&mut self) -> ServerResult<()> {
        // Clean shutdown - in a full implementation, this might save state
        // or perform cleanup operations
        Ok(())
    }
}

impl Default for WindowPlugin {
    fn default() -> Self {
        Self::new()
    }
}
