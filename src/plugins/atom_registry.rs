//! Atom registry
//!
//! This module manages X11 atoms (interned strings) used throughout
//! the X11 protocol for efficient string handling.

use crate::protocol::{Atom, NONE};
use std::collections::HashMap;

/// Registry for X11 atoms
pub struct AtomRegistry {
    atoms: HashMap<String, Atom>,
    strings: HashMap<Atom, String>,
    next_id: Atom,
}

impl AtomRegistry {
    /// Create a new atom registry
    pub fn new() -> Self {
        let mut registry = Self {
            atoms: HashMap::new(),
            strings: HashMap::new(),
            next_id: 1, // Start at 1, 0 is reserved
        };

        // Pre-register common atoms
        registry.register_predefined_atoms();
        registry
    }

    /// Register a string as an atom
    pub fn intern(&mut self, name: &str) -> Atom {
        if let Some(&atom) = self.atoms.get(name) {
            return atom;
        }

        let atom = self.next_id;
        self.next_id += 1;

        self.atoms.insert(name.to_string(), atom);
        self.strings.insert(atom, name.to_string());

        atom
    }

    /// Get the string for an atom
    pub fn get_name(&self, atom: Atom) -> Option<&str> {
        self.strings.get(&atom).map(|s| s.as_str())
    }

    /// Get the atom for a string
    pub fn get_atom(&self, name: &str) -> Option<Atom> {
        self.atoms.get(name).copied()
    }

    pub fn is_valid_atom(&self, atom: Atom) -> bool {
        atom != NONE && self.strings.contains_key(&atom)
    }

    /// Register predefined atoms
    fn register_predefined_atoms(&mut self) {
        // Common X11 atoms
        let predefined = [
            "PRIMARY",
            "SECONDARY",
            "ARC",
            "ATOM",
            "BITMAP",
            "CARDINAL",
            "COLORMAP",
            "CURSOR",
            "CUT_BUFFER0",
            "CUT_BUFFER1",
            "CUT_BUFFER2",
            "CUT_BUFFER3",
            "CUT_BUFFER4",
            "CUT_BUFFER5",
            "CUT_BUFFER6",
            "CUT_BUFFER7",
            "DRAWABLE",
            "FONT",
            "INTEGER",
            "PIXMAP",
            "POINT",
            "RECTANGLE",
            "RESOURCE_MANAGER",
            "RGB_COLOR_MAP",
            "RGB_BEST_MAP",
            "RGB_BLUE_MAP",
            "RGB_DEFAULT_MAP",
            "RGB_GRAY_MAP",
            "RGB_GREEN_MAP",
            "RGB_RED_MAP",
            "STRING",
            "VISUALID",
            "WINDOW",
            "WM_COMMAND",
            "WM_HINTS",
            "WM_CLIENT_MACHINE",
            "WM_ICON_NAME",
            "WM_ICON_SIZE",
            "WM_NAME",
            "WM_NORMAL_HINTS",
            "WM_SIZE_HINTS",
            "WM_ZOOM_HINTS",
            "MIN_SPACE",
            "NORM_SPACE",
            "MAX_SPACE",
            "END_SPACE",
            "SUPERSCRIPT_X",
            "SUPERSCRIPT_Y",
            "SUBSCRIPT_X",
            "SUBSCRIPT_Y",
            "UNDERLINE_POSITION",
            "UNDERLINE_THICKNESS",
            "STRIKEOUT_ASCENT",
            "STRIKEOUT_DESCENT",
            "ITALIC_ANGLE",
            "X_HEIGHT",
            "QUAD_WIDTH",
            "WEIGHT",
            "POINT_SIZE",
            "RESOLUTION",
            "COPYRIGHT",
            "NOTICE",
            "FONT_NAME",
            "FAMILY_NAME",
            "FULL_NAME",
            "CAP_HEIGHT",
            "WM_CLASS",
            "WM_TRANSIENT_FOR",
        ];

        for &name in &predefined {
            self.intern(name);
        }
    }
}

impl Default for AtomRegistry {
    fn default() -> Self {
        Self::new()
    }
}
