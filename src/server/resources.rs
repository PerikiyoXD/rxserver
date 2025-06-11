//! Resource management
//!
//! This module manages X11 resources like windows, pixmaps, graphics contexts, etc.

use crate::protocol::types::*;
use crate::{todo_high, todo_medium, Result};
use std::collections::HashMap;

/// Manages X11 resources
pub struct ResourceManager {
    /// Next available resource ID
    next_id: ResourceId,
    /// Window resources
    windows: HashMap<Window, WindowResource>,
    /// Pixmap resources
    pixmaps: HashMap<Pixmap, PixmapResource>,
    /// Graphics context resources
    graphics_contexts: HashMap<GContext, GraphicsContextResource>,
    /// Font resources
    fonts: HashMap<Font, FontResource>,
    /// Cursor resources
    cursors: HashMap<Cursor, CursorResource>,
    /// Colormap resources
    colormaps: HashMap<Colormap, ColormapResource>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        Self {
            next_id: 2, // Start at 2, root window is 1
            windows: HashMap::new(),
            pixmaps: HashMap::new(),
            graphics_contexts: HashMap::new(),
            fonts: HashMap::new(),
            cursors: HashMap::new(),
            colormaps: HashMap::new(),
        }
    }

    /// Allocate a new resource ID
    pub fn allocate_id(&mut self) -> ResourceId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Create a new window resource
    pub fn create_window(
        &mut self,
        parent: Window,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        class: WindowClass,
    ) -> Result<Window> {
        todo_high!(
            "resource_manager",
            "Window creation incomplete - missing visual, attributes, and validation"
        );

        let id = self.allocate_id();

        let window = WindowResource {
            id,
            parent,
            geometry: Rectangle {
                x,
                y,
                width,
                height,
            },
            border_width,
            class,
            mapped: false,
            children: Vec::new(),
            event_mask: EventMask::empty(),
        };

        // Add to parent's children list
        if let Some(parent_window) = self.windows.get_mut(&parent) {
            parent_window.children.push(id);
        }

        self.windows.insert(id, window);
        log::debug!(
            "Created window {} ({}x{} at {},{}) parent={}",
            id,
            width,
            height,
            x,
            y,
            parent
        );

        Ok(id)
    }

    /// Destroy a window resource
    pub fn destroy_window(&mut self, window: Window) -> Result<()> {
        if let Some(win) = self.windows.remove(&window) {
            // Remove from parent's children list
            if let Some(parent_window) = self.windows.get_mut(&win.parent) {
                parent_window.children.retain(|&child| child != window);
            }

            // Destroy all children
            let children = win.children.clone();
            for child in children {
                self.destroy_window(child)?;
            }

            log::debug!("Destroyed window {}", window);
        }

        Ok(())
    }

    /// Get a window resource
    pub fn get_window(&self, window: Window) -> Option<&WindowResource> {
        self.windows.get(&window)
    }

    /// Get a mutable window resource
    pub fn get_window_mut(&mut self, window: Window) -> Option<&mut WindowResource> {
        self.windows.get_mut(&window)
    }

    /// Map a window (make it visible)
    pub fn map_window(&mut self, window: Window) -> Result<()> {
        if let Some(win) = self.windows.get_mut(&window) {
            win.mapped = true;
            log::debug!("Mapped window {}", window);
        }
        Ok(())
    }

    /// Unmap a window (make it invisible)
    pub fn unmap_window(&mut self, window: Window) -> Result<()> {
        if let Some(win) = self.windows.get_mut(&window) {
            win.mapped = false;
            log::debug!("Unmapped window {}", window);
        }
        Ok(())
    }

    /// Get the window count
    pub async fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Get the pixmap count
    pub async fn pixmap_count(&self) -> usize {
        self.pixmaps.len()
    }

    /// Create a graphics context
    pub fn create_graphics_context(&mut self) -> Result<GContext> {
        todo_medium!(
            "resource_manager",
            "Graphics context creation incomplete - missing full GC attributes"
        );

        let id = self.allocate_id();

        let gc = GraphicsContextResource {
            id,
            foreground: 0,
            background: 1,
            line_width: 0,
            line_style: 0,
            cap_style: 0,
            join_style: 0,
        };

        self.graphics_contexts.insert(id, gc);
        log::debug!("Created graphics context {}", id);

        Ok(id)
    }

    /// Get all mapped windows
    pub fn get_mapped_windows(&self) -> Vec<&WindowResource> {
        self.windows.values().filter(|w| w.mapped).collect()
    }

    /// Check if a resource ID exists
    pub fn resource_exists(&self, id: ResourceId) -> bool {
        self.windows.contains_key(&id)
            || self.pixmaps.contains_key(&id)
            || self.graphics_contexts.contains_key(&id)
            || self.fonts.contains_key(&id)
            || self.cursors.contains_key(&id)
            || self.colormaps.contains_key(&id)
    }

    /// Get the memory usage
    pub async fn memory_usage(&self) -> usize {
        // Calculate a more accurate memory usage, including heap allocations
        let window_memory: usize = self
            .windows
            .values()
            .map(|w| {
                std::mem::size_of_val(w) + w.children.capacity() * std::mem::size_of::<Window>()
            })
            .sum();

        let pixmap_memory: usize = self
            .pixmaps
            .values()
            .map(|p| std::mem::size_of_val(p) + p.data.capacity() * std::mem::size_of::<u8>())
            .sum();

        let gc_memory: usize = self
            .graphics_contexts
            .values()
            .map(|gc| std::mem::size_of_val(gc))
            .sum();

        let font_memory: usize = self
            .fonts
            .values()
            .map(|f| std::mem::size_of_val(f) + f.name.capacity() * std::mem::size_of::<char>())
            .sum();

        let cursor_memory: usize = self
            .cursors
            .values()
            .map(|c| std::mem::size_of_val(c))
            .sum();

        let colormap_memory: usize = self
            .colormaps
            .values()
            .map(|cm| {
                std::mem::size_of_val(cm)
                    + cm.colors.capacity()
                        * (std::mem::size_of::<u32>() + std::mem::size_of::<Color>())
            })
            .sum();

        window_memory + pixmap_memory + gc_memory + font_memory + cursor_memory + colormap_memory
    }
}

