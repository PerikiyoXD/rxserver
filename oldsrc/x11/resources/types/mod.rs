//! Resource type implementations
//!
//! This module provides concrete implementations for different X11 resource types.
//! Each resource type has its own module with specific functionality.

pub mod atom;
pub mod colormap;
pub mod cursor;
pub mod font;
pub mod graphics_context;
pub mod pixmap;
pub mod window;

// Re-export all resource types for convenience
pub use atom::AtomResource;
pub use colormap::ColormapResource;
pub use cursor::CursorResource;
pub use font::FontResource;
pub use graphics_context::GraphicsContextResource;
pub use pixmap::PixmapResource;
pub use window::WindowResource;

use crate::x11::resources::ResourceType;

/// Resource type utilities and extensions
pub trait ResourceTypeExt {
    /// Check if this resource type supports client ownership
    fn has_client_ownership(&self) -> bool;

    /// Get the display name for this resource type
    fn display_name(&self) -> &'static str;

    /// Check if this resource type can be shared between clients
    fn is_shareable(&self) -> bool;

    /// Get the priority for cleanup ordering (lower numbers = higher priority)
    fn cleanup_priority(&self) -> u8;

    /// Check if this resource type requires dependency tracking
    fn requires_dependency_tracking(&self) -> bool;
}

impl ResourceTypeExt for ResourceType {
    fn has_client_ownership(&self) -> bool {
        match self {
            ResourceType::Window
            | ResourceType::Pixmap
            | ResourceType::GraphicsContext
            | ResourceType::Font
            | ResourceType::Cursor
            | ResourceType::Colormap => true,
            ResourceType::Atom => false, // Atoms are global
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            ResourceType::Window => "Window",
            ResourceType::Pixmap => "Pixmap",
            ResourceType::GraphicsContext => "Graphics Context",
            ResourceType::Font => "Font",
            ResourceType::Cursor => "Cursor",
            ResourceType::Colormap => "Colormap",
            ResourceType::Atom => "Atom",
        }
    }

    fn is_shareable(&self) -> bool {
        match self {
            ResourceType::Font | ResourceType::Colormap | ResourceType::Atom => true,
            ResourceType::Window
            | ResourceType::Pixmap
            | ResourceType::GraphicsContext
            | ResourceType::Cursor => false,
        }
    }

    fn cleanup_priority(&self) -> u8 {
        match self {
            ResourceType::Window => 1,          // Windows first (high priority)
            ResourceType::GraphicsContext => 2, // GCs next
            ResourceType::Pixmap => 3,          // Pixmaps next
            ResourceType::Cursor => 4,          // Cursors next
            ResourceType::Font => 5,            // Fonts next
            ResourceType::Colormap => 6,        // Colormaps next
            ResourceType::Atom => 7,            // Atoms last (global, low priority)
        }
    }

    fn requires_dependency_tracking(&self) -> bool {
        match self {
            ResourceType::Window
            | ResourceType::GraphicsContext
            | ResourceType::Pixmap
            | ResourceType::Cursor => true,
            ResourceType::Font | ResourceType::Colormap | ResourceType::Atom => false,
        }
    }
}

/// Statistics about resource types
#[derive(Debug, Default, Clone)]
pub struct ResourceTypeStats {
    pub window_count: usize,
    pub pixmap_count: usize,
    pub graphics_context_count: usize,
    pub font_count: usize,
    pub colormap_count: usize,
    pub cursor_count: usize,
    pub atom_count: usize,
}

impl ResourceTypeStats {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Get total resource count across all types
    pub fn total_count(&self) -> usize {
        self.window_count
            + self.pixmap_count
            + self.graphics_context_count
            + self.font_count
            + self.colormap_count
            + self.cursor_count
            + self.atom_count
    }

    /// Increment count for the given resource type
    pub fn increment(&mut self, resource_type: ResourceType) {
        match resource_type {
            ResourceType::Window => self.window_count += 1,
            ResourceType::Pixmap => self.pixmap_count += 1,
            ResourceType::GraphicsContext => self.graphics_context_count += 1,
            ResourceType::Font => self.font_count += 1,
            ResourceType::Colormap => self.colormap_count += 1,
            ResourceType::Cursor => self.cursor_count += 1,
            ResourceType::Atom => self.atom_count += 1,
        }
    }

    /// Decrement count for the given resource type
    pub fn decrement(&mut self, resource_type: ResourceType) {
        match resource_type {
            ResourceType::Window => self.window_count = self.window_count.saturating_sub(1),
            ResourceType::Pixmap => self.pixmap_count = self.pixmap_count.saturating_sub(1),
            ResourceType::GraphicsContext => {
                self.graphics_context_count = self.graphics_context_count.saturating_sub(1)
            }
            ResourceType::Font => self.font_count = self.font_count.saturating_sub(1),
            ResourceType::Colormap => self.colormap_count = self.colormap_count.saturating_sub(1),
            ResourceType::Cursor => self.cursor_count = self.cursor_count.saturating_sub(1),
            ResourceType::Atom => self.atom_count = self.atom_count.saturating_sub(1),
        }
    }
}
