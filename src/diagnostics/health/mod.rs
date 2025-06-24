//! Health monitoring interface.
//!
//! This module provides comprehensive health monitoring capabilities for the X11 server,
//! including health checks, status reporting, alerts, and recovery mechanisms.

use crate::types::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

pub mod alerts;
pub mod checks;
pub mod recovery;
pub mod status;

pub use alerts::*;
pub use checks::*;
pub use recovery::*;
pub use status::*;

// Re-export HealthStatus from server types
pub use crate::server::types::HealthStatus;

/// Health monitoring service that coordinates all health-related functionality.
#[derive(Debug)]
pub struct HealthMonitor {
    check_names: Vec<String>,
    status_manager: HealthStatusManager,
    alert_manager: AlertManager,
    recovery_manager: RecoveryManager,
    check_interval: Duration,
    last_check: Option<Instant>,
}

impl HealthMonitor {
    /// Creates a new health monitor with default configuration.
    pub fn new() -> Self {
        Self {
            check_names: Vec::new(),
            status_manager: HealthStatusManager::new(),
            alert_manager: AlertManager::new(),
            recovery_manager: RecoveryManager::new(),
            check_interval: Duration::from_secs(30),
            last_check: None,
        }
    }

    /// Adds a health check to the monitor.
    pub fn add_check(&mut self, name: String) {
        if !self.check_names.contains(&name) {
            self.check_names.push(name);
        }
    }

    /// Removes a health check from the monitor.
    pub fn remove_check(&mut self, name: &str) -> bool {
        if let Some(pos) = self.check_names.iter().position(|x| x == name) {
            self.check_names.remove(pos);
            true
        } else {
            false
        }
    }

    /// Starts the health monitoring service.
    pub async fn start(&mut self) -> Result<()> {
        {
            // TODO: Implement health monitoring startup
            self.last_check = Some(Instant::now());
        }

        Ok(())
    }

    /// Stops the health monitoring service.
    pub async fn stop(&mut self) -> Result<()> {
        {
            // TODO: Implement health monitoring shutdown
            self.last_check = None;
        }

        Ok(())
    }

    /// Gets the current health status (alternative name for compatibility).
    pub async fn get_current_status(&self) -> Result<HealthStatus> {
        // TODO: Implement status collection from checks
        Ok(HealthStatus::Healthy)
    }

    /// Runs all health checks and updates status.
    pub async fn run_checks(&mut self) -> Result<OverallHealth> {
        let mut results = HashMap::new();

        // Simulate running checks for each registered check name
        for name in &self.check_names {
            let result = CheckResult {
                status: CheckStatus::Pass,
                message: "Check passed".to_string(),
                duration: Duration::from_millis(10),
                metadata: HashMap::new(),
            };
            results.insert(name.clone(), result);
        }

        let overall = self.status_manager.update_status(results).await?;
        self.last_check = Some(Instant::now()); // Process alerts if needed
        if overall.severity() >= HealthSeverity::Warning {
            self.alert_manager.process_health_status(&overall).await?;
        }

        // Trigger recovery if critical
        if overall.severity() >= HealthSeverity::Critical {
            self.recovery_manager.attempt_recovery(&overall).await?;
        }

        Ok(overall)
    }
    /// Gets the current health status.
    pub async fn get_status(&self) -> Result<OverallHealth> {
        // TODO: Implement status collection
        Ok(OverallHealth {
            status: HealthStatus::Healthy,
            severity: HealthSeverity::Healthy,
            check_results: HashMap::new(),
            message: "System is healthy".to_string(),
            timestamp: std::time::SystemTime::now(),
            details: HashMap::new(),
        })
    }

    /// Checks if health checks should be run based on interval.
    pub fn should_run_checks(&self) -> bool {
        match self.last_check {
            Some(last) => last.elapsed() >= self.check_interval,
            None => true,
        }
    }

