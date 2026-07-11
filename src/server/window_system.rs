// window_system.rs
use crate::protocol::{Atom, PixmapId, WindowId, constants};
use crate::server::client_system::ClientId;
use std::collections::HashMap;
use tracing::debug;

/// A window's `background-pixel`/`background-pixmap` attribute, as set via
/// CreateWindow/ChangeWindowAttributes's value-list (xproto encoding.xml,
/// CreateWindow VALUEs). Drives ClearArea and window-clearing exposure -
/// `None` means "background None" (contents left unchanged when cleared),
/// distinct from this attribute never having been set (which defaults to
/// `None` too, per the spec's CreateWindow defaults).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Background {
    /// Spec's background `None`: clearing the window leaves pixels as-is.
    None,
    /// Spec's background `ParentRelative`: inherits the parent's background.
    /// Resolved by walking up the window tree (see
    /// `WindowSystem::resolve_background`) - never faked as `None`.
    ParentRelative,
    Pixel(u32),
    Pixmap(PixmapId),
}

/// A window's `border-pixel`/`border-pixmap` attribute, as set via
/// CreateWindow/ChangeWindowAttributes's value-list. Same shape as
/// `Background` minus `ParentRelative` (xproto has no CopyFromParent/
/// ParentRelative case for borders - only None/Pixel/Pixmap), default is
/// `CopyFromParent` per the spec (a border pixel this server has no value
/// for yet, rendered the same as `None` until border rendering exists).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Border {
    CopyFromParent,
    Pixel(u32),
    Pixmap(PixmapId),
}

/// X11 window class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum WindowClass {
    CopyFromParent = 0,
    InputOutput = 1,
    InputOnly = 2,
}

/// A window property as set via ChangeProperty/read via GetProperty.
///
/// `data` holds the raw property value; `format` (8, 16, or 32) is the bit
/// width of each element, needed to reply with the right unit count.
#[derive(Debug, Clone)]
pub struct Property {
    pub r#type: Atom,
    pub format: u8,
    pub data: Vec<u8>,
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
    pub properties: HashMap<Atom, Property>,
    pub background: Background,
    /// Bounding-shape mask set via SHAPE's ShapeMask request, if any -
    /// the pixmap defining the window's non-rectangular outline.
    pub bounding_shape: Option<PixmapId>,
    /// Event mask set via CreateWindow/ChangeWindowAttributes's EVENT_MASK
    /// value (`protocol::types::value_mask::EVENT_MASK`), OR'd with
    /// `protocol::events::event_mask` bit constants. Gates which input
    /// events (KeyPress, ButtonPress, PointerMotion, ...) this window's
    /// owner actually receives - 0 means "selected for nothing."
    pub event_mask: u32,
    /// The remaining CW_* attributes from CreateWindow/
    /// ChangeWindowAttributes's VALUEs table. Stored so QueryTree/
    /// GetWindowAttributes-style requests can round-trip what a client set,
    /// even though most of these don't have rendering/dispatch behavior
    /// wired up yet (bit_gravity/win_gravity affect resize repainting,
    /// backing_store affects obscured-window contents, save_under affects
    /// pop-up rendering, do_not_propagate_mask affects event propagation to
    /// ancestors, colormap/cursor affect palette/pointer-shape - none of
    /// that machinery exists yet).
    pub border: Border,
    pub bit_gravity: u8,
    pub win_gravity: u8,
    pub backing_store: u8,
    pub backing_planes: u32,
    pub backing_pixel: u32,
    pub override_redirect: bool,
    pub save_under: bool,
    pub do_not_propagate_mask: u32,
    pub colormap: Option<u32>,
    pub cursor: Option<u32>,
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
        background: Background,
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
            properties: HashMap::new(),
            background,
            bounding_shape: None,
            event_mask: 0,
            border: Border::CopyFromParent,
            bit_gravity: 0,       // ForgetGravity, xproto default
            win_gravity: 1,       // NorthWestGravity, xproto default
            backing_store: 0,     // NotUseful, xproto default
            backing_planes: 0xFFFFFFFF,
            backing_pixel: 0,
            override_redirect: false,
            save_under: false,
            do_not_propagate_mask: 0,
            colormap: None,
            cursor: None,
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
            properties: HashMap::new(),
            background: Background::Pixel(0xFF2E3440),
            bounding_shape: None,
            event_mask: 0,
            border: Border::CopyFromParent,
            bit_gravity: 0,
            win_gravity: 1,
            backing_store: 0,
            backing_planes: 0xFFFFFFFF,
            backing_pixel: 0,
            override_redirect: false,
            save_under: false,
            do_not_propagate_mask: 0,
            colormap: None,
            cursor: None,
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

