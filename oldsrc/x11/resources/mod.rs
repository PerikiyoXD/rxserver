//! Resource Management System
//!
//! This module provides the core resource management infrastructure for the X11 server,
//! including XId allocation, resource registry, lifecycle management, and cleanup.

pub mod allocator;
pub mod cleanup;
pub mod dependencies;
pub mod lifecycle;
pub mod ownership;
pub mod references;
pub mod registry;
pub mod types;

// Re-export core types and traits
pub use allocator::{AllocationError, XIDAllocator};
pub use cleanup::{CleanupPolicy, ResourceCleaner};
pub use dependencies::{DependencyError, DependencyTracker};
pub use lifecycle::{LifecycleError, ResourceLifecycleManager};
pub use ownership::{OwnershipError, OwnershipTracker};
pub use references::{ReferenceCounter, ResourceRef};
pub use registry::{RegistryError, ResourceRegistry};

use crate::x11::protocol::types::{ClientId, XId};

/// Core resource management interface
pub trait ResourceManager {
    /// Allocate a new XId for a client
    fn allocate_xid(&mut self, client: ClientId) -> Result<XId, AllocationError>;

    /// Register a resource with the given XId
    fn register_resource(
        &mut self,
        xid: XId,
        resource: Box<dyn Resource>,
    ) -> Result<(), RegistryError>;

    /// Look up a resource by XId
    fn lookup_resource(&self, xid: XId) -> Option<&dyn Resource>;
    /// Look up a mutable resource by XId
    fn lookup_resource_mut(&mut self, xid: XId) -> Option<&mut Box<dyn Resource>>;

    /// Free a resource and its XId
    fn free_resource(&mut self, xid: XId) -> Result<(), RegistryError>;

    /// Get all resources owned by a client
    fn get_client_resources(&self, client: ClientId) -> Vec<XId>;

    /// Free all resources owned by a client
    fn free_client_resources(&mut self, client: ClientId) -> Result<(), RegistryError>;

    /// Perform garbage collection on unreferenced resources
    /// Returns the number of resources that were cleaned up
    fn garbage_collect(&mut self) -> Result<usize, RegistryError>;
}

/// Base trait for all X11 resources
pub trait Resource: std::fmt::Debug + Send + Sync {
    /// Get the resource type identifier
    fn resource_type(&self) -> ResourceType;

    /// Get the XId of this resource
    fn xid(&self) -> XId;

    /// Get the client that owns this resource
    fn owner(&self) -> ClientId;

    /// Check if this resource can be accessed by the given client
    fn accessible_by(&self, client: ClientId) -> bool {
        self.owner() == client
    }

    /// Prepare the resource for destruction
    fn prepare_destroy(&mut self) -> Result<(), LifecycleError> {
        Ok(())
    }

    /// Get dependencies of this resource
    fn dependencies(&self) -> Vec<XId> {
        Vec::new()
    }

    /// Get dependents of this resource
    fn dependents(&self) -> Vec<XId> {
        Vec::new()
    }

    /// Downcast to Any for type checking
    fn as_any(&self) -> &dyn std::any::Any;

    /// Mutable downcast to Any for type checking
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// Enumeration of X11 resource types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Window,
    Pixmap,
    GraphicsContext,
    Font,
    Cursor,
    Colormap,
    Atom,
}

impl ResourceType {
    /// Get the string name of the resource type
    pub fn name(self) -> &'static str {
        match self {
            ResourceType::Window => "Window",
            ResourceType::Pixmap => "Pixmap",
            ResourceType::GraphicsContext => "Graphics Context",
            ResourceType::Font => "Font",
            ResourceType::Cursor => "Cursor",
            ResourceType::Colormap => "Colormap",
            ResourceType::Atom => "Atom",
        }
    }

    /// Check if this resource type supports client ownership
    pub fn has_client_ownership(self) -> bool {
        match self {
            ResourceType::Atom => false, // Atoms are global
            _ => true,
        }
    }
}

/// Main resource manager implementation
#[derive(Debug)]
pub struct DefaultResourceManager {
    allocator: XIDAllocator,
    registry: ResourceRegistry,
    ownership: OwnershipTracker,
    references: ReferenceCounter,
    dependencies: DependencyTracker,
    cleaner: ResourceCleaner,
}

