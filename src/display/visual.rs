//! Visual management and configuration
//!
//! This module handles visual class configuration, color depth management,
//! and visual resource allocation for display rendering.

use crate::{
    display::types::{VisualClass, VisualInfo, DisplaySettings},
    Result,
};
use std::collections::HashMap;
use tracing::{info, debug, warn};

/// Visual configuration parameters
#[derive(Debug, Clone)]
pub struct VisualConfig {
    /// Visual class type
    pub class: VisualClass,
    /// Color depth in bits
    pub depth: u8,
    /// Bits per RGB component
    pub bits_per_rgb: u8,
    /// Number of colormap entries
    pub colormap_entries: u32,
    /// Color masks for TrueColor/DirectColor
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
}

/// Visual manager for handling visual configurations
pub struct VisualManager {
    /// Visual information by visual ID
    visuals: HashMap<u32, VisualInfo>,
    /// Visual configurations by screen ID
    screen_visuals: HashMap<u32, Vec<u32>>,
    /// Next available visual ID
    next_visual_id: u32,
}

impl VisualManager {
    /// Create a new visual manager
    pub fn new() -> Self {
        info!("Creating new visual manager");
        Self {
            visuals: HashMap::new(),
            screen_visuals: HashMap::new(),
            next_visual_id: 1,
        }
    }

    /// Add a visual configuration
    pub fn add_visual(&mut self, screen_id: u32, config: VisualConfig) -> Result<u32> {
        let visual_id = self.next_visual_id;
        self.next_visual_id += 1;

        debug!("Adding visual {} for screen {} with config: {:?}", visual_id, screen_id, config);

        // Validate visual configuration
        self.validate_visual_config(&config)?;

        // Create visual info
        let visual_info = VisualInfo {
            id: visual_id,
            class: config.class,
            bits_per_rgb: config.bits_per_rgb,
            colormap_entries: config.colormap_entries,
            red_mask: config.red_mask,
            green_mask: config.green_mask,
            blue_mask: config.blue_mask,
            depth: config.depth,
        };

        // Store visual information
        self.visuals.insert(visual_id, visual_info);

        // Associate with screen
        self.screen_visuals.entry(screen_id).or_insert_with(Vec::new).push(visual_id);

        info!("Added visual {} ({:?}) for screen {}", visual_id, &config.class, screen_id);
        Ok(visual_id)
    }

    /// Remove a visual
    pub fn remove_visual(&mut self, visual_id: u32) -> Result<()> {
        if !self.visuals.contains_key(&visual_id) {
            warn!("Attempted to remove non-existent visual {}", visual_id);
            return Ok(());
        }

        info!("Removing visual {}", visual_id);

        // Remove from screen associations
        for visuals in self.screen_visuals.values_mut() {
            visuals.retain(|&id| id != visual_id);
        }

        // Remove visual info
        self.visuals.remove(&visual_id);

        debug!("Visual {} removed successfully", visual_id);
        Ok(())
    }

    /// Get visual information
    pub fn get_visual(&self, visual_id: u32) -> Option<&VisualInfo> {
        self.visuals.get(&visual_id)
    }

