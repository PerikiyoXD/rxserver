//! Window resource implementation
//!
//! Windows are the primary visual elements in X11, representing rectangular areas
//! on the screen that can be drawn to and can receive input events.

use crate::x11::protocol::types::{ClientId, XId};
use crate::x11::resources::{LifecycleError, Resource, ResourceType};
use std::collections::HashMap;

/// Window attributes that can be set and queried
#[derive(Debug, Clone)]
pub struct WindowAttributes {
    /// Window class (InputOutput or InputOnly)
    pub class: WindowClass,
    /// Bit gravity for window contents
    pub bit_gravity: BitGravity,
    /// Window gravity for positioning
    pub win_gravity: WinGravity,
    /// Backing store setting
    pub backing_store: BackingStore,
    /// Backing planes mask
    pub backing_planes: u32,
    /// Backing pixel value
    pub backing_pixel: u32,
    /// Override redirect flag
    pub override_redirect: bool,
    /// Save under flag
    pub save_under: bool,
    /// Event mask for this window
    pub event_mask: u32,
    /// Do not propagate mask
    pub do_not_propagate_mask: u32,
    /// Colormap XId
    pub colormap: Option<XId>,
    /// Cursor XId
    pub cursor: Option<XId>,
}

impl Default for WindowAttributes {
    fn default() -> Self {
        Self {
            class: WindowClass::InputOutput,
            bit_gravity: BitGravity::Forget,
            win_gravity: WinGravity::NorthWest,
            backing_store: BackingStore::Never,
            backing_planes: u32::MAX,
            backing_pixel: 0,
            override_redirect: false,
            save_under: false,
            event_mask: 0,
            do_not_propagate_mask: 0,
            colormap: None,
            cursor: None,
        }
    }
}

/// Window class enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowClass {
    /// Window can receive input and display output
    InputOutput,
    /// Window can only receive input (invisible)
    InputOnly,
}

/// Bit gravity enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitGravity {
    Forget,
    NorthWest,
    North,
    NorthEast,
    West,
    Center,
    East,
    SouthWest,
    South,
    SouthEast,
    Static,
}

/// Window gravity enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinGravity {
    Unmap,
    NorthWest,
    North,
    NorthEast,
    West,
    Center,
    East,
    SouthWest,
    South,
    SouthEast,
    Static,
}


impl From<u8> for BackingStore {
    fn from(value: u8) -> Self {
        match value {
            0 => BackingStore::Never,
            1 => BackingStore::WhenMapped,
            2 => BackingStore::Always,
            _ => BackingStore::Never, // Default case
        }
    }
}

impl From<BackingStore> for u8 {
    fn from(value: BackingStore) -> Self {
        match value {
            BackingStore::Never => 0,
            BackingStore::WhenMapped => 1,
            BackingStore::Always => 2,
        }
    }
}

/// Window geometry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowGeometry {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
}

impl WindowGeometry {
    /// Create new window geometry
    pub fn new(x: i16, y: i16, width: u16, height: u16, border_width: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
            border_width,
        }
    }

    /// Check if a point is inside this window
    pub fn contains_point(&self, x: i16, y: i16) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x + self.width as i16
            && y < self.y + self.height as i16
    }

    /// Get the total width including border
    pub fn total_width(&self) -> u16 {
        self.width + 2 * self.border_width
    }

    /// Get the total height including border
    pub fn total_height(&self) -> u16 {
        self.height + 2 * self.border_width
    }
}

/// Window state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    /// Window is unmapped (not visible)
    Unmapped,
    /// Window is unviewable (mapped but parent unmapped)
    Unviewable,
    /// Window is viewable (mapped and parent mapped)
    Viewable,
}

/// Window resource implementation
#[derive(Debug)]
pub struct WindowResource {
    /// Window XId
    xid: XId,
    /// Owning client
    owner: ClientId,
    /// Parent window (None for root windows)
    parent: Option<XId>,
    /// Child windows
    children: Vec<XId>,
    /// Window geometry
    geometry: WindowGeometry,
    /// Window attributes
    attributes: WindowAttributes,
    /// Window properties
    properties: HashMap<XId, Vec<u8>>, // Atom -> Property data
    /// Current window state
    state: WindowState,
    /// Visual ID for this window
    visual: u32,
    /// Window depth
    depth: u8,
}

impl WindowResource {
    /// Create a new window resource
    pub fn new(
        xid: XId,
        owner: ClientId,
        parent: Option<XId>,
        geometry: WindowGeometry,
        depth: u8,
        visual: u32,
        attributes: Option<WindowAttributes>,
    ) -> Self {
        Self {
            xid,
            owner,
            parent,
            children: Vec::new(),
            geometry,
            attributes: attributes.unwrap_or_default(),
            properties: HashMap::new(),
            state: WindowState::Unmapped,
            visual,
            depth,
        }
    }

    /// Get window geometry
    pub fn geometry(&self) -> WindowGeometry {
        self.geometry
    }

    /// Set window geometry
    pub fn set_geometry(&mut self, geometry: WindowGeometry) {
        self.geometry = geometry;
    }

