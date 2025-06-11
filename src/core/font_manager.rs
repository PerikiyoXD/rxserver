//! Font Management System for X11 Server
//!
//! This module provides a font loading, storage, and management system that handles
//! font resources, maintains the mapping between font IDs and font data, and provides
//! thread-safe access to font information.

use crate::protocol::types::Font;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info, warn};

/// Font information and metrics
#[derive(Debug, Clone)]
pub struct FontInfo {
    /// Font ID
    pub id: Font,
    /// Font name as requested
    pub name: String,
    /// Font properties (simplified for now)
    pub properties: FontProperties,
}

/// Font properties and metrics
#[derive(Debug, Clone)]
pub struct FontProperties {
    /// Font ascent (above baseline)
    pub font_ascent: i16,
    /// Font descent (below baseline)
    pub font_descent: i16,
    /// Character ascent
    pub char_ascent: i16,
    /// Character descent
    pub char_descent: i16,
    /// Maximum character width
    pub max_char_width: u16,
    /// Minimum character width
    pub min_char_width: u16,
    /// Number of characters in font
    pub char_count: u16,
    /// Default character (used for missing glyphs)
    pub default_char: u16,
    /// Font direction (0 = left-to-right, 1 = right-to-left)
    pub draw_direction: u8,
    /// Minimum byte1 value for 2-byte characters
    pub min_byte1: u8,
    /// Maximum byte1 value for 2-byte characters
    pub max_byte1: u8,
    /// Are all characters the same width?
    pub all_chars_exist: bool,
}

impl Default for FontProperties {
    fn default() -> Self {
        FontProperties {
            font_ascent: 12,
            font_descent: 3,
            char_ascent: 12,
            char_descent: 3,
            max_char_width: 8,
            min_char_width: 8,
            char_count: 95,    // ASCII printable characters
            default_char: 32,  // Space character
            draw_direction: 0, // Left-to-right
            min_byte1: 0,
            max_byte1: 0,
            all_chars_exist: true,
        }
    }
}

/// Thread-safe font manager for the X11 server
#[derive(Debug)]
pub struct FontManager {
    /// Map from font ID to font information
    fonts: Arc<RwLock<HashMap<Font, FontInfo>>>,
    /// Map from font name to font ID (for reuse)
    name_to_id: Arc<RwLock<HashMap<String, Font>>>,
}

impl FontManager {
    /// Create a new font manager
    pub fn new() -> Self {
        info!("Initializing FontManager");
        let manager = FontManager {
            fonts: Arc::new(RwLock::new(HashMap::new())),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
        };

        info!("FontManager initialized");
        manager
    }

    /// Load/open a font with the given name and font ID
    pub fn open_font(&self, fid: Font, name: &str) -> Result<(), String> {
        debug!("Opening font: fid={}, name='{}'", fid, name);

        // Check if font ID is already in use
        {
            let fonts = self.fonts.read().unwrap();
            if fonts.contains_key(&fid) {
                let error_msg = format!("Font ID {} already in use", fid);
                warn!("{}", error_msg);
                return Err(error_msg);
            }
        }

        // Check if we already have this font loaded (by name)
        {
            let name_to_id = self.name_to_id.read().unwrap();
            if let Some(&existing_fid) = name_to_id.get(name) {
                debug!(
                    "Font '{}' already loaded with ID {}, creating alias with ID {}",
                    name, existing_fid, fid
                );

                // Create a copy for the new font ID
                let existing_font = {
                    let fonts = self.fonts.read().unwrap();
                    fonts.get(&existing_fid).cloned()
                };

                if let Some(mut font_info) = existing_font {
                    font_info.id = fid; // Update the ID

                    // Add to both maps
                    {
                        let mut fonts = self.fonts.write().unwrap();
                        fonts.insert(fid, font_info);
                    }

                    debug!("Created font alias: '{}' -> fid={}", name, fid);
                    return Ok(());
                }
            }
        }

        // Load the font (simplified - we'll use default properties for now)
        let font_info = self.load_font_info(fid, name)?;

        // Add to both maps
        {
            let mut fonts = self.fonts.write().unwrap();
            fonts.insert(fid, font_info);
        }
        {
            let mut name_to_id = self.name_to_id.write().unwrap();
            name_to_id.insert(name.to_string(), fid);
        }

        info!("Successfully opened font: '{}' with ID {}", name, fid);
        Ok(())
    }

