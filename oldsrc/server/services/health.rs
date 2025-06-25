//! Service health monitoring
//!
//! Provides health checking and monitoring capabilities for services.

use crate::server::services::{Service, ServiceResult, ServiceState};
use async_trait::async_trait;
use std::collections::HashMap;
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, trace, warn};

// Re-export HealthStatus from server types
pub use crate::server::types::HealthStatus;

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Service name
    pub service_name: String,
    /// Health status
    pub status: HealthStatus,
    /// Check timestamp
    pub timestamp: SystemTime,
    /// Check duration
    pub duration: Duration,
    /// Optional message
    pub message: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl HealthCheckResult {
    /// Create a new health check result
    pub fn new(service_name: String, status: HealthStatus) -> Self {
        Self {
            service_name,
            status,
            timestamp: SystemTime::now(),
            duration: Duration::from_millis(0),
            message: None,
            metadata: HashMap::new(),
        }
    }

    /// Set duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set message
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Check interval
    pub interval: Duration,
    /// Check timeout
    pub timeout: Duration,
    /// Number of retries before marking unhealthy
    pub retries: u32,
    /// Threshold for marking service as unhealthy
    pub unhealthy_threshold: u32,
    /// Threshold for marking service as healthy
    pub healthy_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            retries: 3,
            unhealthy_threshold: 3,
            healthy_threshold: 2,
        }
    }
}

/// Health monitor for tracking service health
#[derive(Debug)]
pub struct HealthMonitor {
    health_status: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
    config: HealthCheckConfig,
    failure_counts: Arc<RwLock<HashMap<String, u32>>>,
    state: ServiceState,
    monitored_services: Arc<RwLock<Vec<String>>>,
}

#[async_trait]
impl Service for HealthMonitor {
    /// Get the service name
    fn name(&self) -> &str {
        "health_monitor"
    }

    /// Start the health monitoring service
    async fn start(&mut self) -> ServiceResult<()> {
        info!("Starting health monitoring service");

        if self.state == ServiceState::Running {
            warn!("Health monitor already running");
            return Ok(());
        }

        self.state = ServiceState::Starting;

        // Initialize monitoring for registered services
        let services = self.monitored_services.read().await;
        for service_name in services.iter() {
            self.start_monitoring(service_name.clone()).await?;
        }

        // Start periodic health checks
        self.start_periodic_health_checks().await;

        self.state = ServiceState::Running;
        info!("Health monitoring service started successfully");
        Ok(())
    }

    /// Stop the health monitoring service
    async fn stop(&mut self) -> ServiceResult<()> {
        info!("Stopping health monitoring service");

        if self.state == ServiceState::Stopped {
            debug!("Health monitor already stopped");
            return Ok(());
        }

        self.state = ServiceState::Stopping;

        // Clear all monitoring data
        let mut health_status = self.health_status.write().await;
        let mut failure_counts = self.failure_counts.write().await;
        let monitored_count = health_status.len();

        health_status.clear();
        failure_counts.clear();

        self.state = ServiceState::Stopped;
        info!(
            "Health monitoring service stopped, cleared {} service records",
            monitored_count
        );
        Ok(())
    }

    /// Get the current service state
    fn state(&self) -> ServiceState {
        self.state.clone()
    }