    /// Get window attributes
    pub fn attributes(&self) -> &WindowAttributes {
        &self.attributes
    }

    /// Set window attributes
    pub fn set_attributes(&mut self, attributes: WindowAttributes) {
        self.attributes = attributes;
    }

    /// Get parent window
    pub fn parent(&self) -> Option<XId> {
        self.parent
    }

    /// Set parent window
    pub fn set_parent(&mut self, parent: Option<XId>) {
        self.parent = parent;
    }

    /// Get child windows
    pub fn children(&self) -> &[XId] {
        &self.children
    }

    /// Add a child window
    pub fn add_child(&mut self, child: XId) {
        if !self.children.contains(&child) {
            self.children.push(child);
        }
    }

    /// Remove a child window
    pub fn remove_child(&mut self, child: XId) {
        self.children.retain(|&c| c != child);
    }

    /// Get window state
    pub fn state(&self) -> WindowState {
        self.state
    }

    /// Set window state
    pub fn set_state(&mut self, state: WindowState) {
        self.state = state;
    }

    /// Check if window is mapped
    pub fn is_mapped(&self) -> bool {
        matches!(self.state, WindowState::Viewable | WindowState::Unviewable)
    }

    /// Get visual ID
    pub fn visual(&self) -> u32 {
        self.visual
    }

    /// Get window depth
    pub fn depth(&self) -> u8 {
        self.depth
    }

    /// Set a property on this window
    pub fn set_property(&mut self, atom: XId, data: Vec<u8>) {
        self.properties.insert(atom, data);
    }

    /// Get a property from this window
    pub fn get_property(&self, atom: XId) -> Option<&Vec<u8>> {
        self.properties.get(&atom)
    }

    /// Remove a property from this window
    pub fn remove_property(&mut self, atom: XId) -> Option<Vec<u8>> {
        self.properties.remove(&atom)
    }

    /// List all property atoms for this window
    pub fn list_properties(&self) -> Vec<XId> {
        self.properties.keys().copied().collect()
    }
}

impl Resource for WindowResource {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Window
    }

    fn xid(&self) -> XId {
        self.xid
    }

    fn owner(&self) -> ClientId {
        self.owner
    }

    fn prepare_destroy(&mut self) -> Result<(), LifecycleError> {
        // Clear all properties
        self.properties.clear();

        // Clear children list (actual child destruction handled elsewhere)
        self.children.clear();

        Ok(())
    }

    fn dependencies(&self) -> Vec<XId> {
        let mut deps = Vec::new();

        // Parent window is a dependency
        if let Some(parent) = self.parent {
            deps.push(parent);
        }

        // Colormap and cursor are dependencies if set
        if let Some(colormap) = self.attributes.colormap {
            deps.push(colormap);
        }
        if let Some(cursor) = self.attributes.cursor {
            deps.push(cursor);
        }

        deps
    }

    fn dependents(&self) -> Vec<XId> {
        // Child windows are dependents
        self.children.clone()
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
    fn test_window_creation() {
        let geometry = WindowGeometry::new(10, 20, 100, 200, 1);
        let window = WindowResource::new(123, 1, Some(456), geometry, 24, 0x21, None);

        assert_eq!(window.xid(), 123);
        assert_eq!(window.owner(), 1);
        assert_eq!(window.parent(), Some(456));
        assert_eq!(window.geometry(), geometry);
        assert_eq!(window.depth(), 24);
        assert_eq!(window.visual(), 0x21);
        assert_eq!(window.state(), WindowState::Unmapped);
        assert!(!window.is_mapped());
    }

    #[test]
    fn test_window_children() {
        let geometry = WindowGeometry::new(0, 0, 100, 100, 0);
        let mut window = WindowResource::new(123, 1, None, geometry, 24, 0x21, None);

        window.add_child(456);
        window.add_child(789);
        assert_eq!(window.children(), &[456, 789]);

        window.remove_child(456);
        assert_eq!(window.children(), &[789]);
    }

    #[test]
    fn test_window_properties() {
        let geometry = WindowGeometry::new(0, 0, 100, 100, 0);
        let mut window = WindowResource::new(123, 1, None, geometry, 24, 0x21, None);

        let data = vec![1, 2, 3, 4];
        window.set_property(999, data.clone());
        assert_eq!(window.get_property(999), Some(&data));

        let properties = window.list_properties();
        assert_eq!(properties, vec![999]);

        let removed = window.remove_property(999);
        assert_eq!(removed, Some(data));
        assert!(window.get_property(999).is_none());
    }

    #[test]
    fn test_geometry_utilities() {
        let geometry = WindowGeometry::new(10, 20, 100, 200, 5);

        assert!(geometry.contains_point(10, 20));
        assert!(geometry.contains_point(50, 100));
        assert!(geometry.contains_point(109, 219));
        assert!(!geometry.contains_point(9, 20));
        assert!(!geometry.contains_point(110, 220));

        assert_eq!(geometry.total_width(), 110);
        assert_eq!(geometry.total_height(), 210);
    }
}
