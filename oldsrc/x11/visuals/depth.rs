//! Color depth handling
//!
//! This module manages color depths and their relationship to visuals.

use crate::x11::VisualId;
use crate::x11::visuals::types::DepthInfo;
use std::collections::HashMap;

/// Color depth representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColorDepth(pub u8);

impl ColorDepth {
    /// Create a new color depth
    pub fn new(depth: u8) -> Self {
        Self(depth)
    }

    /// Get the depth value
    pub fn value(self) -> u8 {
        self.0
    }

    /// Check if this is a valid color depth
    pub fn is_valid(self) -> bool {
        matches!(self.0, 1 | 4 | 8 | 15 | 16 | 24 | 30 | 32)
    }

    /// Get the number of possible colors for this depth
    pub fn color_count(self) -> u64 {
        1u64 << self.0
    }

    /// Get the bytes per pixel for this depth
    pub fn bytes_per_pixel(self) -> u8 {
        match self.0 {
            1..=8 => 1,
            9..=16 => 2,
            17..=24 => 3,
            25..=32 => 4,
            _ => (self.0 + 7) / 8,
        }
    }

    /// Check if this depth supports grayscale
    pub fn supports_grayscale(self) -> bool {
        self.0 >= 1
    }

    /// Check if this depth supports color
    pub fn supports_color(self) -> bool {
        self.0 >= 4
    }

    /// Check if this depth supports true color
    pub fn supports_true_color(self) -> bool {
        self.0 >= 15
    }
}

impl From<u8> for ColorDepth {
    fn from(depth: u8) -> Self {
        Self(depth)
    }
}

impl From<ColorDepth> for u8 {
    fn from(depth: ColorDepth) -> Self {
        depth.0
    }
}

/// Manages color depths for the display system
#[derive(Debug)]
pub struct DepthManager {
    /// Available depths by screen
    depths: HashMap<u8, Vec<DepthInfo>>,
    /// Default depth for each screen
    default_depths: HashMap<u8, ColorDepth>,
}

impl DepthManager {
    /// Create a new depth manager
    pub fn new() -> Self {
        let mut manager = Self {
            depths: HashMap::new(),
            default_depths: HashMap::new(),
        };

        // Initialize with common depths for screen 0
        manager.initialize_default_depths();
        manager
    }

    /// Initialize default depths
    fn initialize_default_depths(&mut self) {
        let screen = 0;
        let mut screen_depths = Vec::new();

        // Add common depths
        for &depth_value in &[1, 8, 15, 16, 24, 32] {
            let mut depth_info = DepthInfo::new(depth_value);

            // Add default visuals for each depth
            match depth_value {
                1 => {
                    // Monochrome
                    depth_info.add_visual(VisualId(0x30));
                }
                8 => {
                    // 8-bit indexed color
                    depth_info.add_visual(VisualId(0x23)); // PseudoColor
                    depth_info.add_visual(VisualId(0x24)); // StaticColor
                }
                15 | 16 => {
                    // 15/16-bit high color
                    depth_info.add_visual(VisualId(0x25)); // TrueColor
                }
                24 | 32 => {
                    // 24/32-bit true color
                    depth_info.add_visual(VisualId(0x21)); // TrueColor
                    depth_info.add_visual(VisualId(0x22)); // DirectColor
                }
                _ => {}
            }

            screen_depths.push(depth_info);
        }

        self.depths.insert(screen, screen_depths);
        self.default_depths.insert(screen, ColorDepth::new(24));
    }

    /// Get available depths for a screen
    pub fn get_depths(&self, screen: u8) -> Vec<DepthInfo> {
        self.depths.get(&screen).cloned().unwrap_or_default()
    }

    /// Get the default depth for a screen
    pub fn get_default_depth(&self, screen: u8) -> Option<ColorDepth> {
        self.default_depths.get(&screen).copied()
    }

    /// Set the default depth for a screen
    pub fn set_default_depth(&mut self, screen: u8, depth: ColorDepth) {
        self.default_depths.insert(screen, depth);
    }

    /// Add a depth to a screen
    pub fn add_depth(&mut self, screen: u8, depth_info: DepthInfo) {
        self.depths
            .entry(screen)
            .or_insert_with(Vec::new)
            .push(depth_info);
    }

