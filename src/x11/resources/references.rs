//! Resource reference counting and management
//!
//! This module provides reference counting capabilities for resources,
//! enabling safe sharing and automatic cleanup.

use crate::x11::protocol::types::{ClientId, XID};
use std::collections::HashMap;
use std::sync::{Arc, Weak};

/// A reference to a resource with automatic tracking
#[derive(Debug)]
pub struct ResourceRef<T> {
    /// The actual resource data
    inner: Arc<T>,
    /// XID of the resource
    xid: XID,
    /// Client that holds this reference
    client: ClientId,
}

impl<T> ResourceRef<T> {
    /// Create a new resource reference
    pub fn new(resource: T, xid: XID, client: ClientId) -> Self {
        Self {
            inner: Arc::new(resource),
            xid,
            client,
        }
    }

    /// Get the XID of this resource
    pub fn xid(&self) -> XID {
        self.xid
    }

    /// Get the client that holds this reference
    pub fn client(&self) -> ClientId {
        self.client
    }

    /// Get the reference count
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Create a weak reference to this resource
    pub fn downgrade(&self) -> WeakResourceRef<T> {
        WeakResourceRef {
            inner: Arc::downgrade(&self.inner),
            xid: self.xid,
            client: self.client,
        }
    }

    /// Try to get the resource if it's still valid
    pub fn get(&self) -> &T {
        &self.inner
    }
}

impl<T> Clone for ResourceRef<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            xid: self.xid,
            client: self.client,
        }
    }
}

impl<T> std::ops::Deref for ResourceRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A weak reference to a resource
#[derive(Debug)]
pub struct WeakResourceRef<T> {
    /// Weak reference to the resource data
    inner: Weak<T>,
    /// XID of the resource
    xid: XID,
    /// Client that created this reference
    client: ClientId,
}

impl<T> WeakResourceRef<T> {
    /// Get the XID of this resource
    pub fn xid(&self) -> XID {
        self.xid
    }

    /// Get the client that created this reference
    pub fn client(&self) -> ClientId {
        self.client
    }

    /// Try to upgrade to a strong reference
    pub fn upgrade(&self) -> Option<ResourceRef<T>> {
        self.inner.upgrade().map(|inner| ResourceRef {
            inner,
            xid: self.xid,
            client: self.client,
        })
    }

    /// Check if the resource is still alive
    pub fn is_alive(&self) -> bool {
        self.inner.strong_count() > 0
    }
}

impl<T> Clone for WeakResourceRef<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Weak::clone(&self.inner),
            xid: self.xid,
            client: self.client,
        }
    }
}

/// Reference counter for tracking resource usage
#[derive(Debug, Default)]
pub struct ReferenceCounter {
    /// Reference counts for each resource
    counts: HashMap<XID, usize>,
    /// Client reference tracking
    client_refs: HashMap<ClientId, HashMap<XID, usize>>,
    /// Weak reference tracking
    weak_refs: HashMap<XID, usize>,
}

impl ReferenceCounter {
    /// Create a new reference counter
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a reference to a resource
    pub fn add_ref(&mut self, xid: XID, client: ClientId) {
        *self.counts.entry(xid).or_insert(0) += 1;
        *self
            .client_refs
            .entry(client)
            .or_insert_with(HashMap::new)
            .entry(xid)
            .or_insert(0) += 1;
    }

    /// Remove a reference from a resource
    pub fn remove_ref(&mut self, xid: XID, client: ClientId) -> usize {
        // Update global count
        let global_count = self.counts.entry(xid).or_insert(0);
        if *global_count > 0 {
            *global_count -= 1;
        }
        let remaining_global = *global_count;

        // Update client count
        if let Some(client_resources) = self.client_refs.get_mut(&client) {
            if let Some(client_count) = client_resources.get_mut(&xid) {
                if *client_count > 0 {
                    *client_count -= 1;
                }
                if *client_count == 0 {
                    client_resources.remove(&xid);
                }
            }
            if client_resources.is_empty() {
                self.client_refs.remove(&client);
            }
        }

        // Clean up if no references remain
        if remaining_global == 0 {
            self.counts.remove(&xid);
        }

        remaining_global
    }

    /// Get the reference count for a resource
    pub fn get_count(&self, xid: XID) -> usize {
        self.counts.get(&xid).copied().unwrap_or(0)
    }

    /// Get the reference count for a specific client
    pub fn get_client_count(&self, xid: XID, client: ClientId) -> usize {
        self.client_refs
            .get(&client)
            .and_then(|resources| resources.get(&xid))
            .copied()
            .unwrap_or(0)
    }

    /// Add a weak reference
    pub fn add_weak_ref(&mut self, xid: XID) {
        *self.weak_refs.entry(xid).or_insert(0) += 1;
    }

    /// Remove a weak reference
    pub fn remove_weak_ref(&mut self, xid: XID) -> usize {
        let weak_count = self.weak_refs.entry(xid).or_insert(0);
        if *weak_count > 0 {
            *weak_count -= 1;
        }
        let remaining = *weak_count;

        if remaining == 0 {
            self.weak_refs.remove(&xid);
        }

        remaining
    }