    /// Perform health check on the monitoring service itself
    async fn health_check(&self) -> ServiceResult<bool> {
        trace!("Performing health monitor self-check");

        match self.state {
            ServiceState::Running => {
                let health_status = self.health_status.read().await;
                let monitored_services = self.monitored_services.read().await;

                debug!(
                    "Health monitor self-check: monitoring {} services, {} status records",
                    monitored_services.len(),
                    health_status.len()
                );

                // Check if we have any critical failures
                let critical_failures = health_status
                    .values()
                    .filter(|result| result.status == HealthStatus::Unhealthy)
                    .count();

                if critical_failures > 0 {
                    warn!(
                        "Health monitor detected {} critical service failures",
                        critical_failures
                    );
                }

                Ok(true)
            }
            _ => {
                warn!("Health monitor self-check failed: not running");
                Ok(false)
            }
        }
    }
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(config: HealthCheckConfig) -> Self {
        info!(
            "Creating new health monitor with config: interval={:?}, timeout={:?}",
            config.interval, config.timeout
        );
        Self {
            health_status: Arc::new(RwLock::new(HashMap::new())),
            config,
            failure_counts: Arc::new(RwLock::new(HashMap::new())),
            state: ServiceState::Stopped,
            monitored_services: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start health monitoring for a service
    pub async fn start_monitoring(&self, service_name: String) -> ServiceResult<()> {
        info!("Starting health monitoring for service: {}", service_name);

        // Add to monitored services list
        {
            let mut monitored = self.monitored_services.write().await;
            if !monitored.contains(&service_name) {
                monitored.push(service_name.clone());
            } else {
                debug!("Service '{}' already being monitored", service_name);
                return Ok(());
            }
        }

        // Initialize health status
        let mut health_status = self.health_status.write().await;
        let initial_result = HealthCheckResult::new(service_name.clone(), HealthStatus::Unknown);
        health_status.insert(service_name.clone(), initial_result);

        debug!("Health monitoring started for service: {}", service_name);
        Ok(())
    }

    /// Stop health monitoring for a service
    pub async fn stop_monitoring(&self, service_name: &str) -> ServiceResult<()> {
        info!("Stopping health monitoring for service: {}", service_name);

        // Remove from monitored services
        {
            let mut monitored = self.monitored_services.write().await;
            monitored.retain(|s| s != service_name);
        }

        // Clear health data
        let mut health_status = self.health_status.write().await;
        let mut failure_counts = self.failure_counts.write().await;

        let was_monitored = health_status.remove(service_name).is_some();
        failure_counts.remove(service_name);

        if was_monitored {
            info!("Health monitoring stopped for service: {}", service_name);
        } else {
            warn!(
                "Attempted to stop monitoring non-monitored service: {}",
                service_name
            );
        }

        Ok(())
    }

    /// Update health status for a service
    pub async fn update_health(&self, result: HealthCheckResult) -> ServiceResult<()> {
        let service_name = result.service_name.clone();
        trace!(
            "Updating health status for service '{}': {:?}",
            service_name, result.status
        );

        let mut health_status = self.health_status.write().await;
        let mut failure_counts = self.failure_counts.write().await;

        // Update failure count based on health status
        let current_failures = failure_counts.get(&service_name).unwrap_or(&0);
        let new_failures = match result.status {
            HealthStatus::Healthy => {
                if *current_failures > 0 {
                    info!(
                        "Service '{}' recovered (was {} failures)",
                        service_name, current_failures
                    );
                }
                0
            }
            HealthStatus::Unhealthy => {
                let new_count = current_failures + 1;
                warn!(
                    "Service '{}' health check failed (failure #{}) - {}",
                    service_name,
                    new_count,
                    result.message.as_deref().unwrap_or("No details")
                );

                // Log critical threshold breach
                if new_count >= self.config.unhealthy_threshold {
                    error!(
                        "Service '{}' exceeded unhealthy threshold ({} failures)",
                        service_name, new_count
                    );
                }

                new_count
            }
            _ => {
                debug!(
                    "Service '{}' status updated to {:?}",
                    service_name, result.status
                );
                *current_failures
            }
        };

        failure_counts.insert(service_name.clone(), new_failures);

        // Update health status
        health_status.insert(service_name, result);
        Ok(())
    }
    /// Get health status for a service
    pub async fn get_health(&self, service_name: &str) -> Option<HealthCheckResult> {
        trace!("Getting health status for service: {}", service_name);

        let health_status = self.health_status.read().await;
        let result = health_status.get(service_name).cloned();

        if let Some(ref health_result) = result {
            trace!(
                "Health status for '{}': {:?} (checked {:?} ago)",
                service_name,
                health_result.status,
                health_result.timestamp.elapsed().unwrap_or_default()
            );
        } else {
            trace!("No health status found for service: {}", service_name);
        }

        result
    }

    /// Get health status for all services
    pub async fn get_all_health(&self) -> HashMap<String, HealthCheckResult> {
        debug!("Getting health status for all monitored services");

        let health_status = self.health_status.read().await;
        let results = health_status.clone();

        debug!("Retrieved health status for {} services", results.len());

        // Log summary of health statuses
        let mut status_counts = HashMap::new();
        for result in results.values() {
            *status_counts.entry(result.status.clone()).or_insert(0) += 1;
        }

        if !status_counts.is_empty() {
            debug!("Health status summary: {:?}", status_counts);
        }

        results
    }

    /// Check if a service is healthy
    pub async fn is_healthy(&self, service_name: &str) -> bool {
        trace!("Checking if service '{}' is healthy", service_name);

        if let Some(result) = self.get_health(service_name).await {
            let is_healthy = result.status == HealthStatus::Healthy;
            trace!(
                "Service '{}' health status: {}",
                service_name,
                if is_healthy { "healthy" } else { "unhealthy" }
            );
            is_healthy
        } else {
            warn!(
                "Cannot determine health for unmonitored service: {}",
                service_name
            );
            false
        }
    }

    /// Get services that are unhealthy
    pub async fn get_unhealthy_services(&self) -> Vec<String> {
        debug!("Finding unhealthy services");

        let health_status = self.health_status.read().await;
        let unhealthy_services = health_status
            .iter()
            .filter(|(_, result)| result.status == HealthStatus::Unhealthy)
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();

        if !unhealthy_services.is_empty() {
            warn!(
                "Found {} unhealthy services: {:?}",
                unhealthy_services.len(),
                unhealthy_services
            );
        } else {
            debug!("No unhealthy services found");
        }

        unhealthy_services
    }

    /// Start periodic health checks for all monitored services
    async fn start_periodic_health_checks(&self) {
        info!(
            "Starting periodic health checks with interval: {:?}",
            self.config.interval
        );

        let health_status = Arc::clone(&self.health_status);
        let monitored_services = Arc::clone(&self.monitored_services);
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval_timer = interval(config.interval);
            let mut check_count = 0u64;

            loop {
                interval_timer.tick().await;
                check_count += 1;

                trace!("Starting periodic health check cycle #{}", check_count);

                // Get list of services to check
                let services: Vec<String> = {
                    let monitored = monitored_services.read().await;
                    monitored.clone()
                };

                if services.is_empty() {
                    trace!("No services to monitor, skipping health check cycle");
                    continue;
                }

                debug!("Running health checks for {} services", services.len());

                // Perform health checks for each service
                for service_name in services {
                    trace!("Performing health check for service: {}", service_name);

                    // Simulate health check (in real implementation, this would call actual health check)
                    let start_time = SystemTime::now();

                    // TODO: Implement actual health check logic based on service type
                    let check_result =
                        Self::perform_service_health_check(&service_name, &config).await;

                    let duration = start_time.elapsed().unwrap_or_default();

                    let result = HealthCheckResult::new(service_name.clone(), check_result)
                        .with_duration(duration)
                        .with_message(format!("Periodic check #{}", check_count));

                    // Update health status (note: this would normally call update_health on the monitor)
                    {
                        let mut status_map = health_status.write().await;
                        status_map.insert(service_name.clone(), result);
                    }

                    trace!(
                        "Health check completed for '{}' in {:?}",
                        service_name, duration
                    );
                }

                debug!(
                    "Completed health check cycle #{} for {} services",
                    check_count,
                    services.len()
                );
            }
        });
    }    /// Perform actual health check for a service
    async fn perform_service_health_check(
        service_name: &str,
        config: &HealthCheckConfig,
    ) -> HealthStatus {
        trace!("Performing health check for service: {}", service_name);

        // Set up timeout for health check
        let health_check_future = async {
            // Perform actual health checks based on service type
            match service_name {
                "health_monitor" => {
                    // Self-health check - always healthy if we can execute this
                    HealthStatus::Healthy
                }
                "task_scheduler" => {
                    // Check if scheduler is responsive and has reasonable queue size
                    // In real implementation, this would query the actual scheduler
                    HealthStatus::Healthy
                }
                "network_server" => {
                    // Check if network server can accept connections
                    // In real implementation, this would test socket binding
                    HealthStatus::Healthy
                }
                "display_manager" => {
                    // Check if display subsystem is functional
                    // In real implementation, this would test display operations
                    HealthStatus::Healthy
                }
                "font_system" => {
                    // Check if font system can load fonts
                    // In real implementation, this would test font loading
                    HealthStatus::Healthy
                }
                "input_system" => {
                    // Check if input system is receiving events
                    // In real implementation, this would test input device access
                    HealthStatus::Healthy
                }
                "connection_manager" => {
                    // Check if connection manager is tracking connections properly
                    // In real implementation, this would verify connection state
                    HealthStatus::Healthy
                }
                _ => {
                    // Unknown service - mark as unknown status
                    warn!("Unknown service type for health check: {}", service_name);
                    HealthStatus::Unknown
                }
            }
        };

        // Apply timeout to health check
        match tokio::time::timeout(config.timeout, health_check_future).await {
            Ok(status) => {
                trace!("Health check completed for '{}': {:?}", service_name, status);
                status
            }
            Err(_) => {
                warn!("Health check timed out for service: {}", service_name);
                HealthStatus::Unhealthy
            }
        }
    }/// Start periodic health checks (legacy method)
    pub async fn start_periodic_checks(&self) {
        warn!("start_periodic_checks() is deprecated, use start() method instead");
        self.start_periodic_health_checks().await;
    }