/// Window resource data
#[derive(Debug, Clone)]
pub struct WindowResource {
    pub id: Window,
    pub parent: Window,
    pub geometry: Rectangle,
    pub border_width: u16,
    pub class: WindowClass,
    pub mapped: bool,
    pub children: Vec<Window>,
    pub event_mask: EventMask,
}

/// Pixmap resource data
#[derive(Debug, Clone)]
pub struct PixmapResource {
    pub id: Pixmap,
    pub width: u16,
    pub height: u16,
    pub depth: u8,
    pub data: Vec<u8>,
}

/// Graphics context resource data
#[derive(Debug, Clone)]
pub struct GraphicsContextResource {
    pub id: GContext,
    pub foreground: u32,
    pub background: u32,
    pub line_width: u16,
    pub line_style: u8,
    pub cap_style: u8,
    pub join_style: u8,
}

/// Font resource data
#[derive(Debug, Clone)]
pub struct FontResource {
    pub id: Font,
    pub name: String,
    pub height: u16,
    pub max_width: u16,
}

/// Cursor resource data
#[derive(Debug, Clone)]
pub struct CursorResource {
    pub id: Cursor,
    pub width: u16,
    pub height: u16,
    pub hotspot_x: u16,
    pub hotspot_y: u16,
    pub foreground: Color,
    pub background: Color,
}

/// Colormap resource data
#[derive(Debug, Clone)]
pub struct ColormapResource {
    pub id: Colormap,
    pub visual: u32,
    pub colors: HashMap<u32, Color>,
}
