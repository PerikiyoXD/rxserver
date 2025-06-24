//! Atom resource implementation
//!
//! Atoms in X11 are unique identifiers for strings. They provide a way to refer
//! to strings using integer values, which is more efficient for network protocols.
//! Atoms are globally unique across all clients.

use crate::x11::protocol::types::{ClientId, XID};
use crate::x11::resources::{LifecycleError, Resource, ResourceType};
use std::collections::HashMap;

/// Predefined X11 atoms that always exist
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredefinedAtom {
    Primary = 1,
    Secondary = 2,
    Arc = 3,
    Atom = 4,
    Bitmap = 5,
    Cardinal = 6,
    Colormap = 7,
    Cursor = 8,
    CutBuffer0 = 9,
    CutBuffer1 = 10,
    CutBuffer2 = 11,
    CutBuffer3 = 12,
    CutBuffer4 = 13,
    CutBuffer5 = 14,
    CutBuffer6 = 15,
    CutBuffer7 = 16,
    Drawable = 17,
    Font = 18,
    Integer = 19,
    Pixmap = 20,
    Point = 21,
    Rectangle = 22,
    ResourceManager = 23,
    RgbColorMap = 24,
    RgbBestMap = 25,
    RgbBlueMap = 26,
    RgbDefaultMap = 27,
    RgbGrayMap = 28,
    RgbGreenMap = 29,
    RgbRedMap = 30,
    String = 31,
    VisualId = 32,
    Window = 33,
    WmCommand = 34,
    WmHints = 35,
    WmClientMachine = 36,
    WmIconName = 37,
    WmIconSize = 38,
    WmName = 39,
    WmNormalHints = 40,
    WmSizeHints = 41,
    WmZoomHints = 42,
    MinSpace = 43,
    NormSpace = 44,
    MaxSpace = 45,
    EndSpace = 46,
    SuperscriptX = 47,
    SuperscriptY = 48,
    SubscriptX = 49,
    SubscriptY = 50,
    UnderlinePosition = 51,
    UnderlineThickness = 52,
    StrikeoutAscent = 53,
    StrikeoutDescent = 54,
    ItalicAngle = 55,
    XHeight = 56,
    QuadWidth = 57,
    Weight = 58,
    PointSize = 59,
    Resolution = 60,
    Copyright = 61,
    Notice = 62,
    FontName = 63,
    FamilyName = 64,
    FullName = 65,
    CapHeight = 66,
    WmClass = 67,
    WmTransientFor = 68,
}

