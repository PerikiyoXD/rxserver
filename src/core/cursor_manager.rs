//! Cursor Management System for X11 Server
//!
//! This module provides a cursor loading, storage, and management system that handles
//! cursor resources, maintains the mapping between cursor IDs and cursor data, and provides
//! thread-safe access to cursor information.

use crate::protocol::types::{Cursor, Font};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info, warn};

/// Cursor glyph information
#[derive(Debug, Clone)]
pub struct CursorGlyph {
    /// Font containing the glyph
    pub font: Font,
    /// Character/glyph index in the font
    pub character: u16,
}

/// Cursor color information
#[derive(Debug, Clone)]
pub struct CursorColor {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

/// Cursor information and properties
#[derive(Debug, Clone)]
pub struct CursorInfo {
    /// Cursor ID
    pub id: Cursor,
    /// Cursor type
    pub cursor_type: CursorType,
    /// Cursor width (if known)
    pub width: u16,
    /// Cursor height (if known)
    pub height: u16,
    /// Hotspot X coordinate
    pub hotspot_x: u16,
    /// Hotspot Y coordinate
    pub hotspot_y: u16,
}

/// Types of cursors supported
#[derive(Debug, Clone)]
pub enum CursorType {
    /// Cursor created from font glyphs
    Glyph {
        /// Source glyph (the cursor shape)
        source: CursorGlyph,
        /// Mask glyph (optional, for transparency)
        mask: Option<CursorGlyph>,
        /// Foreground color
        foreground: CursorColor,
        /// Background color
        background: CursorColor,
    },
    /// Cursor created from pixmap (not implemented yet)
    Pixmap {
        /// Source pixmap ID
        source: u32,
        /// Mask pixmap ID (optional)
        mask: Option<u32>,
        /// Foreground color
        foreground: CursorColor,
        /// Background color
        background: CursorColor,
    },
}

/// Thread-safe cursor manager for the X11 server
#[derive(Debug)]
pub struct CursorManager {
    /// Map from cursor ID to cursor information
    cursors: Arc<RwLock<HashMap<Cursor, CursorInfo>>>,
}

impl CursorManager {
    /// Create a new cursor manager
    pub fn new() -> Self {
        info!("Initializing CursorManager");

        let manager = CursorManager {
            cursors: Arc::new(RwLock::new(HashMap::new())),
        };

        info!("CursorManager initialized");
        manager
    }

    /// Create a glyph cursor with the given cursor ID
    pub fn create_glyph_cursor(
        &self,
        cid: Cursor,
        source_font: Font,
        mask_font: Font,
        source_char: u16,
        mask_char: u16,
        fore_red: u16,
        fore_green: u16,
        fore_blue: u16,
        back_red: u16,
        back_green: u16,
        back_blue: u16,
    ) -> Result<(), String> {
        debug!("Creating glyph cursor: cid={}, source_font={}, mask_font={}, source_char={}, mask_char={}", 
               cid, source_font, mask_font, source_char, mask_char);

        // Check if cursor ID is already in use
        {
            let cursors = self.cursors.read().unwrap();
            if cursors.contains_key(&cid) {
                let error_msg = format!("Cursor ID {} already in use", cid);
                warn!("{}", error_msg);
                return Err(error_msg);
            }
        }

        // Create the cursor glyph information
        let source_glyph = CursorGlyph {
            font: source_font,
            character: source_char,
        };

        // Create mask glyph if mask font is not None (0)
        let mask_glyph = if mask_font != 0 {
            Some(CursorGlyph {
                font: mask_font,
                character: mask_char,
            })
        } else {
            None
        };

        let foreground = CursorColor {
            red: fore_red,
            green: fore_green,
            blue: fore_blue,
        };

        let background = CursorColor {
            red: back_red,
            green: back_green,
            blue: back_blue,
        };

        let cursor_info = CursorInfo {
            id: cid,
            cursor_type: CursorType::Glyph {
                source: source_glyph,
                mask: mask_glyph,
                foreground,
                background,
            },
            width: 16, // Default cursor size
            height: 16,
            hotspot_x: 8, // Default hotspot (center)
            hotspot_y: 8,
        };

        // Add to cursor map
        {
            let mut cursors = self.cursors.write().unwrap();
            cursors.insert(cid, cursor_info);
        }

        info!("Successfully created glyph cursor: cid={}", cid);
        Ok(())
    }

