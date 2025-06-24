//! Display capability detection and management

use super::types::{ColorDepth, Resolution};
use crate::types::Result;

/// Display capabilities detector
#[derive(Debug)]
pub struct DisplayCapabilities {
    max_resolution: Resolution,
    supported_depths: Vec<ColorDepth>,
    supports_hardware_acceleration: bool,
}

impl DisplayCapabilities {
    /// Detect capabilities for the current system
    pub fn detect() -> Result<Self> {
        Ok(Self {
            max_resolution: Resolution::new(8192, 8192),
            supported_depths: vec![
                ColorDepth::Depth8,
                ColorDepth::Depth16,
                ColorDepth::Depth24,
                ColorDepth::Depth32,
            ],
            supports_hardware_acceleration: false,
        })
    }

    /// Get maximum supported resolution
    pub fn max_resolution(&self) -> Resolution {
        self.max_resolution
    }

    /// Get supported color depths
    pub fn supported_depths(&self) -> &[ColorDepth] {
        &self.supported_depths
    }

    /// Check if hardware acceleration is supported
    pub fn supports_hardware_acceleration(&self) -> bool {
        self.supports_hardware_acceleration
    }

    /// Check if a resolution is supported
    pub fn supports_resolution(&self, resolution: Resolution) -> bool {
        resolution.width <= self.max_resolution.width
            && resolution.height <= self.max_resolution.height
    }

    /// Check if a color depth is supported
    pub fn supports_color_depth(&self, depth: ColorDepth) -> bool {
        self.supported_depths.contains(&depth)
    }
}