impl PredefinedAtom {
    /// Get the string name of a predefined atom
    pub fn name(self) -> &'static str {
        match self {
            PredefinedAtom::Primary => "PRIMARY",
            PredefinedAtom::Secondary => "SECONDARY",
            PredefinedAtom::Arc => "ARC",
            PredefinedAtom::Atom => "ATOM",
            PredefinedAtom::Bitmap => "BITMAP",
            PredefinedAtom::Cardinal => "CARDINAL",
            PredefinedAtom::Colormap => "COLORMAP",
            PredefinedAtom::Cursor => "CURSOR",
            PredefinedAtom::CutBuffer0 => "CUT_BUFFER0",
            PredefinedAtom::CutBuffer1 => "CUT_BUFFER1",
            PredefinedAtom::CutBuffer2 => "CUT_BUFFER2",
            PredefinedAtom::CutBuffer3 => "CUT_BUFFER3",
            PredefinedAtom::CutBuffer4 => "CUT_BUFFER4",
            PredefinedAtom::CutBuffer5 => "CUT_BUFFER5",
            PredefinedAtom::CutBuffer6 => "CUT_BUFFER6",
            PredefinedAtom::CutBuffer7 => "CUT_BUFFER7",
            PredefinedAtom::Drawable => "DRAWABLE",
            PredefinedAtom::Font => "FONT",
            PredefinedAtom::Integer => "INTEGER",
            PredefinedAtom::Pixmap => "PIXMAP",
            PredefinedAtom::Point => "POINT",
            PredefinedAtom::Rectangle => "RECTANGLE",
            PredefinedAtom::ResourceManager => "RESOURCE_MANAGER",
            PredefinedAtom::RgbColorMap => "RGB_COLOR_MAP",
            PredefinedAtom::RgbBestMap => "RGB_BEST_MAP",
            PredefinedAtom::RgbBlueMap => "RGB_BLUE_MAP",
            PredefinedAtom::RgbDefaultMap => "RGB_DEFAULT_MAP",
            PredefinedAtom::RgbGrayMap => "RGB_GRAY_MAP",
            PredefinedAtom::RgbGreenMap => "RGB_GREEN_MAP",
            PredefinedAtom::RgbRedMap => "RGB_RED_MAP",
            PredefinedAtom::String => "STRING",
            PredefinedAtom::VisualId => "VISUALID",
            PredefinedAtom::Window => "WINDOW",
            PredefinedAtom::WmCommand => "WM_COMMAND",
            PredefinedAtom::WmHints => "WM_HINTS",
            PredefinedAtom::WmClientMachine => "WM_CLIENT_MACHINE",
            PredefinedAtom::WmIconName => "WM_ICON_NAME",
            PredefinedAtom::WmIconSize => "WM_ICON_SIZE",
            PredefinedAtom::WmName => "WM_NAME",
            PredefinedAtom::WmNormalHints => "WM_NORMAL_HINTS",
            PredefinedAtom::WmSizeHints => "WM_SIZE_HINTS",
            PredefinedAtom::WmZoomHints => "WM_ZOOM_HINTS",
            PredefinedAtom::MinSpace => "MIN_SPACE",
            PredefinedAtom::NormSpace => "NORM_SPACE",
            PredefinedAtom::MaxSpace => "MAX_SPACE",
            PredefinedAtom::EndSpace => "END_SPACE",
            PredefinedAtom::SuperscriptX => "SUPERSCRIPT_X",
            PredefinedAtom::SuperscriptY => "SUPERSCRIPT_Y",
            PredefinedAtom::SubscriptX => "SUBSCRIPT_X",
            PredefinedAtom::SubscriptY => "SUBSCRIPT_Y",
            PredefinedAtom::UnderlinePosition => "UNDERLINE_POSITION",
            PredefinedAtom::UnderlineThickness => "UNDERLINE_THICKNESS",
            PredefinedAtom::StrikeoutAscent => "STRIKEOUT_ASCENT",
            PredefinedAtom::StrikeoutDescent => "STRIKEOUT_DESCENT",
            PredefinedAtom::ItalicAngle => "ITALIC_ANGLE",
            PredefinedAtom::XHeight => "X_HEIGHT",
            PredefinedAtom::QuadWidth => "QUAD_WIDTH",
            PredefinedAtom::Weight => "WEIGHT",
            PredefinedAtom::PointSize => "POINT_SIZE",
            PredefinedAtom::Resolution => "RESOLUTION",
            PredefinedAtom::Copyright => "COPYRIGHT",
            PredefinedAtom::Notice => "NOTICE",
            PredefinedAtom::FontName => "FONT_NAME",
            PredefinedAtom::FamilyName => "FAMILY_NAME",
            PredefinedAtom::FullName => "FULL_NAME",
            PredefinedAtom::CapHeight => "CAP_HEIGHT",
            PredefinedAtom::WmClass => "WM_CLASS",
            PredefinedAtom::WmTransientFor => "WM_TRANSIENT_FOR",
        }
    }

    /// Get all predefined atoms
    pub fn all() -> Vec<(XID, &'static str)> {
        vec![
            (
                PredefinedAtom::Primary as XID,
                PredefinedAtom::Primary.name(),
            ),
            (
                PredefinedAtom::Secondary as XID,
                PredefinedAtom::Secondary.name(),
            ),
            (PredefinedAtom::Arc as XID, PredefinedAtom::Arc.name()),
            (PredefinedAtom::Atom as XID, PredefinedAtom::Atom.name()),
            (PredefinedAtom::Bitmap as XID, PredefinedAtom::Bitmap.name()),
            (
                PredefinedAtom::Cardinal as XID,
                PredefinedAtom::Cardinal.name(),
            ),
            (
                PredefinedAtom::Colormap as XID,
                PredefinedAtom::Colormap.name(),
            ),
            (PredefinedAtom::Cursor as XID, PredefinedAtom::Cursor.name()),
            (
                PredefinedAtom::CutBuffer0 as XID,
                PredefinedAtom::CutBuffer0.name(),
            ),
            (
                PredefinedAtom::CutBuffer1 as XID,
                PredefinedAtom::CutBuffer1.name(),
            ),
            (
                PredefinedAtom::CutBuffer2 as XID,
                PredefinedAtom::CutBuffer2.name(),
            ),
            (
                PredefinedAtom::CutBuffer3 as XID,
                PredefinedAtom::CutBuffer3.name(),
            ),
            (
                PredefinedAtom::CutBuffer4 as XID,
                PredefinedAtom::CutBuffer4.name(),
            ),
            (
                PredefinedAtom::CutBuffer5 as XID,
                PredefinedAtom::CutBuffer5.name(),
            ),
            (
                PredefinedAtom::CutBuffer6 as XID,
                PredefinedAtom::CutBuffer6.name(),
            ),
            (
                PredefinedAtom::CutBuffer7 as XID,
                PredefinedAtom::CutBuffer7.name(),
            ),
            (
                PredefinedAtom::Drawable as XID,
                PredefinedAtom::Drawable.name(),
            ),
            (PredefinedAtom::Font as XID, PredefinedAtom::Font.name()),
            (
                PredefinedAtom::Integer as XID,
                PredefinedAtom::Integer.name(),
            ),
            (PredefinedAtom::Pixmap as XID, PredefinedAtom::Pixmap.name()),
            (PredefinedAtom::Point as XID, PredefinedAtom::Point.name()),
            (
                PredefinedAtom::Rectangle as XID,
                PredefinedAtom::Rectangle.name(),
            ),
            (
                PredefinedAtom::ResourceManager as XID,
                PredefinedAtom::ResourceManager.name(),
            ),
            (
                PredefinedAtom::RgbColorMap as XID,
                PredefinedAtom::RgbColorMap.name(),
            ),
            (
                PredefinedAtom::RgbBestMap as XID,
                PredefinedAtom::RgbBestMap.name(),
            ),
            (
                PredefinedAtom::RgbBlueMap as XID,
                PredefinedAtom::RgbBlueMap.name(),
            ),
            (
                PredefinedAtom::RgbDefaultMap as XID,
                PredefinedAtom::RgbDefaultMap.name(),
            ),
            (
                PredefinedAtom::RgbGrayMap as XID,
                PredefinedAtom::RgbGrayMap.name(),
            ),
            (
                PredefinedAtom::RgbGreenMap as XID,
                PredefinedAtom::RgbGreenMap.name(),
            ),
            (
                PredefinedAtom::RgbRedMap as XID,
                PredefinedAtom::RgbRedMap.name(),
            ),
            (PredefinedAtom::String as XID, PredefinedAtom::String.name()),
            (
                PredefinedAtom::VisualId as XID,
                PredefinedAtom::VisualId.name(),
            ),
            (PredefinedAtom::Window as XID, PredefinedAtom::Window.name()),
            (
                PredefinedAtom::WmCommand as XID,
                PredefinedAtom::WmCommand.name(),
            ),
            (
                PredefinedAtom::WmHints as XID,
                PredefinedAtom::WmHints.name(),
            ),
            (
                PredefinedAtom::WmClientMachine as XID,
                PredefinedAtom::WmClientMachine.name(),
            ),
            (
                PredefinedAtom::WmIconName as XID,
                PredefinedAtom::WmIconName.name(),
            ),
            (
                PredefinedAtom::WmIconSize as XID,
                PredefinedAtom::WmIconSize.name(),
            ),
            (PredefinedAtom::WmName as XID, PredefinedAtom::WmName.name()),
            (
                PredefinedAtom::WmNormalHints as XID,
                PredefinedAtom::WmNormalHints.name(),
            ),
            (
                PredefinedAtom::WmSizeHints as XID,
                PredefinedAtom::WmSizeHints.name(),
            ),
            (
                PredefinedAtom::WmZoomHints as XID,
                PredefinedAtom::WmZoomHints.name(),
            ),
            (
                PredefinedAtom::MinSpace as XID,
                PredefinedAtom::MinSpace.name(),
            ),
            (
                PredefinedAtom::NormSpace as XID,
                PredefinedAtom::NormSpace.name(),
            ),
            (
                PredefinedAtom::MaxSpace as XID,
                PredefinedAtom::MaxSpace.name(),
            ),
            (
                PredefinedAtom::EndSpace as XID,
                PredefinedAtom::EndSpace.name(),
            ),
            (
                PredefinedAtom::SuperscriptX as XID,
                PredefinedAtom::SuperscriptX.name(),
            ),
            (
                PredefinedAtom::SuperscriptY as XID,
                PredefinedAtom::SuperscriptY.name(),
            ),
            (
                PredefinedAtom::SubscriptX as XID,
                PredefinedAtom::SubscriptX.name(),
            ),
            (
                PredefinedAtom::SubscriptY as XID,
                PredefinedAtom::SubscriptY.name(),
            ),
            (
                PredefinedAtom::UnderlinePosition as XID,
                PredefinedAtom::UnderlinePosition.name(),
            ),
            (
                PredefinedAtom::UnderlineThickness as XID,
                PredefinedAtom::UnderlineThickness.name(),
            ),
            (
                PredefinedAtom::StrikeoutAscent as XID,
                PredefinedAtom::StrikeoutAscent.name(),
            ),
            (
                PredefinedAtom::StrikeoutDescent as XID,
                PredefinedAtom::StrikeoutDescent.name(),
            ),
            (
                PredefinedAtom::ItalicAngle as XID,
                PredefinedAtom::ItalicAngle.name(),
            ),
            (
                PredefinedAtom::XHeight as XID,
                PredefinedAtom::XHeight.name(),
            ),
            (
                PredefinedAtom::QuadWidth as XID,
                PredefinedAtom::QuadWidth.name(),
            ),
            (PredefinedAtom::Weight as XID, PredefinedAtom::Weight.name()),
            (
                PredefinedAtom::PointSize as XID,
                PredefinedAtom::PointSize.name(),
            ),
            (
                PredefinedAtom::Resolution as XID,
                PredefinedAtom::Resolution.name(),
            ),
            (
                PredefinedAtom::Copyright as XID,
                PredefinedAtom::Copyright.name(),
            ),
            (PredefinedAtom::Notice as XID, PredefinedAtom::Notice.name()),
            (
                PredefinedAtom::FontName as XID,
                PredefinedAtom::FontName.name(),
            ),
            (
                PredefinedAtom::FamilyName as XID,
                PredefinedAtom::FamilyName.name(),
            ),
            (
                PredefinedAtom::FullName as XID,
                PredefinedAtom::FullName.name(),
            ),
            (
                PredefinedAtom::CapHeight as XID,
                PredefinedAtom::CapHeight.name(),
            ),
            (
                PredefinedAtom::WmClass as XID,
                PredefinedAtom::WmClass.name(),
            ),
            (
                PredefinedAtom::WmTransientFor as XID,
                PredefinedAtom::WmTransientFor.name(),
            ),
        ]
    }
}

