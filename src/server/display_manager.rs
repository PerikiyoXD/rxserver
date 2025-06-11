/*!
 * Display Manager for X11 Server
 *
 * Manages display configuration, screens, visuals, and display-specific resources.
 * This is a wrapper around the comprehensive display module.
 */

use crate::{
    config::DisplaySettings as ConfigDisplaySettings, 
    display::{
        DisplayManager as CoreDisplayManager, 
        ScreenInfo, 
        VisualInfo,
        types::{DisplaySettings as CoreDisplaySettings, FramebufferSettings, VisualClass}
    },
    Result
};
use tracing::info;

impl From<&ConfigDisplaySettings> for CoreDisplaySettings {
    fn from(config: &ConfigDisplaySettings) -> Self {
        Self {
            width: config.width,
            height: config.height,
            depth: config.depth,
            dpi: config.dpi,
            screens: 1, // Default to single screen
            visual_class: VisualClass::TrueColor, // Default visual class
            hw_acceleration: false, // Default to software rendering
            framebuffer: FramebufferSettings {
                software: true,
                bpp: config.depth,
                scanline_pad: 32,
                little_endian: true,
            },
        }
    }
}

/// Manages display resources and configuration
/// 
/// This is a wrapper around the core DisplayManager from the display module
/// to maintain compatibility with the server module structure.
pub struct DisplayManager {
    core_manager: CoreDisplayManager,
}

impl DisplayManager {
    /// Create a new display manager
    pub fn new(config: &ConfigDisplaySettings) -> Result<Self> {
        info!(
            "Initializing DisplayManager with configuration: {:?}",
            config
        );

        // Convert config format
        let core_config: CoreDisplaySettings = config.into();
        let core_manager = CoreDisplayManager::new(&core_config)?;
        
        Ok(Self {
            core_manager,
        })
    }

    /// Get screen information
    pub fn get_screens(&self) -> Vec<&ScreenInfo> {
        self.core_manager.get_screens()
    }

    /// Get visual information
    pub fn get_visuals(&self) -> Vec<&VisualInfo> {
        self.core_manager.get_visuals()
    }

    /// Get screen count
    pub fn screen_count(&self) -> usize {
        self.core_manager.screen_count()
    }

    /// Get visual count
    pub fn visual_count(&self) -> usize {
        self.core_manager.visual_count()
    }

    /// Get screen by ID
    pub fn get_screen(&self, screen_id: u32) -> Option<&ScreenInfo> {
        self.core_manager.get_screen(screen_id)
    }

    /// Get visual by ID
    pub fn get_visual(&self, visual_id: u32) -> Option<&VisualInfo> {
        self.core_manager.get_visual(visual_id)
    }

    /// Initialize display resources (already done in constructor)
    pub fn initialize(&mut self) -> Result<()> {
        // Display initialization is already complete in the constructor
        info!("Display resources already initialized");
        Ok(())
    }
}
