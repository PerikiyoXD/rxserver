use crate::protocol::Atom;
use std::collections::HashMap;
use tracing::{debug, trace};

/// Manages X11 atom identifiers and their string mappings.
///
/// Atoms are unique identifiers for strings in the X11 protocol, used for
/// properties, selections, and inter-client communication.
#[derive(Debug)]
pub struct AtomSystem {
    registry: HashMap<String, Atom>,
    next_atom_id: Atom,
}

impl AtomSystem {
    pub fn new() -> Self {
        let mut registry = HashMap::new();

        for &(name, atom_id) in PREDEFINED_ATOMS {
            registry.insert(name.to_string(), atom_id);
        }

        let next_atom_id = PREDEFINED_ATOMS
            .iter()
            .map(|&(_, id)| id)
            .max()
            .unwrap_or(0)
            + 1;

        Self {
            registry,
            next_atom_id,
        }
    }

    /// Returns an atom ID for the given name, creating one if needed.
    ///
    /// If `only_if_exists` is true, returns None for unknown atoms.
    /// Otherwise creates a new atom and returns its ID.
    pub fn intern_atom(&mut self, name: &str, only_if_exists: bool) -> Option<Atom> {
        if let Some(&atom_id) = self.registry.get(name) {
            trace!("Found existing atom '{}' with ID {}", name, atom_id);
            Some(atom_id)
        } else if !only_if_exists {
            let atom_id = self.next_atom_id;
            self.next_atom_id += 1;
            self.registry.insert(name.to_string(), atom_id);
            debug!("Created new atom '{}' with ID {}", name, atom_id);
            Some(atom_id)
        } else {
            trace!("Atom '{}' not found and only_if_exists=true", name);
            None
        }
    }

    pub fn get_atom_name(&self, atom_id: Atom) -> Option<&str> {
        self.registry
            .iter()
            .find(|&(_, &id)| id == atom_id)
            .map(|(name, _)| name.as_str())
    }

    pub fn len(&self) -> usize {
        self.registry.len()
    }
}

impl Default for AtomSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard X11 atoms as defined in the protocol specification
const PREDEFINED_ATOMS: &[(&str, Atom)] = &[
    ("PRIMARY", 1),
    ("SECONDARY", 2),
    ("ARC", 3),
    ("ATOM", 4),
    ("BITMAP", 5),
    ("CARDINAL", 6),
    ("COLORMAP", 7),
    ("CURSOR", 8),
    ("CUT_BUFFER0", 9),
    ("CUT_BUFFER1", 10),
    ("CUT_BUFFER2", 11),
    ("CUT_BUFFER3", 12),
    ("CUT_BUFFER4", 13),
    ("CUT_BUFFER5", 14),
    ("CUT_BUFFER6", 15),
    ("CUT_BUFFER7", 16),
    ("DRAWABLE", 17),
    ("FONT", 18),
    ("INTEGER", 19),
    ("PIXMAP", 20),
    ("POINT", 21),
    ("RECTANGLE", 22),
    ("RESOURCE_MANAGER", 23),
    ("RGB_COLOR_MAP", 24),
    ("RGB_BEST_MAP", 25),
    ("RGB_BLUE_MAP", 26),
    ("RGB_DEFAULT_MAP", 27),
    ("RGB_GRAY_MAP", 28),
    ("RGB_GREEN_MAP", 29),
    ("RGB_RED_MAP", 30),
    ("STRING", 31),
    ("VISUALID", 32),
    ("WINDOW", 33),
    ("WM_COMMAND", 34),
    ("WM_HINTS", 35),
    ("WM_CLIENT_MACHINE", 36),
    ("WM_ICON_NAME", 37),
    ("WM_ICON_SIZE", 38),
    ("WM_NAME", 39),
    ("WM_NORMAL_HINTS", 40),
    ("WM_SIZE_HINTS", 41),
    ("WM_ZOOM_HINTS", 42),
    ("MIN_SPACE", 43),
    ("NORM_SPACE", 44),
    ("MAX_SPACE", 45),
    ("END_SPACE", 46),
    ("SUPERSCRIPT_X", 47),
    ("SUPERSCRIPT_Y", 48),
    ("SUBSCRIPT_X", 49),
    ("SUBSCRIPT_Y", 50),
    ("UNDERLINE_POSITION", 51),
    ("UNDERLINE_THICKNESS", 52),
    ("STRIKEOUT_ASCENT", 53),
    ("STRIKEOUT_DESCENT", 54),
    ("ITALIC_ANGLE", 55),
    ("X_HEIGHT", 56),
    ("QUAD_WIDTH", 57),
    ("WEIGHT", 58),
    ("POINT_SIZE", 59),
    ("RESOLUTION", 60),
    ("COPYRIGHT", 61),
    ("NOTICE", 62),
    ("FONT_NAME", 63),
    ("FAMILY_NAME", 64),
    ("FULL_NAME", 65),
    ("CAP_HEIGHT", 66),
    ("WM_CLASS", 67),
    ("WM_TRANSIENT_FOR", 68),
];