/// Atom usage tracking
#[derive(Debug, Clone)]
pub struct AtomUsage {
    /// Number of times this atom has been referenced
    pub reference_count: u32,
    /// Clients that have referenced this atom
    pub referencing_clients: Vec<ClientId>,
    /// Last time this atom was used
    pub last_used: std::time::SystemTime,
}

impl Default for AtomUsage {
    fn default() -> Self {
        Self {
            reference_count: 0,
            referencing_clients: Vec::new(),
            last_used: std::time::SystemTime::UNIX_EPOCH,
        }
    }
}

/// Atom resource implementation
#[derive(Debug)]
pub struct AtomResource {
    /// Unique identifier for this atom (the atom value itself)
    xid: XID,
    /// String name of the atom
    name: String,
    /// Whether this is a predefined atom
    is_predefined: bool,
    /// Client that first interned this atom (0 for predefined atoms)
    original_client: ClientId,
    /// Usage tracking information
    usage: AtomUsage,
    /// Whether this atom can be deleted
    can_delete: bool,
}

impl AtomResource {
    /// Create a new atom resource for a custom (non-predefined) atom
    pub fn new(xid: XID, name: String, client: ClientId) -> Result<Self, LifecycleError> {
        if name.is_empty() {
            return Err(LifecycleError::InitializationFailed(
                "Atom name cannot be empty".into(),
            ));
        }

        if name.len() > 255 {
            return Err(LifecycleError::InitializationFailed(
                "Atom name too long (max 255 characters)".into(),
            ));
        }

        // Check for invalid characters in atom name
        if name.contains('\0') {
            return Err(LifecycleError::InitializationFailed(
                "Atom name cannot contain null characters".into(),
            ));
        }

        let mut atom = Self {
            xid,
            name,
            is_predefined: false,
            original_client: client,
            usage: AtomUsage::default(),
            can_delete: true, // Custom atoms can be deleted when not referenced
        };

        // Record initial usage
        atom.add_reference(client);

        Ok(atom)
    }

