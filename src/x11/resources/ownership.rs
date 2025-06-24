//! Resource ownership tracking and management
//!
//! This module handles ownership relationships between clients and resources,
//! including ownership transfers and access control.

use crate::x11::protocol::types::{ClientId, XID};
use std::collections::{HashMap, HashSet};

/// Errors that can occur during ownership operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnershipError {
    /// Resource is not owned by the specified client
    NotOwned { resource: XID, client: ClientId },
    /// Client does not have permission to access the resource
    AccessDenied { resource: XID, client: ClientId },
    /// Ownership transfer is not allowed for this resource type
    TransferNotAllowed(XID),
    /// Resource not found in ownership tracking
    ResourceNotFound(XID),
    /// Client not found in ownership tracking
    ClientNotFound(ClientId),
}

impl std::fmt::Display for OwnershipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OwnershipError::NotOwned { resource, client } => {
                write!(f, "Resource {} is not owned by client {}", resource, client)
            }
            OwnershipError::AccessDenied { resource, client } => {
                write!(
                    f,
                    "Client {} does not have access to resource {}",
                    client, resource
                )
            }
            OwnershipError::TransferNotAllowed(resource) => {
                write!(
                    f,
                    "Ownership transfer not allowed for resource {}",
                    resource
                )
            }
            OwnershipError::ResourceNotFound(resource) => {
                write!(f, "Resource {} not found in ownership tracking", resource)
            }
            OwnershipError::ClientNotFound(client) => {
                write!(f, "Client {} not found in ownership tracking", client)
            }
        }
    }
}

impl std::error::Error for OwnershipError {}

/// Access permissions for resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessLevel {
    /// No access
    None,
    /// Read-only access
    Read,
    /// Read and write access
    ReadWrite,
    /// Full control (owner)
    Owner,
}

impl AccessLevel {
    /// Check if this access level includes read permissions
    pub fn can_read(&self) -> bool {
        matches!(
            self,
            AccessLevel::Read | AccessLevel::ReadWrite | AccessLevel::Owner
        )
    }

    /// Check if this access level includes write permissions
    pub fn can_write(&self) -> bool {
        matches!(self, AccessLevel::ReadWrite | AccessLevel::Owner)
    }

    /// Check if this access level includes ownership permissions
    pub fn is_owner(&self) -> bool {
        matches!(self, AccessLevel::Owner)
    }
}

/// Information about resource ownership
#[derive(Debug, Clone)]
pub struct OwnershipInfo {
    /// The resource XID
    pub resource: XID,
    /// The owning client
    pub owner: ClientId,
    /// When ownership was established
    pub owned_since: std::time::Instant,
    /// Whether the resource can be transferred
    pub transferable: bool,
    /// Access permissions for other clients
    pub access_list: HashMap<ClientId, AccessLevel>,
}

impl OwnershipInfo {
    /// Create new ownership info
    pub fn new(resource: XID, owner: ClientId, transferable: bool) -> Self {
        Self {
            resource,
            owner,
            owned_since: std::time::Instant::now(),
            transferable,
            access_list: HashMap::new(),
        }
    }

    /// Check if a client has the specified access level
    pub fn has_access(&self, client: ClientId, required_level: AccessLevel) -> bool {
        if client == self.owner {
            return true; // Owner has all access
        }

        let client_level = self
            .access_list
            .get(&client)
            .copied()
            .unwrap_or(AccessLevel::None);

        match required_level {
            AccessLevel::None => true,
            AccessLevel::Read => client_level.can_read(),
            AccessLevel::ReadWrite => client_level.can_write(),
            AccessLevel::Owner => client_level.is_owner() || client == self.owner,
        }
    }

    /// Grant access to a client
    pub fn grant_access(&mut self, client: ClientId, level: AccessLevel) {
        if client != self.owner {
            self.access_list.insert(client, level);
        }
    }

    /// Revoke access from a client
    pub fn revoke_access(&mut self, client: ClientId) {
        self.access_list.remove(&client);
    }

    /// Get the access level for a client
    pub fn get_access(&self, client: ClientId) -> AccessLevel {
        if client == self.owner {
            AccessLevel::Owner
        } else {
            self.access_list
                .get(&client)
                .copied()
                .unwrap_or(AccessLevel::None)
        }
    }
}

/// Tracks ownership of resources
#[derive(Debug, Default)]
pub struct OwnershipTracker {
    /// Map from resource to ownership info
    ownership: HashMap<XID, OwnershipInfo>,
    /// Map from client to owned resources
    client_resources: HashMap<ClientId, HashSet<XID>>,
}

impl OwnershipTracker {
    /// Create a new ownership tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Register ownership of a resource
    pub fn register_ownership(&mut self, resource: XID, owner: ClientId, transferable: bool) {
        let info = OwnershipInfo::new(resource, owner, transferable);
        self.ownership.insert(resource, info);

        self.client_resources
            .entry(owner)
            .or_insert_with(HashSet::new)
            .insert(resource);
    }

