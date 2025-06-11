//! Centralized logging system for RXServer
//!
//! This module provides a unified logging approach organized into focused submodules:
//! - `init`: Logging initialization and configuration with file logging support
//! - `macros`: Structured logging macros for consistent event formatting
//! - `components`: Component-specific logging utilities for server components
//! - `utils`: Common logging patterns, utilities, and performance monitoring
//! - `types`: Logging-related type definitions and enums
//! - `logging`: Statistics, monitoring, and performance tracking utilities

pub mod components;
pub mod init;
pub mod logging;
pub mod macros;
pub mod types;
pub mod utils;

// Re-export commonly used items for convenience
pub use init::{init_logging, log_shutdown_info, log_startup_info, log_system_info};
pub use components::{NetworkLogger, PerformanceLogger, ProtocolLogger, ResourceLogger, ServerLogger};
pub use utils::{
    log_statistics, with_context, with_timing, with_timing_async, with_context_result,
    with_context_async, with_context_async_result,
    log_memory_usage_stats, log_rate_limited_warning, log_config_change,
    log_retry_warning, log_recovery, ScopedTimer
};
pub use logging::{ConnectionStats, PerformanceMonitor, log_memory_usage};