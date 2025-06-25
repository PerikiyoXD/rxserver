//! Server and client state management for the X11 server

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tracing::{debug, trace};

use crate::protocol::{Atom, ByteOrder, SequenceNumber, WindowId, XId};

/// Global server state shared across all connections
#[derive(Debug)]
pub struct ServerState {
    /// Registry of atoms (string -> ID mapping)
    pub atom_registry: HashMap<String, Atom>,
    /// Next atom ID to allocate
    pub next_atom_id: Atom,
    /// Global window registry
    pub windows: HashMap<WindowId, WindowState>,
    /// Root window ID
    pub root_window_id: WindowId,
    /// Next resource ID to allocate
    pub next_resource_id: XId,
    /// Connected clients
    pub clients: HashMap<ClientId, Arc<Mutex<ClientState>>>,
    /// Next client ID
    pub next_client_id: ClientId,
}

/// Unique identifier for a client connection
pub type ClientId = u32;

/// State for an individual client connection
#[derive(Debug)]
pub struct ClientState {
    /// Client ID
    pub id: ClientId,
    /// Whether this client has completed connection setup
    pub is_authenticated: bool,
    /// Client's resource ID base (from connection setup)
    pub resource_id_base: XId,
    /// Client's resource ID mask (from connection setup)
    pub resource_id_mask: XId,
    /// Current sequence number for this client
    pub sequence_number: SequenceNumber,
    /// Client's preferred byte order
    pub byte_order: ByteOrder,
    /// Client's socket address
    pub address: SocketAddr,
    /// Resources owned by this client
    pub owned_resources: Vec<XId>,
}

/// Window state information
#[derive(Debug, Clone)]
pub struct WindowState {
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

/// X11 window class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum WindowClass {
    CopyFromParent = 0,
    InputOutput = 1,
    InputOnly = 2,
}

impl Default for ServerState {
    fn default() -> Self {
        let mut atom_registry = HashMap::new();
        let mut next_atom_id = 1;

        // Pre-populate with standard atoms
        for (name, _) in PREDEFINED_ATOMS {
            atom_registry.insert(name.to_string(), next_atom_id);
            next_atom_id += 1;
        }

        let root_window_id = 1;
        let mut windows = HashMap::new();

        // Create root window
        windows.insert(
            root_window_id,
            WindowState {
                id: root_window_id,
                parent: None,
                x: 0,
                y: 0,
                width: 1024,
                height: 768,
                border_width: 0,
                depth: 24,
                class: WindowClass::InputOutput,
                owner: None, // Root window has no owner
            },
        );

        Self {
            atom_registry,
            next_atom_id,
            windows,
            root_window_id,
            next_resource_id: 0x00400000, // Start at a reasonable base
            clients: HashMap::new(),
            next_client_id: 1,
        }
    }
}

impl ServerState {
    /// Create a new server state
    pub fn new() -> Arc<Mutex<Self>> {
        trace!("Creating new server state");
        let state = Arc::new(Mutex::new(Self::default()));
        debug!(
            "Server state created with {} predefined atoms",
            PREDEFINED_ATOMS.len()
        );
        state
    }

    /// Register a new client
    pub fn register_client(&mut self, address: SocketAddr) -> (ClientId, Arc<Mutex<ClientState>>) {
        let client_id = self.next_client_id;
        self.next_client_id += 1;

        let client_state = Arc::new(Mutex::new(ClientState {
            id: client_id,
            is_authenticated: false,
            resource_id_base: 0,
            resource_id_mask: 0,
            sequence_number: 0,
            byte_order: ByteOrder::LittleEndian,
            address,
            owned_resources: Vec::new(),
        }));

        self.clients.insert(client_id, client_state.clone());
        debug!("Registered new client {} from {}", client_id, address);
        (client_id, client_state)
    }

    /// Remove a client
    pub fn unregister_client(&mut self, client_id: ClientId) {
        if let Some(client_state) = self.clients.remove(&client_id) {
            // Clean up client resources
            if let Ok(client) = client_state.lock() {
                debug!(
                    "Cleaning up {} resources for client {}",
                    client.owned_resources.len(),
                    client_id
                );
                for resource_id in &client.owned_resources {
                    // Remove windows owned by this client
                    self.windows
                        .retain(|_, window| window.owner != Some(client_id));
                }
            }
            debug!("Unregistered client {}", client_id);
        } else {
            trace!("Attempted to unregister non-existent client {}", client_id);
        }
    }

    /// Get or create an atom
    pub fn intern_atom(&mut self, name: &str, only_if_exists: bool) -> Option<Atom> {
        if let Some(&atom_id) = self.atom_registry.get(name) {
            trace!("Found existing atom '{}' with ID {}", name, atom_id);
            Some(atom_id)
        } else if !only_if_exists {
            let atom_id = self.next_atom_id;
            self.next_atom_id += 1;
            self.atom_registry.insert(name.to_string(), atom_id);
            debug!("Created new atom '{}' with ID {}", name, atom_id);
            Some(atom_id)
        } else {
            trace!("Atom '{}' not found and only_if_exists=true", name);
            None
        }
    }