    /// Transfer ownership of a resource to a new client
    pub fn transfer_ownership(
        &mut self,
        resource: XID,
        new_owner: ClientId,
    ) -> Result<(), OwnershipError> {
        let info = self
            .ownership
            .get_mut(&resource)
            .ok_or(OwnershipError::ResourceNotFound(resource))?;

        if !info.transferable {
            return Err(OwnershipError::TransferNotAllowed(resource));
        }

        let old_owner = info.owner;
        info.owner = new_owner;
        info.owned_since = std::time::Instant::now();
        info.access_list.clear(); // Clear old access list

        // Update client resource tracking
        if let Some(resources) = self.client_resources.get_mut(&old_owner) {
            resources.remove(&resource);
            if resources.is_empty() {
                self.client_resources.remove(&old_owner);
            }
        }

        self.client_resources
            .entry(new_owner)
            .or_insert_with(HashSet::new)
            .insert(resource);

        Ok(())
    }

    /// Check if a client owns a resource
    pub fn is_owner(&self, resource: XID, client: ClientId) -> bool {
        self.ownership
            .get(&resource)
            .map(|info| info.owner == client)
            .unwrap_or(false)
    }

    /// Check if a client has the specified access to a resource
    pub fn has_access(&self, resource: XID, client: ClientId, level: AccessLevel) -> bool {
        self.ownership
            .get(&resource)
            .map(|info| info.has_access(client, level))
            .unwrap_or(false)
    }

    /// Grant access to a resource
    pub fn grant_access(
        &mut self,
        resource: XID,
        owner: ClientId,
        target: ClientId,
        level: AccessLevel,
    ) -> Result<(), OwnershipError> {
        let info = self
            .ownership
            .get_mut(&resource)
            .ok_or(OwnershipError::ResourceNotFound(resource))?;

        if info.owner != owner {
            return Err(OwnershipError::NotOwned {
                resource,
                client: owner,
            });
        }

        info.grant_access(target, level);
        Ok(())
    }

    /// Revoke access to a resource
    pub fn revoke_access(
        &mut self,
        resource: XID,
        owner: ClientId,
        target: ClientId,
    ) -> Result<(), OwnershipError> {
        let info = self
            .ownership
            .get_mut(&resource)
            .ok_or(OwnershipError::ResourceNotFound(resource))?;

        if info.owner != owner {
            return Err(OwnershipError::NotOwned {
                resource,
                client: owner,
            });
        }

        info.revoke_access(target);
        Ok(())
    }

    /// Get the owner of a resource
    pub fn get_owner(&self, resource: XID) -> Option<ClientId> {
        self.ownership.get(&resource).map(|info| info.owner)
    }

    /// Get all resources owned by a client
    pub fn get_client_resources(&self, client: ClientId) -> Vec<XID> {
        self.client_resources
            .get(&client)
            .map(|resources| resources.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get ownership information for a resource
    pub fn get_ownership_info(&self, resource: XID) -> Option<&OwnershipInfo> {
        self.ownership.get(&resource)
    }

    /// Remove ownership tracking for a resource
    pub fn unregister_resource(&mut self, resource: XID) -> Option<OwnershipInfo> {
        if let Some(info) = self.ownership.remove(&resource) {
            // Remove from client tracking
            if let Some(resources) = self.client_resources.get_mut(&info.owner) {
                resources.remove(&resource);
                if resources.is_empty() {
                    self.client_resources.remove(&info.owner);
                }
            }
            Some(info)
        } else {
            None
        }
    }

    /// Remove all resources owned by a client
    pub fn unregister_client(&mut self, client: ClientId) -> Vec<XID> {
        let resources = self.get_client_resources(client);

        for resource in &resources {
            self.ownership.remove(resource);
        }

        self.client_resources.remove(&client);
        resources
    }

    /// Get statistics about ownership
    pub fn get_stats(&self) -> OwnershipStats {
        let mut stats = OwnershipStats::default();

        stats.total_resources = self.ownership.len();
        stats.total_clients = self.client_resources.len();

        // Calculate resource distribution
        for (_, resources) in &self.client_resources {
            let count = resources.len();
            stats.max_resources_per_client = stats.max_resources_per_client.max(count);
            stats.total_owned_resources += count;
        }

        if !self.client_resources.is_empty() {
            stats.avg_resources_per_client =
                stats.total_owned_resources as f64 / self.client_resources.len() as f64;
        }

        // Count transferable resources
        stats.transferable_resources = self
            .ownership
            .values()
            .filter(|info| info.transferable)
            .count();

        // Count resources with shared access
        stats.shared_resources = self
            .ownership
            .values()
            .filter(|info| !info.access_list.is_empty())
            .count();

        stats
    }

    /// Validate access for an operation
    pub fn validate_access(
        &self,
        resource: XID,
        client: ClientId,
        required_level: AccessLevel,
    ) -> Result<(), OwnershipError> {
        if self.has_access(resource, client, required_level) {
            Ok(())
        } else {
            Err(OwnershipError::AccessDenied { resource, client })
        }
    }
}

/// Statistics about resource ownership
#[derive(Debug, Default)]
pub struct OwnershipStats {
    /// Total number of resources being tracked
    pub total_resources: usize,
    /// Total number of clients with resources
    pub total_clients: usize,
    /// Total resources owned by all clients
    pub total_owned_resources: usize,
    /// Maximum resources owned by a single client
    pub max_resources_per_client: usize,
    /// Average resources per client
    pub avg_resources_per_client: f64,
    /// Number of transferable resources
    pub transferable_resources: usize,
    /// Number of resources with shared access
    pub shared_resources: usize,
}
