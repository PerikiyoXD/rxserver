//! Font registry implementation
//!
//! This module provides font resource management for the X11 server.
//! Fonts in X11 are server-side resources that clients can open, use for text rendering,
//! and must close when done. Each font has a unique identifier (Font ID) and is
//! associated with a client.

use crate::network::ConnectionId;
use crate::x11::protocol::types::XID;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, trace, warn};

/// Font resource representing an opened font
#[derive(Debug, Clone)]
pub struct FontResource {
    /// Font ID (XID)
    pub font_id: XID,
    /// Font name or pattern used to open the font
    pub name: String,
    /// Client that opened this font
    pub client_id: ConnectionId,
    /// Font metrics and information
    pub font_info: Arc<FontInfo>,
}

/// Font information and metrics
#[derive(Debug, Clone)]
pub struct FontInfo {
    /// Font ascent in pixels
    pub ascent: i16,
    /// Font descent in pixels  
    pub descent: i16,
    /// Maximum character width
    pub max_char_width: u16,
    /// Minimum character width
    pub min_char_width: u16,
    /// Font properties (encoding, etc.)
    pub properties: HashMap<String, String>,
}

impl FontInfo {
    /// Create new font info with basic metrics
    pub fn new(ascent: i16, descent: i16, min_width: u16, max_width: u16) -> Self {
        Self {
            ascent,
            descent,
            max_char_width: max_width,
            min_char_width: min_width,
            properties: HashMap::new(),
        }
    }

    /// Get font height (ascent + descent)
    pub fn height(&self) -> u16 {
        (self.ascent + self.descent) as u16
    }
}

/// Font registry for managing X11 font resources
#[derive(Debug)]
pub struct FontRegistry {
    /// Map from font XID to font resource
    fonts: HashMap<XID, FontResource>,
    /// Map from font name to opened font XIDs
    name_to_xids: HashMap<String, Vec<XID>>,
    /// Map from client ID to opened font XIDs
    client_fonts: HashMap<ConnectionId, Vec<XID>>,
    /// Next available XID for fonts
    next_xid: XID,
    /// Reference to the system font manager
    font_manager: Option<Arc<super::FontManager>>,
}