    /// Free/delete a cursor by cursor ID
    pub fn free_cursor(&self, cid: Cursor) -> Result<(), String> {
        debug!("Freeing cursor: cid={}", cid);

        let removed_cursor = {
            let mut cursors = self.cursors.write().unwrap();
            cursors.remove(&cid)
        };

        match removed_cursor {
            Some(_cursor_info) => {
                info!("Successfully freed cursor: cid={}", cid);
                Ok(())
            }
            None => {
                let error_msg = format!("Cursor ID {} not found", cid);
                warn!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    /// Get cursor information by cursor ID
    pub fn get_cursor_info(&self, cid: Cursor) -> Option<CursorInfo> {
        let cursors = self.cursors.read().unwrap();
        cursors.get(&cid).cloned()
    }

    /// Check if a cursor ID exists
    pub fn cursor_exists(&self, cid: Cursor) -> bool {
        let cursors = self.cursors.read().unwrap();
        cursors.contains_key(&cid)
    }

    /// Get all loaded cursor IDs
    pub fn get_loaded_cursors(&self) -> Vec<Cursor> {
        let cursors = self.cursors.read().unwrap();
        cursors.keys().cloned().collect()
    }

    /// Get number of loaded cursors
    pub fn cursor_count(&self) -> usize {
        let cursors = self.cursors.read().unwrap();
        cursors.len()
    }

    /// Validate the consistency of internal mappings
    #[cfg(debug_assertions)]
    pub fn validate(&self) -> bool {
        let cursors = self.cursors.read().unwrap();

        // Check that all cursors have valid IDs
        for (&cid, cursor_info) in cursors.iter() {
            if cursor_info.id != cid {
                error!(
                    "Cursor ID mismatch: map key {} != cursor.id {}",
                    cid, cursor_info.id
                );
                return false;
            }
        }

        debug!("CursorManager validation passed");
        true
    }
}

impl Default for CursorManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_manager_creation() {
        let manager = CursorManager::new();
        assert_eq!(manager.cursor_count(), 0);
    }

    #[test]
    fn test_create_and_free_glyph_cursor() {
        let manager = CursorManager::new();

        // Create a glyph cursor
        assert!(manager
            .create_glyph_cursor(
                1, 100, 101, 64, 65, 65535, 0, 0, // red foreground
                0, 65535, 0 // green background
            )
            .is_ok());

        assert_eq!(manager.cursor_count(), 1);
        assert!(manager.cursor_exists(1));

        // Get cursor info
        let cursor_info = manager.get_cursor_info(1);
        assert!(cursor_info.is_some());
        assert_eq!(cursor_info.unwrap().id, 1);

        // Free the cursor
        assert!(manager.free_cursor(1).is_ok());
        assert_eq!(manager.cursor_count(), 0);
        assert!(!manager.cursor_exists(1));
    }

    #[test]
    fn test_cursor_id_reuse_error() {
        let manager = CursorManager::new();

        // Create a cursor
        assert!(manager
            .create_glyph_cursor(1, 100, 101, 64, 65, 65535, 0, 0, 0, 65535, 0)
            .is_ok());

        // Try to create another cursor with the same ID
        assert!(manager
            .create_glyph_cursor(1, 102, 103, 66, 67, 0, 0, 65535, 65535, 0, 0)
            .is_err());
    }

    #[test]
    fn test_free_nonexistent_cursor() {
        let manager = CursorManager::new();

        // Try to free a cursor that doesn't exist
        assert!(manager.free_cursor(999).is_err());
    }

    #[test]
    fn test_glyph_cursor_with_no_mask() {
        let manager = CursorManager::new();

        // Create a glyph cursor with no mask (mask_font = 0)
        assert!(manager
            .create_glyph_cursor(1, 100, 0, 64, 0, 65535, 0, 0, 0, 65535, 0)
            .is_ok());

        let cursor_info = manager.get_cursor_info(1).unwrap();
        match cursor_info.cursor_type {
            CursorType::Glyph { mask, .. } => {
                assert!(mask.is_none());
            }
            _ => panic!("Expected glyph cursor type"),
        }
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_cursor_manager_validation() {
        let manager = CursorManager::new();

        // Initially should be valid
        assert!(manager.validate());

        // After creating a cursor should still be valid
        assert!(manager
            .create_glyph_cursor(1, 100, 101, 64, 65, 65535, 0, 0, 0, 65535, 0)
            .is_ok());
        assert!(manager.validate());
    }
}
