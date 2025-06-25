//! Crash detection and recovery system

use crate::types::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

pub mod detection;
pub mod dumps;
pub mod recovery;
pub mod reporting;

/// Crash detector for monitoring system failures
#[derive(Debug, Clone)]
pub struct CrashDetector {
    config: CrashConfig,
    crash_history: Arc<RwLock<VecDeque<CrashRecord>>>,
    recovery_manager: recovery::RecoveryManager,
    dump_generator: dumps::DumpGenerator,
    reporter: reporting::CrashReporter,
    is_running: Arc<RwLock<bool>>,
}

impl CrashDetector {
    /// Create a new crash detector
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: CrashConfig::default(),
            crash_history: Arc::new(RwLock::new(VecDeque::new())),
            recovery_manager: recovery::RecoveryManager::new()?,
            dump_generator: dumps::DumpGenerator::new()?,
            reporter: reporting::CrashReporter::new()?,
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start crash detection monitoring
    pub async fn start(&mut self) -> Result<()> {
        {
            let mut running = self
                .is_running
                .write()
                .map_err(|_| Error::Internal("Lock poisoned".to_string()))?;
            if *running {
                return Ok(());
            }
            *running = true;
        }

        // Install signal handlers for crash detection
        self.install_signal_handlers().await?;

        // Start background monitoring
        self.start_background_monitoring().await?;

        Ok(())
    }

    /// Stop crash detection monitoring
    pub async fn stop(&mut self) -> Result<()> {
        {
            let mut running = self
                .is_running
                .write()
                .map_err(|_| Error::Internal("Lock poisoned".to_string()))?;
            if !*running {
                return Ok(());
            }
            *running = false;
        }

        // Cleanup signal handlers and background tasks
        Ok(())
    }

    /// Check if crash detection is running
    pub fn is_running(&self) -> bool {
        self.is_running.read().map(|r| *r).unwrap_or(false)
    }

    /// Get crash history
    pub async fn get_crash_history(&self) -> Result<CrashHistory> {
        let history = self
            .crash_history
            .read()
            .map_err(|_| Error::Internal("Lock poisoned".to_string()))?;

        Ok(CrashHistory {
            crashes: history.iter().cloned().collect(),
            total_crashes: history.len(),
            last_crash: history.back().cloned(),
        })
    }

    /// Record a crash event
    pub async fn record_crash(
        &self,
        crash_type: CrashType,
        context: CrashContext,
    ) -> Result<String> {
        let crash_record = CrashRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            crash_type,
            context,
            dump_path: None,
            recovery_attempted: false,
            recovery_successful: false,
        };

        // Generate crash dump
        let dump_path = self.dump_generator.generate_dump(&crash_record).await?;
        let mut crash_record = crash_record;
        crash_record.dump_path = Some(dump_path.clone());

        // Store in history
        {
            let mut history = self
                .crash_history
                .write()
                .map_err(|_| Error::Internal("Lock poisoned".to_string()))?;

            history.push_back(crash_record.clone());

            // Limit history size
            while history.len() > self.config.max_crash_history {
                history.pop_front();
            }
        }

        // Attempt recovery if enabled
        if self.config.enable_auto_recovery {
            let recovery_result = self.recovery_manager.attempt_recovery(&crash_record).await;

            // Update recovery status
            if let Ok(mut history) = self.crash_history.write() {
                if let Some(last_crash) = history.back_mut() {
                    last_crash.recovery_attempted = true;
                    last_crash.recovery_successful = recovery_result.is_ok();
                }
            }
        }

        // Report the crash
        self.reporter.report_crash(&crash_record).await?;

        Ok(crash_record.id)
    }

    /// Install signal handlers for crash detection
    async fn install_signal_handlers(&self) -> Result<()> {
        // Install handlers for common crash signals
        // SIGFPE, SIGSEGV, SIGILL, SIGABRT, etc.
        // This is platform-specific and would need proper implementation
        Ok(())
    }

    /// Start background monitoring for crash indicators
    async fn start_background_monitoring(&self) -> Result<()> {
        // Monitor for memory corruption, stack overflow, etc.
        // Start watchdog timer
        // Monitor resource exhaustion
        Ok(())
    }
}

/// Crash configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashConfig {
    /// Enable automatic crash recovery
    pub enable_auto_recovery: bool,
    /// Maximum number of crash records to keep
    pub max_crash_history: usize,
    /// Generate core dumps on crash
    pub generate_core_dumps: bool,
    /// Generate memory dumps
    pub generate_memory_dumps: bool,
    /// Report crashes to external system
    pub enable_crash_reporting: bool,
    /// Crash dump directory
    pub dump_directory: std::path::PathBuf,
    /// Maximum recovery attempts per component
    pub max_recovery_attempts: u32,
    /// Watchdog timeout in seconds
    pub watchdog_timeout_seconds: u64,
}

