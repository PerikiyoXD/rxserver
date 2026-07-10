// window_system.rs
use crate::protocol::{WindowId, constants};
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
    pub pixel_data: Vec<u32>, // 0xAARRGGBB pixel data
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
        let pixel_count = (width as usize) * (height as usize);
        let pixel_data = vec![0xFF000000; pixel_count]; // Opaque black background

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
            pixel_data,
        })
    }

    /// Create root window (no parent, no owner)
    pub fn new_root(id: WindowId, width: u16, height: u16, depth: u8) -> Self {
        let pixel_count = (width as usize) * (height as usize);
        let pixel_data = vec![0xFF2E3440; pixel_count]; // Opaque dark blue-gray background

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
            pixel_data,
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

    /// Get pixel data
    pub fn pixel_data(&self) -> &[u32] {
        &self.pixel_data
    }

    /// Get mutable pixel data
    pub fn pixel_data_mut(&mut self) -> &mut [u32] {
        &mut self.pixel_data
    }

    /// Set a pixel at the given coordinates
    pub fn set_pixel(&mut self, x: u16, y: u16, color: u32) {
        if x < self.width && y < self.height {
            let index = (y as usize * self.width as usize) + x as usize;
            if index < self.pixel_data.len() {
                self.pixel_data[index] = color;
            }
        }
    }

    /// Get a pixel at the given coordinates
    pub fn get_pixel(&self, x: u16, y: u16) -> Option<u32> {
        if x < self.width && y < self.height {
            let index = (y as usize * self.width as usize) + x as usize;
            self.pixel_data.get(index).copied()
        } else {
            None
        }
    }
}

/// Manages X11 window hierarchy and operations
#[derive(Debug)]
pub struct WindowSystem {
    window_map: HashMap<WindowId, Window>,
}

impl WindowSystem {
    pub fn new() -> Self {
        let mut window_map = HashMap::new();

        // Create root window using the constructor
        window_map.insert(
            constants::WINDOW_ID_ROOT,
            Window::new_root(constants::WINDOW_ID_ROOT, 1024, 768, 24),
        );

        Self { window_map }
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
        if self.window_map.contains_key(&id) {
            return Err(format!("Window ID {} already exists", id));
        }

        if !self.window_map.contains_key(&parent) {
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
        self.window_map.insert(id, window);

        debug!(
            "Created window {} ({}x{} at {},{}) for client {} with parent {}",
            id, width, height, x, y, owner, parent
        );

        Ok(())
    }

    pub fn destroy_window(&mut self, window_id: WindowId) -> Result<(), String> {
        if !self.window_map.contains_key(&window_id) {
            return Err("Window does not exist".to_string());
        }

        self.window_map.remove(&window_id);
        debug!("Destroyed window {}", window_id);
        Ok(())
    }

    pub fn get_window(&self, window_id: WindowId) -> Option<&Window> {
        self.window_map.get(&window_id)
    }

    pub fn get_window_mut(&mut self, window_id: WindowId) -> Option<&mut Window> {
        self.window_map.get_mut(&window_id)
    }

    pub fn get_root_window(&self) -> &Window {
        &self.window_map[&constants::WINDOW_ID_ROOT]
    }

    pub fn window_exists(&self, window_id: WindowId) -> bool {
        self.window_map.contains_key(&window_id)
    }

    /// Remove all windows owned by a client
    pub fn cleanup_client_windows(&mut self, client_id: ClientId) {
        let count = self.window_map.len();
        self.window_map
            .retain(|_, window| window.owner != Some(client_id));
        let removed = count - self.window_map.len();
        if removed > 0 {
            debug!("Removed {} windows for client {}", removed, client_id);
        }
    }

    pub fn windows(&self) -> &HashMap<WindowId, Window> {
        &self.window_map
    }

    pub fn is_window_viewable(&self, window_id: WindowId) -> bool {
        if let Some(window) = self.window_map.get(&window_id) {
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
