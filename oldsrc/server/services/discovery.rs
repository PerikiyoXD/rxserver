//! Service discovery
//!
//! Provides mechanisms for discovering and locating services.

use crate::server::services::{ServiceMetadata, ServiceResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Service discovery criteria
#[derive(Debug, Clone)]
pub struct DiscoveryCriteria {
    /// Service name pattern (supports wildcards)
    pub name_pattern: Option<String>,
    /// Required tags
    pub tags: Vec<String>,
    /// Service version requirements
    pub version_requirement: Option<String>,
    /// Maximum number of results
    pub limit: Option<usize>,
}

impl DiscoveryCriteria {
    /// Create new discovery criteria
    pub fn new() -> Self {
        Self {
            name_pattern: None,
            tags: Vec::new(),
            version_requirement: None,
            limit: None,
        }
    }

    /// Set name pattern
    pub fn with_name_pattern(mut self, pattern: String) -> Self {
        self.name_pattern = Some(pattern);
        self
    }

    /// Add required tag
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Set version requirement
    pub fn with_version(mut self, version: String) -> Self {
        self.version_requirement = Some(version);
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl Default for DiscoveryCriteria {
    fn default() -> Self {
        Self::new()
    }
}

/// Service endpoint information
#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    /// Service name
    pub service_name: String,
    /// Endpoint URL
    pub url: String,
    /// Endpoint type (e.g., "http", "grpc", "websocket")
    pub endpoint_type: String,
    /// Health status
    pub healthy: bool,
    /// Last health check timestamp
    pub last_check: Option<std::time::SystemTime>,
}

/// Service discovery engine
#[derive(Debug)]
pub struct ServiceDiscovery {
    services: Arc<RwLock<HashMap<String, ServiceMetadata>>>,
    endpoints: Arc<RwLock<HashMap<String, Vec<ServiceEndpoint>>>>,
}

impl ServiceDiscovery {
    /// Create a new service discovery instance
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            endpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a service for discovery
    pub async fn register_service(&self, metadata: ServiceMetadata) -> ServiceResult<()> {
        let mut services = self.services.write().await;
        let mut endpoints = self.endpoints.write().await;

        // Create endpoints from metadata
        let service_endpoints: Vec<ServiceEndpoint> = metadata
            .endpoints
            .iter()
            .map(|url| ServiceEndpoint {
                service_name: metadata.name.clone(),
                url: url.clone(),
                endpoint_type: "http".to_string(), // Default type
                healthy: true,
                last_check: Some(std::time::SystemTime::now()),
            })
            .collect();

        services.insert(metadata.name.clone(), metadata.clone());
        endpoints.insert(metadata.name, service_endpoints);

        Ok(())
    }

    /// Unregister a service
    pub async fn unregister_service(&self, name: &str) -> ServiceResult<()> {
        let mut services = self.services.write().await;
        let mut endpoints = self.endpoints.write().await;

        services.remove(name);
        endpoints.remove(name);

        Ok(())
    }

    /// Discover services matching criteria
    pub async fn discover(
        &self,
        criteria: DiscoveryCriteria,
    ) -> ServiceResult<Vec<ServiceMetadata>> {
        let services = self.services.read().await;
        let mut results: Vec<ServiceMetadata> = services
            .values()
            .filter(|service| self.matches_criteria(service, &criteria))
            .cloned()
            .collect();

        // Apply limit if specified
        if let Some(limit) = criteria.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Get service endpoints
    pub async fn get_endpoints(&self, service_name: &str) -> ServiceResult<Vec<ServiceEndpoint>> {
        let endpoints = self.endpoints.read().await;
        let result = endpoints
            .get(service_name)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|endpoint| endpoint.healthy)
            .collect::<Vec<_>>();
        Ok(result)
    }

    /// Get healthy endpoint for a service
    pub async fn get_healthy_endpoint(
        &self,
        service_name: &str,
    ) -> ServiceResult<Option<ServiceEndpoint>> {
        let endpoints = self.get_endpoints(service_name).await?;
        Ok(endpoints.into_iter().find(|e| e.healthy))
    }

    /// Update endpoint health status
    pub async fn update_endpoint_health(
        &self,
        service_name: &str,
        url: &str,
        healthy: bool,
    ) -> ServiceResult<()> {
        let mut endpoints = self.endpoints.write().await;
        if let Some(service_endpoints) = endpoints.get_mut(service_name) {
            if let Some(endpoint) = service_endpoints.iter_mut().find(|e| e.url == url) {
                endpoint.healthy = healthy;
                endpoint.last_check = Some(std::time::SystemTime::now());
            }
        }
        Ok(())
    }

    /// Check if criteria matches service
    fn matches_criteria(&self, service: &ServiceMetadata, criteria: &DiscoveryCriteria) -> bool {
        // Check name pattern
        if let Some(pattern) = &criteria.name_pattern {
            if !self.matches_pattern(&service.name, pattern) {
                return false;
            }
        }

        // Check required tags
        for required_tag in &criteria.tags {
            if !service.tags.contains(required_tag) {
                return false;
            }
        }

        // Check version requirement (simplified)
        if let Some(version_req) = &criteria.version_requirement {
            if service.version != *version_req {
                return false;
            }
        }

        true
    }

    /// Simple pattern matching (supports * wildcard)
    fn matches_pattern(&self, text: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        if pattern.contains('*') {
            // Simple wildcard matching
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return text.starts_with(prefix) && text.ends_with(suffix);
            }
        }
        text == pattern
    }
}

impl Default for ServiceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}
