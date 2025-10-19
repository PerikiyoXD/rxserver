// client_system.rs
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use crate::protocol::{ByteOrder, SequenceNumber, XId};

pub type ClientId = u32;

#[derive(Debug)]
pub struct Client {
    id: ClientId,
    address: SocketAddr,
    is_authenticated: bool,
    byte_order: ByteOrder,
    resource_id_base: XId,
    resource_id_mask: XId,
    sequence_number: SequenceNumber,
    owned_resources: Vec<XId>,
    event_queue: Vec<Vec<u8>>, // Queue of serialized events to send
    big_requests_enabled: bool,
}

/// Manages X11 client connections and authentication
#[derive(Debug)]
pub struct ClientSystem {
    clients: HashMap<ClientId, Arc<Mutex<Client>>>,
    next_client_id: ClientId,
}

impl ClientSystem {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            next_client_id: 1,
        }
    }

    /// Register a new client
    pub fn register_client(&mut self, address: SocketAddr) -> (ClientId, Arc<Mutex<Client>>) {
        let client_id = self.next_client_id;
        self.next_client_id += 1;

        let client_state = Arc::new(Mutex::new(Client::new(client_id, address)));
        self.clients.insert(client_id, client_state.clone());
        debug!("Registered new client {} from {}", client_id, address);
        (client_id, client_state)
    }

    /// Remove a client
    pub fn unregister_client(&mut self, client_id: ClientId) -> Option<Arc<Mutex<Client>>> {
        if let Some(client_state) = self.clients.remove(&client_id) {
            debug!("Unregistered client {}", client_id);
            Some(client_state)
        } else {
            None
        }
    }

    /// Get a client by ID
    pub fn get_client(&self, client_id: ClientId) -> Option<&Arc<Mutex<Client>>> {
        self.clients.get(&client_id)
    }

    /// Get a mutable client by ID
    pub fn get_client_mut(&mut self, client_id: ClientId) -> Option<&mut Arc<Mutex<Client>>> {
        self.clients.get_mut(&client_id)
    }

    /// Get the number of connected clients
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Get all client IDs
    pub fn client_ids(&self) -> Vec<ClientId> {
        self.clients.keys().copied().collect()
    }
}

impl Client {
    fn new(id: ClientId, address: SocketAddr) -> Self {
        Self {
            id,
            is_authenticated: false,
            resource_id_base: 0,
            resource_id_mask: 0,
            sequence_number: 1,
            byte_order: ByteOrder::LittleEndian,
            address,
            owned_resources: Vec::new(),
            event_queue: Vec::new(),
            big_requests_enabled: false,
        }
    }

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
        (resource_id & !self.resource_id_mask) == self.resource_id_base
    }

    // Getters for private fields
    pub fn id(&self) -> ClientId {
        self.id
    }
    pub fn is_authenticated(&self) -> bool {
        self.is_authenticated
    }
    pub fn address(&self) -> SocketAddr {
        self.address
    }
    pub fn byte_order(&self) -> ByteOrder {
        self.byte_order
    }
    pub fn resource_id_base(&self) -> XId {
        self.resource_id_base
    }
    pub fn resource_id_mask(&self) -> XId {
        self.resource_id_mask
    }
    pub fn owned_resources(&self) -> &[XId] {
        &self.owned_resources
    }

    /// Queue an event for this client
    pub fn queue_event(&mut self, event_data: Vec<u8>) {
        self.event_queue.push(event_data);
    }

    /// Get pending events for this client
    pub fn pending_events(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.event_queue)
    }

    /// Check if client has pending events
    pub fn has_pending_events(&self) -> bool {
        !self.event_queue.is_empty()
    }

    /// Enable big requests for this client
    pub fn set_big_requests_enabled(&mut self, enabled: bool) {
        self.big_requests_enabled = enabled;
    }

    /// Check if big requests are enabled for this client
    pub fn big_requests_enabled(&self) -> bool {
        self.big_requests_enabled
    }
}

impl Default for ClientSystem {
    fn default() -> Self {
        Self::new()
    }
}