    /// Create a predefined atom resource
    pub fn new_predefined(predefined: PredefinedAtom) -> Self {
        Self {
            xid: predefined as XID,
            name: predefined.name().to_string(),
            is_predefined: true,
            original_client: 0, // System atom
            usage: AtomUsage::default(),
            can_delete: false, // Predefined atoms cannot be deleted
        }
    }

    /// Get the atom name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Check if this is a predefined atom
    pub fn is_predefined(&self) -> bool {
        self.is_predefined
    }

    /// Get the original client that interned this atom
    pub fn original_client(&self) -> ClientId {
        self.original_client
    }

    /// Add a reference to this atom from a client
    pub fn add_reference(&mut self, client: ClientId) {
        self.usage.reference_count += 1;

        if !self.usage.referencing_clients.contains(&client) {
            self.usage.referencing_clients.push(client);
        }

        self.usage.last_used = std::time::SystemTime::now();
    }

    /// Remove a reference to this atom from a client
    pub fn remove_reference(&mut self, client: ClientId) -> bool {
        if let Some(pos) = self
            .usage
            .referencing_clients
            .iter()
            .position(|&c| c == client)
        {
            self.usage.referencing_clients.remove(pos);

            if self.usage.reference_count > 0 {
                self.usage.reference_count -= 1;
            }

            true
        } else {
            false
        }
    }