impl FontRegistry {
    /// Create a new font registry
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            name_to_xids: HashMap::new(),
            client_fonts: HashMap::new(),
            next_xid: 0x1000000, // Start font XIDs at a high range
            font_manager: None,
        }
    }

    /// Set the font manager reference
    pub fn set_font_manager(&mut self, manager: Arc<super::FontManager>) {
        self.font_manager = Some(manager);
    }

    /// Open a font by name for a client
    pub fn open_font(
        &mut self,
        font_id: XID,
        name: &str,
        client_id: ConnectionId,
    ) -> Result<(), FontError> {
        trace!(
            "open_font: font_id={}, name='{}', client={:?}",
            font_id,
            name,
            client_id
        );

        // Check if font ID is already in use
        if self.fonts.contains_key(&font_id) {
            warn!("Font ID {} already in use", font_id);
            return Err(FontError::FontIdInUse(font_id));
        }

        // For now, create mock font info - in a real implementation,
        // this would query the font manager for actual font metrics
        let font_info = self.get_font_info(name)?;

        let font_resource = FontResource {
            font_id,
            name: name.to_string(),
            client_id,
            font_info: Arc::new(font_info),
        };

        // Register the font
        self.fonts.insert(font_id, font_resource);

        // Update name-to-XIDs mapping
        self.name_to_xids
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(font_id);

        // Update client fonts mapping
        self.client_fonts
            .entry(client_id)
            .or_insert_with(Vec::new)
            .push(font_id);

        debug!("Opened font '{}' with ID {} for client {:?}", name, font_id, client_id);
        Ok(())
    }

    /// Close a font
    pub fn close_font(&mut self, font_id: XID) -> Result<(), FontError> {
        trace!("close_font: font_id={}", font_id);

        let font = self.fonts.remove(&font_id)
            .ok_or(FontError::FontNotFound(font_id))?;

        // Remove from name-to-XIDs mapping
        if let Some(xids) = self.name_to_xids.get_mut(&font.name) {
            xids.retain(|&xid| xid != font_id);
            if xids.is_empty() {
                self.name_to_xids.remove(&font.name);
            }
        }

        // Remove from client fonts mapping
        if let Some(xids) = self.client_fonts.get_mut(&font.client_id) {
            xids.retain(|&xid| xid != font_id);
            if xids.is_empty() {
                self.client_fonts.remove(&font.client_id);
            }
        }

        debug!("Closed font '{}' with ID {}", font.name, font_id);
        Ok(())
    }

    /// Get font resource by XID
    pub fn get_font(&self, font_id: XID) -> Option<&FontResource> {
        self.fonts.get(&font_id)
    }

    /// Get font info by XID
    pub fn get_font_info_by_id(&self, font_id: XID) -> Option<Arc<FontInfo>> {
        self.fonts.get(&font_id).map(|font| font.font_info.clone())
    }

    /// Close all fonts for a client (called when client disconnects)
    pub fn close_client_fonts(&mut self, client_id: ConnectionId) -> Vec<XID> {
        trace!("close_client_fonts: client={:?}", client_id);

        let client_font_ids = self.client_fonts.remove(&client_id).unwrap_or_default();
        let mut closed_fonts = Vec::new();

        for font_id in client_font_ids {
            if let Ok(()) = self.close_font(font_id) {
                closed_fonts.push(font_id);
            }
        }

        if !closed_fonts.is_empty() {
            debug!("Closed {} fonts for disconnected client {:?}", closed_fonts.len(), client_id);
        }

        closed_fonts
    }

    /// List fonts matching a pattern
    pub fn list_fonts(&self, pattern: &str, max_names: u16) -> Vec<String> {
        trace!("list_fonts: pattern='{}', max_names={}", pattern, max_names);

        // For now, return some common font names
        // In a real implementation, this would query the font manager
        let common_fonts = vec![
            "fixed".to_string(),
            "cursor".to_string(),
            "-*-helvetica-*-*-*-*-*-*-*-*-*-*-*-*".to_string(),
            "-*-times-*-*-*-*-*-*-*-*-*-*-*-*".to_string(),
            "-*-courier-*-*-*-*-*-*-*-*-*-*-*-*".to_string(),
        ];

        // Simple pattern matching (would be more sophisticated in real implementation)
        let mut matches: Vec<String> = if pattern == "*" {
            common_fonts
        } else {
            common_fonts
                .into_iter()
                .filter(|name| name.contains(pattern) || pattern.contains('*'))
                .collect()
        };

        // Limit results
        matches.truncate(max_names as usize);
        matches
    }

    /// Get number of opened fonts
    pub fn font_count(&self) -> usize {
        self.fonts.len()
    }

    /// Get number of fonts for a specific client
    pub fn client_font_count(&self, client_id: ConnectionId) -> usize {
        self.client_fonts.get(&client_id).map_or(0, |fonts| fonts.len())
    }

    /// Get font information by name (internal helper)
    fn get_font_info(&self, name: &str) -> Result<FontInfo, FontError> {
        // For common/standard fonts, provide known metrics
        match name {
            "fixed" | "cursor" => Ok(FontInfo::new(13, 4, 6, 6)),
            name if name.contains("helvetica") => Ok(FontInfo::new(12, 3, 4, 10)),
            name if name.contains("times") => Ok(FontInfo::new(14, 4, 3, 12)),
            name if name.contains("courier") => Ok(FontInfo::new(12, 3, 6, 6)),
            _ => {
                // Default font metrics for unknown fonts
                debug!("Using default metrics for unknown font '{}'", name);
                Ok(FontInfo::new(12, 3, 4, 8))
            }
        }
    }

    /// Generate next available font XID
    pub fn next_font_id(&mut self) -> XID {
        let id = self.next_xid;
        self.next_xid += 1;
        id
    }
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Font-related errors
#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("Font not found: {0}")]
    FontNotFound(XID),
    
    #[error("Font ID {0} already in use")]
    FontIdInUse(XID),
    
    #[error("Invalid font name: {0}")]
    InvalidFontName(String),
    
    #[error("Font loading failed: {0}")]
    LoadingFailed(String),
    
    #[error("Client {0:?} does not have permission to access font {1}")]
    PermissionDenied(ConnectionId, XID),
}

#[cfg(test)]
mod tests {
    use super::*;    #[test]
    fn test_font_registry_basic_operations() {
        let mut registry = FontRegistry::new();
        let client_id: ConnectionId = 1;
        let font_id = 0x1000001;

        // Test opening a font
        assert!(registry.open_font(font_id, "fixed", client_id).is_ok());
        assert_eq!(registry.font_count(), 1);
        assert_eq!(registry.client_font_count(client_id), 1);

        // Test getting font
        let font = registry.get_font(font_id);
        assert!(font.is_some());
        assert_eq!(font.unwrap().name, "fixed");

        // Test closing font
        assert!(registry.close_font(font_id).is_ok());
        assert_eq!(registry.font_count(), 0);
        assert_eq!(registry.client_font_count(client_id), 0);
    }

    #[test]
    fn test_font_registry_client_cleanup() {
        let mut registry = FontRegistry::new();
        let client_id: ConnectionId = 1;

        // Open multiple fonts for client
        assert!(registry.open_font(0x1000001, "fixed", client_id).is_ok());
        assert!(registry.open_font(0x1000002, "helvetica", client_id).is_ok());
        assert_eq!(registry.client_font_count(client_id), 2);

        // Close all client fonts
        let closed = registry.close_client_fonts(client_id);
        assert_eq!(closed.len(), 2);
        assert_eq!(registry.font_count(), 0);
    }

    #[test]
    fn test_font_listing() {
        let registry = FontRegistry::new();
        
        let fonts = registry.list_fonts("*", 10);
        assert!(!fonts.is_empty());
        assert!(fonts.contains(&"fixed".to_string()));
        
        let limited = registry.list_fonts("*", 2);
        assert!(limited.len() <= 2);
    }
}
