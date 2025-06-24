//! Service manager
//!
//! Manages the lifecycle and operations of server services.

use crate::server::services::{ServiceError, ServiceResult};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Service state enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceState {
    /// Service is stopped
    Stopped,
    /// Service is starting up
    Starting,
    /// Service is running
    Running,
    /// Service is stopping
    Stopping,
    /// Service has failed
    Failed(String),
}

/// Service trait that all services must implement
#[async_trait::async_trait]
pub trait Service: Send + Sync + Debug {
    /// Get the service name
    fn name(&self) -> &str;

    /// Start the service
    async fn start(&mut self) -> ServiceResult<()>;

    /// Stop the service
    async fn stop(&mut self) -> ServiceResult<()>;

    /// Get the current service state
    fn state(&self) -> ServiceState;

    /// Perform health check
    async fn health_check(&self) -> ServiceResult<bool>;
}

/// Service manager for managing multiple services
#[derive(Debug)]
pub struct ServiceManager {
    services: Arc<RwLock<HashMap<String, Box<dyn Service>>>>,
}

impl ServiceManager {
    /// Create a new service manager
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a service
    pub async fn register_service(&self, service: Box<dyn Service>) -> ServiceResult<()> {
        let name = service.name().to_string();
        let mut services = self.services.write().await;
        services.insert(name, service);
        Ok(())
    }

    /// Start a service by name
    pub async fn start_service(&self, name: &str) -> ServiceResult<()> {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(name) {
            service.start().await
        } else {
            Err(ServiceError::NotFound(name.to_string()))
        }
    }

    /// Stop a service by name
    pub async fn stop_service(&self, name: &str) -> ServiceResult<()> {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(name) {
            service.stop().await
        } else {
            Err(ServiceError::NotFound(name.to_string()))
        }
    }

    /// Start all services
    pub async fn start_all(&self) -> ServiceResult<()> {
        let mut services = self.services.write().await;
        for service in services.values_mut() {
            service.start().await?;
        }
        Ok(())
    }

    /// Stop all services
    pub async fn stop_all(&self) -> ServiceResult<()> {
        let mut services = self.services.write().await;
        for service in services.values_mut() {
            service.stop().await?;
        }
        Ok(())
    }

    /// Get service state
    pub async fn get_service_state(&self, name: &str) -> ServiceResult<ServiceState> {
        let services = self.services.read().await;
        if let Some(service) = services.get(name) {
            Ok(service.state())
        } else {
            Err(ServiceError::NotFound(name.to_string()))
        }
    }

    /// List all registered services
    pub async fn list_services(&self) -> Vec<String> {
        let services = self.services.read().await;
        services.keys().cloned().collect()
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}