    /// Get the current reference count
    pub fn reference_count(&self) -> u32 {
        self.usage.reference_count
    }

    /// Get the list of clients referencing this atom
    pub fn referencing_clients(&self) -> &[ClientId] {
        &self.usage.referencing_clients
    }

    /// Check if this atom is referenced by any client
    pub fn is_referenced(&self) -> bool {
        !self.usage.referencing_clients.is_empty()
    }

    /// Check if this atom can be deleted (not predefined and not referenced)
    pub fn can_be_deleted(&self) -> bool {
        self.can_delete && !self.is_referenced() && !self.is_predefined
    }

    /// Remove all references from a specific client
    pub fn remove_client_references(&mut self, client: ClientId) -> u32 {
        let removed_count = self
            .usage
            .referencing_clients
            .iter()
            .filter(|&&c| c == client)
            .count() as u32;

        self.usage.referencing_clients.retain(|&c| c != client);

        if self.usage.reference_count >= removed_count {
            self.usage.reference_count -= removed_count;
        } else {
            self.usage.reference_count = 0;
        }

        removed_count
    }

    /// Get usage statistics
    pub fn usage_stats(&self) -> &AtomUsage {
        &self.usage
    }

    /// Check if the atom name matches (case-sensitive)
    pub fn matches_name(&self, name: &str) -> bool {
        self.name == name
    }

    /// Check if the atom name matches (case-insensitive)
    pub fn matches_name_ignore_case(&self, name: &str) -> bool {
        self.name.to_lowercase() == name.to_lowercase()
    }

    /// Get the atom value (XID)
    pub fn atom_value(&self) -> XID {
        self.xid
    }
}

/// Global atom registry for managing all atoms
#[derive(Debug)]
pub struct AtomRegistry {
    /// Map from atom XID to atom resource
    atoms: HashMap<XID, AtomResource>,
    /// Map from atom name to XID for quick lookup
    name_to_xid: HashMap<String, XID>,
    /// Next available XID for custom atoms
    next_xid: XID,
}

impl AtomRegistry {
    /// Create a new atom registry with predefined atoms
    pub fn new() -> Self {
        let mut registry = Self {
            atoms: HashMap::new(),
            name_to_xid: HashMap::new(),
            next_xid: 69, // Start after predefined atoms
        };

        // Register all predefined atoms
        registry.initialize_predefined_atoms();

        registry
    }

    /// Initialize predefined atoms
    fn initialize_predefined_atoms(&mut self) {
        for (xid, name) in PredefinedAtom::all() {
            let predefined = PredefinedAtom::try_from(xid as u8).unwrap();
            let atom = AtomResource::new_predefined(predefined);
            self.name_to_xid.insert(name.to_string(), xid);
            self.atoms.insert(xid, atom);
        }
    }

