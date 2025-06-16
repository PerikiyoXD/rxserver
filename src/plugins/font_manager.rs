//! Font manager
//!
//! This module handles font loading, caching, and management
//! for the X11 server.

use crate::core::error::ServerResult;
use std::collections::HashMap;

/// Font identifier
pub type FontId = u32;

/// Font resource
#[derive(Debug, Clone)]
pub struct Font {
    pub id: FontId,
    pub name: String,
    pub size: u16,
    pub style: FontStyle,
}

/// Font style flags
#[derive(Debug, Clone, Copy)]
pub struct FontStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

/// Font manager for handling font resources
pub struct FontManager {
    fonts: HashMap<FontId, Font>,
    font_cache: HashMap<String, FontId>,
    next_id: FontId,
}

impl FontManager {
    /// Create a new font manager
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            font_cache: HashMap::new(),
            next_id: 1,
        }
    }

    /// Load a font by name
    pub fn load_font(&mut self, name: &str) -> ServerResult<FontId> {
        // Check if font is already loaded
        if let Some(&font_id) = self.font_cache.get(name) {
            return Ok(font_id);
        }

        // Create new font resource
        let font_id = self.next_id;
        self.next_id += 1;

        let font = Font {
            id: font_id,
            name: name.to_string(),
            size: 12, // Default size
            style: FontStyle {
                bold: false,
                italic: false,
                underline: false,
            },
        };

        self.fonts.insert(font_id, font);
        self.font_cache.insert(name.to_string(), font_id);

        Ok(font_id)
    }

    /// Get font by ID
    pub fn get_font(&self, font_id: FontId) -> Option<&Font> {
        self.fonts.get(&font_id)
    }

    /// Unload a font
    pub fn unload_font(&mut self, font_id: FontId) -> ServerResult<()> {
        if let Some(font) = self.fonts.remove(&font_id) {
            self.font_cache.remove(&font.name);
        }
        Ok(())
    }

    /// Get all loaded fonts
    pub fn list_fonts(&self) -> Vec<&Font> {
        self.fonts.values().collect()
    }

    /// Check if font is loaded
    pub fn is_font_loaded(&self, font_id: FontId) -> bool {
        self.fonts.contains_key(&font_id)
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}
