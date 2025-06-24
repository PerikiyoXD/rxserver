//! Service health monitoring
//!
//! Provides health checking and monitoring capabilities for services.

use crate::server::services::ServiceResult;
use std::collections::HashMap;
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::RwLock;
use tokio::time::interval;

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
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            health_status: Arc::new(RwLock::new(HashMap::new())),
            config,
            failure_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start health monitoring for a service
    pub async fn start_monitoring(&self, service_name: String) -> ServiceResult<()> {
        let mut health_status = self.health_status.write().await;
        let initial_result = HealthCheckResult::new(service_name.clone(), HealthStatus::Unknown);
        health_status.insert(service_name, initial_result);
        Ok(())
    }

    /// Stop health monitoring for a service
    pub async fn stop_monitoring(&self, service_name: &str) -> ServiceResult<()> {
        let mut health_status = self.health_status.write().await;
        let mut failure_counts = self.failure_counts.write().await;
        health_status.remove(service_name);
        failure_counts.remove(service_name);
        Ok(())
    }

    /// Update health status for a service
    pub async fn update_health(&self, result: HealthCheckResult) -> ServiceResult<()> {
        let service_name = result.service_name.clone();
        let mut health_status = self.health_status.write().await;
        let mut failure_counts = self.failure_counts.write().await;

        // Update failure count based on health status
        let current_failures = failure_counts.get(&service_name).unwrap_or(&0);
        let new_failures = match result.status {
            HealthStatus::Healthy => 0,
            HealthStatus::Unhealthy => current_failures + 1,
            _ => *current_failures,
        };
        failure_counts.insert(service_name.clone(), new_failures);

        // Update health status
        health_status.insert(service_name, result);
        Ok(())
    }

    /// Get health status for a service
    pub async fn get_health(&self, service_name: &str) -> Option<HealthCheckResult> {
        let health_status = self.health_status.read().await;
        health_status.get(service_name).cloned()
    }

    /// Get health status for all services
    pub async fn get_all_health(&self) -> HashMap<String, HealthCheckResult> {
        let health_status = self.health_status.read().await;
        health_status.clone()
    }

    /// Check if a service is healthy
    pub async fn is_healthy(&self, service_name: &str) -> bool {
        if let Some(result) = self.get_health(service_name).await {
            result.status == HealthStatus::Healthy
        } else {
            false
        }
    }

    /// Get services that are unhealthy
    pub async fn get_unhealthy_services(&self) -> Vec<String> {
        let health_status = self.health_status.read().await;
        health_status
            .iter()
            .filter(|(_, result)| result.status == HealthStatus::Unhealthy)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Start periodic health checks
    pub async fn start_periodic_checks(&self) {
        let mut interval_timer = interval(self.config.interval);
        let health_status = Arc::clone(&self.health_status);

        tokio::spawn(async move {
            loop {
                interval_timer.tick().await;

                // Get list of services to check
                let services: Vec<String> = {
                    let status_map = health_status.read().await;
                    status_map.keys().cloned().collect()
                };

                // Perform health checks (placeholder implementation)
                for service_name in services {
                    // TODO: Implement actual health check logic
                    let _result =
                        HealthCheckResult::new(service_name.clone(), HealthStatus::Healthy);

                    todo!("Perform health check for {}", service_name);
                    // This would normally update the health status
                    // self.update_health(result).await;
                }
            }
        });
    }

    /// Get health summary
    pub async fn get_health_summary(&self) -> HashMap<HealthStatus, usize> {
        let health_status = self.health_status.read().await;
        let mut summary = HashMap::new();

        for result in health_status.values() {
            let count = summary.get(&result.status).unwrap_or(&0);
            summary.insert(result.status.clone(), count + 1);
        }

        summary
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new(HealthCheckConfig::default())
    }
}