    /// Set (replace/prepend/append) a property, per ChangeProperty's `mode`.
    /// `mode`: 0 = Replace, 1 = Prepend, 2 = Append.
    pub fn change_property(
        &mut self,
        property: Atom,
        r#type: Atom,
        format: u8,
        mode: u8,
        mut data: Vec<u8>,
    ) {
        match mode {
            1 => {
                // Prepend: new data goes before existing data of the same type/format
                if let Some(existing) = self.properties.get(&property) {
                    if existing.r#type == r#type && existing.format == format {
                        data.extend_from_slice(&existing.data);
                    }
                }
                self.properties.insert(
                    property,
                    Property {
                        r#type,
                        format,
                        data,
                    },
                );
            }
            2 => {
                // Append: new data goes after existing data of the same type/format
                let mut combined = if let Some(existing) = self.properties.get(&property) {
                    if existing.r#type == r#type && existing.format == format {
                        existing.data.clone()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };
                combined.append(&mut data);
                self.properties.insert(
                    property,
                    Property {
                        r#type,
                        format,
                        data: combined,
                    },
                );
            }
            _ => {
                // Replace (mode 0, and the default for anything unrecognized)
                self.properties.insert(
                    property,
                    Property {
                        r#type,
                        format,
                        data,
                    },
                );
            }
        }
    }

    /// Get a property's current value, if set.
    pub fn get_property(&self, property: Atom) -> Option<&Property> {
        self.properties.get(&property)
    }

    /// Delete a property. Returns true if it existed.
    pub fn delete_property(&mut self, property: Atom) -> bool {
        self.properties.remove(&property).is_some()
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
        background: Background,
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
            background,
        )?;
        self.window_map.insert(id, window);

        debug!(
            "Created window {} ({}x{} at {},{}) for client {} with parent {}",
            id, width, height, x, y, owner, parent
        );

        Ok(())
    }

    /// Resolve a window's effective background, following `ParentRelative`
    /// up the window tree until a concrete `Pixel`/`Pixmap`/`None` is found
    /// (or the root is reached). Per xproto's ClearArea semantics, a chain
    /// of `ParentRelative` all the way to a window with background `None`
    /// means "leave contents unchanged", so that's a real, distinct result -
    /// not an error.
    pub fn resolve_background(&self, window_id: WindowId) -> Option<Background> {
        let mut current = self.window_map.get(&window_id)?;
        loop {
            match current.background {
                Background::ParentRelative => {
                    let parent_id = current.parent?;
                    current = self.window_map.get(&parent_id)?;
                }
                resolved => return Some(resolved),
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn window() -> Window {
        Window::new(
            2,
            1,
            0,
            0,
            100,
            100,
            0,
            24,
            WindowClass::InputOutput,
            1,
            Background::None,
        )
        .unwrap()
    }

    #[test]
    fn change_property_replace_sets_value() {
        let mut w = window();
        w.change_property(39, 31, 8, 0, b"hello".to_vec()); // WM_NAME/STRING
        let p = w.get_property(39).unwrap();
        assert_eq!(p.data, b"hello");
        assert_eq!(p.r#type, 31);
        assert_eq!(p.format, 8);
    }

    #[test]
    fn change_property_replace_overwrites() {
        let mut w = window();
        w.change_property(39, 31, 8, 0, b"first".to_vec());
        w.change_property(39, 31, 8, 0, b"second".to_vec());
        assert_eq!(w.get_property(39).unwrap().data, b"second");
    }

    #[test]
    fn change_property_append() {
        let mut w = window();
        w.change_property(39, 31, 8, 0, b"foo".to_vec()); // Replace
        w.change_property(39, 31, 8, 2, b"bar".to_vec()); // Append
        assert_eq!(w.get_property(39).unwrap().data, b"foobar");
    }

    #[test]
    fn change_property_prepend() {
        let mut w = window();
        w.change_property(39, 31, 8, 0, b"bar".to_vec()); // Replace
        w.change_property(39, 31, 8, 1, b"foo".to_vec()); // Prepend
        assert_eq!(w.get_property(39).unwrap().data, b"foobar");
    }

    #[test]
    fn get_property_missing_returns_none() {
        let w = window();
        assert!(w.get_property(39).is_none());
    }

    #[test]
    fn delete_property_removes_it() {
        let mut w = window();
        w.change_property(39, 31, 8, 0, b"hello".to_vec());
        assert!(w.delete_property(39));
        assert!(w.get_property(39).is_none());
        assert!(!w.delete_property(39));
    }
}