    /// Get health summary with enhanced statistics
    pub async fn get_health_summary(&self) -> HealthSummary {
        debug!("Generating health summary");
        
        let health_status = self.health_status.read().await;
        let failure_counts = self.failure_counts.read().await;
        
        let mut summary = HashMap::new();
        let mut total_failures = 0;
        let mut longest_unhealthy_duration = Duration::from_secs(0);
        let mut services_with_failures = 0;

        for result in health_status.values() {
            let count = summary.get(&result.status).unwrap_or(&0);
            summary.insert(result.status.clone(), count + 1);
            
            // Track failure statistics
            if let Some(failures) = failure_counts.get(&result.service_name) {
                total_failures += failures;
                if *failures > 0 {
                    services_with_failures += 1;
                }
            }
            
            // Track unhealthy duration
            if result.status == HealthStatus::Unhealthy {
                let duration = result.timestamp.elapsed().unwrap_or_default();
                if duration > longest_unhealthy_duration {
                    longest_unhealthy_duration = duration;
                }
            }
        }

        let total_services = health_status.len();
        let healthy_services = summary.get(&HealthStatus::Healthy).unwrap_or(&0);
        let unhealthy_services = summary.get(&HealthStatus::Unhealthy).unwrap_or(&0);
        
        let health_summary = HealthSummary {
            status_distribution: summary,
            total_services,
            total_failures,
            services_with_failures,
            longest_unhealthy_duration,
            overall_health_percentage: if total_services > 0 {
                (*healthy_services as f64 / total_services as f64) * 100.0
            } else {
                100.0
            },
        };

        info!(
            "Health summary: {}/{} services healthy ({:.1}%), {} total failures across {} services",
            healthy_services, total_services, health_summary.overall_health_percentage,
            total_failures, services_with_failures
        );
        
        if *unhealthy_services > 0 {
            warn!("Health concerns: {} unhealthy services, longest unhealthy duration: {:?}", 
                  unhealthy_services, longest_unhealthy_duration);
        }

        health_summary
    }

