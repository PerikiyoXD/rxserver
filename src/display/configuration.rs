//! Display configuration types

use super::{
    BackendType,
    types::{ColorDepth, RefreshRate, Resolution},
};
use serde::{Deserialize, Serialize};

/// Display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Preferred backend type
    pub preferred_backend: BackendType,
    /// Default resolution
    pub default_resolution: Resolution,
    /// Default color depth
    pub default_color_depth: ColorDepth,
    /// Default refresh rate
    pub default_refresh_rate: RefreshRate,
    /// Enable multiple displays
    pub enable_multiple_displays: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            preferred_backend: BackendType::Headless,
            default_resolution: Resolution::new(1920, 1080),
            default_color_depth: ColorDepth::Depth24,
            default_refresh_rate: RefreshRate::new(60.0),
            enable_multiple_displays: false,
        }
    }
}

impl DisplayConfig {
    /// Create a new display configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the preferred backend
    pub fn with_backend(mut self, backend: BackendType) -> Self {
        self.preferred_backend = backend;
        self
    }

    /// Set the default resolution
    pub fn with_resolution(mut self, resolution: Resolution) -> Self {
        self.default_resolution = resolution;
        self
    }

    /// Set the default color depth
    pub fn with_color_depth(mut self, depth: ColorDepth) -> Self {
        self.default_color_depth = depth;
        self
    }
}
