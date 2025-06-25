//! Health recovery system.
//!
//! This module provides automated recovery mechanisms for health issues.

use super::{HealthSeverity, OverallHealth};
use crate::types::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Manages automated recovery from health issues.
#[derive(Debug)]
pub struct RecoveryManager {
    recovery_strategies: HashMap<String, Box<dyn RecoveryStrategy>>,
    recovery_history: Vec<RecoveryAttempt>,
    max_recovery_attempts: u32,
    recovery_cooldown: Duration,
    last_recovery_attempt: Option<Instant>,
}

impl RecoveryManager {
    /// Creates a new recovery manager.
    pub fn new() -> Self {
        Self {
            recovery_strategies: HashMap::new(),
            recovery_history: Vec::new(),
            max_recovery_attempts: 3,
            recovery_cooldown: Duration::from_secs(300), // 5 minutes
            last_recovery_attempt: None,
        }
    }

    /// Adds a recovery strategy.
    pub fn add_strategy(&mut self, name: String, strategy: Box<dyn RecoveryStrategy>) {
        self.recovery_strategies.insert(name, strategy);
    }

    /// Removes a recovery strategy.
    pub fn remove_strategy(&mut self, name: &str) -> Option<Box<dyn RecoveryStrategy>> {
        self.recovery_strategies.remove(name)
    }

    /// Attempts recovery based on health status.
    pub async fn attempt_recovery(&mut self, health: &OverallHealth) -> Result<RecoveryResult> {
        // Check if we're in cooldown period
        if let Some(last_attempt) = self.last_recovery_attempt {
            if last_attempt.elapsed() < self.recovery_cooldown {
                return Ok(RecoveryResult::Skipped("In cooldown period".to_string()));
            }
        }

        // Check if we've exceeded max attempts
        let recent_attempts = self.count_recent_attempts(Duration::from_secs(3600)); // 1 hour
        if recent_attempts >= self.max_recovery_attempts {
            return Ok(RecoveryResult::Skipped(
                "Max recovery attempts reached".to_string(),
            ));
        }

        let mut recovery_actions = Vec::new();
        let mut overall_success = true;

        // Determine which recovery strategies to apply
        for (strategy_name, strategy) in &mut self.recovery_strategies {
            if strategy.should_apply(health) {
                let start_time = Instant::now();

                match strategy.execute_recovery(health).await {
                    Ok(result) => {
                        let duration = start_time.elapsed();
                        recovery_actions.push(RecoveryAction {
                            strategy_name: strategy_name.clone(),
                            result: RecoveryActionResult::Success,
                            message: result,
                            duration,
                        });
                    }
                    Err(e) => {
                        let duration = start_time.elapsed();
                        recovery_actions.push(RecoveryAction {
                            strategy_name: strategy_name.clone(),
                            result: RecoveryActionResult::Failed,
                            message: e.to_string(),
                            duration,
                        });
                        overall_success = false;
                    }
                }
            }
        }

        let recovery_result = if recovery_actions.is_empty() {
            RecoveryResult::NoActionNeeded
        } else if overall_success {
            RecoveryResult::Success(recovery_actions)
        } else {
            RecoveryResult::PartialSuccess(recovery_actions)
        };

        // Record the recovery attempt
        let attempt = RecoveryAttempt {
            timestamp: Instant::now(),
            health_severity: health.severity,
            result: recovery_result.clone(),
            triggered_by: self.identify_trigger_cause(health),
        };

        self.recovery_history.push(attempt);
        self.last_recovery_attempt = Some(Instant::now());

        Ok(recovery_result)
    }

    /// Gets recovery history.
    pub fn get_recovery_history(&self) -> &[RecoveryAttempt] {
        &self.recovery_history
    }

    /// Gets recovery statistics.
    pub fn get_recovery_stats(&self) -> RecoveryStats {
        let total_attempts = self.recovery_history.len();
        let successful_attempts = self
            .recovery_history
            .iter()
            .filter(|attempt| matches!(attempt.result, RecoveryResult::Success(_)))
            .count();
        let partial_success_attempts = self
            .recovery_history
            .iter()
            .filter(|attempt| matches!(attempt.result, RecoveryResult::PartialSuccess(_)))
            .count();

        let success_rate = if total_attempts > 0 {
            (successful_attempts + partial_success_attempts) as f64 / total_attempts as f64
        } else {
            0.0
        };

        RecoveryStats {
            total_attempts,
            successful_attempts,
            partial_success_attempts,
            success_rate,
            last_attempt: self.last_recovery_attempt,
        }
    }

    /// Sets the maximum number of recovery attempts per hour.
    pub fn set_max_recovery_attempts(&mut self, max_attempts: u32) {
        self.max_recovery_attempts = max_attempts;
    }

    /// Sets the recovery cooldown period.
    pub fn set_recovery_cooldown(&mut self, cooldown: Duration) {
        self.recovery_cooldown = cooldown;
    }

    fn count_recent_attempts(&self, duration: Duration) -> u32 {
        let cutoff_time = Instant::now() - duration;
        self.recovery_history
            .iter()
            .filter(|attempt| attempt.timestamp >= cutoff_time)
            .count() as u32
    }

