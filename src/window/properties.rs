//! Window properties
//!
//! This module handles window properties and attributes including
//! WM properties, hints, and other window metadata.

use crate::core::error::ServerResult;
use std::collections::HashMap;

/// Property atom identifier
pub type PropertyAtom = u32;

/// Property value with format information
#[derive(Debug, Clone)]
pub struct Property {
    pub type_atom: PropertyAtom,
    pub format: u8, // 8, 16, or 32 bits per unit
    pub data: Vec<u8>,
}

/// Property value (legacy enum for compatibility)
#[derive(Debug, Clone)]
pub enum PropertyValue {
    String(String),
    Cardinal(u32),
    Window(u32),
    Atom(PropertyAtom),
    Data(Vec<u8>),
}

impl PropertyValue {
    /// Convert to new Property format
    pub fn to_property(&self, string_atom: PropertyAtom, cardinal_atom: PropertyAtom) -> Property {
        match self {
            PropertyValue::String(s) => Property {
                type_atom: string_atom,
                format: 8,
                data: s.as_bytes().to_vec(),
            },
            PropertyValue::Cardinal(n) => Property {
                type_atom: cardinal_atom,
                format: 32,
                data: n.to_le_bytes().to_vec(),
            },
            PropertyValue::Window(id) => Property {
                type_atom: 33, // WINDOW atom
                format: 32,
                data: id.to_le_bytes().to_vec(),
            },
            PropertyValue::Atom(atom) => Property {
                type_atom: 4, // ATOM atom
                format: 32,
                data: atom.to_le_bytes().to_vec(),
            },
            PropertyValue::Data(data) => Property {
                type_atom: 0, // None
                format: 8,
                data: data.clone(),
            },
        }
    }
}

/// Window properties manager
pub struct WindowProperties {
    properties: HashMap<u32, HashMap<PropertyAtom, Property>>,
    legacy_properties: HashMap<u32, HashMap<PropertyAtom, PropertyValue>>, // For backward compatibility
}

impl WindowProperties {
    /// Create a new window properties manager
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            legacy_properties: HashMap::new(),
        }
    }

    /// Set a property on a window with full format control
    pub fn set_property_full(
        &mut self,
        window_id: u32,
        property_atom: PropertyAtom,
        type_atom: PropertyAtom,
        format: u8,
        data: Vec<u8>,
    ) -> ServerResult<()> {
        let property = Property {
            type_atom,
            format,
            data,
        };

        self.properties
            .entry(window_id)
            .or_insert_with(HashMap::new)
            .insert(property_atom, property);
        Ok(())
    }

    /// Get a property from a window with full metadata
    pub fn get_property_full(
        &self,
        window_id: u32,
        property_atom: PropertyAtom,
        type_filter: PropertyAtom, // 0 = AnyPropertyType
        offset: u32,
        length: u32,
    ) -> Option<(Property, u32)> {
        // Returns (property, bytes_after)
        let window_props = self.properties.get(&window_id)?;
        let property = window_props.get(&property_atom)?;

        // Check type filter
        if type_filter != 0 && property.type_atom != type_filter {
            return None;
        }

        // Calculate offset and length in bytes
        let bytes_per_unit = match property.format {
            8 => 1,
            16 => 2,
            32 => 4,
            _ => return None,
        };

        let byte_offset = (offset as usize) * bytes_per_unit;
        let max_byte_length = (length as usize) * bytes_per_unit;

        if byte_offset >= property.data.len() {
            // Offset beyond data
            return Some((
                Property {
                    type_atom: property.type_atom,
                    format: property.format,
                    data: Vec::new(),
                },
                0,
            ));
        }

        let end_offset = std::cmp::min(byte_offset + max_byte_length, property.data.len());
        let data_slice = property.data[byte_offset..end_offset].to_vec();
        let bytes_after = property.data.len().saturating_sub(end_offset) as u32;

        Some((
            Property {
                type_atom: property.type_atom,
                format: property.format,
                data: data_slice,
            },
            bytes_after,
        ))
    }

    /// Delete a property from a window
    pub fn delete_property_full(
        &mut self,
        window_id: u32,
        property_atom: PropertyAtom,
    ) -> ServerResult<bool> {
        if let Some(window_props) = self.properties.get_mut(&window_id) {
            Ok(window_props.remove(&property_atom).is_some())
        } else {
            Ok(false)
        }
    }
    /// Set a property on a window (legacy method for compatibility)
    pub fn set_property(
        &mut self,
        window_id: u32,
        property: PropertyAtom,
        value: PropertyValue,
    ) -> ServerResult<()> {
        // Store in legacy format
        self.legacy_properties
            .entry(window_id)
            .or_insert_with(HashMap::new)
            .insert(property, value.clone());

        // Convert to new format - we'll use common atom IDs
        let property_obj = value.to_property(31, 6); // STRING=31, CARDINAL=6
        self.set_property_full(
            window_id,
            property,
            property_obj.type_atom,
            property_obj.format,
            property_obj.data,
        )
    }

    /// Get a property from a window (legacy method for compatibility)
    pub fn get_property(&self, window_id: u32, property: PropertyAtom) -> Option<PropertyValue> {
        let window_props = self.legacy_properties.get(&window_id)?;
        window_props.get(&property).cloned()
    }

    /// Delete a property from a window (legacy method)
    pub fn delete_property(&mut self, window_id: u32, property: PropertyAtom) -> ServerResult<()> {
        if let Some(window_props) = self.legacy_properties.get_mut(&window_id) {
            window_props.remove(&property);
        }
        // Also remove from new format
        self.delete_property_full(window_id, property)?;
        Ok(())
    }
    /// List all properties for a window
    pub fn list_properties(&self, window_id: u32) -> Vec<PropertyAtom> {
        let mut props = Vec::new();

        // Get from new format
        if let Some(window_props) = self.properties.get(&window_id) {
            props.extend(window_props.keys().copied());
        }

        // Get from legacy format
        if let Some(window_props) = self.legacy_properties.get(&window_id) {
            for &key in window_props.keys() {
                if !props.contains(&key) {
                    props.push(key);
                }
            }
        }

        props
    }

    /// Remove all properties for a window
    pub fn remove_window_properties(&mut self, window_id: u32) {
        self.properties.remove(&window_id);
        self.legacy_properties.remove(&window_id);
    }
    /// Get window title (WM_NAME property)
    pub fn get_window_title(&self, window_id: u32) -> Option<String> {
        // WM_NAME is typically atom 39
        if let Some(PropertyValue::String(title)) = self.get_property(window_id, 39) {
            Some(title)
        } else {
            None
        }
    }

    /// Set window title (WM_NAME property)
    pub fn set_window_title(&mut self, window_id: u32, title: String) -> ServerResult<()> {
        // WM_NAME is typically atom 39
        self.set_property(window_id, 39, PropertyValue::String(title))
    }
    /// Get window class (WM_CLASS property)
    pub fn get_window_class(&self, window_id: u32) -> Option<String> {
        // WM_CLASS is typically atom 67
        if let Some(PropertyValue::String(class)) = self.get_property(window_id, 67) {
            Some(class)
        } else {
            None
        }
    }

    /// Set window class (WM_CLASS property)
    pub fn set_window_class(&mut self, window_id: u32, class: String) -> ServerResult<()> {
        // WM_CLASS is typically atom 67
        self.set_property(window_id, 67, PropertyValue::String(class))
    }
}

impl Default for WindowProperties {
    fn default() -> Self {
        Self::new()
    }
}
