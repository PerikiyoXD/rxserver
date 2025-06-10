//! Window properties management
//!
//! This module handles window properties such as window titles, classes,
//! and other metadata associated with windows.

use std::collections::HashMap;
use crate::protocol::types::*;
use crate::{Error, Result};

/// Window property data
#[derive(Debug, Clone)]
pub struct Property {
    /// Property type
    pub property_type: Atom,
    /// Property format (8, 16, or 32 bits per item)
    pub format: u8,
    /// Property data
    pub data: Vec<u8>,
}

/// Manages window properties
pub struct PropertyManager {
    /// Properties for each window
    window_properties: HashMap<Window, HashMap<Atom, Property>>,
}

impl PropertyManager {
    /// Create a new property manager
    pub fn new() -> Self {
        Self {
            window_properties: HashMap::new(),
        }
    }

    /// Set a property on a window
    pub fn set_property(
        &mut self,
        window: Window,
        property: Atom,
        property_type: Atom,
        format: u8,
        data: Vec<u8>,
    ) -> Result<()> {
        if format != 8 && format != 16 && format != 32 {
            return Err(Error::Window("Invalid property format".to_string()));
        }

        let prop = Property {
            property_type,
            format,
            data,
        };

        self.window_properties
            .entry(window)
            .or_insert_with(HashMap::new)
            .insert(property, prop);

        log::debug!("Set property {} on window {}", property, window);
        Ok(())
    }

    /// Get a property from a window
    pub fn get_property(&self, window: Window, property: Atom) -> Option<&Property> {
        self.window_properties
            .get(&window)
            .and_then(|props| props.get(&property))
    }

    /// Delete a property from a window
    pub fn delete_property(&mut self, window: Window, property: Atom) -> Result<bool> {
        if let Some(props) = self.window_properties.get_mut(&window) {
            let removed = props.remove(&property).is_some();
            log::debug!("Deleted property {} from window {}", property, window);
            Ok(removed)
        } else {
            Ok(false)
        }
    }

    /// List all properties for a window
    pub fn list_properties(&self, window: Window) -> Vec<Atom> {
        self.window_properties
            .get(&window)
            .map(|props| props.keys().copied().collect())
            .unwrap_or_default()
    }

    /// Remove all properties for a window
    pub fn remove_window_properties(&mut self, window: Window) {
        if self.window_properties.remove(&window).is_some() {
            log::debug!("Removed all properties for window {}", window);
        }
    }

    /// Set the window title (WM_NAME property)
    pub fn set_window_title(&mut self, window: Window, title: &str) -> Result<()> {
        self.set_property(
            window,
            atoms::WM_NAME,
            atoms::STRING,
            8,
            title.as_bytes().to_vec(),
        )
    }

    /// Get the window title (WM_NAME property)
    pub fn get_window_title(&self, window: Window) -> Option<String> {
        self.get_property(window, atoms::WM_NAME)
            .and_then(|prop| {
                if prop.format == 8 {
                    String::from_utf8(prop.data.clone()).ok()
                } else {
                    None
                }
            })
    }

    /// Set the window class (WM_CLASS property)
    pub fn set_window_class(&mut self, window: Window, instance: &str, class: &str) -> Result<()> {
        let mut data = Vec::new();
        data.extend_from_slice(instance.as_bytes());
        data.push(0); // Null terminator
        data.extend_from_slice(class.as_bytes());
        data.push(0); // Null terminator

        self.set_property(window, atoms::WM_CLASS, atoms::STRING, 8, data)
    }

