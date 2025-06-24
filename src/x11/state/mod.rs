//! X11 Server State Management
//!
//! This module manages the global state of the X11 server including windows,
//! clients, resources, and other server-wide state.

use crate::types::Result;
use std::collections::HashMap;

/// Global X11 server state
#[derive(Debug, Default)]
pub struct X11ServerState {
    /// Connected clients
    pub clients: ClientManager,
    /// Window hierarchy
    pub windows: WindowManager,
    /// Resource registry
    pub resources: ResourceStateTracker,
    /// Next available resource ID
    next_resource_id: u32,
}

impl X11ServerState {
    /// Create new server state
    pub fn new() -> Self {
        Self {
            clients: ClientManager::new(),
            windows: WindowManager::new(),
            resources: ResourceStateTracker::new(),
            next_resource_id: 1,
        }
    }

    /// Allocate a new resource ID
    pub fn allocate_resource_id(&mut self) -> u32 {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        id
    }

    /// Get the root window ID
    pub fn root_window(&self) -> u32 {
        // Root window always has ID 0 or a fixed ID
        0
    }
}

/// Client connection management
#[derive(Debug, Default)]
pub struct ClientManager {
    /// Connected clients
    clients: HashMap<u32, ClientInfo>,
    /// Next client ID
    next_client_id: u32,
}

impl ClientManager {
    /// Create new client manager
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            next_client_id: 1,
        }
    }

    /// Register a new client
    pub fn register_client(&mut self) -> u32 {
        let client_id = self.next_client_id;
        self.next_client_id += 1;

        let client_info = ClientInfo {
            id: client_id,
            windows: Vec::new(),
        };

        self.clients.insert(client_id, client_info);
        client_id
    }

    /// Remove a client
    pub fn remove_client(&mut self, client_id: u32) -> Option<ClientInfo> {
        self.clients.remove(&client_id)
    }

    /// Get client info
    pub fn get_client(&self, client_id: u32) -> Option<&ClientInfo> {
        self.clients.get(&client_id)
    }

    /// Get mutable client info
    pub fn get_client_mut(&mut self, client_id: u32) -> Option<&mut ClientInfo> {
        self.clients.get_mut(&client_id)
    }
}

/// Information about a connected client
#[derive(Debug, Clone)]
pub struct ClientInfo {
    /// Client ID
    pub id: u32,
    /// Windows owned by this client
    pub windows: Vec<u32>,
}

/// Window hierarchy management
#[derive(Debug, Default)]
pub struct WindowManager {
    /// Window information
    windows: HashMap<u32, WindowInfo>,
    /// Root window ID
    root_window: Option<u32>,
}

impl WindowManager {
    /// Create new window manager
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            root_window: None,
        }
    }

    /// Create a new window
    pub fn create_window(
        &mut self,
        window_id: u32,
        parent: u32,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    ) -> Result<()> {
        let window = WindowInfo {
            id: window_id,
            parent,
            x,
            y,
            width,
            height,
            mapped: false,
            children: Vec::new(),
        };

        self.windows.insert(window_id, window);

        // Add to parent's children if not root
        if parent != 0 {
            if let Some(parent_window) = self.windows.get_mut(&parent) {
                parent_window.children.push(window_id);
            }
        }

        Ok(())
    }

    /// Get window info
    pub fn get_window(&self, window_id: u32) -> Option<&WindowInfo> {
        self.windows.get(&window_id)
    }

    /// Get mutable window info
    pub fn get_window_mut(&mut self, window_id: u32) -> Option<&mut WindowInfo> {
        self.windows.get_mut(&window_id)
    }

    /// Map a window
    pub fn map_window(&mut self, window_id: u32) -> Result<()> {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.mapped = true;
            Ok(())
        } else {
            Err(crate::types::Error::Internal(
                "Window not found".to_string(),
            ))
        }
    }

    /// Unmap a window
    pub fn unmap_window(&mut self, window_id: u32) -> Result<()> {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.mapped = false;
            Ok(())
        } else {
            Err(crate::types::Error::Internal(
                "Window not found".to_string(),
            ))
        }
    }
}

/// Information about a window
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// Window ID
    pub id: u32,
    /// Parent window ID
    pub parent: u32,
    /// X position
    pub x: i16,
    /// Y position
    pub y: i16,
    /// Width
    pub width: u16,
    /// Height
    pub height: u16,
    /// Whether window is mapped
    pub mapped: bool,
    /// Child windows
    pub children: Vec<u32>,
}

/// Basic resource state tracking
#[derive(Debug, Default)]
pub struct ResourceStateTracker {
    /// Resource information
    resources: HashMap<u32, ResourceInfo>,
}

impl ResourceStateTracker {
    /// Create new resource manager
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Register a resource
    pub fn register_resource(
        &mut self,
        resource_id: u32,
        client_id: u32,
        resource_type: ResourceType,
    ) {
        let resource = ResourceInfo {
            id: resource_id,
            client_id,
            resource_type,
        };
        self.resources.insert(resource_id, resource);
    }

    /// Get resource info
    pub fn get_resource(&self, resource_id: u32) -> Option<&ResourceInfo> {
        self.resources.get(&resource_id)
    }

    /// Remove a resource
    pub fn remove_resource(&mut self, resource_id: u32) -> Option<ResourceInfo> {
        self.resources.remove(&resource_id)
    }
}

// Re-export ResourceType from resources module
pub use crate::x11::resources::ResourceType;

/// Information about a resource
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    /// Resource ID
    pub id: u32,
    /// Owning client ID
    pub client_id: u32,
    /// Resource type
    pub resource_type: ResourceType,
}
