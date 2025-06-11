//! Atom Management System for X11 Server
//!
//! This module provides a proper atom storage and allocation system that maintains
//! the mapping between atom names and IDs, handles predefined atoms, and ensures
//! thread-safe allocation of new atoms.

use crate::core::ids::AtomId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

/// Thread-safe atom manager for the X11 server
#[derive(Debug)]
pub struct AtomManager {
    /// Map from atom name to atom ID
    name_to_id: Arc<RwLock<HashMap<String, AtomId>>>,
    /// Map from atom ID to atom name (for reverse lookups)
    id_to_name: Arc<RwLock<HashMap<AtomId, String>>>,
    /// Next available atom ID for dynamic allocation
    next_id: Arc<RwLock<u32>>,
}

impl AtomManager {
    /// Create a new atom manager with all predefined atoms
    pub fn new() -> Self {
        info!("Initializing AtomManager with predefined atoms");
        
        let manager = AtomManager {
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
            id_to_name: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1000)), // Start custom atoms at 1000
        };

        // Initialize all predefined atoms
        manager.initialize_predefined_atoms();
        
        info!("AtomManager initialized with {} predefined atoms", 
              manager.name_to_id.read().unwrap().len());
        
        manager
    }

    /// Initialize all X11 predefined atoms
    fn initialize_predefined_atoms(&self) {
        let predefined_atoms = [
            ("PRIMARY", AtomId::PRIMARY),
            ("SECONDARY", AtomId::SECONDARY),
            ("ARC", AtomId::ARC),
            ("ATOM", AtomId::ATOM),
            ("BITMAP", AtomId::BITMAP),
            ("CARDINAL", AtomId::CARDINAL),
            ("COLORMAP", AtomId::COLORMAP),
            ("CURSOR", AtomId::CURSOR),
            ("CUT_BUFFER0", AtomId::CUT_BUFFER0),
            ("CUT_BUFFER1", AtomId::CUT_BUFFER1),
            ("CUT_BUFFER2", AtomId::CUT_BUFFER2),
            ("CUT_BUFFER3", AtomId::CUT_BUFFER3),
            ("CUT_BUFFER4", AtomId::CUT_BUFFER4),
            ("CUT_BUFFER5", AtomId::CUT_BUFFER5),
            ("CUT_BUFFER6", AtomId::CUT_BUFFER6),
            ("CUT_BUFFER7", AtomId::CUT_BUFFER7),
            ("DRAWABLE", AtomId::DRAWABLE),
            ("FONT", AtomId::FONT),
            ("INTEGER", AtomId::INTEGER),
            ("PIXMAP", AtomId::PIXMAP),
            ("POINT", AtomId::POINT),
            ("RECTANGLE", AtomId::RECTANGLE),
            ("RESOURCE_MANAGER", AtomId::RESOURCE_MANAGER),
            ("RGB_COLOR_MAP", AtomId::RGB_COLOR_MAP),
            ("RGB_BEST_MAP", AtomId::RGB_BEST_MAP),
            ("RGB_BLUE_MAP", AtomId::RGB_BLUE_MAP),
            ("RGB_DEFAULT_MAP", AtomId::RGB_DEFAULT_MAP),
            ("RGB_GRAY_MAP", AtomId::RGB_GRAY_MAP),
            ("RGB_GREEN_MAP", AtomId::RGB_GREEN_MAP),
            ("RGB_RED_MAP", AtomId::RGB_RED_MAP),
            ("STRING", AtomId::STRING),
            ("VISUALID", AtomId::VISUALID),
            ("WINDOW", AtomId::WINDOW),
            ("WM_COMMAND", AtomId::WM_COMMAND),
            ("WM_HINTS", AtomId::WM_HINTS),
            ("WM_CLIENT_MACHINE", AtomId::WM_CLIENT_MACHINE),
            ("WM_ICON_NAME", AtomId::WM_ICON_NAME),
            ("WM_ICON_SIZE", AtomId::WM_ICON_SIZE),
            ("WM_NAME", AtomId::WM_NAME),
            ("WM_NORMAL_HINTS", AtomId::WM_NORMAL_HINTS),
            ("WM_SIZE_HINTS", AtomId::WM_SIZE_HINTS),
            ("WM_ZOOM_HINTS", AtomId::WM_ZOOM_HINTS),
            ("MIN_SPACE", AtomId::MIN_SPACE),
            ("NORM_SPACE", AtomId::NORM_SPACE),
            ("MAX_SPACE", AtomId::MAX_SPACE),
            ("END_SPACE", AtomId::END_SPACE),
            ("SUPERSCRIPT_X", AtomId::SUPERSCRIPT_X),
            ("SUPERSCRIPT_Y", AtomId::SUPERSCRIPT_Y),
            ("SUBSCRIPT_X", AtomId::SUBSCRIPT_X),
            ("SUBSCRIPT_Y", AtomId::SUBSCRIPT_Y),
            ("UNDERLINE_POSITION", AtomId::UNDERLINE_POSITION),
            ("UNDERLINE_THICKNESS", AtomId::UNDERLINE_THICKNESS),
            ("STRIKEOUT_ASCENT", AtomId::STRIKEOUT_ASCENT),
            ("STRIKEOUT_DESCENT", AtomId::STRIKEOUT_DESCENT),
            ("ITALIC_ANGLE", AtomId::ITALIC_ANGLE),
            ("X_HEIGHT", AtomId::X_HEIGHT),
            ("QUAD_WIDTH", AtomId::QUAD_WIDTH),
            ("WEIGHT", AtomId::WEIGHT),
            ("POINT_SIZE", AtomId::POINT_SIZE),
            ("RESOLUTION", AtomId::RESOLUTION),
            ("COPYRIGHT", AtomId::COPYRIGHT),
            ("NOTICE", AtomId::NOTICE),
            ("FONT_NAME", AtomId::FONT_NAME),
            ("FAMILY_NAME", AtomId::FAMILY_NAME),
            ("FULL_NAME", AtomId::FULL_NAME),
            ("CAP_HEIGHT", AtomId::CAP_HEIGHT),
            ("WM_CLASS", AtomId::WM_CLASS),
            ("WM_TRANSIENT_FOR", AtomId::WM_TRANSIENT_FOR),
        ];

        let mut name_map = self.name_to_id.write().unwrap();
        let mut id_map = self.id_to_name.write().unwrap();

        for (name, atom_id) in predefined_atoms.iter() {
            name_map.insert(name.to_string(), *atom_id);
            id_map.insert(*atom_id, name.to_string());
            debug!("Registered predefined atom: '{}' = {}", name, atom_id.0);
        }
    }

    /// Look up an atom by name, returning None if not found
    pub fn lookup_atom(&self, name: &str) -> Option<AtomId> {
        let name_map = self.name_to_id.read().unwrap();
        let result = name_map.get(name).copied();
        
        if let Some(atom_id) = result {
            debug!("Found atom '{}' with ID {}", name, atom_id.0);
        } else {
            debug!("Atom '{}' not found", name);
        }
        
        result
    }

    /// Look up an atom name by ID, returning None if not found
    pub fn lookup_name(&self, atom_id: AtomId) -> Option<String> {
        let id_map = self.id_to_name.read().unwrap();
        let result = id_map.get(&atom_id).cloned();
        
        if let Some(ref name) = result {
            debug!("Found name '{}' for atom ID {}", name, atom_id.0);
        } else {
            debug!("No name found for atom ID {}", atom_id.0);
        }
        
        result
    }

    /// Intern an atom (create if it doesn't exist, or return existing ID)
    pub fn intern_atom(&self, name: &str, only_if_exists: bool) -> Option<AtomId> {
        // First try to find existing atom
        if let Some(atom_id) = self.lookup_atom(name) {
            debug!("InternAtom: Found existing atom '{}' with ID {}", name, atom_id.0);
            return Some(atom_id);
        }

        // If only_if_exists is true and atom doesn't exist, return None
        if only_if_exists {
            debug!("InternAtom: Atom '{}' not found and only_if_exists=true", name);
            return None;
        }

        // Create new atom
        let atom_id = self.create_atom(name);
        info!("InternAtom: Created new atom '{}' with ID {}", name, atom_id.0);
        Some(atom_id)
    }

    /// Create a new atom with the given name
    fn create_atom(&self, name: &str) -> AtomId {
        let mut name_map = self.name_to_id.write().unwrap();
        let mut id_map = self.id_to_name.write().unwrap();
        let mut next_id = self.next_id.write().unwrap();

        // Double-check that the atom doesn't exist (race condition protection)
        if let Some(existing_id) = name_map.get(name) {
            debug!("Atom '{}' was created by another thread with ID {}", name, existing_id.0);
            return *existing_id;
        }

        // Allocate new ID
        let atom_id = AtomId(*next_id);
        *next_id += 1;

        // Store in both maps
        name_map.insert(name.to_string(), atom_id);
        id_map.insert(atom_id, name.to_string());

        debug!("Created new atom: '{}' = {}", name, atom_id.0);
        atom_id
    }

    /// Get the total number of atoms (for debugging/stats)
    pub fn atom_count(&self) -> usize {
        self.name_to_id.read().unwrap().len()
    }

    /// Get all atom names (for debugging)
    pub fn get_all_names(&self) -> Vec<String> {
        self.name_to_id.read().unwrap().keys().cloned().collect()
    }

    /// Validate that the atom manager is in a consistent state
    pub fn validate_consistency(&self) -> bool {
        let name_map = self.name_to_id.read().unwrap();
        let id_map = self.id_to_name.read().unwrap();

        if name_map.len() != id_map.len() {
            warn!("AtomManager inconsistency: name_to_id has {} entries, id_to_name has {} entries",
                  name_map.len(), id_map.len());
            return false;
        }

        for (name, atom_id) in name_map.iter() {
            if let Some(reverse_name) = id_map.get(atom_id) {
                if reverse_name != name {
                    warn!("AtomManager inconsistency: {} -> {} but {} -> {}",
                          name, atom_id.0, atom_id.0, reverse_name);
                    return false;
                }
            } else {
                warn!("AtomManager inconsistency: {} -> {} but no reverse mapping",
                      name, atom_id.0);
                return false;
            }
        }

        debug!("AtomManager consistency check passed");
        true
    }
}

