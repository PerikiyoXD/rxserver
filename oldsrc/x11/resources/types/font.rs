//! Font resource implementation
//!
//! Fonts in X11 represent typefaces used for text rendering operations.
//! They contain glyph information and metrics for drawing text.

use crate::x11::protocol::types::{ClientId, XId};
use crate::x11::resources::{LifecycleError, Resource, ResourceType};
use std::collections::HashMap;

/// Font character information
#[derive(Debug, Clone)]
pub struct CharInfo {
    /// Left side bearing
    pub left_side_bearing: i16,
    /// Right side bearing
    pub right_side_bearing: i16,
    /// Character width
    pub character_width: i16,
    /// Ascent
    pub ascent: i16,
    /// Descent
    pub descent: i16,
    /// Character attributes
    pub attributes: u16,
}

impl Default for CharInfo {
    fn default() -> Self {
        Self {
            left_side_bearing: 0,
            right_side_bearing: 0,
            character_width: 0,
            ascent: 0,
            descent: 0,
            attributes: 0,
        }
    }
}

/// Font properties
#[derive(Debug, Clone)]
pub struct FontProperty {
    /// Property name atom
    pub name: XId,
    /// Property value
    pub value: u32,
}

/// Font metrics and properties
#[derive(Debug, Clone)]
pub struct FontInfo {
    /// Font ascent
    pub font_ascent: i16,
    /// Font descent
    pub font_descent: i16,
    /// Character metrics (if per-character metrics are available)
    pub char_infos: Option<Vec<CharInfo>>,
    /// Default character info (for missing characters)
    pub default_char: CharInfo,
    /// Minimum character bounds
    pub min_bounds: CharInfo,
    /// Maximum character bounds
    pub max_bounds: CharInfo,
    /// First character code
    pub min_char_or_byte2: u16,
    /// Last character code
    pub max_char_or_byte2: u16,
    /// Default character code
    pub default_char_code: u16,
    /// Font properties
    pub properties: Vec<FontProperty>,
    /// Drawing direction (0 = left-to-right, 1 = right-to-left)
    pub draw_direction: u8,
    /// First byte of two-byte characters
    pub min_byte1: u8,
    /// Last byte of two-byte characters
    pub max_byte1: u8,
    /// All characters exist flag
    pub all_chars_exist: bool,
}

impl Default for FontInfo {
    fn default() -> Self {
        Self {
            font_ascent: 0,
            font_descent: 0,
            char_infos: None,
            default_char: CharInfo::default(),
            min_bounds: CharInfo::default(),
            max_bounds: CharInfo::default(),
            min_char_or_byte2: 0,
            max_char_or_byte2: 255,
            default_char_code: 0,
            properties: Vec::new(),
            draw_direction: 0,
            min_byte1: 0,
            max_byte1: 0,
            all_chars_exist: false,
        }
    }
}

/// Font loading states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontState {
    /// Font is being loaded
    Loading,
    /// Font is ready for use
    Ready,
    /// Font loading failed
    Failed(String),
    /// Font is being unloaded
    Unloading,
}

/// Font resource implementation
#[derive(Debug)]
pub struct FontResource {
    /// Unique identifier for this font
    xid: XId,
    /// Client that owns this font
    owner: ClientId,
    /// Font name pattern used to load this font
    name: String,
    /// Font information and metrics
    font_info: FontInfo,
    /// Glyph bitmap data
    glyph_data: HashMap<u16, Vec<u8>>,
    /// Font loading state
    state: FontState,
    /// Reference count for this font
    ref_count: u32,
    /// Cache of rendered glyphs for performance
    glyph_cache: HashMap<u16, CachedGlyph>,
}

/// Cached glyph information for performance
#[derive(Debug, Clone)]
struct CachedGlyph {
    /// Rendered bitmap
    bitmap: Vec<u8>,
    /// Glyph width
    width: u16,
    /// Glyph height
    height: u16,
    /// Glyph metrics
    metrics: CharInfo,
}

impl FontResource {
    /// Create a new font resource
    pub fn new(xid: XId, owner: ClientId, name: String) -> Result<Self, LifecycleError> {
        if name.is_empty() {
            return Err(LifecycleError::InitializationFailed(
                "Font name cannot be empty".into(),
            ));
        }

        let mut font = Self {
            xid,
            owner,
            name: name.clone(),
            font_info: FontInfo::default(),
            glyph_data: HashMap::new(),
            state: FontState::Loading,
            ref_count: 1,
            glyph_cache: HashMap::new(),
        };

        // Attempt to load the font
        font.load_font(&name)?;

        Ok(font)
    }

    /// Get the font name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get font information
    pub fn font_info(&self) -> &FontInfo {
        &self.font_info
    }

    /// Get the current font state
    pub fn state(&self) -> &FontState {
        &self.state
    }

    /// Check if the font is ready for use
    pub fn is_ready(&self) -> bool {
        matches!(self.state, FontState::Ready)
    }

