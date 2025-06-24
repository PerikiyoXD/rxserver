//! Display-related types and structures

use serde::{Deserialize, Serialize};

/// Display resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    /// Create a new resolution
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Get the aspect ratio
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    /// Calculate total pixels
    pub fn total_pixels(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

/// Color depth in bits per pixel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorDepth {
    Depth1 = 1,
    Depth4 = 4,
    Depth8 = 8,
    Depth15 = 15,
    Depth16 = 16,
    Depth24 = 24,
    Depth32 = 32,
}

impl ColorDepth {
    /// Get the number of bytes per pixel
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            ColorDepth::Depth1 => 1,
            ColorDepth::Depth4 => 1,
            ColorDepth::Depth8 => 1,
            ColorDepth::Depth15 => 2,
            ColorDepth::Depth16 => 2,
            ColorDepth::Depth24 => 3,
            ColorDepth::Depth32 => 4,
        }
    }
}

/// Refresh rate in Hz
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RefreshRate(pub f64);

impl RefreshRate {
    pub fn new(hz: f64) -> Self {
        Self(hz)
    }

    pub fn hz(&self) -> f64 {
        self.0
    }
}

/// Display information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub resolution: Resolution,
    pub color_depth: ColorDepth,
    pub refresh_rate: RefreshRate,
    pub name: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
}

impl DisplayInfo {
    /// Create new display info
    pub fn new(
        resolution: Resolution,
        color_depth: ColorDepth,
        refresh_rate: RefreshRate,
        name: String,
    ) -> Self {
        Self {
            resolution,
            color_depth,
            refresh_rate,
            name,
            manufacturer: None,
            model: None,
        }
    }
}
