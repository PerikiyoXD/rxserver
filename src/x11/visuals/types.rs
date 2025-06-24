//! Visual type definitions
//!
//! This module defines the core types for X11 visuals, including visual classes,
//! color depths, and pixel formats.

use crate::x11::VisualId;

/// X11 Visual Class types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VisualClass {
    /// Static Gray - fixed colormap, grayscale
    StaticGray = 0,
    /// Gray Scale - changeable colormap, grayscale
    GrayScale = 1,
    /// Static Color - fixed colormap, color
    StaticColor = 2,
    /// Pseudo Color - changeable colormap, color indices
    PseudoColor = 3,
    /// True Color - fixed colormap, RGB values
    TrueColor = 4,
    /// Direct Color - changeable colormap, RGB indices
    DirectColor = 5,
}
impl VisualClass {
    /// Create from protocol value
    pub fn from_protocol_value(value: u8) -> Option<Self> {
        match value {
            0 => Some(VisualClass::StaticGray),
            1 => Some(VisualClass::GrayScale),
            2 => Some(VisualClass::StaticColor),
            3 => Some(VisualClass::PseudoColor),
            4 => Some(VisualClass::TrueColor),
            5 => Some(VisualClass::DirectColor),
            _ => None,
        }
    }

    /// Get the protocol value for this visual class
    pub fn to_protocol_value(self) -> u8 {
        match self {
            VisualClass::StaticGray => 0,
            VisualClass::GrayScale => 1,
            VisualClass::StaticColor => 2,
            VisualClass::PseudoColor => 3,
            VisualClass::TrueColor => 4,
            VisualClass::DirectColor => 5,
        }
    }

    /// Check if this visual class supports modifiable colormaps
    pub fn supports_modifiable_colormap(self) -> bool {
        matches!(
            self,
            VisualClass::GrayScale | VisualClass::PseudoColor | VisualClass::DirectColor
        )
    }

    /// Check if this visual class is color (not grayscale)
    pub fn is_color(self) -> bool {
        !matches!(self, VisualClass::StaticGray | VisualClass::GrayScale)
    }

    /// Check if this visual class uses direct color mapping
    pub fn is_direct_color(self) -> bool {
        matches!(self, VisualClass::TrueColor | VisualClass::DirectColor)
    }

    /// Get the name of this visual class
    pub fn name(self) -> &'static str {
        match self {
            VisualClass::StaticGray => "StaticGray",
            VisualClass::GrayScale => "GrayScale",
            VisualClass::StaticColor => "StaticColor",
            VisualClass::PseudoColor => "PseudoColor",
            VisualClass::TrueColor => "TrueColor",
            VisualClass::DirectColor => "DirectColor",
        }
    }
}

/// X11 Visual definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Visual {
    /// Visual identifier
    pub visual_id: VisualId,
    /// Visual class
    pub class: VisualClass,
    /// Bits per RGB component
    pub bits_per_rgb: u8,
    /// Number of colormap entries
    pub colormap_entries: u16,
    /// Red color mask (for direct color visuals)
    pub red_mask: u32,
    /// Green color mask (for direct color visuals)
    pub green_mask: u32,
    /// Blue color mask (for direct color visuals)
    pub blue_mask: u32,
}

impl Visual {
    /// Create a new visual
    pub fn new(
        visual_id: VisualId,
        class: VisualClass,
        bits_per_rgb: u8,
        colormap_entries: u16,
        red_mask: u32,
        green_mask: u32,
        blue_mask: u32,
    ) -> Self {
        Self {
            visual_id,
            class,
            bits_per_rgb,
            colormap_entries,
            red_mask,
            green_mask,
            blue_mask,
        }
    }

    /// Get the total bits per pixel for this visual
    pub fn bits_per_pixel(&self) -> u8 {
        match self.class {
            VisualClass::StaticGray | VisualClass::GrayScale => self.bits_per_rgb,
            VisualClass::StaticColor | VisualClass::PseudoColor => {
                // For indexed color, calculate from colormap entries
                (self.colormap_entries.next_power_of_two().trailing_zeros() as u8).max(1)
            }
            VisualClass::TrueColor | VisualClass::DirectColor => {
                // For direct color, sum the RGB components
                self.bits_per_rgb * 3
            }
        }
    }

    /// Check if this visual uses a colormap
    pub fn uses_colormap(&self) -> bool {
        !self.class.is_direct_color()
    }

