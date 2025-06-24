//! Resource Registry and Lookup
//!
//! This module implements the central registry for all X11 resources,
//! providing fast lookup and management capabilities.

use crate::x11::protocol::types::{ClientId, XID};
use crate::x11::resources::{Resource, ResourceType};
use std::collections::HashMap;

/// Errors that can occur during registry operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// Resource with the given XID already exists
    ResourceExists(XID),
    /// Resource with the given XID was not found
    ResourceNotFound(XID),
    /// Resource has dependents and cannot be removed
    HasDependents(XID, Vec<XID>),
    /// Type mismatch when casting resource
    TypeMismatch {
        xid: XID,
        expected: ResourceType,
        actual: ResourceType,
    },
    /// Resource cleanup failed
    CleanupFailed(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::ResourceExists(xid) => {
                write!(f, "Resource {} already exists", xid)
            }
            RegistryError::ResourceNotFound(xid) => {
                write!(f, "Resource {} not found", xid)
            }
            RegistryError::HasDependents(xid, dependents) => {
                write!(f, "Resource {} has dependents: {:?}", xid, dependents)
            }
            RegistryError::TypeMismatch {
                xid,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Type mismatch for resource {}: expected {}, found {}",
                    xid,
                    expected.name(),
                    actual.name()
                )
            }
            RegistryError::CleanupFailed(msg) => {
                write!(f, "Resource cleanup failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for RegistryError {}

/// Central registry for all X11 resources
#[derive(Debug)]
pub struct ResourceRegistry {
    /// Map from XID to resource
    resources: HashMap<XID, Box<dyn Resource>>,
    /// Map from resource type to list of XIDs
    by_type: HashMap<ResourceType, Vec<XID>>,
    /// Map from client to list of XIDs
    by_client: HashMap<ClientId, Vec<XID>>,
}

impl ResourceRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            by_type: HashMap::new(),
            by_client: HashMap::new(),
        }
    }

    /// Register a new resource
    pub fn register(&mut self, xid: XID, resource: Box<dyn Resource>) -> Result<(), RegistryError> {
        if self.resources.contains_key(&xid) {
            return Err(RegistryError::ResourceExists(xid));
        }

        let resource_type = resource.resource_type();
        let client = resource.owner();

        // Add to main registry
        self.resources.insert(xid, resource);

        // Add to type index
        self.by_type
            .entry(resource_type)
            .or_insert_with(Vec::new)
            .push(xid);

        // Add to client index (only for client-owned resources)
        if resource_type.has_client_ownership() {
            self.by_client
                .entry(client)
                .or_insert_with(Vec::new)
                .push(xid);
        }

        Ok(())
    }

    /// Unregister a resource
    pub fn unregister(&mut self, xid: XID) -> Result<Box<dyn Resource>, RegistryError> {
        let resource = self
            .resources
            .remove(&xid)
            .ok_or(RegistryError::ResourceNotFound(xid))?;

        let resource_type = resource.resource_type();
        let client = resource.owner();

        // Remove from type index
        if let Some(type_list) = self.by_type.get_mut(&resource_type) {
            type_list.retain(|&x| x != xid);
            if type_list.is_empty() {
                self.by_type.remove(&resource_type);
            }
        }

        // Remove from client index
        if resource_type.has_client_ownership() {
            if let Some(client_list) = self.by_client.get_mut(&client) {
                client_list.retain(|&x| x != xid);
                if client_list.is_empty() {
                    self.by_client.remove(&client);
                }
            }
        }

        Ok(resource)
    }

    /// Get a resource by XID
    pub fn get(&self, xid: XID) -> Option<&dyn Resource> {
        self.resources.get(&xid).map(|r| r.as_ref())
    }

    /// Get a mutable reference to a resource by XID
    pub fn get_mut(&mut self, xid: XID) -> Option<&mut Box<dyn Resource>> {
        self.resources.get_mut(&xid)
    }

    /// Get a resource cast to a specific type
    pub fn get_typed<T: Resource + 'static>(&self, xid: XID) -> Result<&T, RegistryError> {
        let resource = self.get(xid).ok_or(RegistryError::ResourceNotFound(xid))?;

        resource
            .as_any()
            .downcast_ref::<T>()
            .ok_or_else(|| RegistryError::TypeMismatch {
                xid,
                expected: resource.resource_type(), // Use instance method instead
                actual: resource.resource_type(),
            })
    }

    /// Get a mutable resource cast to a specific type
    pub fn get_typed_mut<T: Resource + 'static>(
        &mut self,
        xid: XID,
    ) -> Result<&mut T, RegistryError> {
        let resource = self
            .get_mut(xid)
            .ok_or(RegistryError::ResourceNotFound(xid))?;

        let resource_type = resource.resource_type();

        resource
            .as_any_mut()
            .downcast_mut::<T>()
            .ok_or_else(|| RegistryError::TypeMismatch {
                xid,
                expected: resource_type, // Use stored resource type
                actual: resource_type,
            })
    }

    /// Check if a resource exists
    pub fn contains(&self, xid: XID) -> bool {
        self.resources.contains_key(&xid)
    }

    /// Get all resources of a specific type
    pub fn get_by_type(&self, resource_type: ResourceType) -> Vec<XID> {
        self.by_type
            .get(&resource_type)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all resources owned by a client
    pub fn get_by_client(&self, client: ClientId) -> Vec<XID> {
        self.by_client.get(&client).cloned().unwrap_or_default()
    }

    /// Get the number of resources in the registry
    pub fn count(&self) -> usize {
        self.resources.len()
    }

    /// Get the number of resources of a specific type
    pub fn count_by_type(&self, resource_type: ResourceType) -> usize {
        self.by_type
            .get(&resource_type)
            .map_or(0, |list| list.len())
    }

    /// Get the number of resources owned by a client
    pub fn count_by_client(&self, client: ClientId) -> usize {
        self.by_client.get(&client).map_or(0, |list| list.len())
    }

    /// Get all XIDs in the registry
    pub fn all_xids(&self) -> Vec<XID> {
        self.resources.keys().copied().collect()
    }

    /// Get all clients that own resources
    pub fn all_clients(&self) -> Vec<ClientId> {
        self.by_client.keys().copied().collect()
    }

    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        let mut type_counts = HashMap::new();
        let mut client_counts = HashMap::new();

        for (&resource_type, list) in &self.by_type {
            type_counts.insert(resource_type, list.len());
        }

        for (&client, list) in &self.by_client {
            client_counts.insert(client, list.len());
        }

        RegistryStats {
            total_resources: self.resources.len(),
            type_counts,
            client_counts,
        }
    }

    /// Validate registry consistency
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check that all resources in type indices exist in main registry
        for (&resource_type, xids) in &self.by_type {
            for &xid in xids {
                if let Some(resource) = self.resources.get(&xid) {
                    if resource.resource_type() != resource_type {
                        errors.push(format!(
                            "Resource {} has type mismatch in type index: expected {}, found {}",
                            xid,
                            resource_type.name(),
                            resource.resource_type().name()
                        ));
                    }
                } else {
                    errors.push(format!(
                        "Resource {} in type index {} does not exist in main registry",
                        xid,
                        resource_type.name()
                    ));
                }
            }
        }

        // Check that all resources in client indices exist in main registry
        for (&client, xids) in &self.by_client {
            for &xid in xids {
                if let Some(resource) = self.resources.get(&xid) {
                    if resource.owner() != client {
                        errors.push(format!(
                            "Resource {} has client mismatch in client index: expected {}, found {}",
                            xid, client, resource.owner()
                        ));
                    }
                } else {
                    errors.push(format!(
                        "Resource {} in client index {} does not exist in main registry",
                        xid, client
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the resource registry
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Total number of resources
    pub total_resources: usize,
    /// Number of resources by type
    pub type_counts: HashMap<ResourceType, usize>,
    /// Number of resources by client
    pub client_counts: HashMap<ClientId, usize>,
}

impl RegistryStats {
    /// Get the most common resource type
    pub fn most_common_type(&self) -> Option<(ResourceType, usize)> {
        self.type_counts
            .iter()
            .max_by_key(|&(_, &count)| count)
            .map(|(&resource_type, &count)| (resource_type, count))
    }

    /// Get the client with the most resources
    pub fn client_with_most_resources(&self) -> Option<(ClientId, usize)> {
        self.client_counts
            .iter()
            .max_by_key(|&(_, &count)| count)
            .map(|(&client, &count)| (client, count))
    }
}

// Extension trait for downcasting resources
pub trait ResourceDowncast {
    /// Get the static resource type for this concrete type
    fn static_resource_type() -> ResourceType;

    /// Downcast to Any for type checking
    fn as_any(&self) -> &dyn std::any::Any;

    /// Mutable downcast to Any for type checking
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

// Implement the downcast trait for the base Resource trait
impl dyn Resource {
    /// Attempt to downcast to a specific resource type
    pub fn downcast_ref<T: Resource + 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }

    /// Attempt to mutably downcast to a specific resource type
    pub fn downcast_mut<T: Resource + 'static>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut::<T>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x11::resources::types::WindowResource;

    #[test]
    fn test_registry_basic_operations() {
        let mut registry = ResourceRegistry::new();
        let xid = 0x00400001;
        let client = 1; // Create and register a resource
        let geometry = crate::x11::resources::types::window::WindowGeometry {
            x: 0,
            y: 0,
            width: 100,
            height: 100,
            border_width: 1,
        };
        let window = Box::new(WindowResource::new(
            xid,
            client,
            Some(0),
            geometry,
            24,
            0x21,
            None,
        ));
        registry
            .register(xid, window)
            .expect("Should register resource");

        // Check that it exists
        assert!(registry.contains(xid));
        assert_eq!(registry.count(), 1);

        // Get the resource
        let resource = registry.get(xid).expect("Should find resource");
        assert_eq!(resource.xid(), xid);
        assert_eq!(resource.owner(), client);

        // Unregister the resource
        let removed = registry
            .unregister(xid)
            .expect("Should unregister resource");
        assert_eq!(removed.xid(), xid);
        assert!(!registry.contains(xid));
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_indices() {
        let mut registry = ResourceRegistry::new();
        let client = 1; // Register multiple windows
        for i in 0..3 {
            let xid = 0x00400001 + i;
            let geometry = crate::x11::resources::types::window::WindowGeometry {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
                border_width: 1,
            };
            let window = Box::new(WindowResource::new(
                xid,
                client,
                Some(0),
                geometry,
                24,
                0x21,
                None,
            ));
            registry
                .register(xid, window)
                .expect("Should register resource");
        }

        // Check type index
        let windows = registry.get_by_type(ResourceType::Window);
        assert_eq!(windows.len(), 3);

        // Check client index
        let client_resources = registry.get_by_client(client);
        assert_eq!(client_resources.len(), 3);

        // Check counts
        assert_eq!(registry.count_by_type(ResourceType::Window), 3);
        assert_eq!(registry.count_by_client(client), 3);
    }

    #[test]
    fn test_registry_validation() {
        let registry = ResourceRegistry::new();

        // Empty registry should be valid
        assert!(registry.validate().is_ok());

        // TODO: Add tests for invalid states when we have more complex scenarios
    }
}
