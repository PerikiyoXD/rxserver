//! Display-related type definitions and enums
//!
//! This module contains all the data structures and enums used throughout
//! the display management system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Display configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    /// Screen width in pixels
    pub width: u32,
    /// Screen height in pixels
    pub height: u32,
    /// Color depth in bits per pixel
    pub depth: u8,
    /// Dots per inch resolution
    pub dpi: u16,
    /// Number of screens to create
    pub screens: u32,
    /// Default visual class
    pub visual_class: VisualClass,
    /// Enable hardware acceleration
    pub hw_acceleration: bool,
    /// Framebuffer configuration
    pub framebuffer: FramebufferSettings,
}

/// Framebuffer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FramebufferSettings {
    /// Use software framebuffer
    pub software: bool,
    /// Bits per pixel
    pub bpp: u8,
    /// Scanline padding
    pub scanline_pad: u8,
    /// Byte order (true for little endian)
    pub little_endian: bool,
}

/// Information about a screen
#[derive(Debug, Clone)]
pub struct ScreenInfo {
    /// Screen identifier
    pub id: u32,
    /// Screen width in pixels
    pub width: u32,
    /// Screen height in pixels
    pub height: u32,
    /// Screen width in millimeters
    pub width_mm: u32,
    /// Screen height in millimeters
    pub height_mm: u32,
    /// Color depth in bits
    pub depth: u8,
    /// Root window ID
    pub root_window: u32,
    /// Default colormap ID
    pub default_colormap: u32,
    /// White pixel value
    pub white_pixel: u32,
    /// Black pixel value
    pub black_pixel: u32,
    /// Available visuals
    pub visuals: Vec<VisualInfo>,
    /// Screen position (for multi-screen setups)
    pub position: ScreenPosition,
    /// DPI settings
    pub dpi_x: u16,
    pub dpi_y: u16,
}

/// Screen position for multi-screen configurations
#[derive(Debug, Clone)]
pub struct ScreenPosition {
    pub x: i32,
    pub y: i32,
}

/// Information about a visual
#[derive(Debug, Clone)]
pub struct VisualInfo {
    /// Visual ID
    pub id: u32,
    /// Visual class
    pub class: VisualClass,
    /// Bits per RGB value
    pub bits_per_rgb: u8,
    /// Colormap entries
    pub colormap_entries: u32,
    /// Red mask for TrueColor/DirectColor
    pub red_mask: u32,
    /// Green mask for TrueColor/DirectColor
    pub green_mask: u32,
    /// Blue mask for TrueColor/DirectColor
    pub blue_mask: u32,
    /// Depth in bits
    pub depth: u8,
}

/// Visual class types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum VisualClass {
    /// Static Gray visual
    StaticGray,
    /// Gray Scale visual
    GrayScale,
    /// Static Color visual
    StaticColor,
    /// Pseudo Color visual
    PseudoColor,
    /// True Color visual
    TrueColor,
    /// Direct Color visual
    DirectColor,
}

/// Colormap information
#[derive(Debug, Clone)]
pub struct ColormapInfo {
    /// Colormap ID
    pub id: u32,
    /// Visual this colormap is for
    pub visual_id: u32,
    /// Color entries
    pub entries: Vec<ColorEntry>,
    /// Is this the default colormap
    pub is_default: bool,
}

/// Color entry in a colormap
#[derive(Debug, Clone)]
pub struct ColorEntry {
    /// Red component (0-65535)
    pub red: u16,
    /// Green component (0-65535)
    pub green: u16,
    /// Blue component (0-65535)
    pub blue: u16,
    /// Flags indicating which components are allocated
    pub flags: ColorFlags,
}

/// Color component flags
#[derive(Debug, Clone)]
pub struct ColorFlags {
    pub red: bool,
    pub green: bool,
    pub blue: bool,
}

/// Display state enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayState {
    /// Display is uninitialized
    Uninitialized,
    /// Display is initializing
    Initializing,
    /// Display is active and ready
    Active,
    /// Display is suspended
    Suspended,
    /// Display has encountered an error
    Error(String),
}