    pub fn intern_atom(
        &mut self,
        name: &str,
        only_if_exists: bool,
        client: ClientId,
    ) -> Option<XID> {
        tracing::trace!(
            "intern_atom: name='{}', only_if_exists={}, client={:?}",
            name,
            only_if_exists,
            client
        );

        // Check if atom already exists
        if let Some(&xid) = self.name_to_xid.get(name) {
            tracing::trace!("Atom '{}' exists with xid={}", name, xid);
            // Atom exists, add reference
            if let Some(atom) = self.atoms.get_mut(&xid) {
                atom.add_reference(client);
                tracing::trace!(
                    "Added reference to atom xid={} for client {:?}",
                    xid,
                    client
                );
                return Some(xid);
            } else {
                tracing::trace!(
                    "Inconsistent state: xid {} in name_to_xid but not in atoms!",
                    xid
                );
            }
        }

        // Atom doesn't exist
        if only_if_exists {
            tracing::trace!(
                "Atom '{}' not found and only_if_exists=true, returning None",
                name
            );
            return None; // Don't create new atom
        }

        // Create new atom
        let xid = self.next_xid;
        self.next_xid += 1;
        tracing::trace!("Creating new atom: name='{}', xid={}", name, xid);

        match AtomResource::new(xid, name.to_string(), client) {
            Ok(atom) => {
                self.name_to_xid.insert(name.to_string(), xid);
                self.atoms.insert(xid, atom);
                tracing::trace!("Inserted new atom: name='{}', xid={}", name, xid);
                Some(xid)
            }
            Err(e) => {
                tracing::trace!("Failed to create atom: name='{}', error={:?}", name, e);
                None
            }
        }
    }

    /// Get atom name by XID
    pub fn get_atom_name(&self, xid: XID) -> Option<&str> {
        self.atoms.get(&xid).map(|atom| atom.name())
    }

    /// Get atom by XID
    pub fn get_atom(&self, xid: XID) -> Option<&AtomResource> {
        self.atoms.get(&xid)
    }

    /// Get mutable atom by XID
    pub fn get_atom_mut(&mut self, xid: XID) -> Option<&mut AtomResource> {
        self.atoms.get_mut(&xid)
    }

    /// Remove all references from a client
    pub fn remove_client_references(&mut self, client: ClientId) -> u32 {
        let mut total_removed = 0;
        let mut atoms_to_delete = Vec::new();

        for (xid, atom) in &mut self.atoms {
            total_removed += atom.remove_client_references(client);

            // Mark for deletion if it can be deleted
            if atom.can_be_deleted() {
                atoms_to_delete.push(*xid);
            }
        }

        // Delete unreferenced custom atoms
        for xid in atoms_to_delete {
            if let Some(atom) = self.atoms.remove(&xid) {
                self.name_to_xid.remove(&atom.name);
            }
        }

        total_removed
    }

    /// Get all atoms
    pub fn all_atoms(&self) -> Vec<(XID, &str)> {
        self.atoms
            .iter()
            .map(|(xid, atom)| (*xid, atom.name()))
            .collect()
    }

    /// Get atoms by client
    pub fn get_client_atoms(&self, client: ClientId) -> Vec<XID> {
        self.atoms
            .iter()
            .filter(|(_, atom)| atom.referencing_clients().contains(&client))
            .map(|(xid, _)| *xid)
            .collect()
    }
}

