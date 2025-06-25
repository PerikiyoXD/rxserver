//! XId Allocation and Tracking
//!
//! This module implements the X11 resource identifier (XId) allocation system
//! following the X11 protocol specification.

use crate::x11::protocol::types::{ClientId, XId};
use std::collections::{BinaryHeap, HashMap};
use std::ops::Range;

/// Errors that can occur during XId allocation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocationError {
    /// No more XIDs available for the client
    Exhausted(ClientId),
    /// Invalid XId mask configuration
    InvalidMask,
    /// Client not configured for allocation
    ClientNotConfigured(ClientId),
    /// XId already allocated
    AlreadyAllocated(XId),
}

impl std::fmt::Display for AllocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AllocationError::Exhausted(client) => {
                write!(f, "No more XIDs available for client {}", client)
            }
            AllocationError::InvalidMask => {
                write!(f, "Invalid XId mask configuration")
            }
            AllocationError::ClientNotConfigured(client) => {
                write!(f, "Client {} not configured for XId allocation", client)
            }
            AllocationError::AlreadyAllocated(xid) => {
                write!(f, "XId {} is already allocated", xid)
            }
        }
    }
}

impl std::error::Error for AllocationError {}

/// XId allocator implementation
#[derive(Debug)]
pub struct XIDAllocator {
    /// Base XId value (server's resource base)
    base: XId,
    /// Mask for client XIDs (defines client bit range)
    mask: XId,
    /// Next XId to try for each client
    client_next: HashMap<ClientId, XId>,
    /// Freed XIDs that can be reused
    freed: BinaryHeap<XId>,
    /// Currently allocated XIDs
    allocated: HashMap<XId, ClientId>,
    /// Per-client XId ranges
    client_ranges: HashMap<ClientId, Range<XId>>,
}

impl XIDAllocator {
    /// Create a new XId allocator
    ///
    /// # Arguments
    /// * `base` - Base XId value for this server
    /// * `mask` - XId mask defining the client ID bits
    pub fn new(base: XId, mask: XId) -> Self {
        Self {
            base,
            mask,
            client_next: HashMap::new(),
            freed: BinaryHeap::new(),
            allocated: HashMap::new(),
            client_ranges: HashMap::new(),
        }
    }

    /// Configure a client for XId allocation
    pub fn configure_client(&mut self, client: ClientId) -> Result<(), AllocationError> {
        if self.mask == 0 {
            return Err(AllocationError::InvalidMask);
        }

        // Calculate the client's XId range
        let client_base = self.base | ((client & self.mask) << self.mask.trailing_zeros());
        let client_limit = client_base | self.mask;

        let range = client_base..client_limit + 1;

        self.client_ranges.insert(client, range.clone());
        self.client_next.insert(client, range.start);

        Ok(())
    }

    /// Allocate a new XId for the specified client
    pub fn allocate_for_client(&mut self, client: ClientId) -> Result<XId, AllocationError> {
        // Ensure client is configured
        if !self.client_ranges.contains_key(&client) {
            self.configure_client(client)?;
        }

        // Try to reuse a freed XId first
        while let Some(freed_xid) = self.freed.pop() {
            if self.is_valid_for_client(freed_xid, client)
                && !self.allocated.contains_key(&freed_xid)
            {
                self.allocated.insert(freed_xid, client);
                return Ok(freed_xid);
            }
        }

        // Allocate a new XId
        let range = self
            .client_ranges
            .get(&client)
            .ok_or(AllocationError::ClientNotConfigured(client))?
            .clone();

        let start_xid = self
            .client_next
            .get(&client)
            .copied()
            .unwrap_or(range.start);

        // Search for an available XId
        for xid in (start_xid..range.end).chain(range.start..start_xid) {
            if !self.allocated.contains_key(&xid) {
                self.allocated.insert(xid, client);
                self.client_next.insert(client, xid + 1);
                return Ok(xid);
            }
        }

        Err(AllocationError::Exhausted(client))
    }

    /// Free an allocated XId
    pub fn free(&mut self, xid: XId) {
        if let Some(client) = self.allocated.remove(&xid) {
            // Add to freed list for potential reuse
            self.freed.push(xid);

            // Update client's next pointer if this was the most recent allocation
            if let Some(&next_xid) = self.client_next.get(&client) {
                if xid + 1 == next_xid {
                    self.client_next.insert(client, xid);
                }
            }
        }
    }

    /// Check if an XId is valid for the given client
    pub fn is_valid_for_client(&self, xid: XId, client: ClientId) -> bool {
        if let Some(range) = self.client_ranges.get(&client) {
            range.contains(&xid)
        } else {
            false
        }
    }

    /// Check if an XId is currently allocated
    pub fn is_allocated(&self, xid: XId) -> bool {
        self.allocated.contains_key(&xid)
    }

    /// Get the client that owns the given XId
    pub fn get_owner(&self, xid: XId) -> Option<ClientId> {
        self.allocated.get(&xid).copied()
    }

    /// Get all XIDs allocated to a client
    pub fn get_client_xids(&self, client: ClientId) -> Vec<XId> {
        self.allocated
            .iter()
            .filter_map(
                |(xid, owner)| {
                    if *owner == client { Some(*xid) } else { None }
                },
            )
            .collect()
    }