    /// Get the weak reference count for a resource
    pub fn get_weak_count(&self, xid: XID) -> usize {
        self.weak_refs.get(&xid).copied().unwrap_or(0)
    }

    /// Check if a resource has any references
    pub fn has_references(&self, xid: XID) -> bool {
        self.get_count(xid) > 0 || self.get_weak_count(xid) > 0
    }

    /// Get all resources referenced by a client
    pub fn get_client_resources(&self, client: ClientId) -> Vec<(XID, usize)> {
        self.client_refs
            .get(&client)
            .map(|resources| {
                resources
                    .iter()
                    .map(|(&xid, &count)| (xid, count))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all clients that reference a resource
    pub fn get_resource_clients(&self, xid: XID) -> Vec<(ClientId, usize)> {
        let mut result = Vec::new();

        for (&client, resources) in &self.client_refs {
            if let Some(&count) = resources.get(&xid) {
                if count > 0 {
                    result.push((client, count));
                }
            }
        }

        result
    }

    /// Remove all references from a client
    pub fn remove_client(&mut self, client: ClientId) -> Vec<XID> {
        let mut affected_resources = Vec::new();

        if let Some(client_resources) = self.client_refs.remove(&client) {
            for (xid, count) in client_resources {
                affected_resources.push(xid);

                // Update global counts
                if let Some(global_count) = self.counts.get_mut(&xid) {
                    *global_count = global_count.saturating_sub(count);
                    if *global_count == 0 {
                        self.counts.remove(&xid);
                    }
                }
            }
        }

        affected_resources
    }
    /// Remove all references to a resource
    pub fn remove_resource(&mut self, xid: XID) -> Vec<(ClientId, usize)> {
        let mut affected_clients = Vec::new();

        // Remove from global count
        self.counts.remove(&xid);
        self.weak_refs.remove(&xid);

        // Remove from client tracking
        for (client, resources) in &mut self.client_refs {
            if let Some(count) = resources.remove(&xid) {
                affected_clients.push((*client, count));
            }
        }

        // Clean up empty client entries
        self.client_refs
            .retain(|_, resources| !resources.is_empty());

        affected_clients
    }

    /// Get statistics about reference usage
    pub fn get_stats(&self) -> ReferenceStats {
        let mut stats = ReferenceStats::default();

        stats.total_resources = self.counts.len();
        stats.total_strong_refs = self.counts.values().sum();
        stats.total_weak_refs = self.weak_refs.values().sum();
        stats.total_clients = self.client_refs.len();
        // Find resource with most references
        if let Some((&xid, &count)) = self.counts.iter().max_by_key(|&(_, &count)| count) {
            stats.max_refs_resource = Some(xid);
            stats.max_refs_count = count;
        }

        // Calculate average references per resource
        if stats.total_resources > 0 {
            stats.avg_refs_per_resource =
                stats.total_strong_refs as f64 / stats.total_resources as f64;
        }

        // Find client with most references
        let mut max_client_refs = 0;
        let mut max_client = None;

        for (&client, resources) in &self.client_refs {
            let total: usize = resources.values().sum();
            if total > max_client_refs {
                max_client_refs = total;
                max_client = Some(client);
            }
        }

        stats.max_refs_client = max_client;
        stats.max_client_refs = max_client_refs;

        stats
    }

    /// Check for resources that can be cleaned up (no strong references)
    pub fn find_unreferenced(&self) -> Vec<XID> {
        self.weak_refs
            .keys()
            .filter(|&&xid| self.get_count(xid) == 0)
            .copied()
            .collect()
    }
}

/// Statistics about reference counting
#[derive(Debug, Default)]
pub struct ReferenceStats {
    /// Total number of resources being tracked
    pub total_resources: usize,
    /// Total strong references
    pub total_strong_refs: usize,
    /// Total weak references
    pub total_weak_refs: usize,
    /// Total clients with references
    pub total_clients: usize,
    /// Resource with the most references
    pub max_refs_resource: Option<XID>,
    /// Maximum reference count for a single resource
    pub max_refs_count: usize,
    /// Average references per resource
    pub avg_refs_per_resource: f64,
    /// Client with the most total references
    pub max_refs_client: Option<ClientId>,
    /// Maximum references held by a single client
    pub max_client_refs: usize,
}

impl ReferenceStats {
    /// Get the reference efficiency (strong refs / total refs)
    pub fn reference_efficiency(&self) -> f64 {
        let total_refs = self.total_strong_refs + self.total_weak_refs;
        if total_refs > 0 {
            self.total_strong_refs as f64 / total_refs as f64
        } else {
            0.0
        }
    }

    /// Get the average references per client
    pub fn avg_refs_per_client(&self) -> f64 {
        if self.total_clients > 0 {
            self.total_strong_refs as f64 / self.total_clients as f64
        } else {
            0.0
        }
    }
}