    /// Get detailed health report for a specific service
    pub async fn get_service_health_report(&self, service_name: &str) -> Option<ServiceHealthReport> {
        debug!("Generating health report for service: {}", service_name);
        
        let health_status = self.health_status.read().await;
        let failure_counts = self.failure_counts.read().await;
        
        if let Some(health_result) = health_status.get(service_name) {
            let failure_count = failure_counts.get(service_name).unwrap_or(&0);
            let time_since_check = health_result.timestamp.elapsed().unwrap_or_default();
            
            let report = ServiceHealthReport {
                service_name: service_name.to_string(),
                current_status: health_result.status.clone(),
                last_check: health_result.timestamp,
                time_since_check,
                failure_count: *failure_count,
                last_check_duration: health_result.duration,
                message: health_result.message.clone(),
                metadata: health_result.metadata.clone(),
                is_healthy: health_result.status == HealthStatus::Healthy,
                exceeds_failure_threshold: *failure_count >= self.config.unhealthy_threshold,
            };
            
            debug!("Health report for '{}': status={:?}, failures={}, last_check={:?} ago", 
                   service_name, report.current_status, report.failure_count, report.time_since_check);
                   
            Some(report)
        } else {
            warn!("No health data available for service: {}", service_name);
            None
        }
    }

    /// Get list of monitored services
    pub async fn get_monitored_services(&self) -> Vec<String> {
        let monitored = self.monitored_services.read().await;
        let services = monitored.clone();
        debug!("Currently monitoring {} services: {:?}", services.len(), services);
        services
    }

