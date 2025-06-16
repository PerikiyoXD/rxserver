//! Window manager
//!
//! This module provides core window management functionality including
//! window creation, hierarchy management, and focus handling.

use crate::core::error::ServerResult;
use std::collections::HashMap;

/// Window identifier
pub type WindowId = u32;

/// Window manager for handling window operations
pub struct WindowManager {
    windows: HashMap<WindowId, Window>,
    root_window: WindowId,
    focused_window: Option<WindowId>,
    next_id: WindowId,
}

/// Window resource
#[derive(Debug, Clone)]
pub struct Window {
    pub id: WindowId,
    pub parent: Option<WindowId>,
    pub children: Vec<WindowId>,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub mapped: bool,
    pub class: WindowClass,
}

/// Window class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowClass {
    CopyFromParent,
    InputOutput,
    InputOnly,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new() -> Self {
        let mut manager = Self {
            windows: HashMap::new(),
            root_window: 1,
            focused_window: None,
            next_id: 2, // Start at 2, root is 1
        };

        // Create root window
        manager.create_root_window();
        manager
    }

    /// Create the root window
    fn create_root_window(&mut self) {
        let root = Window {
            id: self.root_window,
            parent: None,
            children: Vec::new(),
            x: 0,
            y: 0,
            width: 1920, // Default screen size
            height: 1080,
            border_width: 0,
            mapped: true,
            class: WindowClass::InputOutput,
        };

        self.windows.insert(self.root_window, root);
    }

    /// Create a new window
    pub fn create_window(
        &mut self,
        parent: WindowId,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        class: WindowClass,
    ) -> ServerResult<WindowId> {
        let window_id = self.next_id;
        self.next_id += 1;

        let window = Window {
            id: window_id,
            parent: Some(parent),
            children: Vec::new(),
            x,
            y,
            width,
            height,
            border_width,
            mapped: false,
            class,
        };

        // Add to parent's children
        if let Some(parent_window) = self.windows.get_mut(&parent) {
            parent_window.children.push(window_id);
        }

        self.windows.insert(window_id, window);
        Ok(window_id)
    }

    /// Get window by ID
    pub fn get_window(&self, window_id: WindowId) -> Option<&Window> {
        self.windows.get(&window_id)
    }

    /// Get mutable window by ID
    pub fn get_window_mut(&mut self, window_id: WindowId) -> Option<&mut Window> {
        self.windows.get_mut(&window_id)
    }

    /// Map a window (make it visible)
    pub fn map_window(&mut self, window_id: WindowId) -> ServerResult<()> {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.mapped = true;
        }
        Ok(())
    }

    /// Unmap a window (make it invisible)
    pub fn unmap_window(&mut self, window_id: WindowId) -> ServerResult<()> {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.mapped = false;
        }
        Ok(())
    }

    /// Destroy a window
    pub fn destroy_window(&mut self, window_id: WindowId) -> ServerResult<()> {
        if let Some(window) = self.windows.remove(&window_id) {
            // Remove from parent's children
            if let Some(parent_id) = window.parent {
                if let Some(parent) = self.windows.get_mut(&parent_id) {
                    parent.children.retain(|&id| id != window_id);
                }
            }

            // Destroy all children
            let children = window.children.clone();
            for child_id in children {
                self.destroy_window(child_id)?;
            }
        }
        Ok(())
    }

    /// Set focus to a window
    pub fn set_focus(&mut self, window_id: WindowId) -> ServerResult<()> {
        if self.windows.contains_key(&window_id) {
            self.focused_window = Some(window_id);
        }
        Ok(())
    }

    /// Get currently focused window
    pub fn get_focused_window(&self) -> Option<WindowId> {
        self.focused_window
    }

    /// Get root window ID
    pub fn get_root_window(&self) -> WindowId {
        self.root_window
    }

    /// Get all mapped windows
    pub fn get_mapped_windows(&self) -> Vec<&Window> {
        self.windows.values().filter(|w| w.mapped).collect()
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}