    /// Sets the health check interval.
    pub fn set_check_interval(&mut self, interval: Duration) {
        self.check_interval = interval;
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Overall health status of the system.
#[derive(Debug, Clone)]
pub struct OverallHealth {
    /// Overall health status.
    pub status: HealthStatus,
    /// Overall health severity.
    pub severity: HealthSeverity,
    /// Individual check results.
    pub check_results: HashMap<String, CheckResult>,
    /// Health status message.
    pub message: String,
    /// Timestamp of the health check.
    pub timestamp: std::time::SystemTime,
    /// Additional details about the health status.
    pub details: HashMap<String, String>,
}

impl OverallHealth {
    /// Gets the health severity level.
    pub fn severity(&self) -> HealthSeverity {
        self.severity
    }
}

/// Health severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HealthSeverity {
    /// System is healthy.
    Healthy,
    /// System has minor issues.
    Warning,
    /// System has significant issues.
    Critical,
    /// System is non-functional.
    Fatal,
}

/// Result of a health check.
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// Check status.
    pub status: CheckStatus,
    /// Status message.
    pub message: String,
    /// Duration of the check.
    pub duration: Duration,
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

/// Status of a health check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    /// Check passed successfully.
    Pass,
    /// Check failed with warnings.
    Warning,
    /// Check failed critically.
    Fail,
    /// Check could not be executed.
    Error,
}

/// Trait for implementing health checks.
#[async_trait]
pub trait HealthCheck: Send + Sync + std::fmt::Debug {
    /// Executes the health check.
    async fn check(&mut self) -> CheckResult;

    /// Gets the name of the health check.
    fn name(&self) -> &str;

    /// Gets the description of the health check.
    fn description(&self) -> &str;
}

/// Health monitoring service entrypoint and coordinator
#[derive(Debug)]
pub struct HealthService {
    monitor: Arc<RwLock<HealthMonitor>>,
    command_tx: mpsc::UnboundedSender<HealthCommand>,
    command_rx: Option<mpsc::UnboundedReceiver<HealthCommand>>,
    is_running: Arc<RwLock<bool>>,
}

/// Commands for controlling the health service
#[derive(Debug)]
pub enum HealthCommand {
    /// Start monitoring
    Start,
    /// Stop monitoring
    Stop,
    /// Add a health check
    AddCheck { name: String },
    /// Remove a health check
    RemoveCheck { name: String },
    /// Force run all checks
    RunChecks,
    /// Get current status
    GetStatus {
        response_tx: tokio::sync::oneshot::Sender<Result<OverallHealth>>,
    },
    /// Update check interval
    SetInterval { interval: Duration },
}

impl HealthService {
    /// Create a new health service
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        Self {
            monitor: Arc::new(RwLock::new(HealthMonitor::new())),
            command_tx,
            command_rx: Some(command_rx),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the health monitoring service
    pub async fn start(&mut self) -> Result<()> {
        {
            let mut running = self.is_running.write().await;
            if *running {
                return Ok(());
            }
            *running = true;
        }

        info!("Starting health monitoring service");

        // Start the monitor
        {
            let mut monitor = self.monitor.write().await;
            monitor.start().await?;
        }

        // Set up default health checks
        self.setup_default_checks().await?;

        // Start the command processing loop
        if let Some(command_rx) = self.command_rx.take() {
            let monitor = Arc::clone(&self.monitor);
            let is_running = Arc::clone(&self.is_running);

            tokio::spawn(async move {
                Self::command_loop(monitor, command_rx, is_running).await;
            });
        }

        // Start periodic health check task
        self.start_periodic_checks().await;

        info!("Health monitoring service started successfully");
        Ok(())
    }

    /// Stop the health monitoring service
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping health monitoring service");

        {
            let mut running = self.is_running.write().await;
            *running = false;
        }

        {
            let mut monitor = self.monitor.write().await;
            monitor.stop().await?;
        }

        info!("Health monitoring service stopped");
        Ok(())
    }

    /// Get a handle to send commands to the health service
    pub fn get_command_sender(&self) -> mpsc::UnboundedSender<HealthCommand> {
        self.command_tx.clone()
    }

    /// Get current health status
    pub async fn get_health_status(&self) -> Result<OverallHealth> {
        let monitor = self.monitor.read().await;
        monitor.get_status().await
    }

    /// Check if the service is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Setup default health checks
    async fn setup_default_checks(&mut self) -> Result<()> {
        let mut monitor = self.monitor.write().await;

        // Add basic system health checks
        monitor.add_check("memory_usage".to_string());
        monitor.add_check("connection_count".to_string());
        monitor.add_check("server_responsiveness".to_string());
        monitor.add_check("resource_leaks".to_string());

        info!("Default health checks configured");
        Ok(())
    }