    /// Close/free a font by font ID
    pub fn close_font(&self, fid: Font) -> Result<(), String> {
        debug!("Closing font: fid={}", fid);

        let removed_font = {
            let mut fonts = self.fonts.write().unwrap();
            fonts.remove(&fid)
        };

        match removed_font {
            Some(font_info) => {
                // Remove from name mapping only if this was the primary ID for the name
                {
                    let mut name_to_id = self.name_to_id.write().unwrap();
                    if let Some(&mapped_fid) = name_to_id.get(&font_info.name) {
                        if mapped_fid == fid {
                            name_to_id.remove(&font_info.name);
                        }
                    }
                }

                info!(
                    "Successfully closed font: '{}' with ID {}",
                    font_info.name, fid
                );
                Ok(())
            }
            None => {
                let error_msg = format!("Font ID {} not found", fid);
                warn!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    /// Get font information by font ID
    pub fn get_font_info(&self, fid: Font) -> Option<FontInfo> {
        let fonts = self.fonts.read().unwrap();
        fonts.get(&fid).cloned()
    }

    /// Check if a font ID exists
    pub fn font_exists(&self, fid: Font) -> bool {
        let fonts = self.fonts.read().unwrap();
        fonts.contains_key(&fid)
    }

    /// Get all loaded font IDs
    pub fn get_loaded_fonts(&self) -> Vec<Font> {
        let fonts = self.fonts.read().unwrap();
        fonts.keys().cloned().collect()
    }

    /// Get number of loaded fonts
    pub fn font_count(&self) -> usize {
        let fonts = self.fonts.read().unwrap();
        fonts.len()
    }

    /// Internal method to load font information
    fn load_font_info(&self, fid: Font, name: &str) -> Result<FontInfo, String> {
        debug!("Loading font info for: '{}'", name);

        // For now, we'll return default font properties for any font name
        // In a real implementation, this would load actual font files
        let properties = match name {
            "fixed" | "cursor" | "6x10" | "6x12" | "6x13" | "6x9" | "7x13" | "7x14" | "8x13"
            | "8x16" | "9x15" => {
                // Common fixed-width fonts
                FontProperties {
                    font_ascent: 12,
                    font_descent: 3,
                    char_ascent: 12,
                    char_descent: 3,
                    max_char_width: 8,
                    min_char_width: 8,
                    char_count: 95,
                    default_char: 32,
                    draw_direction: 0,
                    min_byte1: 0,
                    max_byte1: 0,
                    all_chars_exist: true,
                }
            }
            _ => {
                // Default properties for unknown fonts
                debug!("Using default properties for unknown font: '{}'", name);
                FontProperties::default()
            }
        };

        Ok(FontInfo {
            id: fid,
            name: name.to_string(),
            properties,
        })
    }

    /// Validate the consistency of internal mappings
    #[cfg(debug_assertions)]
    pub fn validate(&self) -> bool {
        let fonts = self.fonts.read().unwrap();
        let name_to_id = self.name_to_id.read().unwrap();

        // Check that all name mappings point to existing fonts
        for (name, &fid) in name_to_id.iter() {
            if !fonts.contains_key(&fid) {
                error!(
                    "Name mapping '{}' -> {} points to non-existent font",
                    name, fid
                );
                return false;
            }
        }

        debug!("FontManager validation passed");
        true
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_manager_creation() {
        let manager = FontManager::new();
        assert_eq!(manager.font_count(), 0);
    }

    #[test]
    fn test_open_and_close_font() {
        let manager = FontManager::new();

        // Open a font
        assert!(manager.open_font(1, "fixed").is_ok());
        assert_eq!(manager.font_count(), 1);
        assert!(manager.font_exists(1));

        // Get font info
        let font_info = manager.get_font_info(1);
        assert!(font_info.is_some());
        assert_eq!(font_info.unwrap().name, "fixed");

        // Close the font
        assert!(manager.close_font(1).is_ok());
        assert_eq!(manager.font_count(), 0);
        assert!(!manager.font_exists(1));
    }

    #[test]
    fn test_font_id_reuse_error() {
        let manager = FontManager::new();

        // Open a font
        assert!(manager.open_font(1, "fixed").is_ok());

        // Try to open another font with the same ID
        assert!(manager.open_font(1, "cursor").is_err());
    }

    #[test]
    fn test_font_name_reuse() {
        let manager = FontManager::new();

        // Open a font
        assert!(manager.open_font(1, "fixed").is_ok());

        // Open the same font name with different ID (should create alias)
        assert!(manager.open_font(2, "fixed").is_ok());
        assert_eq!(manager.font_count(), 2);
    }

    #[test]
    fn test_close_nonexistent_font() {
        let manager = FontManager::new();

        // Try to close a font that doesn't exist
        assert!(manager.close_font(999).is_err());
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_font_manager_validation() {
        let manager = FontManager::new();

        // Initially should be valid
        assert!(manager.validate());

        // After opening a font should still be valid
        assert!(manager.open_font(1, "fixed").is_ok());
        assert!(manager.validate());
    }
}
