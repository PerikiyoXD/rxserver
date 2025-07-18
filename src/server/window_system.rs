// window_system.rs
use crate::protocol::WindowId;
use crate::server::client_system::ClientId;
use std::collections::HashMap;
use tracing::debug;

/// X11 window class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum WindowClass {
    CopyFromParent = 0,
    InputOutput = 1,
    InputOnly = 2,
}

/// Internal representation of a window in the X11 server
#[derive(Debug, Clone)]
pub struct Window {
    pub id: WindowId,
    pub parent: Option<WindowId>,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub depth: u8,
    pub class: WindowClass,
    pub owner: Option<ClientId>,
}

impl Window {
    pub fn new(
        id: WindowId,
        parent: WindowId,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        depth: u8,
        class: WindowClass,
        owner: ClientId,
    ) -> Result<Self, String> {
        Ok(Self {
            id,
            parent: Some(parent),
            x,
            y,
            width,
            height,
            border_width,
            depth,
            class,
            owner: Some(owner),
        })
    }

    /// Create root window (no parent, no owner)
    pub fn new_root(id: WindowId, width: u16, height: u16, depth: u8) -> Self {
        Self {
            id,
            parent: None,
            x: 0,
            y: 0,
            width,
            height,
            border_width: 0,
            depth,
            class: WindowClass::InputOutput,
            owner: None,
        }
    }

    /// Check if this is the root window
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// Get window area as (x, y, width, height)
    pub fn bounds(&self) -> (i16, i16, u16, u16) {
        (self.x, self.y, self.width, self.height)
    }

    /// Check if a point is within this window
    pub fn contains_point(&self, x: i16, y: i16) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x + self.width as i16
            && y < self.y + self.height as i16
    }
}

/// Manages X11 window hierarchy and operations
#[derive(Debug)]
pub struct WindowSystem {
    windows: HashMap<WindowId, Window>,
    root_window_id: WindowId,
}

impl WindowSystem {
    pub fn new() -> Self {
        let root_window_id = 1;
        let mut windows = HashMap::new();

        // Create root window using the constructor
        windows.insert(
            root_window_id,
            Window::new_root(root_window_id, 1024, 768, 24),
        );

        Self {
            windows,
            root_window_id,
        }
    }

    pub fn create_window(
        &mut self,
        id: WindowId,
        parent: WindowId,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        depth: u8,
        class: WindowClass,
        owner: ClientId,
    ) -> Result<(), String> {
        if self.windows.contains_key(&id) {
            return Err(format!("Window ID {} already exists", id));
        }

        if !self.windows.contains_key(&parent) {
            return Err(format!("Parent window {} does not exist", parent));
        }

        let window = Window::new(
            id,
            parent,
            x,
            y,
            width,
            height,
            border_width,
            depth,
            class,
            owner,
        )?;
        self.windows.insert(id, window);

        debug!(
            "Created window {} ({}x{} at {},{}) for client {} with parent {}",
            id, width, height, x, y, owner, parent
        );

        Ok(())
    }

    pub fn destroy_window(&mut self, window_id: WindowId) -> Result<(), String> {
        if !self.windows.contains_key(&window_id) {
            return Err("Window does not exist".to_string());
        }

        self.windows.remove(&window_id);
        debug!("Destroyed window {}", window_id);
        Ok(())
    }

    pub fn get_window(&self, window_id: WindowId) -> Option<&Window> {
        self.windows.get(&window_id)
    }

    pub fn get_root_window(&self) -> &Window {
        &self.windows[&self.root_window_id]
    }

    pub fn window_exists(&self, window_id: WindowId) -> bool {
        self.windows.contains_key(&window_id)
    }

    /// Remove all windows owned by a client
    pub fn cleanup_client_windows(&mut self, client_id: ClientId) {
        let count = self.windows.len();
        self.windows
            .retain(|_, window| window.owner != Some(client_id));
        let removed = count - self.windows.len();
        if removed > 0 {
            debug!("Removed {} windows for client {}", removed, client_id);
        }
    }

    pub fn windows(&self) -> &HashMap<WindowId, Window> {
        &self.windows
    }

    pub fn is_window_viewable(&self, window_id: WindowId) -> bool {
        if let Some(window) = self.windows.get(&window_id) {
            // A window is viewable if it has a parent and is not obscured
            window.parent.is_some() && window.width > 0 && window.height > 0
        } else {
            false
        }
    }
}

impl Default for WindowSystem {
    fn default() -> Self {
        Self::new()
    }
}