    /// Get atom name by ID
    pub fn get_atom_name(&self, atom_id: Atom) -> Option<&String> {
        self.atom_registry
            .iter()
            .find(|(_, id)| **id == atom_id)
            .map(|(name, _)| name)
    }

    /// Create a new window
    pub fn create_window(
        &mut self,
        window_id: WindowId,
        parent: WindowId,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        depth: u8,
        class: WindowClass,
        owner: ClientId,
    ) -> Result<(), &'static str> {
        if self.windows.contains_key(&window_id) {
            debug!("Window creation failed: ID {} already exists", window_id);
            return Err("Window ID already exists");
        }

        if !self.windows.contains_key(&parent) {
            debug!("Window creation failed: parent {} does not exist", parent);
            return Err("Parent window does not exist");
        }

        let window = WindowState {
            id: window_id,
            parent: Some(parent),
            x,
            y,
            width,
            height,
            border_width,
            depth,
            class,
            owner: Some(owner),
        };

        self.windows.insert(window_id, window);
        debug!(
            "Created window {} ({}x{} at {},{}) for client {} with parent {}",
            window_id, width, height, x, y, owner, parent
        );
        Ok(())
    }

    /// Get window state
    pub fn get_window(&self, window_id: WindowId) -> Option<&WindowState> {
        self.windows.get(&window_id)
    }

    /// Get the root window
    pub fn get_root_window(&self) -> &WindowState {
        self.windows
            .get(&self.root_window_id)
            .expect("Root window should always exist")
    }
}

impl ClientState {
    /// Mark client as authenticated after successful connection setup
    pub fn authenticate(
        &mut self,
        resource_id_base: XId,
        resource_id_mask: XId,
        byte_order: ByteOrder,
    ) {
        self.is_authenticated = true;
        self.resource_id_base = resource_id_base;
        self.resource_id_mask = resource_id_mask;
        self.byte_order = byte_order;
        debug!(
            "Client {} authenticated with resource base 0x{:08x}, mask 0x{:08x}",
            self.id, resource_id_base, resource_id_mask
        );
    }

    /// Get next sequence number for this client
    pub fn next_sequence_number(&mut self) -> SequenceNumber {
        let seq = self.sequence_number;
        self.sequence_number = self.sequence_number.wrapping_add(1);
        seq
    }

    /// Check if a resource ID belongs to this client
    pub fn owns_resource(&self, resource_id: XId) -> bool {
        (resource_id & self.resource_id_mask) == self.resource_id_base
    }
}

/// Predefined atoms as per X11 specification
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_server_state_creation() {
        let server_state = ServerState::new();
        let state = server_state.lock().unwrap();

        // Should have predefined atoms
        assert!(state.atom_registry.len() > 0);
        assert_eq!(state.atom_registry.get("PRIMARY"), Some(&1));

        // Should have root window
        assert_eq!(state.root_window_id, 1);
        assert!(state.windows.contains_key(&1));
    }

    #[test]
    fn test_client_registration() {
        let server_state = ServerState::new();
        let mut state = server_state.lock().unwrap();

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12345);
        let (client_id, client_state) = state.register_client(addr);

        assert_eq!(client_id, 1);
        assert!(state.clients.contains_key(&client_id));

        let client = client_state.lock().unwrap();
        assert_eq!(client.id, client_id);
        assert!(!client.is_authenticated);
    }

    #[test]
    fn test_atom_interning() {
        let server_state = ServerState::new();
        let mut state = server_state.lock().unwrap();

        // Test creating new atom
        let atom_id = state.intern_atom("TEST_ATOM", false).unwrap();
        assert!(atom_id > 0);

        // Test getting existing atom
        let same_atom_id = state.intern_atom("TEST_ATOM", false).unwrap();
        assert_eq!(atom_id, same_atom_id);

        // Test only_if_exists = true for non-existent atom
        let non_existent = state.intern_atom("NON_EXISTENT", true);
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_window_creation() {
        let server_state = ServerState::new();
        let mut state = server_state.lock().unwrap();

        let result = state.create_window(
            2, // window_id
            1, // parent (root)
            10,
            20, // x, y
            100,
            200, // width, height
            1,   // border_width
            24,  // depth
            WindowClass::InputOutput,
            1, // owner client_id
        );

        assert!(result.is_ok());

        let window = state.get_window(2).unwrap();
        assert_eq!(window.id, 2);
        assert_eq!(window.parent, Some(1));
        assert_eq!(window.width, 100);
        assert_eq!(window.height, 200);
    }
}