impl Default for CrashConfig {
    fn default() -> Self {
        Self {
            enable_auto_recovery: true,
            max_crash_history: 100,
            generate_core_dumps: true,
            generate_memory_dumps: true,
            enable_crash_reporting: false,
            dump_directory: std::path::PathBuf::from("/tmp/rxserver_crashes"),
            max_recovery_attempts: 3,
            watchdog_timeout_seconds: 30,
        }
    }
}

/// Type of crash that occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrashType {
    /// Segmentation fault
    SegmentationFault,
    /// Null pointer dereference
    NullPointerDereference,
    /// Stack overflow
    StackOverflow,
    /// Heap corruption
    HeapCorruption,
    /// Assertion failure
    AssertionFailure,
    /// Unhandled exception
    UnhandledException(String),
    /// Out of memory
    OutOfMemory,
    /// Deadlock detected
    Deadlock,
    /// Infinite loop detected
    InfiniteLoop,
    /// Signal received
    Signal(i32),
    /// Unknown crash
    Unknown,
}

/// Context information about a crash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashContext {
    /// Component where crash occurred
    pub component: String,
    /// Function/method where crash occurred
    pub location: Option<String>,
    /// Stack trace at time of crash
    pub stack_trace: Option<String>,
    /// Memory state information
    pub memory_info: Option<MemoryInfo>,
    /// Thread information
    pub thread_info: Option<ThreadInfo>,
    /// Register state (platform-specific)
    pub registers: std::collections::HashMap<String, u64>,
    /// Additional context data
    pub metadata: std::collections::HashMap<String, String>,
}

/// Memory state information at crash time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Virtual memory size in bytes
    pub virtual_size: u64,
    /// Resident set size in bytes
    pub resident_size: u64,
    /// Heap size in bytes
    pub heap_size: u64,
    /// Stack size in bytes
    pub stack_size: u64,
    /// Memory map information
    pub memory_maps: Vec<MemoryMapping>,
}

/// Memory mapping information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMapping {
    /// Start address
    pub start_addr: u64,
    /// End address
    pub end_addr: u64,
    /// Permissions (rwx)
    pub permissions: String,
    /// Backing file path
    pub file_path: Option<String>,
}

/// Thread information at crash time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    /// Thread ID
    pub thread_id: u64,
    /// Thread name
    pub thread_name: Option<String>,
    /// Thread state
    pub thread_state: String,
    /// Stack pointer
    pub stack_pointer: u64,
    /// Instruction pointer
    pub instruction_pointer: u64,
}

/// Record of a crash event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashRecord {
    /// Unique crash identifier
    pub id: String,
    /// Crash timestamp
    pub timestamp: SystemTime,
    /// Type of crash
    pub crash_type: CrashType,
    /// Crash context
    pub context: CrashContext,
    /// Path to crash dump file
    pub dump_path: Option<String>,
    /// Whether recovery was attempted
    pub recovery_attempted: bool,
    /// Whether recovery was successful
    pub recovery_successful: bool,
}

/// Collection of crash records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashHistory {
    /// List of crash records
    pub crashes: Vec<CrashRecord>,
    /// Total number of crashes
    pub total_crashes: usize,
    /// Most recent crash
    pub last_crash: Option<CrashRecord>,
}

impl CrashHistory {
    /// Get crashes within a time period
    pub fn get_crashes_in_period(&self, start: SystemTime, end: SystemTime) -> Vec<&CrashRecord> {
        self.crashes
            .iter()
            .filter(|crash| crash.timestamp >= start && crash.timestamp <= end)
            .collect()
    }

    /// Get crashes by type
    pub fn get_crashes_by_type(&self, crash_type: &CrashType) -> Vec<&CrashRecord> {
        self.crashes
            .iter()
            .filter(|crash| {
                std::mem::discriminant(&crash.crash_type) == std::mem::discriminant(crash_type)
            })
            .collect()
    }

    /// Calculate crash rate (crashes per hour)
    pub fn calculate_crash_rate(&self, period: Duration) -> f64 {
        let now = SystemTime::now();
        let start_time = now - period;

        let crashes_in_period = self.get_crashes_in_period(start_time, now).len();
        let hours = period.as_secs_f64() / 3600.0;

        if hours > 0.0 {
            crashes_in_period as f64 / hours
        } else {
            0.0
        }
    }
}
