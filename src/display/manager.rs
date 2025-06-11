//! Main display manager coordinating all display components
//!
//! This module provides the central DisplayManager that coordinates
//! screens, visuals, framebuffers, and other display resources.

use crate::{
    display::{
        framebuffer::Framebuffer,
        init::{init_display, DisplayInitConfig},
        screen::{ScreenConfig, ScreenManager},
        types::{DisplaySettings, DisplayState, ScreenInfo, VisualInfo},
        visual::VisualManager,
    },
    logging::ServerLogger,
    Result,
};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Main display manager coordinating all display components
pub struct DisplayManager {
    /// Screen management
    screen_manager: ScreenManager,
    /// Visual management
    visual_manager: VisualManager,
    /// Framebuffers by screen ID
    framebuffers: HashMap<u32, Framebuffer>,
    /// Current display settings
    settings: DisplaySettings,
    /// Display state
    state: DisplayState,
    /// Server logger for display operations
    logger: ServerLogger,
}

impl DisplayManager {
    /// Create a new display manager
    pub fn new(config: &DisplaySettings) -> Result<Self> {
        let logger = ServerLogger::new("DisplayManager");

        info!(
            "Initializing DisplayManager with configuration: {:?}",
            config
        );
        let mut display_manager = Self {
            screen_manager: ScreenManager::new(),
            visual_manager: VisualManager::new(),
            framebuffers: HashMap::new(),
            settings: config.clone(),
            state: DisplayState::Uninitialized,
            logger,
        };

        // Initialize display system
        display_manager.initialize_display(config)?;

        Ok(display_manager)
    }

    /// Initialize the display system
    fn initialize_display(&mut self, config: &DisplaySettings) -> Result<()> {
        self.logger.log_startup(
            "Starting display system initialization",
            "Configuring display components",
        );

        // Set state to initializing
        self.state = DisplayState::Initializing;

        // Initialize display subsystem
        let init_config = DisplayInitConfig {
            settings: config.clone(),
            verbose: true,
            force_software: true,
            timeout_seconds: 30,
        };

        init_display(&init_config)?;

        // Configure screens from settings
        self.screen_manager.configure_from_settings(config)?;

        // Create default screens and visuals
        self.create_default_screens_and_visuals(config)?;

        // Initialize framebuffers
        self.initialize_framebuffers(config)?;

        // Set state to active
        self.state = DisplayState::Active;

        self.logger.log_startup(
            "Display system initialization completed",
            "All components ready",
        );
        info!(
            "DisplayManager initialized successfully with {} screen(s)",
            self.screen_count()
        );

        Ok(())
    }

    /// Create default screens and visuals
    fn create_default_screens_and_visuals(&mut self, config: &DisplaySettings) -> Result<()> {
        info!("Creating default screens and visuals");

        for screen_id in 0..config.screens {
            // Create default visuals for this screen
            let visual_ids = self
                .visual_manager
                .create_default_visuals(screen_id, config)?;

            // Update screen with visual information
            if let Some(screen) = self.screen_manager.get_screen_mut(screen_id) {
                screen.visuals = visual_ids
                    .iter()
                    .filter_map(|&id| self.visual_manager.get_visual(id))
                    .cloned()
                    .collect();

                debug!(
                    "Screen {} configured with {} visuals",
                    screen_id,
                    screen.visuals.len()
                );
            }
        }

        Ok(())
    }

    /// Initialize framebuffers for all screens
    fn initialize_framebuffers(&mut self, config: &DisplaySettings) -> Result<()> {
        info!("Initializing framebuffers for {} screen(s)", config.screens);

        for screen_id in 0..config.screens {
            let framebuffer =
                Framebuffer::from_settings(config.width, config.height, &config.framebuffer)?;

            // Clear framebuffer with custom color
            framebuffer.clear(0xFACADE)?;

            self.framebuffers.insert(screen_id, framebuffer);
            debug!("Initialized framebuffer for screen {}", screen_id);
        }

        Ok(())
    }

    /// Get screen information
    pub fn get_screens(&self) -> Vec<&ScreenInfo> {
        self.screen_manager.get_all_screens()
    }

    /// Get visual information
    pub fn get_visuals(&self) -> Vec<&VisualInfo> {
        self.visual_manager.get_all_visuals()
    }

    /// Get screen count
    pub fn screen_count(&self) -> usize {
        self.screen_manager.screen_count()
    }

    /// Get visual count
    pub fn visual_count(&self) -> usize {
        self.visual_manager.get_all_visuals().len()
    }

    /// Get screen by ID
    pub fn get_screen(&self, screen_id: u32) -> Option<&ScreenInfo> {
        self.screen_manager.get_screen(screen_id)
    }

    /// Get visual by ID
    pub fn get_visual(&self, visual_id: u32) -> Option<&VisualInfo> {
        self.visual_manager.get_visual(visual_id)
    }

    /// Get framebuffer for screen
    pub fn get_framebuffer(&self, screen_id: u32) -> Option<&Framebuffer> {
        self.framebuffers.get(&screen_id)
    }

    /// Get screen visuals
    pub fn get_screen_visuals(&self, screen_id: u32) -> Vec<&VisualInfo> {
        self.visual_manager.get_screen_visuals(screen_id)
    }

    /// Initialize display resources
    pub fn initialize(&mut self) -> Result<()> {
        if self.state == DisplayState::Active {
            debug!("Display manager already initialized");
            return Ok(());
        }

        info!("Initializing display resources");
        // Validate current state
        match &self.state {
            DisplayState::Error(msg) => {
                return Err(crate::Error::Display(format!(
                    "Cannot initialize display in error state: {}",
                    msg
                )));
            }
            _ => {}
        }

        // Re-initialize if needed
        self.initialize_display(&self.settings.clone())?;

        info!("Display resources initialized successfully");
        Ok(())
    }