    /// Check if a depth is supported on a screen
    pub fn supports_depth(&self, screen: u8, depth: ColorDepth) -> bool {
        if let Some(depths) = self.depths.get(&screen) {
            depths.iter().any(|d| d.depth == depth.value())
        } else {
            false
        }
    }

    /// Get visuals for a specific depth on a screen
    pub fn get_visuals_for_depth(&self, screen: u8, depth: ColorDepth) -> Vec<VisualId> {
        if let Some(depths) = self.depths.get(&screen) {
            for depth_info in depths {
                if depth_info.depth == depth.value() {
                    return depth_info.visuals.clone();
                }
            }
        }
        Vec::new()
    }

    /// Add a visual to a depth
    pub fn add_visual_to_depth(&mut self, screen: u8, depth: ColorDepth, visual_id: VisualId) {
        if let Some(depths) = self.depths.get_mut(&screen) {
            let mut found = false;
            for depth_info in depths.iter_mut() {
                if depth_info.depth == depth.value() {
                    depth_info.add_visual(visual_id);
                    found = true;
                    break;
                }
            }

            // Depth not found, create it
            if !found {
                let mut depth_info = DepthInfo::new(depth.value());
                depth_info.add_visual(visual_id);
                depths.push(depth_info);
            }
        }
    }

    /// Remove a visual from a depth
    pub fn remove_visual_from_depth(&mut self, screen: u8, depth: ColorDepth, visual_id: VisualId) {
        if let Some(depths) = self.depths.get_mut(&screen) {
            for depth_info in depths.iter_mut() {
                if depth_info.depth == depth.value() {
                    depth_info.remove_visual(visual_id);
                    break;
                }
            }

            // Clean up empty depths
            depths.retain(|d| d.has_visuals());
        }
    }

    /// Get all supported depths for a screen
    pub fn get_supported_depths(&self, screen: u8) -> Vec<ColorDepth> {
        if let Some(depths) = self.depths.get(&screen) {
            depths.iter().map(|d| ColorDepth::new(d.depth)).collect()
        } else {
            Vec::new()
        }
    }

    /// Find the best depth for a given requirement
    pub fn find_best_depth(
        &self,
        screen: u8,
        min_depth: ColorDepth,
        prefer_true_color: bool,
    ) -> Option<ColorDepth> {
        let depths = self.get_supported_depths(screen);

        if prefer_true_color {
            // Find the best true color depth
            depths
                .into_iter()
                .filter(|&d| d >= min_depth && d.supports_true_color())
                .min() // Prefer the smallest sufficient depth
        } else {
            // Find any suitable depth
            depths.into_iter().filter(|&d| d >= min_depth).min()
        }
    }

    /// Get statistics about depth usage
    pub fn get_depth_statistics(&self, screen: u8) -> DepthStatistics {
        let depths = self.get_depths(screen);
        let total_visuals: usize = depths.iter().map(|d| d.visual_count()).sum();
        let depth_count = depths.len();
        let max_depth = depths.iter().map(|d| d.depth).max().unwrap_or(0);

        DepthStatistics {
            depth_count,
            total_visuals,
            max_depth: ColorDepth::new(max_depth),
            depths,
        }
    }
}

impl Default for DepthManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about depth usage
#[derive(Debug, Clone)]
pub struct DepthStatistics {
    /// Number of different depths
    pub depth_count: usize,
    /// Total number of visuals across all depths
    pub total_visuals: usize,
    /// Maximum supported depth
    pub max_depth: ColorDepth,
    /// Detailed depth information
    pub depths: Vec<DepthInfo>,
}

impl DepthStatistics {
    /// Get the most common depth (by visual count)
    pub fn most_common_depth(&self) -> Option<ColorDepth> {
        self.depths
            .iter()
            .max_by_key(|d| d.visual_count())
            .map(|d| ColorDepth::new(d.depth))
    }

    /// Check if true color is supported
    pub fn supports_true_color(&self) -> bool {
        self.depths
            .iter()
            .any(|d| ColorDepth::new(d.depth).supports_true_color())
    }

    /// Get average visuals per depth
    pub fn average_visuals_per_depth(&self) -> f64 {
        if self.depth_count > 0 {
            self.total_visuals as f64 / self.depth_count as f64
        } else {
            0.0
        }
    }
}
