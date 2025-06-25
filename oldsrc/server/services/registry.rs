//! Service registry
//!
//! Maintains a registry of available services and their metadata.

use crate::server::services::{Service, ServiceError, ServiceResult, ServiceState};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};

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

    /// Check if service has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    /// Check if service depends on another service
    pub fn depends_on(&self, service_name: &str) -> bool {
        self.dependencies.contains(&service_name.to_string())
    }

    /// Get service identifier (name:version)
    pub fn identifier(&self) -> String {
        format!("{}:{}", self.name, self.version)
    }

    /// Check if this is a core service
    pub fn is_core_service(&self) -> bool {
        self.has_tag("core")
    }
}

/// Service registry for managing service metadata
#[derive(Debug)]
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, ServiceMetadata>>>,
    state: ServiceState,
}

#[async_trait]
impl Service for ServiceRegistry {
    /// Get the service name
    fn name(&self) -> &str {
        "service_registry"
    }
    /// Start the service registry
    async fn start(&mut self) -> ServiceResult<()> {
        info!("Starting service registry");

        if self.state == ServiceState::Running {
            warn!("Service registry already running");
            return Ok(());
        }

        self.state = ServiceState::Starting;

        // Initialize core services
        self.register_core_services().await?;

        // Validate the registry after core service registration
        let validation = self.validate_registry().await;

        if !validation.is_valid {
            error!(
                "Registry validation failed during startup: {:?}",
                validation.errors
            );
            self.state = ServiceState::Failed("Validation failed".to_string());
            return Err(ServiceError::RegistryValidation(
                validation.errors.join("; "),
            ));
        }

        if !validation.warnings.is_empty() {
            warn!(
                "Registry validation warnings during startup: {:?}",
                validation.warnings
            );
        }

        // Check if we have sufficient core services
        if !validation.statistics.has_sufficient_core_services() {
            error!("Insufficient core services for startup");
            self.state = ServiceState::Failed("Insufficient core services".to_string());
            return Err(ServiceError::InsufficientCoreServices);
        }

        self.state = ServiceState::Running;

        // Log startup summary
        let health_summary = self.get_health_summary().await;
        info!(
            "Service registry started successfully - Status: {}, Services: {}, Core: {}",
            health_summary.health_status(),
            health_summary.total_services,
            health_summary.core_services
        );

        Ok(())
    }

    /// Stop the service registry
    async fn stop(&mut self) -> ServiceResult<()> {
        info!("Stopping service registry");

        if self.state == ServiceState::Stopped {
            debug!("Service registry already stopped");
            return Ok(());
        }

        // Cleanup and unregister services
        let service_count = self.service_count().await;
        if service_count > 0 {
            info!("Unregistering {} services before shutdown", service_count);
            let mut services = self.services.write().await;
            services.clear();
        }

        self.state = ServiceState::Stopped;
        info!("Service registry stopped successfully");
        Ok(())
    }
    /// Get the current service state
    fn state(&self) -> ServiceState {
        self.state.clone()
    }