    /// Add a new screen
    pub fn add_screen(&mut self, config: ScreenConfig) -> Result<u32> {
        info!("Adding new screen with configuration: {:?}", config);

        let screen_id = self.screen_manager.add_screen(config.clone())?;

        // Create visuals for the new screen
        let visual_ids = self
            .visual_manager
            .create_default_visuals(screen_id, &self.settings)?;

        // Update screen with visual information
        if let Some(screen) = self.screen_manager.get_screen_mut(screen_id) {
            screen.visuals = visual_ids
                .iter()
                .filter_map(|&id| self.visual_manager.get_visual(id))
                .cloned()
                .collect();
        }

        // Create framebuffer for the new screen
        let framebuffer =
            Framebuffer::from_settings(config.width, config.height, &self.settings.framebuffer)?;
        framebuffer.clear(0x000000)?;
        self.framebuffers.insert(screen_id, framebuffer);

        info!("Added screen {} successfully", screen_id);
        Ok(screen_id)
    }

    /// Remove a screen
    pub fn remove_screen(&mut self, screen_id: u32) -> Result<()> {
        info!("Removing screen {}", screen_id);

        // Remove framebuffer
        self.framebuffers.remove(&screen_id);

        // Clear visuals for this screen
        self.visual_manager.clear_screen_visuals(screen_id);

        // Remove screen
        self.screen_manager.remove_screen(screen_id)?;

        info!("Removed screen {} successfully", screen_id);
        Ok(())
    }

    /// Get active screen ID
    pub fn get_active_screen(&self) -> Option<u32> {
        self.screen_manager.get_active_screen()
    }

    /// Set active screen
    pub fn set_active_screen(&mut self, screen_id: u32) -> Result<()> {
        self.screen_manager.set_active_screen(screen_id)
    }

    /// Get display state
    pub fn get_state(&self) -> &DisplayState {
        &self.state
    }

    /// Get display settings
    pub fn get_settings(&self) -> &DisplaySettings {
        &self.settings
    }

    /// Update display settings
    pub fn update_settings(&mut self, new_settings: DisplaySettings) -> Result<()> {
        info!("Updating display settings: {:?}", new_settings);

        // Store old settings for rollback if needed
        let old_settings = self.settings.clone();
        self.settings = new_settings.clone();

        // Try to apply new settings
        match self.apply_settings_changes(&old_settings, &new_settings) {
            Ok(_) => {
                info!("Display settings updated successfully");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to apply new settings, rolling back: {}", e);
                self.settings = old_settings;
                Err(e)
            }
        }
    }

    /// Apply settings changes
    fn apply_settings_changes(
        &mut self,
        old_settings: &DisplaySettings,
        new_settings: &DisplaySettings,
    ) -> Result<()> {
        // Check if screen count changed
        if old_settings.screens != new_settings.screens {
            debug!(
                "Screen count changed from {} to {}",
                old_settings.screens, new_settings.screens
            );
            self.screen_manager.configure_from_settings(new_settings)?;
            self.create_default_screens_and_visuals(new_settings)?;
            self.initialize_framebuffers(new_settings)?;
        }

        // Check if screen dimensions changed
        if old_settings.width != new_settings.width || old_settings.height != new_settings.height {
            debug!(
                "Screen dimensions changed to {}x{}",
                new_settings.width, new_settings.height
            );
            self.resize_framebuffers(new_settings.width, new_settings.height)?;
        }

        // Check if visual settings changed
        if old_settings.visual_class != new_settings.visual_class
            || old_settings.depth != new_settings.depth
        {
            debug!("Visual settings changed, recreating visuals");
            // Clear existing visuals and recreate
            for screen_id in 0..new_settings.screens {
                self.visual_manager.clear_screen_visuals(screen_id);
                let visual_ids = self
                    .visual_manager
                    .create_default_visuals(screen_id, new_settings)?;

                if let Some(screen) = self.screen_manager.get_screen_mut(screen_id) {
                    screen.visuals = visual_ids
                        .iter()
                        .filter_map(|&id| self.visual_manager.get_visual(id))
                        .cloned()
                        .collect();
                }
            }
        }

        Ok(())
    }
    /// Resize all framebuffers
    fn resize_framebuffers(&mut self, new_width: u32, new_height: u32) -> Result<()> {
        for _framebuffer in self.framebuffers.values_mut() {
            // Note: This would require making Framebuffer methods mutable
            // For now, we'll recreate the framebuffers
        }

        // Recreate framebuffers with new dimensions
        let screen_ids: Vec<u32> = self.framebuffers.keys().cloned().collect();
        self.framebuffers.clear();

        for screen_id in screen_ids {
            let framebuffer =
                Framebuffer::from_settings(new_width, new_height, &self.settings.framebuffer)?;
            framebuffer.clear(0x000000)?;
            self.framebuffers.insert(screen_id, framebuffer);
        }

        Ok(())
    }

    /// Find screen by coordinates
    pub fn find_screen_by_coordinates(&self, x: i32, y: i32) -> Option<&ScreenInfo> {
        self.screen_manager.get_screen_by_coordinates(x, y)
    }

    /// Get total display area
    pub fn get_total_display_area(&self) -> Option<(i32, i32, u32, u32)> {
        self.screen_manager.get_total_display_area()
    }

    /// Check if display manager is ready
    pub fn is_ready(&self) -> bool {
        matches!(self.state, DisplayState::Active)
    }

    /// Shutdown display manager
    pub fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down display manager");

        self.state = DisplayState::Uninitialized;
        self.framebuffers.clear();

        self.logger
            .log_shutdown("Display manager shutdown completed");

        Ok(())
    }
}
