//! Cursor manager
//!
//! This module handles cursor loading, caching, and management
//! for the X11 server.

use crate::core::error::ServerResult;
use std::collections::HashMap;

/// Cursor identifier
pub type CursorId = u32;

/// Cursor resource
#[derive(Debug, Clone)]
pub struct Cursor {
    pub id: CursorId,
    pub name: String,
    pub width: u16,
    pub height: u16,
    pub hotspot_x: u16,
    pub hotspot_y: u16,
    pub data: Vec<u8>,
}

/// Cursor manager for handling cursor resources
pub struct CursorManager {
    cursors: HashMap<CursorId, Cursor>,
    cursor_cache: HashMap<String, CursorId>,
    next_id: CursorId,
}

impl CursorManager {
    /// Create a new cursor manager
    pub fn new() -> Self {
        let mut manager = Self {
            cursors: HashMap::new(),
            cursor_cache: HashMap::new(),
            next_id: 1,
        };

        // Load default cursors
        manager.load_default_cursors();
        manager
    }

    /// Load a cursor by name
    pub fn load_cursor(&mut self, name: &str) -> ServerResult<CursorId> {
        // Check if cursor is already loaded
        if let Some(&cursor_id) = self.cursor_cache.get(name) {
            return Ok(cursor_id);
        }

        // Create new cursor resource
        let cursor_id = self.next_id;
        self.next_id += 1;

        let cursor = Cursor {
            id: cursor_id,
            name: name.to_string(),
            width: 16,
            height: 16,
            hotspot_x: 8,
            hotspot_y: 8,
            data: vec![0; 16 * 16 / 8], // 1 bit per pixel
        };

        self.cursors.insert(cursor_id, cursor);
        self.cursor_cache.insert(name.to_string(), cursor_id);

        Ok(cursor_id)
    }

    /// Get cursor by ID
    pub fn get_cursor(&self, cursor_id: CursorId) -> Option<&Cursor> {
        self.cursors.get(&cursor_id)
    }

    /// Unload a cursor
    pub fn unload_cursor(&mut self, cursor_id: CursorId) -> ServerResult<()> {
        if let Some(cursor) = self.cursors.remove(&cursor_id) {
            self.cursor_cache.remove(&cursor.name);
        }
        Ok(())
    }

    /// Get all loaded cursors
    pub fn list_cursors(&self) -> Vec<&Cursor> {
        self.cursors.values().collect()
    }

    /// Check if cursor is loaded
    pub fn is_cursor_loaded(&self, cursor_id: CursorId) -> bool {
        self.cursors.contains_key(&cursor_id)
    }

    /// Load default cursors
    fn load_default_cursors(&mut self) {
        let default_cursors = [
            "arrow",
            "cross",
            "hand",
            "ibeam",
            "wait",
            "resize_ns",
            "resize_ew",
            "resize_nwse",
            "resize_nesw",
        ];

        for &cursor_name in &default_cursors {
            let _ = self.load_cursor(cursor_name);
        }
    }
}

impl Default for CursorManager {
    fn default() -> Self {
        Self::new()
    }
}