    /// Perform health check
    async fn health_check(&self) -> ServiceResult<bool> {
        trace!("Performing service registry health check");

        match self.state {
            ServiceState::Running => {
                let stats = self.get_statistics().await;

                // Check if we have sufficient core services
                if !stats.has_sufficient_core_services() {
                    warn!(
                        "Insufficient core services: found {}, expected at least 3",
                        stats.core_services
                    );
                    return Ok(false);
                }

                // Validate service dependencies
                if let Err(validation_errors) = self.validate_service_dependencies().await {
                    warn!(
                        "Service dependency validation failed: {:?}",
                        validation_errors
                    );
                    return Ok(false);
                }

                debug!(
                    "Service registry health check passed: {} services registered, {} core services",
                    stats.total_services, stats.core_services
                );
                Ok(true)
            }
            _ => {
                warn!("Service registry health check failed: not running");
                Ok(false)
            }
        }
    }
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        info!("Creating new service registry");
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            state: ServiceState::Stopped,
        }
    }

    /// Initialize the service registry
    pub async fn initialize(&mut self) -> ServiceResult<()> {
        info!("Initializing service registry");

        // Initialization logic for the service registry
        // This could include loading service definitions, setting up watchers, etc.

        debug!("Service registry initialization completed");
        Ok(())
    }

    /// Register core system services
    async fn register_core_services(&self) -> ServiceResult<()> {
        info!("Registering core system services");

        // Register essential services
        let core_services = vec![
            ServiceMetadata::new(
                "health_monitor".to_string(),
                "1.0.0".to_string(),
                "Health monitoring service".to_string(),
            )
            .with_tag("core".to_string())
            .with_tag("monitoring".to_string()),
            ServiceMetadata::new(
                "task_scheduler".to_string(),
                "1.0.0".to_string(),
                "Task scheduling service".to_string(),
            )
            .with_tag("core".to_string())
            .with_tag("scheduler".to_string()),
            ServiceMetadata::new(
                "network_server".to_string(),
                "1.0.0".to_string(),
                "Network connection server".to_string(),
            )
            .with_tag("core".to_string())
            .with_tag("network".to_string()),
        ];

        for service in core_services {
            self.register(service).await?;
        }

        info!("Core system services registered successfully");
        Ok(())
    }

    /// Register a service in the registry
    pub async fn register(&self, metadata: ServiceMetadata) -> ServiceResult<()> {
        info!(
            "Registering service: {} v{}",
            metadata.name, metadata.version
        );
        debug!(
            "Service details: description='{}', tags={:?}, dependencies={:?}",
            metadata.description, metadata.tags, metadata.dependencies
        );

        let mut services = self.services.write().await;

        // Check if service already exists
        if services.contains_key(&metadata.name) {
            warn!(
                "Service '{}' already registered, updating metadata",
                metadata.name
            );
        }

        services.insert(metadata.name.clone(), metadata.clone());

        let total_services = services.len();
        drop(services); // Release lock early

        info!(
            "Service registered successfully. Total services: {}",
            total_services
        );

        // If this is a core service registration, check if we now have sufficient core services
        if metadata.is_core_service() {
            let stats = self.get_statistics().await;
            if stats.has_sufficient_core_services() {
                info!(
                    "Core services threshold reached: {} core services registered",
                    stats.core_services
                );
            } else {
                debug!("Core services: {}/3 registered", stats.core_services);
            }
        }

        Ok(())
    }

    /// Unregister a service from the registry
    pub async fn unregister(&self, name: &str) -> ServiceResult<()> {
        info!("Unregistering service: {}", name);

        let mut services = self.services.write().await;

        match services.remove(name) {
            Some(metadata) => {
                info!(
                    "Service '{}' v{} unregistered successfully",
                    metadata.name, metadata.version
                );

                // Check for dependent services
                let dependents = services
                    .values()
                    .filter(|service| service.dependencies.contains(&name.to_string()))
                    .map(|s| &s.name)
                    .collect::<Vec<_>>();

                if !dependents.is_empty() {
                    warn!(
                        "Warning: {} dependent services still registered: {:?}",
                        dependents.len(),
                        dependents
                    );
                }

                Ok(())
            }
            None => {
                warn!("Attempted to unregister non-existent service: {}", name);
                Err(ServiceError::NotFound(name.to_string()))
            }
        }
    }
    /// Get service metadata by name
    pub async fn get_service(&self, name: &str) -> ServiceResult<ServiceMetadata> {
        trace!("Getting service metadata for: {}", name);

        let services = self.services.read().await;
        match services.get(name).cloned() {
            Some(metadata) => {
                debug!("Found service '{}' v{}", metadata.name, metadata.version);
                Ok(metadata)
            }
            None => {
                debug!("Service '{}' not found in registry", name);
                Err(ServiceError::NotFound(name.to_string()))
            }
        }
    }

    /// List all registered services
    pub async fn list_services(&self) -> Vec<ServiceMetadata> {
        trace!("Listing all registered services");

        let services = self.services.read().await;
        let service_list = services.values().cloned().collect::<Vec<_>>();

        debug!("Retrieved {} registered services", service_list.len());
        service_list
    }

    /// Find services by tag
    pub async fn find_by_tag(&self, tag: &str) -> Vec<ServiceMetadata> {
        debug!("Finding services with tag: {}", tag);

        let services = self.services.read().await;
        let filtered_services = services
            .values()
            .filter(|service| service.tags.contains(&tag.to_string()))
            .cloned()
            .collect::<Vec<_>>();

        debug!(
            "Found {} services with tag '{}'",
            filtered_services.len(),
            tag
        );
        filtered_services
    }

    /// Find services that depend on a given service
    pub async fn find_dependents(&self, service_name: &str) -> Vec<ServiceMetadata> {
        debug!("Finding services dependent on: {}", service_name);

        let services = self.services.read().await;
        let dependents = services
            .values()
            .filter(|service| service.dependencies.contains(&service_name.to_string()))
            .cloned()
            .collect::<Vec<_>>();

        debug!(
            "Found {} services dependent on '{}'",
            dependents.len(),
            service_name
        );

        if !dependents.is_empty() {
            let dependent_names: Vec<&String> = dependents.iter().map(|s| &s.name).collect();
            debug!("Dependent services: {:?}", dependent_names);
        }

        dependents
    }

    /// Check if a service is registered
    pub async fn is_registered(&self, name: &str) -> bool {
        trace!("Checking if service is registered: {}", name);

        let services = self.services.read().await;
        let is_registered = services.contains_key(name);

        trace!("Service '{}' registration status: {}", name, is_registered);
        is_registered
    }

    /// Get service count
    pub async fn service_count(&self) -> usize {
        let services = self.services.read().await;
        let count = services.len();
        trace!("Current service count: {}", count);
        count
    }

    /// Get registry statistics
    pub async fn get_statistics(&self) -> RegistryStatistics {
        debug!("Collecting service registry statistics");

        let services = self.services.read().await;
        let total_services = services.len();

        // Count services by tags
        let mut tag_counts = HashMap::new();
        let mut dependency_count = 0;

        for service in services.values() {
            dependency_count += service.dependencies.len();

            for tag in &service.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        let stats = RegistryStatistics {
            total_services,
            core_services: tag_counts.get("core").copied().unwrap_or(0),
            total_dependencies: dependency_count,
            tag_distribution: tag_counts,
        };

        debug!(
            "Registry statistics: {} total, {} core, {} dependencies",
            stats.total_services, stats.core_services, stats.total_dependencies
        );

        stats
    }

    /// Validate service dependencies
    async fn validate_service_dependencies(&self) -> Result<(), Vec<String>> {
        debug!("Validating service dependencies");

        let services = self.services.read().await;
        let mut errors = Vec::new();

        for service in services.values() {
            for dependency in &service.dependencies {
                if !services.contains_key(dependency) {
                    let error = format!(
                        "Service '{}' depends on missing service '{}'",
                        service.name, dependency
                    );
                    errors.push(error);
                }
            }
        }

        if errors.is_empty() {
            debug!("All service dependencies validated successfully");
            Ok(())
        } else {
            warn!("Found {} dependency validation errors", errors.len());
            Err(errors)
        }
    }

    /// Perform comprehensive registry validation
    pub async fn validate_registry(&self) -> RegistryValidationResult {
        info!("Performing comprehensive registry validation");

        let stats = self.get_statistics().await;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Check core services
        if !stats.has_sufficient_core_services() {
            errors.push(format!(
                "Insufficient core services: found {}, expected at least 3",
                stats.core_services
            ));
        }

        // Check for services without tags
        let services = self.services.read().await;
        let untagged_services: Vec<_> = services
            .values()
            .filter(|service| service.tags.is_empty())
            .map(|service| &service.name)
            .collect();

        if !untagged_services.is_empty() {
            warnings.push(format!(
                "Found {} services without tags: {:?}",
                untagged_services.len(),
                untagged_services
            ));
        }

        // Check for circular dependencies (basic check)
        for service in services.values() {
            for dependency in &service.dependencies {
                if let Some(dep_service) = services.get(dependency) {
                    if dep_service.dependencies.contains(&service.name) {
                        warnings.push(format!(
                            "Potential circular dependency between '{}' and '{}'",
                            service.name, dependency
                        ));
                    }
                }
            }
        }

        // Validate dependencies
        if let Err(dep_errors) = self.validate_service_dependencies().await {
            errors.extend(dep_errors);
        }

        let result = RegistryValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            statistics: stats,
        };

        if result.is_valid {
            info!(
                "Registry validation passed with {} warnings",
                result.warnings.len()
            );
        } else {
            warn!(
                "Registry validation failed with {} errors and {} warnings",
                result.errors.len(),
                result.warnings.len()
            );
        }

        result
    }

    /// Get registry health summary
    pub async fn get_health_summary(&self) -> RegistryHealthSummary {
        let stats = self.get_statistics().await;
        let validation = self.validate_registry().await;

        RegistryHealthSummary {
            state: self.state.clone(),
            total_services: stats.total_services,
            core_services: stats.core_services,
            has_sufficient_core: stats.has_sufficient_core_services(),
            validation_errors: validation.errors.len(),
            validation_warnings: validation.warnings.len(),
            top_tags: stats.top_tags(5),
        }
    }

    /// Perform periodic health monitoring and report issues
    pub async fn perform_health_monitoring(&self) -> RegistryHealthSummary {
        debug!("Performing periodic health monitoring");

        let summary = self.get_health_summary().await;

        // Log health status
        match summary.health_status() {
            "Healthy" => {
                trace!("Registry health monitoring: All systems operational");
            }
            "Warning" => {
                warn!(
                    "Registry health monitoring: {} warnings detected - Core services: {}/{}, Validation warnings: {}",
                    if summary.has_sufficient_core { 0 } else { 1 } + summary.validation_warnings,
                    summary.core_services,
                    3, // Expected core services
                    summary.validation_warnings
                );
            }
            "Critical" => {
                error!(
                    "Registry health monitoring: CRITICAL - {} validation errors, Core services: {}/{}",
                    summary.validation_errors, summary.core_services, 3
                );
            }
            _ => {
                warn!("Registry health monitoring: Unknown status");
            }
        }

        // Log top service categories
        if !summary.top_tags.is_empty() {
            debug!("Service distribution by tags: {:?}", summary.top_tags);
        }

        summary
    }

    /// Generate a detailed registry report
    pub async fn generate_registry_report(&self) -> String {
        let validation = self.validate_registry().await;
        let summary = self.get_health_summary().await;

        let mut report = String::new();
        report.push_str("=== Service Registry Report ===\n");
        report.push_str(&format!("Status: {}\n", summary.health_status()));
        report.push_str(&format!("State: {:?}\n", summary.state));
        report.push_str(&format!("Total Services: {}\n", summary.total_services));
        report.push_str(&format!(
            "Core Services: {} (sufficient: {})\n",
            summary.core_services, summary.has_sufficient_core
        ));

        if !summary.top_tags.is_empty() {
            report.push_str("\nService Categories:\n");
            for (tag, count) in &summary.top_tags {
                report.push_str(&format!("  {}: {} services\n", tag, count));
            }
        }

        if !validation.errors.is_empty() {
            report.push_str("\nValidation Errors:\n");
            for error in &validation.errors {
                report.push_str(&format!("  - {}\n", error));
            }
        }

        if !validation.warnings.is_empty() {
            report.push_str("\nValidation Warnings:\n");
            for warning in &validation.warnings {
                report.push_str(&format!("  - {}\n", warning));
            }
        }

        report.push_str("=== End Report ===\n");
        report
    }
}

