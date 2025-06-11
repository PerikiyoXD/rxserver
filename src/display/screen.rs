//! Screen management and configuration
//!
//! This module handles individual screen setup, management,
//! and resource allocation for each display screen.

use crate::{
    display::types::{DisplaySettings, ScreenInfo, ScreenResources, ScreenPosition},
    Result,
};
use std::collections::HashMap;
use tracing::{info, debug, warn};

/// Screen configuration parameters
#[derive(Debug, Clone)]
pub struct ScreenConfig {
    /// Screen dimensions
    pub width: u32,
    pub height: u32,
    /// Physical dimensions in millimeters
    pub width_mm: u32,
    pub height_mm: u32,
    /// Color depth
    pub depth: u8,
    /// Screen position for multi-screen setups
    pub position: ScreenPosition,
    /// DPI settings
    pub dpi_x: u16,
    pub dpi_y: u16,
    /// Enable backing store
    pub backing_store: bool,
    /// Enable save under
    pub save_under: bool,
}

/// Screen manager for handling multiple screens
pub struct ScreenManager {
    /// Screen information by screen ID
    screens: HashMap<u32, ScreenInfo>,
    /// Screen resources by screen ID
    resources: HashMap<u32, ScreenResources>,
    /// Current active screen
    active_screen: Option<u32>,
    /// Next available screen ID
    next_screen_id: u32,
}

impl ScreenManager {
    /// Create a new screen manager
    pub fn new() -> Self {
        info!("Creating new screen manager");
        Self {
            screens: HashMap::new(),
            resources: HashMap::new(),
            active_screen: None,
            next_screen_id: 0,
        }
    }

    /// Add a new screen with the given configuration
    pub fn add_screen(&mut self, config: ScreenConfig) -> Result<u32> {
        let screen_id = self.next_screen_id;
        self.next_screen_id += 1;

        info!("Adding screen {} with configuration: {:?}", screen_id, config);

        // Create screen info
        let screen_info = ScreenInfo {
            id: screen_id,
            width: config.width,
            height: config.height,
            width_mm: config.width_mm,
            height_mm: config.height_mm,
            depth: config.depth,
            root_window: screen_id * 1000 + 1,
            default_colormap: screen_id * 1000 + 2,
            white_pixel: 0xFFFFFF,
            black_pixel: 0x000000,
            visuals: Vec::new(),
            position: config.position,
            dpi_x: config.dpi_x,
            dpi_y: config.dpi_y,
        };

        // Create screen resources
        let screen_resources = ScreenResources::default();

        // Store screen information
        self.screens.insert(screen_id, screen_info);
        self.resources.insert(screen_id, screen_resources);

        // Set as active screen if it's the first one
        if self.active_screen.is_none() {
            self.active_screen = Some(screen_id);
            info!("Set screen {} as active screen", screen_id);
        }

        debug!("Screen {} added successfully", screen_id);
        Ok(screen_id)
    }

    /// Remove a screen
    pub fn remove_screen(&mut self, screen_id: u32) -> Result<()> {
        info!("Removing screen {}", screen_id);

        if !self.screens.contains_key(&screen_id) {
            warn!("Attempted to remove non-existent screen {}", screen_id);
            return Err(crate::ServerError::InvalidParameter(
                format!("Screen {} does not exist", screen_id)
            ));
        }

        // Clean up screen resources
        if let Some(resources) = self.resources.remove(&screen_id) {
            self.cleanup_screen_resources(screen_id, resources)?;
        }

        // Remove screen info
        self.screens.remove(&screen_id);

        // Update active screen if necessary
        if self.active_screen == Some(screen_id) {
            self.active_screen = self.screens.keys().next().copied();
            if let Some(new_active) = self.active_screen {
                info!("Set screen {} as new active screen", new_active);
            } else {
                info!("No active screen remaining");
            }
        }

        debug!("Screen {} removed successfully", screen_id);
        Ok(())
    }

    /// Get screen information
    pub fn get_screen(&self, screen_id: u32) -> Option<&ScreenInfo> {
        self.screens.get(&screen_id)
    }

    /// Get mutable screen information
    pub fn get_screen_mut(&mut self, screen_id: u32) -> Option<&mut ScreenInfo> {
        self.screens.get_mut(&screen_id)
    }

    /// Get all screens
    pub fn get_all_screens(&self) -> Vec<&ScreenInfo> {
        self.screens.values().collect()
    }

    /// Get screen count
    pub fn screen_count(&self) -> usize {
        self.screens.len()
    }

    /// Get active screen ID
    pub fn get_active_screen(&self) -> Option<u32> {
        self.active_screen
    }

    /// Set active screen
    pub fn set_active_screen(&mut self, screen_id: u32) -> Result<()> {
        if !self.screens.contains_key(&screen_id) {
            return Err(crate::ServerError::InvalidParameter(
                format!("Screen {} does not exist", screen_id)
            ));
        }

        self.active_screen = Some(screen_id);
        info!("Set screen {} as active screen", screen_id);
        Ok(())
    }