    /// Start periodic health checks
    async fn start_periodic_checks(&self) {
        let monitor = Arc::clone(&self.monitor);
        let is_running = Arc::clone(&self.is_running);

        tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(30));

            loop {
                check_interval.tick().await;

                // Check if we should continue running
                {
                    let running = is_running.read().await;
                    if !*running {
                        break;
                    }
                }

                // Run health checks
                {
                    let mut monitor_guard = monitor.write().await;
                    if monitor_guard.should_run_checks() {
                        match monitor_guard.run_checks().await {
                            Ok(health) => {
                                debug!("Health check completed: {:?}", health.severity());
                                if health.severity() >= HealthSeverity::Warning {
                                    warn!("Health issues detected: {}", health.message);
                                }
                            }
                            Err(e) => {
                                error!("Health check failed: {}", e);
                            }
                        }
                    }
                }
            }

            info!("Periodic health checks stopped");
        });
    }

    /// Command processing loop
    async fn command_loop(
        monitor: Arc<RwLock<HealthMonitor>>,
        mut command_rx: mpsc::UnboundedReceiver<HealthCommand>,
        is_running: Arc<RwLock<bool>>,
    ) {
        while let Some(command) = command_rx.recv().await {
            let running = *is_running.read().await;
            if !running {
                break;
            }

            match command {
                HealthCommand::Start => {
                    let mut monitor_guard = monitor.write().await;
                    if let Err(e) = monitor_guard.start().await {
                        error!("Failed to start health monitor: {}", e);
                    }
                }
                HealthCommand::Stop => {
                    let mut monitor_guard = monitor.write().await;
                    if let Err(e) = monitor_guard.stop().await {
                        error!("Failed to stop health monitor: {}", e);
                    }
                }
                HealthCommand::AddCheck { name } => {
                    let mut monitor_guard = monitor.write().await;
                    monitor_guard.add_check(name.clone());
                    info!("Added health check: {}", name);
                }
                HealthCommand::RemoveCheck { name } => {
                    let mut monitor_guard = monitor.write().await;
                    if monitor_guard.remove_check(&name) {
                        info!("Removed health check: {}", name);
                    } else {
                        warn!("Health check not found: {}", name);
                    }
                }
                HealthCommand::RunChecks => {
                    let mut monitor_guard = monitor.write().await;
                    match monitor_guard.run_checks().await {
                        Ok(health) => {
                            info!("Forced health check completed: {:?}", health.severity());
                        }
                        Err(e) => {
                            error!("Forced health check failed: {}", e);
                        }
                    }
                }
                HealthCommand::GetStatus { response_tx } => {
                    let monitor_guard = monitor.read().await;
                    let result = monitor_guard.get_status().await;
                    let _ = response_tx.send(result);
                }
                HealthCommand::SetInterval { interval } => {
                    let mut monitor_guard = monitor.write().await;
                    monitor_guard.set_check_interval(interval);
                    info!("Health check interval updated: {:?}", interval);
                }
            }
        }

        info!("Health service command loop stopped");
    }
}

impl Default for HealthService {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize health monitoring for the server
pub async fn init_health_monitoring() -> Result<HealthService> {
    info!("Initializing health monitoring system");

    let mut service = HealthService::new();
    service.start().await?;

    info!("Health monitoring system initialized successfully");
    Ok(service)
}

/// Global health service instance
static mut GLOBAL_HEALTH_SERVICE: Option<HealthService> = None;
static HEALTH_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize global health service
#[allow(static_mut_refs)]
pub async fn init_global_health() -> Result<()> {
    HEALTH_INIT.call_once(|| {
        // This will be set in the async context
    });

    unsafe {
        if GLOBAL_HEALTH_SERVICE.is_none() {
            GLOBAL_HEALTH_SERVICE = Some(init_health_monitoring().await?);
        }
    }

    Ok(())
}

/// Get reference to global health service
#[allow(static_mut_refs)]
pub fn global_health_service() -> Option<&'static HealthService> {
    unsafe { GLOBAL_HEALTH_SERVICE.as_ref() }
}

/// Get command sender for global health service
pub fn global_health_command_sender() -> Option<mpsc::UnboundedSender<HealthCommand>> {
    global_health_service().map(|service| service.get_command_sender())
}