    /// Get the window class (WM_CLASS property)
    pub fn get_window_class(&self, window: Window) -> Option<(String, String)> {
        self.get_property(window, atoms::WM_CLASS)
            .and_then(|prop| {
                if prop.format == 8 {
                    let parts: Vec<&str> = std::str::from_utf8(&prop.data)
                        .ok()?
                        .split('\0')
                        .collect();
                    if parts.len() >= 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
    }
}

/// Common X11 atom constants
pub mod atoms {
    use crate::protocol::types::Atom;

    pub const PRIMARY: Atom = 1;
    pub const SECONDARY: Atom = 2;
    pub const ARC: Atom = 3;
    pub const ATOM: Atom = 4;
    pub const BITMAP: Atom = 5;
    pub const CARDINAL: Atom = 6;
    pub const COLORMAP: Atom = 7;
    pub const CURSOR: Atom = 8;
    pub const CUT_BUFFER0: Atom = 9;
    pub const CUT_BUFFER1: Atom = 10;
    pub const CUT_BUFFER2: Atom = 11;
    pub const CUT_BUFFER3: Atom = 12;
    pub const CUT_BUFFER4: Atom = 13;
    pub const CUT_BUFFER5: Atom = 14;
    pub const CUT_BUFFER6: Atom = 15;
    pub const CUT_BUFFER7: Atom = 16;
    pub const DRAWABLE: Atom = 17;
    pub const FONT: Atom = 18;
    pub const INTEGER: Atom = 19;
    pub const PIXMAP: Atom = 20;
    pub const POINT: Atom = 21;
    pub const RECTANGLE: Atom = 22;
    pub const RESOURCE_MANAGER: Atom = 23;
    pub const RGB_COLOR_MAP: Atom = 24;
    pub const RGB_BEST_MAP: Atom = 25;
    pub const RGB_BLUE_MAP: Atom = 26;
    pub const RGB_DEFAULT_MAP: Atom = 27;
    pub const RGB_GRAY_MAP: Atom = 28;
    pub const RGB_GREEN_MAP: Atom = 29;
    pub const RGB_RED_MAP: Atom = 30;
    pub const STRING: Atom = 31;
    pub const VISUALID: Atom = 32;
    pub const WINDOW: Atom = 33;
    pub const WM_COMMAND: Atom = 34;
    pub const WM_HINTS: Atom = 35;
    pub const WM_CLIENT_MACHINE: Atom = 36;
    pub const WM_ICON_NAME: Atom = 37;
    pub const WM_ICON_SIZE: Atom = 38;
    pub const WM_NAME: Atom = 39;
    pub const WM_NORMAL_HINTS: Atom = 40;
    pub const WM_SIZE_HINTS: Atom = 41;
    pub const WM_ZOOM_HINTS: Atom = 42;
    pub const MIN_SPACE: Atom = 43;
    pub const NORM_SPACE: Atom = 44;
    pub const MAX_SPACE: Atom = 45;
    pub const END_SPACE: Atom = 46;
    pub const SUPERSCRIPT_X: Atom = 47;
    pub const SUPERSCRIPT_Y: Atom = 48;
    pub const SUBSCRIPT_X: Atom = 49;
    pub const SUBSCRIPT_Y: Atom = 50;
    pub const UNDERLINE_POSITION: Atom = 51;
    pub const UNDERLINE_THICKNESS: Atom = 52;
    pub const STRIKEOUT_ASCENT: Atom = 53;
    pub const STRIKEOUT_DESCENT: Atom = 54;
    pub const ITALIC_ANGLE: Atom = 55;
    pub const X_HEIGHT: Atom = 56;
    pub const QUAD_WIDTH: Atom = 57;
    pub const WEIGHT: Atom = 58;
    pub const POINT_SIZE: Atom = 59;
    pub const RESOLUTION: Atom = 60;
    pub const COPYRIGHT: Atom = 61;
    pub const NOTICE: Atom = 62;
    pub const FONT_NAME: Atom = 63;
    pub const FAMILY_NAME: Atom = 64;
    pub const FULL_NAME: Atom = 65;
    pub const CAP_HEIGHT: Atom = 66;
    pub const WM_CLASS: Atom = 67;
    pub const WM_TRANSIENT_FOR: Atom = 68;

    pub const LAST_PREDEFINED: Atom = 68;
}