    /// Get screen resources
    pub fn get_screen_resources(&self, screen_id: u32) -> Option<&ScreenResources> {
        self.resources.get(&screen_id)
    }

    /// Get mutable screen resources
    pub fn get_screen_resources_mut(&mut self, screen_id: u32) -> Option<&mut ScreenResources> {
        self.resources.get_mut(&screen_id)
    }

    /// Configure screen from display settings
    pub fn configure_from_settings(&mut self, settings: &DisplaySettings) -> Result<()> {
        info!("Configuring screens from display settings");

        // Clear existing screens
        self.screens.clear();
        self.resources.clear();
        self.active_screen = None;
        self.next_screen_id = 0;

        // Create screens based on settings
        for screen_index in 0..settings.screens {
            let config = ScreenConfig {
                width: settings.width,
                height: settings.height,
                width_mm: (settings.width as f32 / settings.dpi as f32 * 25.4) as u32,
                height_mm: (settings.height as f32 / settings.dpi as f32 * 25.4) as u32,
                depth: settings.depth,
                position: ScreenPosition {
                    x: (screen_index * settings.width) as i32,
                    y: 0,
                },
                dpi_x: settings.dpi,
                dpi_y: settings.dpi,
                backing_store: false,
                save_under: false,
            };

            self.add_screen(config)?;
        }

        info!("Configured {} screen(s) from settings", settings.screens);
        Ok(())
    }

    /// Get screen by coordinates (for multi-screen setups)
    pub fn get_screen_by_coordinates(&self, x: i32, y: i32) -> Option<&ScreenInfo> {
        for screen in self.screens.values() {
            let screen_x = screen.position.x;
            let screen_y = screen.position.y;
            let screen_right = screen_x + screen.width as i32;
            let screen_bottom = screen_y + screen.height as i32;

            if x >= screen_x && x < screen_right && y >= screen_y && y < screen_bottom {
                return Some(screen);
            }
        }
        None
    }

    /// Update screen position (for dynamic screen arrangements)
    pub fn update_screen_position(&mut self, screen_id: u32, position: ScreenPosition) -> Result<()> {
        if let Some(screen) = self.screens.get_mut(&screen_id) {
            info!("Updating screen {} position to ({}, {})", screen_id, position.x, position.y);
            screen.position = position;
            Ok(())
        } else {
            Err(crate::ServerError::InvalidParameter(
                format!("Screen {} does not exist", screen_id)
            ))
        }
    }

    /// Validate screen configuration
    pub fn validate_screen_config(&self, config: &ScreenConfig) -> Result<()> {
        if config.width == 0 || config.height == 0 {
            return Err(crate::ServerError::ConfigurationError(
                "Screen dimensions must be greater than 0".to_string()
            ));
        }

        if ![1, 4, 8, 15, 16, 24, 32].contains(&config.depth) {
            return Err(crate::ServerError::ConfigurationError(
                format!("Unsupported color depth: {}", config.depth)
            ));
        }

        if config.dpi_x < 50 || config.dpi_x > 500 || config.dpi_y < 50 || config.dpi_y > 500 {
            warn!("Unusual DPI settings: {}x{}", config.dpi_x, config.dpi_y);
        }

        Ok(())
    }

    /// Clean up screen resources
    fn cleanup_screen_resources(&self, screen_id: u32, _resources: ScreenResources) -> Result<()> {
        debug!("Cleaning up resources for screen {}", screen_id);
        
        // TODO: Implement proper resource cleanup
        // - Close framebuffer
        // - Free graphics contexts
        // - Free pixmaps
        // - Free cursors
        
        Ok(())
    }

    /// Get total display area (bounding box of all screens)
    pub fn get_total_display_area(&self) -> Option<(i32, i32, u32, u32)> {
        if self.screens.is_empty() {
            return None;
        }

        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;

        for screen in self.screens.values() {
            min_x = min_x.min(screen.position.x);
            min_y = min_y.min(screen.position.y);
            max_x = max_x.max(screen.position.x + screen.width as i32);
            max_y = max_y.max(screen.position.y + screen.height as i32);
        }

        Some((min_x, min_y, (max_x - min_x) as u32, (max_y - min_y) as u32))
    }
}

impl Default for ScreenManager {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&DisplaySettings> for ScreenConfig {
    fn from(settings: &DisplaySettings) -> Self {
        Self {
            width: settings.width,
            height: settings.height,
            width_mm: (settings.width as f32 / settings.dpi as f32 * 25.4) as u32,
            height_mm: (settings.height as f32 / settings.dpi as f32 * 25.4) as u32,
            depth: settings.depth,
            position: ScreenPosition { x: 0, y: 0 },
            dpi_x: settings.dpi,
            dpi_y: settings.dpi,
            backing_store: false,
            save_under: false,
        }
    }
}
