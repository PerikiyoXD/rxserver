//! Resource cleanup policies and management
//!
//! This module provides mechanisms for resource cleanup, including policies
//! for when and how resources should be cleaned up.

use crate::x11::protocol::types::{ClientId, XId};
use crate::x11::resources::{RegistryError, Resource};
use std::collections::HashSet;

/// Policy for resource cleanup
#[derive(Debug, Clone, PartialEq)]
pub enum CleanupPolicy {
    /// Immediate cleanup when resource is freed
    Immediate,
    /// Deferred cleanup during maintenance cycles
    Deferred,
    /// Cleanup when client disconnects
    OnClientDisconnect,
    /// Custom cleanup with specific delay
    Delayed(std::time::Duration),
}

impl Default for CleanupPolicy {
    fn default() -> Self {
        CleanupPolicy::Immediate
    }
}

/// Resource cleanup manager
#[derive(Debug)]
pub struct ResourceCleaner {
    /// Policy for cleanup
    policy: CleanupPolicy,
    /// Resources pending cleanup
    pending_cleanup: HashSet<XId>,
    /// Cleanup dependencies (resources that must be cleaned up together)
    cleanup_dependencies: std::collections::HashMap<XId, Vec<XId>>,
}

impl ResourceCleaner {
    /// Create a new resource cleaner with the given policy
    pub fn new(policy: CleanupPolicy) -> Self {
        Self {
            policy,
            pending_cleanup: HashSet::new(),
            cleanup_dependencies: std::collections::HashMap::new(),
        }
    }

    /// Schedule a resource for cleanup
    pub fn schedule_cleanup(&mut self, xid: XId) {
        self.pending_cleanup.insert(xid);
    }

    /// Add a cleanup dependency (dependent must be cleaned up when dependency is cleaned)
    pub fn add_cleanup_dependency(&mut self, dependent: XId, dependency: XId) {
        self.cleanup_dependencies
            .entry(dependency)
            .or_insert_with(Vec::new)
            .push(dependent);
    }

    /// Remove a cleanup dependency
    pub fn remove_cleanup_dependency(&mut self, dependent: XId, dependency: XId) {
        if let Some(deps) = self.cleanup_dependencies.get_mut(&dependency) {
            deps.retain(|&dep| dep != dependent);
            if deps.is_empty() {
                self.cleanup_dependencies.remove(&dependency);
            }
        }
    }

    /// Execute cleanup for a specific resource
    pub fn cleanup_resource(
        &mut self,
        xid: XId,
        resource: &mut dyn Resource,
    ) -> Result<(), RegistryError> {
        // Prepare the resource for destruction
        if let Err(e) = resource.prepare_destroy() {
            return Err(RegistryError::CleanupFailed(format!(
                "Failed to prepare resource {} for cleanup: {}",
                xid, e
            )));
        }

        // Schedule dependent resources for cleanup
        if let Some(dependents) = self.cleanup_dependencies.remove(&xid) {
            for dependent in dependents {
                self.schedule_cleanup(dependent);
            }
        }

        // Remove from pending cleanup
        self.pending_cleanup.remove(&xid);

        Ok(())
    }

    /// Execute all pending cleanups
    pub fn execute_pending_cleanups<F>(&mut self, mut cleanup_fn: F) -> Result<(), RegistryError>
    where
        F: FnMut(XId) -> Result<(), RegistryError>,
    {
        let pending: Vec<XId> = self.pending_cleanup.iter().copied().collect();

        for xid in pending {
            cleanup_fn(xid)?;
            self.pending_cleanup.remove(&xid);
        }

        Ok(())
    }

    /// Get the current cleanup policy
    pub fn policy(&self) -> &CleanupPolicy {
        &self.policy
    }

    /// Set a new cleanup policy
    pub fn set_policy(&mut self, policy: CleanupPolicy) {
        self.policy = policy;
    }

    /// Check if a resource is pending cleanup
    pub fn is_pending_cleanup(&self, xid: XId) -> bool {
        self.pending_cleanup.contains(&xid)
    }

    /// Get the number of resources pending cleanup
    pub fn pending_count(&self) -> usize {
        self.pending_cleanup.len()
    }
    /// Schedule cleanup for all resources owned by a client
    pub fn schedule_client_cleanup(&mut self, _client: ClientId, resource_xids: &[XId]) {
        for &xid in resource_xids {
            self.schedule_cleanup(xid);
        }
    }
}

impl Default for ResourceCleaner {
    fn default() -> Self {
        Self::new(CleanupPolicy::default())
    }
}
