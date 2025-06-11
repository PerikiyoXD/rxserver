//! Logging utilities and statistics
//!
//! This module provides logging configuration, statistics tracking, and monitoring
//! utilities for the X server.

use crate::todo_medium;
use tracing::{info, warn};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Memory usage logging with system information
pub fn log_memory_usage() {
    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines().take(3) {
                if line.starts_with("MemTotal:") || 
                   line.starts_with("MemFree:") || 
                   line.starts_with("MemAvailable:") {
                    info!("System memory: {}", line.trim());
                }
            }
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        todo_medium!(
            "logging",
            "Memory usage tracking not implemented for this platform"
        );
        tracing::debug!("Memory usage tracking not yet implemented for this platform");
    }
}

/// Advanced connection statistics with performance metrics
#[derive(Debug, Default, Clone)]
pub struct ConnectionStats {
    pub total_connections: u64,
    pub active_connections: u64,
    pub requests_processed: u64,
    pub responses_sent: u64,
    pub events_sent: u64,
    pub errors_sent: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub avg_response_time_ms: f64,
    pub peak_connections: u64,
    pub connection_errors: u64,
    pub protocol_errors: u64,
}

impl ConnectionStats {
    /// Log current statistics with performance analysis
    pub fn log_stats(&self) {
        info!("=== Connection Statistics ===");
        info!("  Total connections: {}", self.total_connections);
        info!("  Active connections: {}", self.active_connections);
        info!("  Peak connections: {}", self.peak_connections);
        info!("  Requests processed: {}", self.requests_processed);
        info!("  Responses sent: {}", self.responses_sent);
        info!("  Events sent: {}", self.events_sent);
        info!("  Errors sent: {}", self.errors_sent);
        info!("  Data transferred: {} bytes sent, {} bytes received", 
              self.bytes_sent, self.bytes_received);
        info!("  Average response time: {:.2}ms", self.avg_response_time_ms);
        info!("  Connection errors: {}", self.connection_errors);
        info!("  Protocol errors: {}", self.protocol_errors);
        
        // Performance analysis
        if self.total_connections > 0 {
            let error_rate = (self.connection_errors + self.protocol_errors) as f64 / 
                           self.total_connections as f64 * 100.0;
            info!("  Error rate: {:.2}%", error_rate);
            
            if error_rate > 5.0 {
                warn!("High error rate detected: {:.2}%", error_rate);
            }
        }
        
        if self.avg_response_time_ms > 100.0 {
            warn!("High average response time: {:.2}ms", self.avg_response_time_ms);
        }
    }

    /// Log a compact one-line summary
    pub fn log_summary(&self) {
        info!("Stats: {}c/{}peak {}req {:.1}ms avg {}err", 
              self.active_connections, 
              self.peak_connections,
              self.requests_processed,
              self.avg_response_time_ms,
              self.connection_errors + self.protocol_errors);
    }

    /// Reset statistics
    pub fn reset(&mut self) {
        *self = Default::default();
    }
    
    /// Update response time with new measurement
    pub fn update_response_time(&mut self, time_ms: f64) {
        // Simple moving average
        if self.requests_processed == 0 {
            self.avg_response_time_ms = time_ms;
        } else {
            let alpha = 0.1; // Smoothing factor
            self.avg_response_time_ms = alpha * time_ms + (1.0 - alpha) * self.avg_response_time_ms;
        }
    }
    
    /// Record a new connection
    pub fn connection_opened(&mut self) {
        self.total_connections += 1;
        self.active_connections += 1;
        if self.active_connections > self.peak_connections {
            self.peak_connections = self.active_connections;
        }
    }
    
    /// Record a connection closure
    pub fn connection_closed(&mut self) {
        if self.active_connections > 0 {
            self.active_connections -= 1;
        }
    }
}

/// Server performance monitoring
#[derive(Debug, Default)]
pub struct PerformanceMonitor {
    stats: Arc<Mutex<PerformanceStats>>,
}

#[derive(Debug, Default)]
struct PerformanceStats {
    operation_timings: HashMap<String, TimingStats>,
    memory_usage: HashMap<String, usize>,
    last_report: Option<Instant>,
}

#[derive(Debug, Default)]
struct TimingStats {
    total_time: Duration,
    count: u64,
    min_time: Option<Duration>,
    max_time: Option<Duration>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(PerformanceStats::default())),
        }
    }
    
    /// Record timing for an operation
    pub fn record_timing(&self, operation: &str, duration: Duration) {
        let mut stats = self.stats.lock().unwrap();
        let timing_stats = stats.operation_timings.entry(operation.to_string())
            .or_insert_with(TimingStats::default);
        
        timing_stats.total_time += duration;
        timing_stats.count += 1;
        
        match timing_stats.min_time {
            None => timing_stats.min_time = Some(duration),
            Some(min) if duration < min => timing_stats.min_time = Some(duration),
            _ => {}
        }
        
        match timing_stats.max_time {
            None => timing_stats.max_time = Some(duration),
            Some(max) if duration > max => timing_stats.max_time = Some(duration),
            _ => {}
        }
    }
    
    /// Record memory usage for a component
    pub fn record_memory_usage(&self, component: &str, bytes: usize) {
        let mut stats = self.stats.lock().unwrap();
        stats.memory_usage.insert(component.to_string(), bytes);
    }
    
    /// Log performance report if enough time has passed
    pub fn maybe_log_report(&self, force: bool) {
        let mut stats = self.stats.lock().unwrap();
        let should_report = force || match stats.last_report {
            None => true,
            Some(last) => last.elapsed() > Duration::from_secs(60), // Report every minute
        };
        
        if should_report {
            self.log_performance_report(&stats);
            stats.last_report = Some(Instant::now());
        }
    }
    
    fn log_performance_report(&self, stats: &PerformanceStats) {
        info!("=== Performance Report ===");
        
        // Operation timings
        if !stats.operation_timings.is_empty() {
            info!("Operation Timings:");
            for (operation, timing) in &stats.operation_timings {
                let avg_ms = if timing.count > 0 {
                    timing.total_time.as_millis() as f64 / timing.count as f64
                } else {
                    0.0
                };
                
                let min_ms = timing.min_time.map(|d| d.as_millis()).unwrap_or(0);
                let max_ms = timing.max_time.map(|d| d.as_millis()).unwrap_or(0);
                
                info!("  {}: {:.1}ms avg ({} calls, {}ms min, {}ms max)", 
                      operation, avg_ms, timing.count, min_ms, max_ms);
            }
        }
        
        // Memory usage
        if !stats.memory_usage.is_empty() {
            info!("Memory Usage:");
            for (component, bytes) in &stats.memory_usage {
                info!("  {}: {:.1} MB", component, *bytes as f64 / 1_048_576.0);
            }
        }
    }
}