impl TryFrom<u8> for PredefinedAtom {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(PredefinedAtom::Primary),
            2 => Ok(PredefinedAtom::Secondary),
            3 => Ok(PredefinedAtom::Arc),
            4 => Ok(PredefinedAtom::Atom),
            5 => Ok(PredefinedAtom::Bitmap),
            6 => Ok(PredefinedAtom::Cardinal),
            7 => Ok(PredefinedAtom::Colormap),
            8 => Ok(PredefinedAtom::Cursor),
            9 => Ok(PredefinedAtom::CutBuffer0),
            10 => Ok(PredefinedAtom::CutBuffer1),
            11 => Ok(PredefinedAtom::CutBuffer2),
            12 => Ok(PredefinedAtom::CutBuffer3),
            13 => Ok(PredefinedAtom::CutBuffer4),
            14 => Ok(PredefinedAtom::CutBuffer5),
            15 => Ok(PredefinedAtom::CutBuffer6),
            16 => Ok(PredefinedAtom::CutBuffer7),
            17 => Ok(PredefinedAtom::Drawable),
            18 => Ok(PredefinedAtom::Font),
            19 => Ok(PredefinedAtom::Integer),
            20 => Ok(PredefinedAtom::Pixmap),
            21 => Ok(PredefinedAtom::Point),
            22 => Ok(PredefinedAtom::Rectangle),
            23 => Ok(PredefinedAtom::ResourceManager),
            24 => Ok(PredefinedAtom::RgbColorMap),
            25 => Ok(PredefinedAtom::RgbBestMap),
            26 => Ok(PredefinedAtom::RgbBlueMap),
            27 => Ok(PredefinedAtom::RgbDefaultMap),
            28 => Ok(PredefinedAtom::RgbGrayMap),
            29 => Ok(PredefinedAtom::RgbGreenMap),
            30 => Ok(PredefinedAtom::RgbRedMap),
            31 => Ok(PredefinedAtom::String),
            32 => Ok(PredefinedAtom::VisualId),
            33 => Ok(PredefinedAtom::Window),
            34 => Ok(PredefinedAtom::WmCommand),
            35 => Ok(PredefinedAtom::WmHints),
            36 => Ok(PredefinedAtom::WmClientMachine),
            37 => Ok(PredefinedAtom::WmIconName),
            38 => Ok(PredefinedAtom::WmIconSize),
            39 => Ok(PredefinedAtom::WmName),
            40 => Ok(PredefinedAtom::WmNormalHints),
            41 => Ok(PredefinedAtom::WmSizeHints),
            42 => Ok(PredefinedAtom::WmZoomHints),
            43 => Ok(PredefinedAtom::MinSpace),
            44 => Ok(PredefinedAtom::NormSpace),
            45 => Ok(PredefinedAtom::MaxSpace),
            46 => Ok(PredefinedAtom::EndSpace),
            47 => Ok(PredefinedAtom::SuperscriptX),
            48 => Ok(PredefinedAtom::SuperscriptY),
            49 => Ok(PredefinedAtom::SubscriptX),
            50 => Ok(PredefinedAtom::SubscriptY),
            51 => Ok(PredefinedAtom::UnderlinePosition),
            52 => Ok(PredefinedAtom::UnderlineThickness),
            53 => Ok(PredefinedAtom::StrikeoutAscent),
            54 => Ok(PredefinedAtom::StrikeoutDescent),
            55 => Ok(PredefinedAtom::ItalicAngle),
            56 => Ok(PredefinedAtom::XHeight),
            57 => Ok(PredefinedAtom::QuadWidth),
            58 => Ok(PredefinedAtom::Weight),
            59 => Ok(PredefinedAtom::PointSize),
            60 => Ok(PredefinedAtom::Resolution),
            61 => Ok(PredefinedAtom::Copyright),
            62 => Ok(PredefinedAtom::Notice),
            63 => Ok(PredefinedAtom::FontName),
            64 => Ok(PredefinedAtom::FamilyName),
            65 => Ok(PredefinedAtom::FullName),
            66 => Ok(PredefinedAtom::CapHeight),
            67 => Ok(PredefinedAtom::WmClass),
            68 => Ok(PredefinedAtom::WmTransientFor),
            _ => Err(()),
        }
    }
}