    /// Get character info for a specific character
    pub fn get_char_info(&self, char_code: u16) -> CharInfo {
        // Check if we have per-character metrics
        if let Some(ref char_infos) = self.font_info.char_infos {
            if char_code >= self.font_info.min_char_or_byte2
                && char_code <= self.font_info.max_char_or_byte2
            {
                let index = (char_code - self.font_info.min_char_or_byte2) as usize;
                if index < char_infos.len() {
                    return char_infos[index].clone();
                }
            }
        }

        // Return default character info
        self.font_info.default_char.clone()
    }

    /// Get glyph bitmap data for a character
    pub fn get_glyph_data(&self, char_code: u16) -> Option<&Vec<u8>> {
        self.glyph_data.get(&char_code)
    }

    /// Get cached glyph (creates cache entry if needed)
    pub fn get_cached_glyph(&mut self, char_code: u16) -> Option<&CachedGlyph> {
        if !self.glyph_cache.contains_key(&char_code) {
            self.cache_glyph(char_code)?;
        }
        self.glyph_cache.get(&char_code)
    }

    /// Calculate text width in pixels
    pub fn text_width(&self, text: &str) -> u32 {
        let mut width = 0u32;
        for ch in text.chars() {
            let char_code = ch as u16;
            let char_info = self.get_char_info(char_code);
            width += char_info.character_width as u32;
        }
        width
    }

    /// Calculate text extents
    pub fn text_extents(&self, text: &str) -> TextExtents {
        let mut extents = TextExtents::default();

        for ch in text.chars() {
            let char_code = ch as u16;
            let char_info = self.get_char_info(char_code);

            extents.width += char_info.character_width as u32;
            extents.ascent = extents.ascent.max(char_info.ascent as u32);
            extents.descent = extents.descent.max(char_info.descent as u32);
            extents.left_bearing = extents.left_bearing.min(char_info.left_side_bearing as i32);
            extents.right_bearing = extents
                .right_bearing
                .max(char_info.right_side_bearing as i32);
        }

        extents.height = extents.ascent + extents.descent;
        extents
    }

    /// Add a reference to this font
    pub fn add_ref(&mut self) {
        self.ref_count += 1;
    }

    /// Remove a reference from this font
    pub fn remove_ref(&mut self) -> u32 {
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
        self.ref_count
    }

    /// Get the current reference count
    pub fn ref_count(&self) -> u32 {
        self.ref_count
    }

    /// Load font data from the given name pattern
    fn load_font(&mut self, name: &str) -> Result<(), LifecycleError> {
        // This is a simplified font loading implementation
        // In a real implementation, this would parse font files or query the font system

        self.state = FontState::Loading;

        // Simulate font loading based on name patterns
        if name.contains("fixed") || name.contains("courier") {
            self.load_fixed_font()?;
        } else if name.contains("helvetica") || name.contains("arial") {
            self.load_proportional_font()?;
        } else if name == "*" || name.is_empty() {
            // Load default font
            self.load_default_font()?;
        } else {
            // Try to load as a pattern
            self.load_pattern_font(name)?;
        }

        self.state = FontState::Ready;
        Ok(())
    }

    /// Load a fixed-width font
    fn load_fixed_font(&mut self) -> Result<(), LifecycleError> {
        self.font_info.font_ascent = 11;
        self.font_info.font_descent = 3;
        self.font_info.min_char_or_byte2 = 32;
        self.font_info.max_char_or_byte2 = 126;
        self.font_info.default_char_code = 32; // Space character

        // Create uniform character metrics for fixed-width font
        let char_info = CharInfo {
            left_side_bearing: 0,
            right_side_bearing: 8,
            character_width: 8,
            ascent: 11,
            descent: 3,
            attributes: 0,
        };

        self.font_info.default_char = char_info.clone();
        self.font_info.min_bounds = char_info.clone();
        self.font_info.max_bounds = char_info;

        // Generate simple glyph data for ASCII characters
        for ch in 32..=126u16 {
            self.glyph_data
                .insert(ch, self.generate_glyph_data(ch as u8));
        }

        Ok(())
    }

    /// Load a proportional font
    fn load_proportional_font(&mut self) -> Result<(), LifecycleError> {
        self.font_info.font_ascent = 12;
        self.font_info.font_descent = 4;
        self.font_info.min_char_or_byte2 = 32;
        self.font_info.max_char_or_byte2 = 126;
        self.font_info.default_char_code = 32;

        // Variable character metrics for proportional font
        self.font_info.min_bounds = CharInfo {
            left_side_bearing: 0,
            right_side_bearing: 4,
            character_width: 4,
            ascent: 12,
            descent: 4,
            attributes: 0,
        };

        self.font_info.max_bounds = CharInfo {
            left_side_bearing: 0,
            right_side_bearing: 12,
            character_width: 12,
            ascent: 12,
            descent: 4,
            attributes: 0,
        };

        self.font_info.default_char = CharInfo {
            left_side_bearing: 0,
            right_side_bearing: 6,
            character_width: 6,
            ascent: 12,
            descent: 4,
            attributes: 0,
        };

        // Generate variable-width glyph data
        for ch in 32..=126u16 {
            self.glyph_data
                .insert(ch, self.generate_glyph_data(ch as u8));
        }

        Ok(())
    }