/// Registry statistics for monitoring and analysis
#[derive(Debug, Clone)]
pub struct RegistryStatistics {
    /// Total number of registered services
    pub total_services: usize,
    /// Number of core services
    pub core_services: usize,
    /// Total number of dependencies across all services
    pub total_dependencies: usize,
    /// Distribution of services by tag
    pub tag_distribution: HashMap<String, usize>,
}

impl RegistryStatistics {
    /// Get the most common tags
    pub fn top_tags(&self, limit: usize) -> Vec<(String, usize)> {
        let mut tags: Vec<_> = self
            .tag_distribution
            .iter()
            .map(|(tag, count)| (tag.clone(), *count))
            .collect();

        tags.sort_by(|a, b| b.1.cmp(&a.1));
        tags.truncate(limit);
        tags
    }

    /// Check if the registry has sufficient core services
    pub fn has_sufficient_core_services(&self) -> bool {
        self.core_services >= 3 // Expecting at least health, scheduler, network
    }
}

/// Registry validation result
#[derive(Debug, Clone)]
pub struct RegistryValidationResult {
    /// Whether the registry is valid
    pub is_valid: bool,
    /// Validation errors that must be fixed
    pub errors: Vec<String>,
    /// Validation warnings that should be addressed
    pub warnings: Vec<String>,
    /// Current registry statistics
    pub statistics: RegistryStatistics,
}

/// Registry health summary for monitoring
#[derive(Debug, Clone)]
pub struct RegistryHealthSummary {
    /// Current service state
    pub state: ServiceState,
    /// Total number of services
    pub total_services: usize,
    /// Number of core services
    pub core_services: usize,
    /// Whether we have sufficient core services
    pub has_sufficient_core: bool,
    /// Number of validation errors
    pub validation_errors: usize,
    /// Number of validation warnings
    pub validation_warnings: usize,
    /// Most common service tags
    pub top_tags: Vec<(String, usize)>,
}

impl RegistryHealthSummary {
    /// Check if the registry is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.state, ServiceState::Running)
            && self.has_sufficient_core
            && self.validation_errors == 0
    }

    /// Get health status as string
    pub fn health_status(&self) -> &'static str {
        if self.is_healthy() {
            "Healthy"
        } else if self.validation_errors > 0 {
            "Critical"
        } else if self.validation_warnings > 0 || !self.has_sufficient_core {
            "Warning"
        } else {
            "Unknown"
        }
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