impl Resource for AtomResource {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Atom
    }

    fn xid(&self) -> XID {
        self.xid
    }

    fn owner(&self) -> ClientId {
        // Atoms don't have traditional ownership like other resources
        // Return the original client that interned it, or 0 for predefined atoms
        self.original_client
    }

    fn accessible_by(&self, _client: ClientId) -> bool {
        // Atoms are globally accessible by all clients
        true
    }

    fn prepare_destroy(&mut self) -> Result<(), LifecycleError> {
        // Predefined atoms cannot be destroyed
        if self.is_predefined {
            return Err(LifecycleError::FinalizationFailed(
                "Cannot destroy predefined atom".into(),
            ));
        }

        // Atoms with references cannot be destroyed
        if self.is_referenced() {
            return Err(LifecycleError::FinalizationFailed(
                "Cannot destroy atom with active references".into(),
            ));
        }

        // Clear usage data
        self.usage.referencing_clients.clear();
        self.usage.reference_count = 0;

        Ok(())
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
    fn test_predefined_atom() {
        let atom = AtomResource::new_predefined(PredefinedAtom::Primary);
        assert_eq!(atom.xid(), PredefinedAtom::Primary as XID);
        assert_eq!(atom.name(), "PRIMARY");
        assert!(atom.is_predefined());
        assert_eq!(atom.resource_type(), ResourceType::Atom);
        assert!(!atom.can_be_deleted());
    }

    #[test]
    fn test_custom_atom() {
        let atom = AtomResource::new(100, "MY_CUSTOM_ATOM".to_string(), 1).unwrap();
        assert_eq!(atom.xid(), 100);
        assert_eq!(atom.name(), "MY_CUSTOM_ATOM");
        assert!(!atom.is_predefined());
        assert_eq!(atom.original_client(), 1);
        assert_eq!(atom.reference_count(), 1);
    }

    #[test]
    fn test_atom_invalid_name() {
        let result = AtomResource::new(100, "".to_string(), 1);
        assert!(result.is_err());

        let result = AtomResource::new(100, "x".repeat(256), 1);
        assert!(result.is_err());

        let result = AtomResource::new(100, "test\0atom".to_string(), 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_atom_references() {
        let mut atom = AtomResource::new(100, "TEST_ATOM".to_string(), 1).unwrap();
        assert_eq!(atom.reference_count(), 1);
        assert!(atom.referencing_clients().contains(&1));

        atom.add_reference(2);
        assert_eq!(atom.reference_count(), 2);
        assert!(atom.referencing_clients().contains(&2));

        atom.remove_reference(1);
        assert_eq!(atom.reference_count(), 1);
        assert!(!atom.referencing_clients().contains(&1));
        assert!(atom.referencing_clients().contains(&2));
    }

    #[test]
    fn test_atom_registry() {
        let mut registry = AtomRegistry::new();

        // Test predefined atom
        let primary_xid = registry.intern_atom("PRIMARY", true, 1).unwrap();
        assert_eq!(primary_xid, PredefinedAtom::Primary as XID);

        // Test new atom creation
        let custom_xid = registry.intern_atom("CUSTOM_ATOM", false, 1).unwrap();
        assert!(custom_xid > 68); // Should be after predefined atoms

        // Test existing atom lookup
        let same_xid = registry.intern_atom("CUSTOM_ATOM", false, 2).unwrap();
        assert_eq!(custom_xid, same_xid);

        // Test name lookup
        assert_eq!(registry.get_atom_name(primary_xid), Some("PRIMARY"));
        assert_eq!(registry.get_atom_name(custom_xid), Some("CUSTOM_ATOM"));
    }

    #[test]
    fn test_atom_registry_only_if_exists() {
        let mut registry = AtomRegistry::new();

        // Should return None for non-existent atom with only_if_exists=true
        assert!(registry.intern_atom("NON_EXISTENT", true, 1).is_none());

        // Should return Some for predefined atom with only_if_exists=true
        assert!(registry.intern_atom("PRIMARY", true, 1).is_some());
    }

    #[test]
    fn test_client_atom_cleanup() {
        let mut registry = AtomRegistry::new();

        let atom1 = registry.intern_atom("CLIENT1_ATOM", false, 1).unwrap();
        let atom2 = registry.intern_atom("CLIENT2_ATOM", false, 2).unwrap();
        let shared = registry.intern_atom("SHARED_ATOM", false, 1).unwrap();
        registry.intern_atom("SHARED_ATOM", false, 2); // Add reference from client 2

        let removed = registry.remove_client_references(1);
        assert!(removed >= 2); // At least 2 references removed

        // Client 1's exclusive atom should be deleted
        assert!(registry.get_atom_name(atom1).is_none());

        // Client 2's atom should still exist
        assert!(registry.get_atom_name(atom2).is_some());

        // Shared atom should still exist (client 2 still references it)
        assert!(registry.get_atom_name(shared).is_some());
    }

    #[test]
    fn test_predefined_atom_names() {
        assert_eq!(PredefinedAtom::Primary.name(), "PRIMARY");
        assert_eq!(PredefinedAtom::WmName.name(), "WM_NAME");
        assert_eq!(PredefinedAtom::String.name(), "STRING");
    }
}