    /// Load the default system font
    fn load_default_font(&mut self) -> Result<(), LifecycleError> {
        self.name = "fixed".to_string();
        self.load_fixed_font()
    }

    /// Load font based on pattern matching
    fn load_pattern_font(&mut self, _pattern: &str) -> Result<(), LifecycleError> {
        // For now, fall back to default font for unknown patterns
        self.load_default_font()
    }

    /// Generate simple glyph bitmap data for a character
    fn generate_glyph_data(&self, ch: u8) -> Vec<u8> {
        // This is a very simplified glyph generation
        // In a real implementation, this would load actual font glyph data
        let width = 8;
        let height = 14;
        let mut data = vec![0u8; (width * height + 7) / 8]; // Bitmap data

        // Generate a simple pattern based on the character
        if ch.is_ascii_graphic() {
            let pattern = ch as usize % 16;
            for y in 0..height {
                for x in 0..width {
                    if (x + y + pattern) % 3 == 0 {
                        let bit_index = y * width + x;
                        let byte_index = bit_index / 8;
                        let bit_offset = bit_index % 8;
                        data[byte_index] |= 1 << bit_offset;
                    }
                }
            }
        }

        data
    }

    /// Cache a glyph for faster access
    fn cache_glyph(&mut self, char_code: u16) -> Option<()> {
        let char_info = self.get_char_info(char_code);
        let bitmap = self.glyph_data.get(&char_code)?.clone();

        let cached_glyph = CachedGlyph {
            bitmap,
            width: char_info.character_width as u16,
            height: (char_info.ascent + char_info.descent) as u16,
            metrics: char_info,
        };

        self.glyph_cache.insert(char_code, cached_glyph);
        Some(())
    }
}

/// Text measurement results
#[derive(Debug, Default, Clone)]
pub struct TextExtents {
    /// Total text width
    pub width: u32,
    /// Text height
    pub height: u32,
    /// Text ascent
    pub ascent: u32,
    /// Text descent
    pub descent: u32,
    /// Left bearing
    pub left_bearing: i32,
    /// Right bearing
    pub right_bearing: i32,
}

impl Resource for FontResource {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Font
    }

    fn xid(&self) -> XId {
        self.xid
    }

    fn owner(&self) -> ClientId {
        self.owner
    }

    fn prepare_destroy(&mut self) -> Result<(), LifecycleError> {
        self.state = FontState::Unloading;

        // Clear glyph data and cache
        self.glyph_data.clear();
        self.glyph_cache.clear();

        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_creation() {
        let font = FontResource::new(100, 1, "fixed".to_string()).unwrap();
        assert_eq!(font.xid(), 100);
        assert_eq!(font.owner(), 1);
        assert_eq!(font.name(), "fixed");
        assert_eq!(font.resource_type(), ResourceType::Font);
        assert!(font.is_ready());
    }

    #[test]
    fn test_font_creation_empty_name() {
        let result = FontResource::new(100, 1, "".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_text_width_calculation() {
        let font = FontResource::new(100, 1, "fixed".to_string()).unwrap();
        let width = font.text_width("Hello");
        assert_eq!(width, 5 * 8); // 5 characters * 8 pixels each (fixed width)
    }

    #[test]
    fn test_char_info() {
        let font = FontResource::new(100, 1, "fixed".to_string()).unwrap();
        let char_info = font.get_char_info(65); // 'A'
        assert_eq!(char_info.character_width, 8);
        assert_eq!(char_info.ascent, 11);
        assert_eq!(char_info.descent, 3);
    }

    #[test]
    fn test_font_reference_counting() {
        let mut font = FontResource::new(100, 1, "fixed".to_string()).unwrap();
        assert_eq!(font.ref_count(), 1);

        font.add_ref();
        assert_eq!(font.ref_count(), 2);

        font.remove_ref();
        assert_eq!(font.ref_count(), 1);

        font.remove_ref();
        assert_eq!(font.ref_count(), 0);
    }

    #[test]
    fn test_text_extents() {
        let font = FontResource::new(100, 1, "fixed".to_string()).unwrap();
        let extents = font.text_extents("Test");
        assert_eq!(extents.width, 4 * 8); // 4 characters * 8 pixels each
        assert_eq!(extents.ascent, 11);
        assert_eq!(extents.descent, 3);
        assert_eq!(extents.height, 14); // ascent + descent
    }

    #[test]
    fn test_glyph_data() {
        let font = FontResource::new(100, 1, "fixed".to_string()).unwrap();
        let glyph_data = font.get_glyph_data(65); // 'A'
        assert!(glyph_data.is_some());
        assert!(!glyph_data.unwrap().is_empty());
    }
}
