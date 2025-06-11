//! Display and screen management
//!
//! This module manages the virtual display and screen configuration.

use crate::{config::DisplaySettings, Result};
use crate::protocol::types::*;

/// Represents the X11 display/screen
pub struct Display {
    /// Screen width in pixels
    pub width: u32,
    /// Screen height in pixels
    pub height: u32,
    /// Color depth in bits per pixel
    pub depth: u8,
    /// DPI (dots per inch)
    pub dpi: u16,
    /// Root window ID
    pub root_window: Window,
    /// Default colormap
    pub default_colormap: Colormap,
    /// White pixel value
    pub white_pixel: u32,
    /// Black pixel value
    pub black_pixel: u32,
}

impl Display {
    /// Create a new display with the given settings
    pub fn new(settings: &DisplaySettings) -> Result<Self> {
        log::info!(
            "Creating display {}x{} at {} bpp, {} DPI",
            settings.width,
            settings.height,
            settings.depth,
            settings.dpi
        );

        Ok(Self {
            width: settings.width,
            height: settings.height,
            depth: settings.depth,
            dpi: settings.dpi,
            root_window: 1, // Root window is always ID 1
            default_colormap: 1, // Default colormap ID
            white_pixel: 0xFFFFFF,
            black_pixel: 0x000000,
        })
    }

    /// Get display information for connection setup
    pub fn get_setup_info(&self) -> DisplaySetupInfo {
        DisplaySetupInfo {
            width: self.width,
            height: self.height,
            width_mm: (self.width * 254 / (self.dpi as u32 * 10)) as u16, // Convert to mm
            height_mm: (self.height * 254 / (self.dpi as u32 * 10)) as u16,
            root_window: self.root_window,
            default_colormap: self.default_colormap,
            white_pixel: self.white_pixel,
            black_pixel: self.black_pixel,
            root_depth: self.depth,
        }
    }

    /// Get the root window bounds
    pub fn root_bounds(&self) -> Rectangle {
        Rectangle {
            x: 0,
            y: 0,
            width: self.width as u16,
            height: self.height as u16,
        }
    }

    /// Check if a point is within the display bounds
    pub fn contains_point(&self, x: i16, y: i16) -> bool {
        x >= 0 && y >= 0 && x < self.width as i16 && y < self.height as i16
    }

    /// Check if a rectangle is within the display bounds
    pub fn contains_rectangle(&self, rect: &Rectangle) -> bool {
        rect.x >= 0
            && rect.y >= 0
            && rect.x + rect.width as i16 <= self.width as i16
            && rect.y + rect.height as i16 <= self.height as i16
    }

    /// Clip a rectangle to the display bounds
    pub fn clip_rectangle(&self, rect: &Rectangle) -> Option<Rectangle> {
        let x1 = rect.x.max(0);
        let y1 = rect.y.max(0);
        let x2 = (rect.x + rect.width as i16).min(self.width as i16);
        let y2 = (rect.y + rect.height as i16).min(self.height as i16);

        if x2 > x1 && y2 > y1 {
            Some(Rectangle {
                x: x1,
                y: y1,
                width: (x2 - x1) as u16,
                height: (y2 - y1) as u16,
            })
        } else {
            None
        }
    }
}

/// Display setup information sent to clients
#[derive(Debug, Clone)]
pub struct DisplaySetupInfo {
    pub width: u32,
    pub height: u32,
    pub width_mm: u16,
    pub height_mm: u16,
    pub root_window: Window,
    pub default_colormap: Colormap,
    pub white_pixel: u32,
    pub black_pixel: u32,
    pub root_depth: u8,
}