    /// Get the number of color planes for this visual
    pub fn color_planes(&self) -> u8 {
        match self.class {
            VisualClass::StaticGray | VisualClass::GrayScale => 1,
            VisualClass::StaticColor | VisualClass::PseudoColor => 1,
            VisualClass::TrueColor | VisualClass::DirectColor => 3,
        }
    }

    /// Extract red component from a pixel value
    pub fn extract_red(&self, pixel: u32) -> u8 {
        if self.red_mask == 0 {
            0
        } else {
            let shifted = (pixel & self.red_mask) >> self.red_mask.trailing_zeros();
            let max_value = self.red_mask >> self.red_mask.trailing_zeros();
            ((shifted * 255) / max_value) as u8
        }
    }

    /// Extract green component from a pixel value
    pub fn extract_green(&self, pixel: u32) -> u8 {
        if self.green_mask == 0 {
            0
        } else {
            let shifted = (pixel & self.green_mask) >> self.green_mask.trailing_zeros();
            let max_value = self.green_mask >> self.green_mask.trailing_zeros();
            ((shifted * 255) / max_value) as u8
        }
    }

    /// Extract blue component from a pixel value
    pub fn extract_blue(&self, pixel: u32) -> u8 {
        if self.blue_mask == 0 {
            0
        } else {
            let shifted = (pixel & self.blue_mask) >> self.blue_mask.trailing_zeros();
            let max_value = self.blue_mask >> self.blue_mask.trailing_zeros();
            ((shifted * 255) / max_value) as u8
        }
    }

    /// Pack RGB components into a pixel value
    pub fn pack_rgb(&self, red: u8, green: u8, blue: u8) -> u32 {
        let mut pixel = 0;

        if self.red_mask != 0 {
            let max_value = self.red_mask >> self.red_mask.trailing_zeros();
            let scaled = ((red as u32) * max_value) / 255;
            pixel |= (scaled << self.red_mask.trailing_zeros()) & self.red_mask;
        }

        if self.green_mask != 0 {
            let max_value = self.green_mask >> self.green_mask.trailing_zeros();
            let scaled = ((green as u32) * max_value) / 255;
            pixel |= (scaled << self.green_mask.trailing_zeros()) & self.green_mask;
        }

        if self.blue_mask != 0 {
            let max_value = self.blue_mask >> self.blue_mask.trailing_zeros();
            let scaled = ((blue as u32) * max_value) / 255;
            pixel |= (scaled << self.blue_mask.trailing_zeros()) & self.blue_mask;
        }

        pixel
    }
}

/// Visual information including depth and screen
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualInfo {
    /// The visual definition
    pub visual: Visual,
    /// Color depth for this visual
    pub depth: u8,
    /// Screen number this visual belongs to
    pub screen: u8,
}

impl VisualInfo {
    /// Create new visual info
    pub fn new(visual: Visual, depth: u8, screen: u8) -> Self {
        Self {
            visual,
            depth,
            screen,
        }
    }

    /// Get the visual ID
    pub fn visual_id(&self) -> VisualId {
        self.visual.visual_id
    }

    /// Check if this visual is suitable for a given depth
    pub fn supports_depth(&self, depth: u8) -> bool {
        self.depth == depth
    }

    /// Get a human-readable description of this visual
    pub fn description(&self) -> String {
        format!(
            "{} {}bpp (depth {}) on screen {}",
            self.visual.class.name(),
            self.visual.bits_per_pixel(),
            self.depth,
            self.screen
        )
    }
}

/// Depth information for a screen
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepthInfo {
    /// Color depth in bits
    pub depth: u8,
    /// Available visuals at this depth
    pub visuals: Vec<VisualId>,
}

impl DepthInfo {
    /// Create new depth info
    pub fn new(depth: u8) -> Self {
        Self {
            depth,
            visuals: Vec::new(),
        }
    }

    /// Add a visual to this depth
    pub fn add_visual(&mut self, visual_id: VisualId) {
        if !self.visuals.contains(&visual_id) {
            self.visuals.push(visual_id);
        }
    }

    /// Remove a visual from this depth
    pub fn remove_visual(&mut self, visual_id: VisualId) {
        self.visuals.retain(|&id| id != visual_id);
    }

    /// Check if this depth has any visuals
    pub fn has_visuals(&self) -> bool {
        !self.visuals.is_empty()
    }

    /// Get the number of visuals at this depth
    pub fn visual_count(&self) -> usize {
        self.visuals.len()
    }
}