    /// Get all visuals for a screen
    pub fn get_screen_visuals(&self, screen_id: u32) -> Vec<&VisualInfo> {
        if let Some(visual_ids) = self.screen_visuals.get(&screen_id) {
            visual_ids.iter()
                .filter_map(|&id| self.visuals.get(&id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all visuals
    pub fn get_all_visuals(&self) -> Vec<&VisualInfo> {
        self.visuals.values().collect()
    }

    /// Create default visuals for a screen based on settings
    pub fn create_default_visuals(&mut self, screen_id: u32, settings: &DisplaySettings) -> Result<Vec<u32>> {
        info!("Creating default visuals for screen {} with depth {}", screen_id, settings.depth);

        let mut created_visuals = Vec::new();

        // Create primary visual based on requested class
        let primary_visual_id = match settings.visual_class {
            VisualClass::TrueColor => {
                self.create_truecolor_visual(screen_id, settings.depth)?
            }
            VisualClass::PseudoColor => {
                self.create_pseudocolor_visual(screen_id, settings.depth)?
            }
            VisualClass::DirectColor => {
                self.create_directcolor_visual(screen_id, settings.depth)?
            }
            VisualClass::StaticColor => {
                self.create_staticcolor_visual(screen_id, settings.depth)?
            }
            VisualClass::GrayScale => {
                self.create_grayscale_visual(screen_id, settings.depth)?
            }
            VisualClass::StaticGray => {
                self.create_staticgray_visual(screen_id, settings.depth)?
            }
        };
        created_visuals.push(primary_visual_id);

        // Create additional common visuals for compatibility
        if settings.depth >= 24 {
            // Add 16-bit TrueColor visual for compatibility
            if let Ok(visual_id) = self.create_truecolor_visual(screen_id, 16) {
                created_visuals.push(visual_id);
            }
        }

        if settings.depth >= 16 {
            // Add 8-bit PseudoColor visual for legacy applications
            if let Ok(visual_id) = self.create_pseudocolor_visual(screen_id, 8) {
                created_visuals.push(visual_id);
            }
        }

        // Add 1-bit StaticGray visual for monochrome support
        if let Ok(visual_id) = self.create_staticgray_visual(screen_id, 1) {
            created_visuals.push(visual_id);
        }

        info!("Created {} default visuals for screen {}", created_visuals.len(), screen_id);
        Ok(created_visuals)
    }

    /// Create a TrueColor visual
    pub fn create_truecolor_visual(&mut self, screen_id: u32, depth: u8) -> Result<u32> {
        let config = VisualConfig::new_truecolor(depth)?;
        self.add_visual(screen_id, config)
    }

    /// Create a PseudoColor visual
    pub fn create_pseudocolor_visual(&mut self, screen_id: u32, depth: u8) -> Result<u32> {
        let config = VisualConfig::new_pseudocolor(depth)?;
        self.add_visual(screen_id, config)
    }

    /// Create a DirectColor visual
    pub fn create_directcolor_visual(&mut self, screen_id: u32, depth: u8) -> Result<u32> {
        let config = VisualConfig::new_directcolor(depth)?;
        self.add_visual(screen_id, config)
    }

    /// Create a StaticColor visual
    pub fn create_staticcolor_visual(&mut self, screen_id: u32, depth: u8) -> Result<u32> {
        let config = VisualConfig::new_staticcolor(depth)?;
        self.add_visual(screen_id, config)
    }

    /// Create a GrayScale visual
    pub fn create_grayscale_visual(&mut self, screen_id: u32, depth: u8) -> Result<u32> {
        let config = VisualConfig::new_grayscale(depth)?;
        self.add_visual(screen_id, config)
    }

    /// Create a StaticGray visual
    pub fn create_staticgray_visual(&mut self, screen_id: u32, depth: u8) -> Result<u32> {
        let config = VisualConfig::new_staticgray(depth)?;
        self.add_visual(screen_id, config)
    }

    /// Find best matching visual for requirements
    pub fn find_matching_visual(&self, screen_id: u32, preferred_class: Option<VisualClass>, min_depth: u8) -> Option<&VisualInfo> {
        let screen_visuals = self.get_screen_visuals(screen_id);
        
        // First try to find exact class match with sufficient depth
        if let Some(class) = preferred_class {
            if let Some(visual) = screen_visuals.iter()
                .find(|v| v.class == class && v.depth >= min_depth) {
                return Some(visual);
            }
        }        // Fall back to any visual with sufficient depth
        screen_visuals.iter()
            .filter(|v| v.depth >= min_depth)
            .max_by_key(|v| v.depth) // Prefer higher depth
            .map(|v| *v)
    }

    /// Get visual count for screen
    pub fn get_screen_visual_count(&self, screen_id: u32) -> usize {
        self.screen_visuals.get(&screen_id).map_or(0, |v| v.len())
    }

    /// Validate visual configuration
    fn validate_visual_config(&self, config: &VisualConfig) -> Result<()> {
        // Validate depth
        if ![1, 4, 8, 15, 16, 24, 32].contains(&config.depth) {
            return Err(crate::ServerError::ConfigurationError(
                format!("Unsupported visual depth: {}", config.depth)
            ));
        }

        // Validate bits per RGB
        if config.bits_per_rgb > config.depth {
            return Err(crate::ServerError::ConfigurationError(
                "Bits per RGB cannot exceed visual depth".to_string()
            ));
        }

        // Validate color masks for TrueColor/DirectColor
        match config.class {
            VisualClass::TrueColor | VisualClass::DirectColor => {
                if config.red_mask == 0 && config.green_mask == 0 && config.blue_mask == 0 {
                    return Err(crate::ServerError::ConfigurationError(
                        "TrueColor/DirectColor visuals must have color masks".to_string()
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Clear all visuals for a screen
    pub fn clear_screen_visuals(&mut self, screen_id: u32) {
        if let Some(visual_ids) = self.screen_visuals.remove(&screen_id) {
            for visual_id in visual_ids {
                self.visuals.remove(&visual_id);
            }
            info!("Cleared all visuals for screen {}", screen_id);
        }
    }
}

impl VisualConfig {
    /// Create a new TrueColor visual configuration
    pub fn new_truecolor(depth: u8) -> Result<Self> {
        let (red_mask, green_mask, blue_mask, bits_per_rgb) = match depth {
            16 => (0xF800, 0x07E0, 0x001F, 5),
            24 => (0xFF0000, 0x00FF00, 0x0000FF, 8),
            32 => (0xFF0000, 0x00FF00, 0x0000FF, 8),
            _ => return Err(crate::ServerError::ConfigurationError(
                format!("Unsupported TrueColor depth: {}", depth)
            )),
        };

        Ok(Self {
            class: VisualClass::TrueColor,
            depth,
            bits_per_rgb,
            colormap_entries: 0,
            red_mask,
            green_mask,
            blue_mask,
        })
    }

    /// Create a new PseudoColor visual configuration
    pub fn new_pseudocolor(depth: u8) -> Result<Self> {
        if depth > 8 {
            return Err(crate::ServerError::ConfigurationError(
                format!("PseudoColor depth limited to 8 bits, got {}", depth)
            ));
        }

        Ok(Self {
            class: VisualClass::PseudoColor,
            depth,
            bits_per_rgb: depth,
            colormap_entries: 1 << depth,
            red_mask: 0,
            green_mask: 0,
            blue_mask: 0,
        })
    }

    /// Create a new DirectColor visual configuration
    pub fn new_directcolor(depth: u8) -> Result<Self> {
        let (red_mask, green_mask, blue_mask, bits_per_rgb) = match depth {
            16 => (0xF800, 0x07E0, 0x001F, 5),
            24 => (0xFF0000, 0x00FF00, 0x0000FF, 8),
            32 => (0xFF0000, 0x00FF00, 0x0000FF, 8),
            _ => return Err(crate::ServerError::ConfigurationError(
                format!("Unsupported DirectColor depth: {}", depth)
            )),
        };

        Ok(Self {
            class: VisualClass::DirectColor,
            depth,
            bits_per_rgb,
            colormap_entries: 1 << bits_per_rgb,
            red_mask,
            green_mask,
            blue_mask,
        })
    }

    /// Create a new StaticColor visual configuration
    pub fn new_staticcolor(depth: u8) -> Result<Self> {
        if depth > 8 {
            return Err(crate::ServerError::ConfigurationError(
                format!("StaticColor depth limited to 8 bits, got {}", depth)
            ));
        }

        Ok(Self {
            class: VisualClass::StaticColor,
            depth,
            bits_per_rgb: depth,
            colormap_entries: 1 << depth,
            red_mask: 0,
            green_mask: 0,
            blue_mask: 0,
        })
    }

    /// Create a new GrayScale visual configuration
    pub fn new_grayscale(depth: u8) -> Result<Self> {
        if depth > 8 {
            return Err(crate::ServerError::ConfigurationError(
                format!("GrayScale depth limited to 8 bits, got {}", depth)
            ));
        }

        Ok(Self {
            class: VisualClass::GrayScale,
            depth,
            bits_per_rgb: depth,
            colormap_entries: 1 << depth,
            red_mask: 0,
            green_mask: 0,
            blue_mask: 0,
        })
    }

    /// Create a new StaticGray visual configuration
    pub fn new_staticgray(depth: u8) -> Result<Self> {
        Ok(Self {
            class: VisualClass::StaticGray,
            depth,
            bits_per_rgb: if depth == 1 { 1 } else { depth },
            colormap_entries: 1 << depth,
            red_mask: 0,
            green_mask: 0,
            blue_mask: 0,
        })
    }
}

impl Default for VisualManager {
    fn default() -> Self {
        Self::new()
    }
}
