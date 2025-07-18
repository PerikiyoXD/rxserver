// resource_system.rs
use crate::protocol::XId;

/// Manages X11 resource ID allocation
#[derive(Debug)]
pub struct ResourceSystem {
    next_resource_id: XId,
}

impl ResourceSystem {
    pub fn new() -> Self {
        Self {
            next_resource_id: 0x00400000, // Standard X11 server resource base
        }
    }

    /// Allocate a new unique resource ID
    pub fn allocate_resource_id(&mut self) -> XId {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        id
    }

    /// Check if a resource ID is valid (within allocated range)
    pub fn is_valid_resource_id(&self, resource_id: XId) -> bool {
        resource_id < self.next_resource_id && resource_id >= 0x00400000
    }
}
