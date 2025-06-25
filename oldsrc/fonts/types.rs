//! Font-related types and structures

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Font information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    pub family: FontFamily,
    pub style: FontStyle,
    pub weight: FontWeight,
    pub size: f32,
    pub path: Option<PathBuf>,
}

/// Font family name
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FontFamily(pub String);

impl FontFamily {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

impl From<&str> for FontFamily {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

/// Font style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self::Normal
    }
}

/// Font weight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

impl Default for FontWeight {
    fn default() -> Self {
        Self::Normal
    }
}

impl FontWeight {
    pub fn value(&self) -> u16 {
        *self as u16
    }
}

/// Font metrics
#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub line_height: f32,
    pub max_advance: f32,
}

/// Glyph information
#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub codepoint: u32,
    pub advance_width: f32,
    pub advance_height: f32,
    pub bitmap_left: i32,
    pub bitmap_top: i32,
    pub bitmap_width: u32,
    pub bitmap_height: u32,
}

/// Text rendering context
#[derive(Debug, Clone)]
pub struct TextContext {
    pub font: FontInfo,
    pub color: u32,              // RGBA
    pub background: Option<u32>, // Optional background color
    pub anti_aliasing: bool,
}

impl Default for TextContext {
    fn default() -> Self {
        Self {
            font: FontInfo {
                family: FontFamily::new("monospace"),
                style: FontStyle::Normal,
                weight: FontWeight::Normal,
                size: 12.0,
                path: None,
            },
            color: 0x000000FF, // Black
            background: None,
            anti_aliasing: true,
        }
    }
}
