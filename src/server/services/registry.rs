//! Service registry
//!
//! Maintains a registry of available services and their metadata.

use crate::server::services::{ServiceError, ServiceResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Service metadata
#[derive(Debug, Clone)]
pub struct ServiceMetadata {
    /// Service name
    pub name: String,
    /// Service version
    pub version: String,
    /// Service description
    pub description: String,
    /// Service endpoints
    pub endpoints: Vec<String>,
    /// Service tags for categorization
    pub tags: Vec<String>,
    /// Service dependencies
    pub dependencies: Vec<String>,
}

impl ServiceMetadata {
    /// Create new service metadata
    pub fn new(name: String, version: String, description: String) -> Self {
        Self {
            name,
            version,
            description,
            endpoints: Vec::new(),
            tags: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Add an endpoint
    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoints.push(endpoint);
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Add a dependency
    pub fn with_dependency(mut self, dependency: String) -> Self {
        self.dependencies.push(dependency);
        self
    }
}

/// Service registry for managing service metadata
#[derive(Debug)]
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, ServiceMetadata>>>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the service registry
    pub async fn initialize(&mut self) -> ServiceResult<()> {
        // Initialization logic for the service registry
        // This could include loading service definitions, setting up watchers, etc.
        Ok(())
    }

    /// Register a service in the registry
    pub async fn register(&self, metadata: ServiceMetadata) -> ServiceResult<()> {
        let mut services = self.services.write().await;
        services.insert(metadata.name.clone(), metadata);
        Ok(())
    }

    /// Unregister a service from the registry
    pub async fn unregister(&self, name: &str) -> ServiceResult<()> {
        let mut services = self.services.write().await;
        services.remove(name);
        Ok(())
    }

    /// Get service metadata by name
    pub async fn get_service(&self, name: &str) -> ServiceResult<ServiceMetadata> {
        let services = self.services.read().await;
        services
            .get(name)
            .cloned()
            .ok_or_else(|| ServiceError::NotFound(name.to_string()))
    }

    /// List all registered services
    pub async fn list_services(&self) -> Vec<ServiceMetadata> {
        let services = self.services.read().await;
        services.values().cloned().collect()
    }

    /// Find services by tag
    pub async fn find_by_tag(&self, tag: &str) -> Vec<ServiceMetadata> {
        let services = self.services.read().await;
        services
            .values()
            .filter(|service| service.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }

    /// Find services that depend on a given service
    pub async fn find_dependents(&self, service_name: &str) -> Vec<ServiceMetadata> {
        let services = self.services.read().await;
        services
            .values()
            .filter(|service| service.dependencies.contains(&service_name.to_string()))
            .cloned()
            .collect()
    }

    /// Check if a service is registered
    pub async fn is_registered(&self, name: &str) -> bool {
        let services = self.services.read().await;
        services.contains_key(name)
    }

    /// Get service count
    pub async fn service_count(&self) -> usize {
        let services = self.services.read().await;
        services.len()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