impl Default for AtomManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AtomManager {
    fn clone(&self) -> Self {
        AtomManager {
            name_to_id: Arc::clone(&self.name_to_id),
            id_to_name: Arc::clone(&self.id_to_name),
            next_id: Arc::clone(&self.next_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predefined_atoms() {
        let manager = AtomManager::new();
        
        // Test some predefined atoms
        assert_eq!(manager.lookup_atom("PRIMARY"), Some(AtomId::PRIMARY));
        assert_eq!(manager.lookup_atom("STRING"), Some(AtomId::STRING));
        assert_eq!(manager.lookup_atom("WM_NAME"), Some(AtomId::WM_NAME));
        
        // Test reverse lookup
        assert_eq!(manager.lookup_name(AtomId::PRIMARY), Some("PRIMARY".to_string()));
        assert_eq!(manager.lookup_name(AtomId::STRING), Some("STRING".to_string()));
    }

    #[test]
    fn test_custom_atoms() {
        let manager = AtomManager::new();
        
        // Test creating new atoms
        let atom1 = manager.intern_atom("MY_CUSTOM_ATOM", false);
        assert!(atom1.is_some());
        let atom1 = atom1.unwrap();
        
        // Test that the same atom is returned for the same name
        let atom2 = manager.intern_atom("MY_CUSTOM_ATOM", false);
        assert_eq!(atom1, atom2.unwrap());
        
        // Test reverse lookup of custom atom
        assert_eq!(manager.lookup_name(atom1), Some("MY_CUSTOM_ATOM".to_string()));
    }

    #[test]
    fn test_only_if_exists() {
        let manager = AtomManager::new();
        
        // Test only_if_exists=true with non-existent atom
        assert_eq!(manager.intern_atom("NON_EXISTENT", true), None);
        
        // Test only_if_exists=true with existing atom
        assert_eq!(manager.intern_atom("PRIMARY", true), Some(AtomId::PRIMARY));
    }

    #[test]
    fn test_consistency() {
        let manager = AtomManager::new();
        assert!(manager.validate_consistency());
        
        // Create some custom atoms
        manager.intern_atom("TEST1", false);
        manager.intern_atom("TEST2", false);
        manager.intern_atom("TEST3", false);
        
        assert!(manager.validate_consistency());
    }
}
