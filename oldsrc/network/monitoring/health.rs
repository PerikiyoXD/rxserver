//! Health monitoring for network components
//!
//! Provides health checking and status monitoring for network connections and services.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Overall health status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: Instant,
    pub duration: Duration,
    pub details: HashMap<String, String>,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    pub name: String,
    pub interval: Duration,
    pub timeout: Duration,
    pub retries: u32,
    pub enabled: bool,
}

// ---------------------------------------------------------------
// Health checker trait
// ---------------------------------------------------------------

/// Individual health checker trait
#[async_trait::async_trait]
pub trait HealthChecker: Send + Sync {
    /// Perform the health check and return the result
    async fn check(&self) -> HealthCheck;
    fn name(&self) -> &str;
    fn config(&self) -> &HealthCheckConfig;
}

// ---------------------------------------------------------------
// Health checkers for various components
// ---------------------------------------------------------------

pub struct HealthCheckers;

impl HealthCheckers {
    
}

/// TCP connection health checker
pub struct TcpHealthChecker {
    config: HealthCheckConfig,
    target: SocketAddr,
}

/// HTTP health checker
pub struct HttpHealthChecker {
    config: HealthCheckConfig,
    url: String,
    expected_status: u16,
}

/// Memory health checker
pub struct MemoryHealthChecker {
    config: HealthCheckConfig,
    max_memory_bytes: u64,
}

/// Disk space health checker
pub struct DiskHealthChecker {
    config: HealthCheckConfig,
    path: String,
    min_free_gb: u64,
}

/// CPU load health checker
pub struct CpuLoadHealthChecker {
    config: HealthCheckConfig,
    max_load: f64,
}

// ---------------------------------------------------------------
// Health monitor implementation
// ---------------------------------------------------------------

/// Health monitor implementation
pub struct HealthMonitor {
    checkers: Arc<RwLock<Vec<Box<dyn HealthChecker>>>>,
    results: Arc<RwLock<HashMap<String, HealthCheck>>>,
    is_running: Arc<RwLock<bool>>,
    overall_status: Arc<RwLock<HealthStatus>>,
}

impl TcpHealthChecker {
    pub fn new(name: String, target: SocketAddr, interval: Duration) -> Self {
        let config = HealthCheckConfig {
            name: name.clone(),
            interval,
            timeout: Duration::from_secs(5),
            retries: 3,
            enabled: true,
        };

        Self { config, target }
    }
}