impl DefaultResourceManager {
    /// Create a new resource manager
    pub fn new(base_xid: XId, xid_mask: XId) -> Self {
        Self {
            allocator: XIDAllocator::new(base_xid, xid_mask),
            registry: ResourceRegistry::new(),
            ownership: OwnershipTracker::new(),
            references: ReferenceCounter::new(),
            dependencies: DependencyTracker::new(),
            cleaner: ResourceCleaner::new(CleanupPolicy::OnClientDisconnect), // TODO: Decide on a cleanup policy
        }
    }
    /// Get resource statistics
    pub fn stats(&self) -> ResourceTypeStats {
        let mut stats = ResourceTypeStats::new();
        // Collect stats from registry
        for xid in self.registry.all_xids() {
            if let Some(resource) = self.registry.get(xid) {
                let resource_type = resource.resource_type();
                stats.increment(resource_type);
            }
        }
        stats
    }

    /// Add a dependency between resources
    pub fn add_dependency(
        &mut self,
        dependent: XId,
        dependency: XId,
    ) -> Result<(), DependencyError> {
        self.dependencies.add_dependency(dependent, dependency)
    }

    /// Remove a dependency between resources
    pub fn remove_dependency(
        &mut self,
        dependent: XId,
        dependency: XId,
    ) -> Result<(), DependencyError> {
        self.dependencies.remove_dependency(dependent, dependency);
        Ok(())
    }

    /// Get all resources that depend on the given resource
    pub fn get_dependents(&self, xid: XId) -> Vec<XId> {
        self.dependencies.get_dependents(xid)
    }

    /// Get all resources that the given resource depends on
    pub fn get_dependencies(&self, xid: XId) -> Vec<XId> {
        self.dependencies.get_dependencies(xid)
    }
}

impl ResourceManager for DefaultResourceManager {
    fn allocate_xid(&mut self, client: ClientId) -> Result<XId, AllocationError> {
        self.allocator.allocate_for_client(client)
    }

    fn register_resource(
        &mut self,
        xid: XId,
        resource: Box<dyn Resource>,
    ) -> Result<(), RegistryError> {
        let client = resource.owner();
        let resource_type = resource.resource_type();

        // Register the resource
        self.registry.register(xid, resource)?; // Track ownership
        if resource_type.has_client_ownership() {
            self.ownership.register_ownership(xid, client, true); // Assuming resources are transferable by default
        } // Initialize reference count
        self.references.add_ref(xid, client);

        Ok(())
    }

    fn lookup_resource(&self, xid: XId) -> Option<&dyn Resource> {
        self.registry.get(xid)
    }
    fn lookup_resource_mut(&mut self, xid: XId) -> Option<&mut Box<dyn Resource>> {
        self.registry.get_mut(xid)
    }

    fn free_resource(&mut self, xid: XId) -> Result<(), RegistryError> {
        // Check if resource exists
        let resource = self
            .registry
            .get(xid)
            .ok_or(RegistryError::ResourceNotFound(xid))?;

        let resource_type = resource.resource_type();

        // Check for dependencies
        let dependents = self.dependencies.get_dependents(xid);
        if !dependents.is_empty() {
            return Err(RegistryError::HasDependents(xid, dependents));
        } // Remove from dependency tracking
        self.dependencies.remove_resource(xid);

        // Remove from ownership tracking
        if resource_type.has_client_ownership() {
            self.ownership.unregister_resource(xid);
        }

        // Remove from reference counting
        self.references.remove_resource(xid);

        // Free the XId
        self.allocator.free(xid);

        // Remove from registry (this will drop the resource)
        self.registry.unregister(xid)?;

        Ok(())
    }

    fn get_client_resources(&self, client: ClientId) -> Vec<XId> {
        self.ownership.get_client_resources(client)
    }

    fn free_client_resources(&mut self, client: ClientId) -> Result<(), RegistryError> {
        let resources = self.get_client_resources(client);

        // Free resources in dependency order (dependents first)
        let sorted_resources = self
            .dependencies
            .destruction_order(&resources)
            .unwrap_or_else(|_| resources); // Fall back to original order if there are dependency issues

        for xid in sorted_resources {
            if let Err(e) = self.free_resource(xid) {
                // Log error but continue with cleanup
                eprintln!("Error freeing resource {}: {:?}", xid, e);
            }
        }
        Ok(())
    }

