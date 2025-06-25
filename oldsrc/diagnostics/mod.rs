//! Diagnostics module for the X11 server
//!
//! This module provides comprehensive diagnostic capabilities including:
//! - Crash detection and recovery
//! - Health monitoring and status checks  
//! - Performance metrics collection
//! - Debugging and troubleshooting tools

use crate::types::{Error, Result};

pub mod crash;
pub mod debugging;
pub mod health;
pub mod metrics;
pub mod troubleshooting;
pub mod types;

/// Central diagnostics manager
#[derive(Debug)]
pub struct DiagnosticsManager {
    crash_detector: crash::CrashDetector,
    health_monitor: health::HealthMonitor,
    metrics_collector: metrics::MetricsCollector,
    debug_session: Option<debugging::DebugSession>,
    troubleshooter: troubleshooting::TroubleshooterEngine,
}

impl DiagnosticsManager {
    /// Create a new diagnostics manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            crash_detector: crash::CrashDetector::new()?,
            health_monitor: health::HealthMonitor::new(),
            metrics_collector: metrics::MetricsCollector::new(),
            debug_session: None,
            troubleshooter: troubleshooting::TroubleshooterEngine::new(),
        })
    }

    /// Start all diagnostic services
    pub async fn start(&mut self) -> Result<()> {
        self.crash_detector.start().await?;
        self.health_monitor.start().await?;
        self.metrics_collector.start().await?;
        self.troubleshooter.start().await?;
        Ok(())
    }

    /// Stop all diagnostic services
    pub async fn stop(&mut self) -> Result<()> {
        self.crash_detector.stop().await?;
        self.health_monitor.stop().await?;
        self.metrics_collector.stop().await?;
        self.troubleshooter.stop().await?;
        if let Some(ref mut session) = self.debug_session {
            session.stop();
        }
        Ok(())
    }

    /// Get current system health status
    pub async fn get_health_status(&self) -> Result<health::HealthStatus> {
        self.health_monitor.get_current_status().await
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> Result<metrics::MetricsSnapshot> {
        self.metrics_collector.get_snapshot().await
    }
    /// Start a debugging session
    pub async fn start_debug_session(&mut self, config: debugging::DebugConfig) -> Result<()> {
        let mut session = debugging::DebugSession::new(config);
        session.start().await?;
        self.debug_session = Some(session);
        Ok(())
    }

    /// Run automated troubleshooting
    pub async fn run_troubleshooting(
        &mut self,
        problem: troubleshooting::ProblemDescription,
    ) -> Result<troubleshooting::TroubleshootingReport> {
        self.troubleshooter.diagnose(problem).await
    }

    /// Generate comprehensive diagnostic report
    pub async fn generate_report(&self) -> Result<DiagnosticReport> {
        let health = self.get_health_status().await?;
        let metrics = self.get_metrics().await?;
        let crash_history = self.crash_detector.get_crash_history().await?;

        Ok(DiagnosticReport {
            timestamp: std::time::SystemTime::now(),
            health,
            metrics,
            crash_history,
            debug_info: self.debug_session.as_ref().map(|s| s.get_summary()),
        })
    }
}

/// Comprehensive diagnostic report
#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    /// Report generation timestamp
    pub timestamp: std::time::SystemTime,
    /// Current health status
    pub health: health::HealthStatus,
    /// Performance metrics snapshot
    pub metrics: metrics::MetricsSnapshot,
    /// Recent crash history
    pub crash_history: crash::CrashHistory,
    /// Debug session information
    pub debug_info: Option<debugging::DebugSummary>,
}

/// Global diagnostics instance
static mut GLOBAL_DIAGNOSTICS: Option<DiagnosticsManager> = None;

/// Initialize global diagnostics
pub async fn init_global_diagnostics() -> Result<()> {
    unsafe {
        if GLOBAL_DIAGNOSTICS.is_some() {
            return Err(Error::Internal(
                "Diagnostics already initialized".to_string(),
            ));
        }
        let mut diagnostics = DiagnosticsManager::new()?;
        diagnostics.start().await?;
        GLOBAL_DIAGNOSTICS = Some(diagnostics);
    }
    Ok(())
}

/// Get reference to global diagnostics
pub fn global_diagnostics() -> Result<&'static DiagnosticsManager> {
    unsafe {
        GLOBAL_DIAGNOSTICS
            .as_ref()
            .ok_or_else(|| Error::Internal("Diagnostics not initialized".to_string()))
    }
}

/// Emergency diagnostic utilities for critical situations
pub mod emergency {
    use super::*;

    /// Perform emergency diagnostic dump
    pub async fn emergency_dump() -> Result<String> {
        // Create minimal diagnostic information without relying on full diagnostics system
        let mut report = String::new();
        report.push_str("=== EMERGENCY DIAGNOSTIC DUMP ===\n");
        report.push_str(&format!("Timestamp: {:?}\n", std::time::SystemTime::now()));

        // Basic memory info
        if let Ok(process) = std::process::Command::new("ps")
            .args(&[
                "-o",
                "pid,vsz,rss,comm",
                "-p",
                &std::process::id().to_string(),
            ])
            .output()
        {
            report.push_str("Memory usage:\n");
            report.push_str(&String::from_utf8_lossy(&process.stdout));
        }

        // Thread count
        report.push_str(&format!("Process ID: {}\n", std::process::id()));

        Ok(report)
    }

    /// Force crash dump generation
    pub async fn force_crash_dump(reason: &str) -> Result<String> {
        let dump_path = format!("/tmp/rxserver_crash_{}.dump", std::process::id());
        let mut dump_content = String::new();

        dump_content.push_str(&format!("Forced crash dump: {}\n", reason));
        dump_content.push_str(&emergency_dump().await?);

        std::fs::write(&dump_path, dump_content)?;
        Ok(dump_path)
    }
}
