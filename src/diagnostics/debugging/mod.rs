//! Debugging interface.
//!
//! This module provides comprehensive debugging capabilities for the X11 server,
//! including protocol debugging, state inspection, event monitoring, and performance profiling.

use crate::types::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub mod events;
pub mod performance;
pub mod protocol;
pub mod state;

pub use events::*;
pub use performance::*;
pub use protocol::*;
pub use state::*;

/// Configuration for debugging sessions.
#[derive(Debug, Clone)]
pub struct DebugConfig {
    pub protocol_logging: bool,
    pub state_tracking: bool,
    pub event_monitoring: bool,
    pub performance_profiling: bool,
    pub max_events: usize,
    pub output_format: String,
    pub verbosity_level: u8,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            protocol_logging: true,
            state_tracking: true,
            event_monitoring: true,
            performance_profiling: false,
            max_events: 10000,
            output_format: "json".to_string(),
            verbosity_level: 1,
        }
    }
}

/// Summary of debugging session results.
#[derive(Debug, Clone)]
pub struct DebugSummary {
    pub session_id: String,
    pub duration: Duration,
    pub events_captured: usize,
    pub states_tracked: usize,
    pub performance_samples: usize,
    pub issues_found: Vec<String>,
    pub recommendations: Vec<String>,
}

impl DebugSummary {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            duration: Duration::from_secs(0),
            events_captured: 0,
            states_tracked: 0,
            performance_samples: 0,
            issues_found: Vec::new(),
            recommendations: Vec::new(),
        }
    }
}

/// Central debugging manager that coordinates all debugging functionality.
#[derive(Debug)]
pub struct DebuggingManager {
    protocol_debugger: ProtocolDebugger,
    state_debugger: StateDebugger,
    event_debugger: EventDebugger,
    performance_debugger: PerformanceDebugger,
    debug_level: DebugLevel,
    enabled_categories: HashMap<DebugCategory, bool>,
}

impl DebuggingManager {
    /// Creates a new debugging manager.
    pub fn new() -> Self {
        Self {
            protocol_debugger: ProtocolDebugger::new(),
            state_debugger: StateDebugger::new(),
            event_debugger: EventDebugger::new(),
            performance_debugger: PerformanceDebugger::new(),
            debug_level: DebugLevel::Info,
            enabled_categories: HashMap::new(),
        }
    }

    /// Sets the debug level.
    pub fn set_debug_level(&mut self, level: DebugLevel) {
        self.debug_level = level;
    }

    /// Enables or disables a debug category.
    pub fn set_category_enabled(&mut self, category: DebugCategory, enabled: bool) {
        self.enabled_categories.insert(category, enabled);
    }

    /// Checks if a debug category is enabled.
    pub fn is_category_enabled(&self, category: DebugCategory) -> bool {
        self.enabled_categories
            .get(&category)
            .copied()
            .unwrap_or(true)
    }

    /// Gets the protocol debugger.
    pub fn protocol_debugger(&mut self) -> &mut ProtocolDebugger {
        &mut self.protocol_debugger
    }

    /// Gets the state debugger.
    pub fn state_debugger(&mut self) -> &mut StateDebugger {
        &mut self.state_debugger
    }

    /// Gets the event debugger.
    pub fn event_debugger(&mut self) -> &mut EventDebugger {
        &mut self.event_debugger
    }

    /// Gets the performance debugger.
    pub fn performance_debugger(&mut self) -> &mut PerformanceDebugger {
        &mut self.performance_debugger
    }

    /// Logs a debug message if the category and level are enabled.
    pub fn log_debug(&self, category: DebugCategory, level: DebugLevel, message: &str) {
        if self.should_log(category, level) {
            match level {
                DebugLevel::Error => log::error!("[{:?}] {}", category, message),
                DebugLevel::Warning => log::warn!("[{:?}] {}", category, message),
                DebugLevel::Info => log::info!("[{:?}] {}", category, message),
                DebugLevel::Debug => log::debug!("[{:?}] {}", category, message),
                DebugLevel::Trace => log::trace!("[{:?}] {}", category, message),
            }
        }
    }

    /// Starts a debug session with comprehensive monitoring.
    pub async fn start_debug_session(
        &mut self,
        config: DebugSessionConfig,
    ) -> Result<DebugSession> {
        let session = DebugSession {
            id: format!("debug_{}", Instant::now().elapsed().as_nanos()),
            config,
            start_time: Instant::now(),
            active: true,
        };

        // Configure debuggers based on session config
        if session.config.protocol_debugging {
            self.protocol_debugger.enable();
        }
        if session.config.state_debugging {
            self.state_debugger.enable();
        }
        if session.config.event_debugging {
            self.event_debugger.enable();
        }
        if session.config.performance_debugging {
            self.performance_debugger.enable();
        }

        Ok(session)
    }

    /// Stops a debug session and generates a report.
    pub async fn stop_debug_session(&mut self, session: &mut DebugSession) -> Result<DebugReport> {
        session.active = false;
        let duration = session.start_time.elapsed();

        let mut report = DebugReport {
            session_id: session.id.clone(),
            duration,
            protocol_data: None,
            state_data: None,
            event_data: None,
            performance_data: None,
            summary: String::new(),
        };

        // Collect data from each debugger
        if session.config.protocol_debugging {
            report.protocol_data = Some(self.protocol_debugger.generate_report().await?);
            self.protocol_debugger.disable();
        }
        if session.config.state_debugging {
            report.state_data = Some(self.state_debugger.generate_report().await?);
            self.state_debugger.disable();
        }
        if session.config.event_debugging {
            report.event_data = Some(self.event_debugger.generate_report().await?);
            self.event_debugger.disable();
        }
        if session.config.performance_debugging {
            report.performance_data = Some(self.performance_debugger.generate_report().await?);
            self.performance_debugger.disable();
        }

        // Generate summary
        report.summary = self.generate_session_summary(&report);

        Ok(report)
    }