    /// Free all XIDs for a client
    pub fn free_client_xids(&mut self, client: ClientId) {
        let client_xids: Vec<XId> = self.get_client_xids(client);
        for xid in client_xids {
            self.free(xid);
        }
    }

    /// Get the number of allocated XIDs
    pub fn allocated_count(&self) -> usize {
        self.allocated.len()
    }

    /// Get the number of freed XIDs available for reuse
    pub fn freed_count(&self) -> usize {
        self.freed.len()
    }

    /// Get allocation statistics for a client
    pub fn client_stats(&self, client: ClientId) -> Option<ClientStats> {
        let range = self.client_ranges.get(&client)?.clone();
        let allocated = self.get_client_xids(client).len();
        let total_available = range.len();

        Some(ClientStats {
            client,
            allocated,
            total_available,
            next_xid: self.client_next.get(&client).copied(),
        })
    }

    /// Compact the freed XId list by removing duplicates and invalid entries
    pub fn compact_freed_list(&mut self) {
        let mut new_freed = BinaryHeap::new();

        while let Some(xid) = self.freed.pop() {
            if !self.allocated.contains_key(&xid) {
                new_freed.push(xid);
            }
        }

        self.freed = new_freed;
    }
}

/// Statistics for a client's XId allocation
#[derive(Debug, Clone)]
pub struct ClientStats {
    /// Client identifier
    pub client: ClientId,
    /// Number of currently allocated XIDs
    pub allocated: usize,
    /// Total XIDs available to this client
    pub total_available: usize,
    /// Next XId that will be tried for allocation
    pub next_xid: Option<XId>,
}

impl ClientStats {
    /// Calculate the utilization percentage
    pub fn utilization(&self) -> f64 {
        if self.total_available == 0 {
            0.0
        } else {
            (self.allocated as f64 / self.total_available as f64) * 100.0
        }
    }

    /// Check if the client is close to exhaustion (>90% utilized)
    pub fn is_near_exhaustion(&self) -> bool {
        self.utilization() > 90.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocator_creation() {
        let allocator = XIDAllocator::new(0x00400000, 0x001FFFFF);
        assert_eq!(allocator.allocated_count(), 0);
        assert_eq!(allocator.freed_count(), 0);
    }

    #[test]
    fn test_client_configuration() {
        let mut allocator = XIDAllocator::new(0x00400000, 0x001FFFFF);
        let client_id = 1;

        allocator
            .configure_client(client_id)
            .expect("Should configure client");

        let stats = allocator
            .client_stats(client_id)
            .expect("Should have stats");
        assert_eq!(stats.client, client_id);
        assert_eq!(stats.allocated, 0);
        assert!(stats.total_available > 0);
    }

    #[test]
    fn test_xid_allocation() {
        let mut allocator = XIDAllocator::new(0x00400000, 0x001FFFFF);
        let client_id = 1;

        let xid = allocator
            .allocate_for_client(client_id)
            .expect("Should allocate XId");

        assert!(allocator.is_allocated(xid));
        assert_eq!(allocator.get_owner(xid), Some(client_id));
        assert!(allocator.is_valid_for_client(xid, client_id));
        assert_eq!(allocator.allocated_count(), 1);
    }

    #[test]
    fn test_xid_free_and_reuse() {
        let mut allocator = XIDAllocator::new(0x00400000, 0x001FFFFF);
        let client_id = 1;

        let xid = allocator
            .allocate_for_client(client_id)
            .expect("Should allocate XId");
        allocator.free(xid);

        assert!(!allocator.is_allocated(xid));
        assert_eq!(allocator.freed_count(), 1);

        // Allocating again should reuse the freed XId
        let new_xid = allocator
            .allocate_for_client(client_id)
            .expect("Should allocate XId");
        assert_eq!(xid, new_xid);
        assert_eq!(allocator.freed_count(), 0);
    }

    #[test]
    fn test_client_xid_isolation() {
        let mut allocator = XIDAllocator::new(0x00400000, 0x001FFFFF);
        let client1 = 1;
        let client2 = 2;

        let xid1 = allocator
            .allocate_for_client(client1)
            .expect("Should allocate for client 1");
        let xid2 = allocator
            .allocate_for_client(client2)
            .expect("Should allocate for client 2");

        assert_ne!(xid1, xid2);
        assert!(allocator.is_valid_for_client(xid1, client1));
        assert!(!allocator.is_valid_for_client(xid1, client2));
        assert!(allocator.is_valid_for_client(xid2, client2));
        assert!(!allocator.is_valid_for_client(xid2, client1));
    }

    #[test]
    fn test_client_cleanup() {
        let mut allocator = XIDAllocator::new(0x00400000, 0x001FFFFF);
        let client_id = 1;

        // Allocate multiple XIDs
        for _ in 0..5 {
            allocator
                .allocate_for_client(client_id)
                .expect("Should allocate XId");
        }

        assert_eq!(allocator.get_client_xids(client_id).len(), 5);

        // Free all client XIDs
        allocator.free_client_xids(client_id);

        assert_eq!(allocator.get_client_xids(client_id).len(), 0);
        assert_eq!(allocator.allocated_count(), 0);
        assert_eq!(allocator.freed_count(), 5);
    }
}
