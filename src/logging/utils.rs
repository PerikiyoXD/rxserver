//! Logging utility functions
//!
//! This module provides common logging patterns and helper functions
//! that can be used across the RXServer codebase.

use tracing::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::{Mutex, LazyLock};
use std::time::Instant;

/// Execute an operation with debug logging of start/completion
pub fn with_context<F>(context: &str, operation: F)
where
    F: FnOnce(),
{
    debug!("Starting: {}", context);
    operation();
    debug!("Completed: {}", context);
}

/// Execute an operation with debug logging of start/completion and return result
pub fn with_context_result<F, R>(context: &str, operation: F) -> R
where
    F: FnOnce() -> R,
{
    debug!("Starting: {}", context);
    let result = operation();
    debug!("Completed: {}", context);
    result
}

/// Execute an async operation with debug logging of start/completion
pub async fn with_context_async<F, Fut>(context: &str, operation: F)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    debug!("Starting: {}", context);
    operation().await;
    debug!("Completed: {}", context);
}

/// Execute an async operation with debug logging of start/completion and return result
pub async fn with_context_async_result<F, Fut, R>(context: &str, operation: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    debug!("Starting: {}", context);
    let result = operation().await;
    debug!("Completed: {}", context);
    result
}

/// Execute an operation with timing logging
pub fn with_timing<F, R>(operation_name: &str, operation: F) -> R
where
    F: FnOnce() -> R,
{
    let start = std::time::Instant::now();
    let result = operation();
    let duration = start.elapsed();
    
    crate::log_performance!(timing, operation_name, duration.as_millis() as f64);
    result
}

/// Execute an async operation with timing logging
pub async fn with_timing_async<F, Fut, R>(operation_name: &str, operation: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    let start = std::time::Instant::now();
    let result = operation().await;
    let duration = start.elapsed();
    
    crate::log_performance!(timing, operation_name, duration.as_millis() as f64);
    result
}

/// Log an error with full context and return it
pub fn log_and_return_error<E>(error: E, context: &str) -> E
where
    E: std::error::Error,
{
    error!(
        error = %error,
        context = context,
        "Operation failed"
    );
    error
}

/// Log statistics in a standardized format
pub fn log_statistics(component: &str, stats: &[(&str, u64)]) {
    info!("=== {} Statistics ===", component);
    for (name, value) in stats {
        info!("  {}: {}", name, value);
    }
}

/// Log debug message with multiple key-value pairs
pub fn log_debug_kv(message: &str, kvs: &[(&str, &dyn std::fmt::Display)]) {
    let mut fields = Vec::new();
    for (key, value) in kvs {
        fields.push(format!("{}={}", key, value));
    }
    debug!("{}: {}", message, fields.join(", "));
}

/// Log configuration changes
pub fn log_config_change(setting: &str, old_value: &str, new_value: &str) {
    info!(
        setting = setting,
        old_value = old_value,
        new_value = new_value,
        "Configuration changed"
    );
}

/// Log a warning with retry information
pub fn log_retry_warning(operation: &str, attempt: u32, max_attempts: u32, error: &dyn std::error::Error) {
    warn!(
        operation = operation,
        attempt = attempt,
        max_attempts = max_attempts,
        error = %error,
        "Operation failed, retrying"
    );
}

/// Log successful recovery from an error
pub fn log_recovery(operation: &str, attempts: u32) {
    info!(
        operation = operation,
        attempts = attempts,
        "Operation recovered after retries"
    );
}

/// Log a rate-limited warning (useful for frequent errors)
pub fn log_rate_limited_warning(key: &str, message: &str) {
    static COUNTER: LazyLock<Mutex<HashMap<String, (u64, std::time::Instant)>>> = 
        LazyLock::new(|| Mutex::new(HashMap::new()));
    
    let now = std::time::Instant::now();
    let mut counter = COUNTER.lock().unwrap();
    
    let entry = counter.entry(key.to_string()).or_insert((0, now));
    
    if now.duration_since(entry.1).as_secs() > 60 {
        // Reset counter every minute
        entry.0 = 0;
        entry.1 = now;
    }
    
    entry.0 += 1;
    
    // Log only the first occurrence and then every 10th occurrence
    if entry.0 == 1 || entry.0 % 10 == 0 {
        warn!(
            key = key,
            count = entry.0,
            "Rate-limited warning: {}",
            message
        );
    }
}

/// Log memory usage information
pub fn log_memory_usage_stats(component: &str, bytes_used: usize, bytes_allocated: usize) {
    info!(
        component = component,
        bytes_used = bytes_used,
        bytes_allocated = bytes_allocated,
        efficiency_percent = if bytes_allocated > 0 { 
            bytes_used as f64 / bytes_allocated as f64 * 100.0 
        } else { 0.0 },
        "Memory usage statistics"
    );
}

/// Create a scoped timer that automatically logs duration when dropped
pub struct ScopedTimer {
    name: String,
    start: Instant,
}

impl ScopedTimer {
    pub fn new(name: &str) -> Self {
        debug!("Starting timer: {}", name);
        Self {
            name: name.to_string(),
            start: Instant::now(),
        }
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        crate::log_performance!(timing, &self.name, duration.as_millis() as f64);
    }
}

/// Macro for creating scoped timers
#[macro_export]
macro_rules! scoped_timer {
    ($name:expr) => {
        let _timer = $crate::logging::utils::ScopedTimer::new($name);
    };
}