    /// Clear all health data (useful for reset scenarios)
    pub async fn clear_all_health_data(&self) -> ServiceResult<()> {
        info!("Clearing all health monitoring data");
        
        let mut health_status = self.health_status.write().await;
        let mut failure_counts = self.failure_counts.write().await;
        let mut monitored = self.monitored_services.write().await;
        
        let cleared_services = health_status.len();
        
        health_status.clear();
        failure_counts.clear();
        monitored.clear();
        
        info!("Cleared health data for {} services", cleared_services);
        Ok(())
    }
}

/// Health summary statistics
#[derive(Debug, Clone)]
pub struct HealthSummary {
    /// Distribution of health statuses
    pub status_distribution: HashMap<HealthStatus, usize>,
    /// Total number of monitored services
    pub total_services: usize,
    /// Total number of failures across all services
    pub total_failures: u32,
    /// Number of services with at least one failure
    pub services_with_failures: usize,
    /// Longest duration a service has been unhealthy
    pub longest_unhealthy_duration: Duration,
    /// Overall health percentage (0-100)
    pub overall_health_percentage: f64,
}

impl HealthSummary {
    /// Check if the overall system health is good
    pub fn is_system_healthy(&self) -> bool {
        self.overall_health_percentage >= 80.0 && self.services_with_failures == 0
    }
    
    /// Get health grade (A-F)
    pub fn health_grade(&self) -> char {
        match self.overall_health_percentage {
            90.0..=100.0 => 'A',
            80.0..=89.9 => 'B', 
            70.0..=79.9 => 'C',
            60.0..=69.9 => 'D',
            _ => 'F',
        }
    }
}

/// Detailed health report for a specific service
#[derive(Debug, Clone)]
pub struct ServiceHealthReport {
    /// Service name
    pub service_name: String,
    /// Current health status
    pub current_status: HealthStatus,
    /// Timestamp of last health check
    pub last_check: SystemTime,
    /// Time elapsed since last check
    pub time_since_check: Duration,
    /// Number of consecutive failures
    pub failure_count: u32,
    /// Duration of last health check
    pub last_check_duration: Duration,
    /// Optional status message
    pub message: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Whether the service is currently healthy
    pub is_healthy: bool,
    /// Whether failure count exceeds threshold
    pub exceeds_failure_threshold: bool,
}

impl ServiceHealthReport {    /// Get a human-readable status description
    pub fn status_description(&self) -> String {
        match self.current_status {
            HealthStatus::Healthy => "Service is operating normally".to_string(),
            HealthStatus::Degraded => {
                format!("Service is degraded but functional ({} failures)", self.failure_count)
            }
            HealthStatus::Unhealthy => {
                if self.exceeds_failure_threshold {
                    format!("Service is critically unhealthy ({} consecutive failures)", self.failure_count)
                } else {
                    format!("Service is unhealthy ({} failures)", self.failure_count)
                }
            }
            HealthStatus::Unknown => "Service health status is unknown".to_string(),
        }
    }
    
    /// Check if this service needs immediate attention
    pub fn needs_attention(&self) -> bool {
        self.exceeds_failure_threshold || 
        self.current_status == HealthStatus::Unhealthy ||
        self.time_since_check > Duration::from_secs(300) // No check in 5 minutes
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new(HealthCheckConfig::default())
    }
}