/// Screen resources
#[derive(Debug)]
pub struct ScreenResources {
    /// Framebuffer reference
    pub framebuffer: Option<crate::display::framebuffer::Framebuffer>,
    /// Graphics contexts
    pub graphics_contexts: HashMap<u32, GraphicsContext>,
    /// Pixmaps
    pub pixmaps: HashMap<u32, Pixmap>,
    /// Cursor information
    pub cursors: HashMap<u32, Cursor>,
}

/// Graphics context information
#[derive(Debug, Clone)]
pub struct GraphicsContext {
    /// GC ID
    pub id: u32,
    /// Foreground pixel
    pub foreground: u32,
    /// Background pixel
    pub background: u32,
    /// Line width
    pub line_width: u32,
    /// Line style
    pub line_style: LineStyle,
    /// Fill style
    pub fill_style: FillStyle,
}

/// Line style enumeration
#[derive(Debug, Clone)]
pub enum LineStyle {
    Solid,
    OnOffDash,
    DoubleDash,
}

/// Fill style enumeration
#[derive(Debug, Clone)]
pub enum FillStyle {
    Solid,
    Tiled,
    Stippled,
    OpaqueStippled,
}

/// Pixmap information
#[derive(Debug, Clone)]
pub struct Pixmap {
    /// Pixmap ID
    pub id: u32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Depth in bits
    pub depth: u8,
    /// Pixel data
    pub data: Vec<u8>,
}

/// Cursor information
#[derive(Debug, Clone)]
pub struct Cursor {
    /// Cursor ID
    pub id: u32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Hot spot X coordinate
    pub hotspot_x: u32,
    /// Hot spot Y coordinate
    pub hotspot_y: u32,
    /// Cursor image data
    pub image: Vec<u8>,
    /// Cursor mask data
    pub mask: Vec<u8>,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            depth: 24,
            dpi: 96,
            screens: 1,
            visual_class: VisualClass::TrueColor,
            hw_acceleration: false,
            framebuffer: FramebufferSettings::default(),
        }
    }
}

impl Default for FramebufferSettings {
    fn default() -> Self {
        Self {
            software: true,
            bpp: 32,
            scanline_pad: 32,
            little_endian: true,
        }
    }
}

impl ScreenInfo {
    /// Create a new screen info with default values
    pub fn new(id: u32, config: &DisplaySettings) -> Self {
        Self {
            id,
            width: config.width,
            height: config.height,
            width_mm: (config.width as f32 / config.dpi as f32 * 25.4) as u32,
            height_mm: (config.height as f32 / config.dpi as f32 * 25.4) as u32,
            depth: config.depth,
            root_window: id * 1000 + 1, // Simple root window ID generation
            default_colormap: id * 1000 + 2, // Simple colormap ID generation
            white_pixel: 0xFFFFFF,
            black_pixel: 0x000000,
            visuals: Vec::new(),
            position: ScreenPosition { x: 0, y: 0 },
            dpi_x: config.dpi,
            dpi_y: config.dpi,
        }
    }
}

impl VisualInfo {
    /// Create a new TrueColor visual
    pub fn new_truecolor(id: u32, depth: u8) -> Self {
        let (red_mask, green_mask, blue_mask, colormap_entries) = match depth {
            16 => (0xF800, 0x07E0, 0x001F, 0),
            24 | 32 => (0xFF0000, 0x00FF00, 0x0000FF, 0),
            _ => (0, 0, 0, 1 << depth),
        };

        Self {
            id,
            class: VisualClass::TrueColor,
            bits_per_rgb: match depth {
                16 => 5,
                24 | 32 => 8,
                _ => depth,
            },
            colormap_entries,
            red_mask,
            green_mask,
            blue_mask,
            depth,
        }
    }

    /// Create a new PseudoColor visual
    pub fn new_pseudocolor(id: u32, depth: u8) -> Self {
        Self {
            id,
            class: VisualClass::PseudoColor,
            bits_per_rgb: depth,
            colormap_entries: 1 << depth,
            red_mask: 0,
            green_mask: 0,
            blue_mask: 0,
            depth,
        }
    }
}

impl Default for ScreenResources {
    fn default() -> Self {
        Self {
            framebuffer: None,
            graphics_contexts: HashMap::new(),
            pixmaps: HashMap::new(),
            cursors: HashMap::new(),
        }
    }
}