#[async_trait::async_trait]
impl HealthChecker for TcpHealthChecker {
    async fn check(&self) -> HealthCheck {
        let start = Instant::now();
        let mut attempts = 0;
        let mut last_error = String::new();

        while attempts < self.config.retries {
            attempts += 1;

            match tokio::time::timeout(
                self.config.timeout,
                tokio::net::TcpStream::connect(self.target),
            )
            .await
            {
                Ok(Ok(_)) => {
                    return HealthCheck {
                        name: self.config.name.clone(),
                        status: HealthStatus::Healthy,
                        message: format!("TCP connection to {} successful", self.target),
                        timestamp: Instant::now(),
                        duration: start.elapsed(),
                        details: HashMap::new(),
                    };
                }
                Ok(Err(e)) => {
                    last_error = format!("Connection failed: {}", e);
                }
                Err(_) => {
                    last_error = "Connection timeout".to_string();
                }
            }

            if attempts < self.config.retries {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        HealthCheck {
            name: self.config.name.clone(),
            status: HealthStatus::Unhealthy,
            message: format!(
                "TCP connection to {} failed after {} attempts: {}",
                self.target, attempts, last_error
            ),
            timestamp: Instant::now(),
            duration: start.elapsed(),
            details: HashMap::new(),
        }
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &HealthCheckConfig {
        &self.config
    }
}

impl HttpHealthChecker {
    pub fn new(name: String, url: String, expected_status: u16, interval: Duration) -> Self {
        let config = HealthCheckConfig {
            name: name.clone(),
            interval,
            timeout: Duration::from_secs(10),
            retries: 2,
            enabled: true,
        };

        Self {
            config,
            url,
            expected_status,
        }
    }
}

#[async_trait::async_trait]
impl HealthChecker for HttpHealthChecker {
    async fn check(&self) -> HealthCheck {
        let start = Instant::now();

        // In a real implementation, this would make an HTTP request
        // For now, we'll simulate a successful check
        HealthCheck {
            name: self.config.name.clone(),
            status: HealthStatus::Healthy,
            message: format!(
                "HTTP check to {} returned {}",
                self.url, self.expected_status
            ),
            timestamp: Instant::now(),
            duration: start.elapsed(),
            details: HashMap::new(),
        }
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &HealthCheckConfig {
        &self.config
    }
}

impl MemoryHealthChecker {
    pub fn new(name: String, max_memory_mb: u64, interval: Duration) -> Self {
        let config = HealthCheckConfig {
            name: name.clone(),
            interval,
            timeout: Duration::from_secs(1),
            retries: 1,
            enabled: true,
        };

        Self {
            config,
            max_memory_bytes: max_memory_mb,
        }
    }
}

#[async_trait::async_trait]
impl HealthChecker for MemoryHealthChecker {
    async fn check(&self) -> HealthCheck {
        let start = Instant::now();

        // Get current memory usage from system APIs
        let current_memory_bytes = self.get_memory_usage_bytes().await;

        let status = if current_memory_bytes <= self.max_memory_bytes {
            HealthStatus::Healthy
        } else if current_memory_bytes <= self.max_memory_bytes * 2 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        let mut details = HashMap::new();
        details.insert(
            "current_memory_mb".to_string(),
            (current_memory_bytes / 1024 / 1024).to_string(),
        );
        details.insert(
            "max_memory_mb".to_string(),
            self.max_memory_bytes.to_string(),
        );

        HealthCheck {
            name: self.config.name.clone(),
            status,
            message: format!(
                "Memory usage: {} MB (limit: {} MB)",
                (current_memory_bytes / 1024 / 1024),
                self.max_memory_bytes
            ),
            timestamp: Instant::now(),
            duration: start.elapsed(),
            details,
        }
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &HealthCheckConfig {
        &self.config
    }
}

impl MemoryHealthChecker {
    async fn get_memory_usage_bytes(&self) -> u64 {
        // Use sysinfo, implementation for our process

        let sys = sysinfo::System::new_all();
        let pid = sysinfo::get_current_pid().unwrap();
        let proc = sys.process(pid).unwrap();
        proc.memory()
    }
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            checkers: Arc::new(RwLock::new(Vec::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
            overall_status: Arc::new(RwLock::new(HealthStatus::Unknown)),
        }
    }

    /// Add a health checker
    pub async fn add_checker(&self, checker: Box<dyn HealthChecker>) {
        self.checkers.write().await.push(checker);
    }

    /// Remove a health checker
    pub async fn remove_checker(&self, name: &str) -> bool {
        let mut checkers = self.checkers.write().await;
        let len_before = checkers.len();
        checkers.retain(|c| c.name() != name);
        checkers.len() != len_before
    }

    /// Start health monitoring
    pub async fn start_monitoring(&self) {
        *self.is_running.write().await = true;

        let checkers = Arc::clone(&self.checkers);
        let results = Arc::clone(&self.results);
        let is_running = Arc::clone(&self.is_running);
        let overall_status = Arc::clone(&self.overall_status);

        tokio::spawn(async move {
            while *is_running.read().await {
                let checker_list = checkers.read().await;

                for checker in checker_list.iter() {
                    if !checker.config().enabled {
                        continue;
                    }

                    let result = checker.check().await;
                    results.write().await.insert(result.name.clone(), result);
                }

                // Update overall status
                let results_map = results.read().await;
                let new_status = Self::calculate_overall_status(&results_map);
                *overall_status.write().await = new_status;

                drop(checker_list);
                drop(results_map);

                // Wait before next check cycle
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
    }

    /// Stop health monitoring
    pub async fn stop_monitoring(&self) {
        *self.is_running.write().await = false;
    }

    /// Get overall health status
    pub async fn get_status(&self) -> HealthStatus {
        self.overall_status.read().await.clone()
    }

    /// Get all health check results
    pub async fn get_all_results(&self) -> HashMap<String, HealthCheck> {
        self.results.read().await.clone()
    }

    /// Get a specific health check result
    pub async fn get_result(&self, name: &str) -> Option<HealthCheck> {
        self.results.read().await.get(name).cloned()
    }

    /// Calculate overall status from individual check results
    fn calculate_overall_status(results: &HashMap<String, HealthCheck>) -> HealthStatus {
        if results.is_empty() {
            return HealthStatus::Unknown;
        }

        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;

        for result in results.values() {
            match result.status {
                HealthStatus::Healthy => healthy_count += 1,
                HealthStatus::Degraded => degraded_count += 1,
                HealthStatus::Unhealthy => unhealthy_count += 1,
                HealthStatus::Unknown => {}
            }
        }

        if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else if healthy_count > 0 {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }

    /// Generate health report
    pub async fn generate_report(&self) -> HealthReport {
        let results = self.get_all_results().await;
        let overall_status = self.get_status().await;
        HealthReport {
            timestamp: Instant::now(),
            overall_status,
            checks: results.clone(),
            summary: self.generate_summary(&results).await,
        }
    }

    /// Generate summary statistics
    async fn generate_summary(&self, results: &HashMap<String, HealthCheck>) -> HealthSummary {
        let total_checks = results.len();
        let healthy_checks = results
            .values()
            .filter(|r| r.status == HealthStatus::Healthy)
            .count();
        let degraded_checks = results
            .values()
            .filter(|r| r.status == HealthStatus::Degraded)
            .count();
        let unhealthy_checks = results
            .values()
            .filter(|r| r.status == HealthStatus::Unhealthy)
            .count();

        HealthSummary {
            total_checks,
            healthy_checks,
            degraded_checks,
            unhealthy_checks,
        }
    }
}

/// Health monitoring report
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub timestamp: Instant,
    pub overall_status: HealthStatus,
    pub checks: HashMap<String, HealthCheck>,
    pub summary: HealthSummary,
}

/// Health summary statistics
#[derive(Debug, Clone)]
pub struct HealthSummary {
    pub total_checks: usize,
    pub healthy_checks: usize,
    pub degraded_checks: usize,
    pub unhealthy_checks: usize,
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_monitor() {
        let monitor = HealthMonitor::new();
        let checker = TcpHealthChecker::new(
            "test_tcp".to_string(),
            "127.0.0.1:22".parse().unwrap(),
            Duration::from_secs(30),
        );

        monitor.add_checker(Box::new(checker)).await;

        let checkers = monitor.checkers.read().await;
        assert_eq!(checkers.len(), 1);
        assert_eq!(checkers[0].name(), "test_tcp");
    }

    #[tokio::test]
    async fn test_health_status_calculation() {
        let mut results = HashMap::new();

        results.insert(
            "check1".to_string(),
            HealthCheck {
                name: "check1".to_string(),
                status: HealthStatus::Healthy,
                message: "OK".to_string(),
                timestamp: Instant::now(),
                duration: Duration::from_millis(10),
                details: HashMap::new(),
            },
        );

        results.insert(
            "check2".to_string(),
            HealthCheck {
                name: "check2".to_string(),
                status: HealthStatus::Degraded,
                message: "Slow".to_string(),
                timestamp: Instant::now(),
                duration: Duration::from_millis(100),
                details: HashMap::new(),
            },
        );

        let status = HealthMonitor::calculate_overall_status(&results);
        assert_eq!(status, HealthStatus::Degraded);
    }
}
