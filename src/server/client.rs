//! Client Management
//!
//! This module handles the lifecycle and management of X11 clients.

use dashmap::DashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::protocol::types::{ResourceId, Window};
use crate::server::ServerEvent;
use crate::{todo_high, Error, Result};

/// Manages all connected X11 clients
pub struct ClientManager {
    /// Map of client ID to client info
    clients: DashMap<u32, ClientInfo>,
    /// Next client ID to assign
    next_client_id: AtomicU32,
    /// Event broadcaster
    event_sender: Arc<broadcast::Sender<ServerEvent>>,
}

/// Information about a connected client
#[derive(Debug, Clone)]
pub struct ClientInfo {
    /// Unique client ID
    pub id: u32,
    /// Client name/identification
    pub name: String,
    /// Process ID (if available)
    pub pid: Option<u32>,
    /// When the client connected
    pub connected_at: std::time::Instant,
    /// Windows owned by this client
    pub windows: Vec<Window>,
    /// Whether the client is still active
    pub active: bool,
    /// Resource ID base for this client
    pub resource_base: ResourceId,
    /// Resource ID mask for this client
    pub resource_mask: ResourceId,
}

impl ClientManager {
    /// Create a new client manager
    pub fn new(event_sender: broadcast::Sender<ServerEvent>) -> Self {
        Self {
            clients: DashMap::new(),
            next_client_id: AtomicU32::new(1),
            event_sender: Arc::new(event_sender),
        }
    }

    /// Register a new client
    pub async fn register_client(&self, name: String, pid: Option<u32>) -> Result<u32> {
        let client_id = self.next_client_id.fetch_add(1, Ordering::SeqCst);

        // Calculate resource base and mask for this client
        // Each client gets a unique range of resource IDs
        let resource_base = (client_id << 20) as ResourceId;
        let resource_mask = 0x000FFFFF; // 20 bits for resources per client

        let client_info = ClientInfo {
            id: client_id,
            name: name.clone(),
            pid,
            connected_at: std::time::Instant::now(),
            windows: Vec::new(),
            active: true,
            resource_base,
            resource_mask,
        };

        self.clients.insert(client_id, client_info);

        info!("Client {} registered: {} (PID: {:?})", client_id, name, pid);
        // Broadcast client connected event
        let _ = self.event_sender.send(ServerEvent::ClientConnected {
            client_id,
            address: "unknown".to_string(), // TODO: Get actual client address
        });

        Ok(client_id)
    }

    /// Unregister a client
    pub async fn unregister_client(&self, client_id: u32) -> Result<()> {
        if let Some((_, client_info)) = self.clients.remove(&client_id) {
            info!("Client {} disconnected: {}", client_id, client_info.name);

            // Broadcast client disconnected event
            let _ = self
                .event_sender
                .send(ServerEvent::ClientDisconnected { client_id });

            Ok(())
        } else {
            warn!("Attempted to unregister unknown client: {}", client_id);
            Err(Error::Server(format!("Unknown client: {}", client_id)))
        }
    }

    /// Get client information
    pub async fn get_client(&self, client_id: u32) -> Option<ClientInfo> {
        self.clients.get(&client_id).map(|entry| entry.clone())
    }

    /// Add a window to a client
    pub async fn add_window(&self, client_id: u32, window: Window) -> Result<()> {
        if let Some(mut client) = self.clients.get_mut(&client_id) {
            client.windows.push(window);
            debug!("Added window {} to client {}", window, client_id);
            Ok(())
        } else {
            Err(Error::Server(format!("Unknown client: {}", client_id)))
        }
    }

    /// Remove a window from a client
    pub async fn remove_window(&self, client_id: u32, window: Window) -> Result<()> {
        if let Some(mut client) = self.clients.get_mut(&client_id) {
            client.windows.retain(|&w| w != window);
            debug!("Removed window {} from client {}", window, client_id);
            Ok(())
        } else {
            Err(Error::Server(format!("Unknown client: {}", client_id)))
        }
    }

    /// Find the client that owns a window
    pub async fn find_window_owner(&self, window: Window) -> Option<u32> {
        for entry in self.clients.iter() {
            if entry.windows.contains(&window) {
                return Some(entry.id);
            }
        }
        None
    }

    /// Get the number of connected clients
    pub async fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Get all client IDs
    pub async fn client_ids(&self) -> Vec<u32> {
        self.clients.iter().map(|entry| entry.id).collect()
    }

    /// Check if a resource ID belongs to a client
    pub async fn validate_resource(&self, client_id: u32, resource_id: ResourceId) -> bool {
        if let Some(client) = self.clients.get(&client_id) {
            let masked_id = resource_id & client.resource_mask;
            masked_id != 0 && (resource_id & !client.resource_mask) == client.resource_base
        } else {
            false
        }
    }

    /// Generate a new resource ID for a client
    pub async fn allocate_resource_id(&self, client_id: u32) -> Option<ResourceId> {
        if let Some(client) = self.clients.get(&client_id) {
            debug!("Allocating resource ID for client {}", client_id);
            todo_high!(
                "client_manager",
                "Proper resource ID allocation logic not implemented - using simple increment"
            );
            let base_id = client.resource_base;
            // TODO: IMPLEMENT PROPER ALLOCATION LOGIC
            Some(base_id | 1)
        } else {
            None
        }
    }

    /// Estimate memory usage
    pub async fn memory_usage(&self) -> usize {
        // Estimate memory usage of all clients and their windows
        self.clients
            .iter()
            .map(|entry| {
                let client = entry.value();
                std::mem::size_of_val(client)
                    + client.windows.capacity() * std::mem::size_of::<Window>()
                    + client.name.capacity()
            })
            .sum()
    }

    /// Get all clients for debugging
    pub async fn debug_clients(&self) -> Vec<ClientInfo> {
        self.clients.iter().map(|entry| entry.clone()).collect()
    }
}

impl ClientInfo {
    /// Get client session duration
    pub fn session_duration(&self) -> std::time::Duration {
        self.connected_at.elapsed()
    }

    /// Get number of windows owned by this client
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }
}