    fn garbage_collect(&mut self) -> Result<usize, RegistryError> {
        // Find all resources with zero references
        let mut unreferenced_resources = Vec::new();

        // Get all registered resources
        for xid in self.registry.all_xids() {
            if !self.references.has_references(xid) {
                unreferenced_resources.push(xid);
            }
        }

        let cleanup_count = unreferenced_resources.len();

        // Clean up unreferenced resources
        for xid in unreferenced_resources {
            if let Err(e) = self.free_resource(xid) {
                // Log error but continue cleanup
                eprintln!(
                    "Error during garbage collection of resource {}: {:?}",
                    xid, e
                );
            }
        }

        Ok(cleanup_count)
    }
}

// Re-export ResourceStats from types module
pub use types::ResourceTypeStats;

#[cfg(test)]
mod tests {
    use super::*; // Test helper: Mock window resource implementation
    #[derive(Debug)]
    struct TestWindowResource {
        xid: XId,
        owner: ClientId,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    }

    impl TestWindowResource {
        pub fn new(
            xid: XId,
            owner: ClientId,
            x: i16,
            y: i16,
            _z: i16,
            width: u16,
            height: u16,
        ) -> Self {
            Self {
                xid,
                owner,
                x,
                y,
                width,
                height,
            }
        }
    }

    impl Resource for TestWindowResource {
        fn resource_type(&self) -> ResourceType {
            ResourceType::Window
        }

        fn xid(&self) -> XId {
            self.xid
        }

        fn owner(&self) -> ClientId {
            self.owner
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_resource_manager_basic_operations() {
        let mut manager = DefaultResourceManager::new(0x00400000, 0x001FFFFF);
        let client_id = 1;

        // Allocate XId
        let xid = manager
            .allocate_xid(client_id)
            .expect("Should allocate XId"); // Create and register a resource
        let window = Box::new(TestWindowResource::new(xid, client_id, 0, 0, 0, 100, 100));
        manager
            .register_resource(xid, window)
            .expect("Should register resource");

        // Look up the resource
        let resource = manager.lookup_resource(xid).expect("Should find resource");
        assert_eq!(resource.xid(), xid);
        assert_eq!(resource.owner(), client_id);

        // Free the resource
        manager.free_resource(xid).expect("Should free resource");

        // Resource should no longer exist
        assert!(manager.lookup_resource(xid).is_none());
    }

    #[test]
    fn test_client_resource_cleanup() {
        let mut manager = DefaultResourceManager::new(0x00400000, 0x001FFFFF);
        let client_id = 1;

        // Create multiple resources for the client
        for _ in 0..5 {
            let xid = manager
                .allocate_xid(client_id)
                .expect("Should allocate XId");
            let window = Box::new(TestWindowResource::new(xid, client_id, 0, 0, 0, 100, 100));
            manager
                .register_resource(xid, window)
                .expect("Should register resource");
        }

        // Check that client has resources
        let resources = manager.get_client_resources(client_id);
        assert_eq!(resources.len(), 5);

        // Free all client resources
        manager
            .free_client_resources(client_id)
            .expect("Should free all resources");

        // Client should have no resources left
        let resources = manager.get_client_resources(client_id);
        assert_eq!(resources.len(), 0);
    }

    #[test]
    fn test_garbage_collection() {
        let mut manager = DefaultResourceManager::new(0x00400000, 0x001FFFFF);
        let client_id = 1;

        // Create some resources
        let mut xids = Vec::new();
        for _ in 0..3 {
            let xid = manager
                .allocate_xid(client_id)
                .expect("Should allocate XId");
            let window = Box::new(TestWindowResource::new(xid, client_id, 0, 0, 0, 100, 100));
            manager
                .register_resource(xid, window)
                .expect("Should register resource");
            xids.push(xid);
        }

        // Remove references to some resources by manipulating the reference counter
        manager.references.remove_ref(xids[1], client_id);
        manager.references.remove_ref(xids[2], client_id);

        // Run garbage collection
        let cleaned_count = manager
            .garbage_collect()
            .expect("Garbage collection should succeed");

        // Should have cleaned up the unreferenced resources
        assert_eq!(cleaned_count, 2);

        // First resource should still exist (has references)
        assert!(manager.lookup_resource(xids[0]).is_some());

        // Second and third resources should be gone
        assert!(manager.lookup_resource(xids[1]).is_none());
        assert!(manager.lookup_resource(xids[2]).is_none());
    }
}
