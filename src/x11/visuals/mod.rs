//! Visual and color depth support
//!
//! This module provides support for X11 visuals, color depths, and pixel formats.
//! Visuals define how colors are represented and displayed on the screen.

pub mod conversion;
pub mod depth;
pub mod formats;
pub mod selection;
pub mod types;
pub mod validation;

// Re-export core types
pub use depth::{ColorDepth, DepthManager};
pub use formats::{FormatManager, PixelFormat};
pub use selection::{SelectionCriteria, VisualSelector};
pub use types::{DepthInfo, Visual, VisualClass, VisualInfo};
pub use validation::{ValidationResult, VisualValidator};

use crate::x11::VisualId;

/// Visual system interface for managing display visuals
pub trait VisualSystem {
    /// Get all available visuals for a screen
    fn get_visuals(&self, screen: u8) -> Vec<VisualInfo>;

    /// Get the default visual for a screen
    fn get_default_visual(&self, screen: u8) -> Option<VisualInfo>;

    /// Find a visual by ID
    fn find_visual(&self, visual_id: VisualId) -> Option<VisualInfo>;

    /// Check if a visual supports a specific depth
    fn supports_depth(&self, visual_id: VisualId, depth: u8) -> bool;

    /// Get supported depths for a visual
    fn get_supported_depths(&self, visual_id: VisualId) -> Vec<u8>;

    /// Validate visual compatibility
    fn validate_visual(&self, visual_id: VisualId, depth: u8) -> ValidationResult;
}

/// Default implementation of the visual system
#[derive(Debug)]
pub struct DefaultVisualSystem {
    /// Available visuals by screen
    visuals: std::collections::HashMap<u8, Vec<VisualInfo>>,
    /// Default visual for each screen
    default_visuals: std::collections::HashMap<u8, VisualId>,
    /// Visual validator
    validator: VisualValidator,
}

impl DefaultVisualSystem {
    /// Create a new visual system with default visuals
    pub fn new() -> Self {
        let mut system = Self {
            visuals: std::collections::HashMap::new(),
            default_visuals: std::collections::HashMap::new(),
            validator: VisualValidator::new(),
        };

        // Initialize with default visuals for screen 0
        system.initialize_default_visuals();
        system
    }

    /// Initialize default visuals for the system
    fn initialize_default_visuals(&mut self) {
        let screen = 0;
        let mut screen_visuals = Vec::new();
        // Create a basic TrueColor visual (24-bit RGB)
        let truecolor_visual = VisualInfo {
            visual: Visual {
                visual_id: VisualId(0x21),
                class: VisualClass::TrueColor,
                bits_per_rgb: 8,
                colormap_entries: 256,
                red_mask: 0xFF0000,
                green_mask: 0x00FF00,
                blue_mask: 0x0000FF,
            },
            depth: 24,
            screen,
        };
        screen_visuals.push(truecolor_visual.clone());
        self.default_visuals
            .insert(screen, truecolor_visual.visual.visual_id);

        // Create a DirectColor visual (24-bit RGB with separate colormaps)
        let directcolor_visual = VisualInfo {
            visual: Visual {
                visual_id: VisualId(0x22),
                class: VisualClass::DirectColor,
                bits_per_rgb: 8,
                colormap_entries: 256,
                red_mask: 0xFF0000,
                green_mask: 0x00FF00,
                blue_mask: 0x0000FF,
            },
            depth: 24,
            screen,
        };
        screen_visuals.push(directcolor_visual);

        // Create a PseudoColor visual (8-bit indexed)
        let pseudocolor_visual = VisualInfo {
            visual: Visual {
                visual_id: VisualId(0x23),
                class: VisualClass::PseudoColor,
                bits_per_rgb: 8,
                colormap_entries: 256,
                red_mask: 0,
                green_mask: 0,
                blue_mask: 0,
            },
            depth: 8,
            screen,
        };
        screen_visuals.push(pseudocolor_visual);

        // Create a StaticColor visual (8-bit fixed colormap)
        let staticcolor_visual = VisualInfo {
            visual: Visual {
                visual_id: VisualId(0x24),
                class: VisualClass::StaticColor,
                bits_per_rgb: 8,
                colormap_entries: 256,
                red_mask: 0,
                green_mask: 0,
                blue_mask: 0,
            },
            depth: 8,
            screen,
        };
        screen_visuals.push(staticcolor_visual);

        self.visuals.insert(screen, screen_visuals);
    }

    /// Add a custom visual to a screen
    pub fn add_visual(&mut self, screen: u8, visual_info: VisualInfo) {
        self.visuals
            .entry(screen)
            .or_insert_with(Vec::new)
            .push(visual_info);
    }

    /// Set the default visual for a screen
    pub fn set_default_visual(&mut self, screen: u8, visual_id: VisualId) {
        self.default_visuals.insert(screen, visual_id);
    }
}

impl VisualSystem for DefaultVisualSystem {
    fn get_visuals(&self, screen: u8) -> Vec<VisualInfo> {
        self.visuals.get(&screen).cloned().unwrap_or_default()
    }

    fn get_default_visual(&self, screen: u8) -> Option<VisualInfo> {
        let visual_id = self.default_visuals.get(&screen)?;
        self.find_visual(*visual_id)
    }

    fn find_visual(&self, visual_id: VisualId) -> Option<VisualInfo> {
        for visuals in self.visuals.values() {
            for visual_info in visuals {
                if visual_info.visual.visual_id == visual_id {
                    return Some(visual_info.clone());
                }
            }
        }
        None
    }

    fn supports_depth(&self, visual_id: VisualId, depth: u8) -> bool {
        if let Some(visual_info) = self.find_visual(visual_id) {
            visual_info.depth == depth
        } else {
            false
        }
    }

    fn get_supported_depths(&self, visual_id: VisualId) -> Vec<u8> {
        if let Some(visual_info) = self.find_visual(visual_id) {
            vec![visual_info.depth]
        } else {
            Vec::new()
        }
    }

    fn validate_visual(&self, visual_id: VisualId, depth: u8) -> ValidationResult {
        self.validator.validate(visual_id, depth, self)
    }
}

impl Default for DefaultVisualSystem {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the validation::VisualSystem trait for DefaultVisualSystem
impl validation::VisualSystem for DefaultVisualSystem {
    fn find_visual(&self, visual_id: VisualId) -> Option<VisualInfo> {
        <DefaultVisualSystem as VisualSystem>::find_visual(self, visual_id)
    }
}