    fn should_log(&self, category: DebugCategory, level: DebugLevel) -> bool {
        if !self.is_category_enabled(category) {
            return false;
        }
        level <= self.debug_level
    }

    fn generate_session_summary(&self, report: &DebugReport) -> String {
        let mut summary = format!(
            "Debug session {} completed in {:?}",
            report.session_id, report.duration
        );

        if let Some(ref protocol_data) = report.protocol_data {
            summary.push_str(&format!(
                "\nProtocol: {} requests processed",
                protocol_data.request_count
            ));
        }
        if let Some(ref state_data) = report.state_data {
            summary.push_str(&format!(
                "\nState: {} snapshots captured",
                state_data.snapshot_count
            ));
        }
        if let Some(ref event_data) = report.event_data {
            summary.push_str(&format!(
                "\nEvents: {} events captured",
                event_data.event_count
            ));
        }
        if let Some(ref performance_data) = report.performance_data {
            summary.push_str(&format!(
                "\nPerformance: {} profiles captured",
                performance_data.profile_count
            ));
        }

        summary
    }
}

impl Default for DebuggingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Debug levels in order of severity/verbosity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DebugLevel {
    /// Error messages only.
    Error,
    /// Warning and error messages.
    Warning,
    /// Info, warning, and error messages.
    Info,
    /// Debug, info, warning, and error messages.
    Debug,
    /// All messages including trace.
    Trace,
}

/// Debug categories for filtering messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugCategory {
    /// X11 protocol debugging.
    Protocol,
    /// Server state debugging.
    State,
    /// Event system debugging.
    Events,
    /// Performance debugging.
    Performance,
    /// Resource management debugging.
    Resources,
    /// Network debugging.
    Network,
    /// Security debugging.
    Security,
    /// General debugging.
    General,
}

/// Configuration for a debug session.
#[derive(Debug, Clone)]
pub struct DebugSessionConfig {
    /// Enable protocol debugging.
    pub protocol_debugging: bool,
    /// Enable state debugging.
    pub state_debugging: bool,
    /// Enable event debugging.
    pub event_debugging: bool,
    /// Enable performance debugging.
    pub performance_debugging: bool,
    /// Maximum duration for the session.
    pub max_duration: Option<Duration>,
    /// Output format for debug data.
    pub output_format: DebugOutputFormat,
}

/// Debug session handle.
#[derive(Debug)]
pub struct DebugSession {
    /// Unique session identifier.
    pub id: String,
    /// Session configuration.
    pub config: DebugSessionConfig,
    /// When the session started.
    pub start_time: Instant,
    /// Whether the session is active.
    pub active: bool,
}

impl DebugSession {
    /// Creates a new debug session with the specified configuration.
    pub fn new(config: DebugConfig) -> Self {
        let session_config = DebugSessionConfig {
            protocol_debugging: config.protocol_logging,
            state_debugging: config.state_tracking,
            event_debugging: config.event_monitoring,
            performance_debugging: config.performance_profiling,
            max_duration: None,
            output_format: match config.output_format.as_str() {
                "json" => DebugOutputFormat::Json,
                "binary" => DebugOutputFormat::Binary,
                _ => DebugOutputFormat::Text,
            },
        };
        Self {
            id: format!(
                "debug_session_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
            config: session_config,
            start_time: Instant::now(),
            active: true,
        }
    }

    /// Starts the debug session.
    pub async fn start(&mut self) -> Result<()> {
        self.active = true;
        Ok(())
    }

    /// Stops the debug session.
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Gets a summary of the debug session.
    pub fn get_summary(&self) -> DebugSummary {
        DebugSummary {
            session_id: self.id.clone(),
            duration: self.start_time.elapsed(),
            events_captured: 0, // TODO: track actual counts
            states_tracked: 0,
            performance_samples: 0,
            issues_found: Vec::new(),
            recommendations: Vec::new(),
        }
    }
}

impl Clone for DebugSession {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            config: DebugSessionConfig {
                protocol_debugging: self.config.protocol_debugging,
                state_debugging: self.config.state_debugging,
                event_debugging: self.config.event_debugging,
                performance_debugging: self.config.performance_debugging,
                max_duration: self.config.max_duration,
                output_format: self.config.output_format,
            },
            start_time: self.start_time,
            active: self.active,
        }
    }
}

/// Comprehensive debug report.
#[derive(Debug)]
pub struct DebugReport {
    /// Session identifier.
    pub session_id: String,
    /// Session duration.
    pub duration: Duration,
    /// Protocol debugging data.
    pub protocol_data: Option<ProtocolDebugData>,
    /// State debugging data.
    pub state_data: Option<StateDebugData>,
    /// Event debugging data.
    pub event_data: Option<EventDebugData>,
    /// Performance debugging data.
    pub performance_data: Option<PerformanceDebugData>,
    /// Summary of the debug session.
    pub summary: String,
}

/// Output format for debug data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugOutputFormat {
    /// Human-readable text format.
    Text,
    /// JSON format.
    Json,
    /// Binary format.
    Binary,
}