    fn identify_trigger_cause(&self, health: &OverallHealth) -> String {
        let failed_checks: Vec<String> = health
            .check_results
            .iter()
            .filter_map(|(name, result)| {
                if matches!(
                    result.status,
                    super::CheckStatus::Fail | super::CheckStatus::Error
                ) {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        if failed_checks.is_empty() {
            format!("Overall health: {:?}", health.severity)
        } else {
            format!("Failed checks: {}", failed_checks.join(", "))
        }
    }
}

impl Default for RecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Recovery strategy trait.
#[async_trait]
pub trait RecoveryStrategy: Send + Sync + std::fmt::Debug {
    /// Determines if this strategy should be applied for the given health status.
    fn should_apply(&self, health: &OverallHealth) -> bool;

    /// Executes the recovery strategy.
    async fn execute_recovery(&mut self, health: &OverallHealth) -> Result<String>;

    /// Gets the name of the recovery strategy.
    fn name(&self) -> &str;

    /// Gets the description of the recovery strategy.
    fn description(&self) -> &str;
}

/// Memory cleanup recovery strategy.
#[derive(Debug)]
pub struct MemoryCleanupStrategy {
    memory_threshold: f64,
}

impl MemoryCleanupStrategy {
    /// Creates a new memory cleanup strategy.
    pub fn new(memory_threshold: f64) -> Self {
        Self { memory_threshold }
    }
}

#[async_trait]
impl RecoveryStrategy for MemoryCleanupStrategy {
    fn should_apply(&self, health: &OverallHealth) -> bool {
        // Check if memory-related checks are failing
        health.check_results.iter().any(|(name, result)| {
            name.contains("memory")
                && matches!(
                    result.status,
                    super::CheckStatus::Fail | super::CheckStatus::Warning
                )
        })
    }

    async fn execute_recovery(&mut self, _health: &OverallHealth) -> Result<String> {
        // Simulate memory cleanup
        tokio::time::sleep(Duration::from_millis(100)).await;

        // In a real implementation, this would:
        // - Force garbage collection
        // - Clear caches
        // - Release unused resources

        Ok("Memory cleanup completed successfully".to_string())
    }

    fn name(&self) -> &str {
        "memory_cleanup"
    }

    fn description(&self) -> &str {
        "Performs memory cleanup and garbage collection"
    }
}

/// Connection reset recovery strategy.
#[derive(Debug)]
pub struct ConnectionResetStrategy {
    max_connections: u32,
}

impl ConnectionResetStrategy {
    /// Creates a new connection reset strategy.
    pub fn new(max_connections: u32) -> Self {
        Self { max_connections }
    }
}

#[async_trait]
impl RecoveryStrategy for ConnectionResetStrategy {
    fn should_apply(&self, health: &OverallHealth) -> bool {
        health.check_results.iter().any(|(name, result)| {
            name.contains("connection") && matches!(result.status, super::CheckStatus::Fail)
        })
    }

    async fn execute_recovery(&mut self, _health: &OverallHealth) -> Result<String> {
        // Simulate connection reset
        tokio::time::sleep(Duration::from_millis(200)).await;

        // In a real implementation, this would:
        // - Close idle connections
        // - Reset connection pools
        // - Clear connection state

        Ok("Connection reset completed successfully".to_string())
    }

    fn name(&self) -> &str {
        "connection_reset"
    }

    fn description(&self) -> &str {
        "Resets connections and clears connection state"
    }
}

/// Service restart recovery strategy.
#[derive(Debug)]
pub struct ServiceRestartStrategy {
    restart_threshold: HealthSeverity,
}

impl ServiceRestartStrategy {
    /// Creates a new service restart strategy.
    pub fn new(restart_threshold: HealthSeverity) -> Self {
        Self { restart_threshold }
    }
}

#[async_trait]
impl RecoveryStrategy for ServiceRestartStrategy {
    fn should_apply(&self, health: &OverallHealth) -> bool {
        health.severity >= self.restart_threshold
    }

    async fn execute_recovery(&mut self, _health: &OverallHealth) -> Result<String> {
        // Simulate service restart
        tokio::time::sleep(Duration::from_millis(500)).await;

        // In a real implementation, this would:
        // - Gracefully restart affected services
        // - Reinitialize components
        // - Restore service state

        Ok("Service restart completed successfully".to_string())
    }

    fn name(&self) -> &str {
        "service_restart"
    }

    fn description(&self) -> &str {
        "Restarts services when critical issues are detected"
    }
}

/// Result of a recovery attempt.
#[derive(Debug, Clone)]
pub enum RecoveryResult {
    /// Recovery was successful.
    Success(Vec<RecoveryAction>),
    /// Recovery was partially successful.
    PartialSuccess(Vec<RecoveryAction>),
    /// No recovery action was needed.
    NoActionNeeded,
    /// Recovery was skipped (e.g., due to cooldown).
    Skipped(String),
}

/// Individual recovery action.
#[derive(Debug, Clone)]
pub struct RecoveryAction {
    /// Name of the strategy that performed this action.
    pub strategy_name: String,
    /// Result of the action.
    pub result: RecoveryActionResult,
    /// Description of what was done.
    pub message: String,
    /// Time taken to execute the action.
    pub duration: Duration,
}

/// Result of a recovery action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryActionResult {
    /// Action was successful.
    Success,
    /// Action failed.
    Failed,
}

/// Record of a recovery attempt.
#[derive(Debug, Clone)]
pub struct RecoveryAttempt {
    /// When the recovery was attempted.
    pub timestamp: Instant,
    /// Health severity that triggered the recovery.
    pub health_severity: HealthSeverity,
    /// Result of the recovery attempt.
    pub result: RecoveryResult,
    /// What triggered this recovery attempt.
    pub triggered_by: String,
}

/// Recovery statistics.
#[derive(Debug, Clone)]
pub struct RecoveryStats {
    /// Total number of recovery attempts.
    pub total_attempts: usize,
    /// Number of successful recovery attempts.
    pub successful_attempts: usize,
    /// Number of partially successful recovery attempts.
    pub partial_success_attempts: usize,
    /// Overall success rate (0.0 to 1.0).
    pub success_rate: f64,
    /// Timestamp of the last recovery attempt.
    pub last_attempt: Option<Instant>,
}
