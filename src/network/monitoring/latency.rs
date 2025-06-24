//! Latency monitoring and analysis
//!
//! Provides latency measurement, tracking, and alerting capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Latency statistics for a connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub address: SocketAddr,
    pub sample_count: u64,
    pub avg_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub current_latency_ms: f64,
    pub last_updated: u64,
    pub window_size: Duration,
}

/// Latency threshold configuration
#[derive(Debug, Clone)]
pub struct LatencyThreshold {
    pub warning_ms: Option<f64>,
    pub critical_ms: Option<f64>,
    pub consecutive_violations: u32,
    pub enabled: bool,
}

/// Latency measurement entry
#[derive(Debug, Clone)]
struct LatencyEntry {
    samples: Vec<Duration>,
    last_latency: Duration,
    total_samples: u64,
    threshold: Option<LatencyThreshold>,
    violation_count: u32,
    last_updated: Instant,
    start_time: Instant,
}

/// Latency alert
#[derive(Debug, Clone)]
pub struct LatencyAlert {
    pub address: SocketAddr,
    pub level: AlertLevel,
    pub current_latency_ms: f64,
    pub threshold_ms: f64,
    pub consecutive_violations: u32,
    pub timestamp: Instant,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum AlertLevel {
    Warning,
    Critical,
}

/// Latency monitor implementation
pub struct LatencyMonitor {
    connections: Arc<RwLock<HashMap<SocketAddr, LatencyEntry>>>,
    window_size: Duration,
    max_samples: usize,
    is_monitoring: Arc<RwLock<bool>>,
    alert_callback: Arc<RwLock<Option<Box<dyn Fn(LatencyAlert) + Send + Sync>>>>,
}

impl Default for LatencyThreshold {
    fn default() -> Self {
        Self {
            warning_ms: Some(100.0),
            critical_ms: Some(500.0),
            consecutive_violations: 3,
            enabled: true,
        }
    }
}

impl LatencyMonitor {
    /// Create a new latency monitor
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            window_size: Duration::from_secs(60), // 1 minute window
            max_samples: 1000,                    // Keep last 1000 samples
            is_monitoring: Arc::new(RwLock::new(false)),
            alert_callback: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the monitoring window size
    pub fn set_window_size(&mut self, window_size: Duration) {
        self.window_size = window_size;
    }

    /// Set maximum number of samples to keep
    pub fn set_max_samples(&mut self, max_samples: usize) {
        self.max_samples = max_samples;
    }

    /// Set alert callback function
    pub async fn set_alert_callback<F>(&self, callback: F)
    where
        F: Fn(LatencyAlert) + Send + Sync + 'static,
    {
        *self.alert_callback.write().await = Some(Box::new(callback));
    }

    /// Set latency threshold for a connection
    pub async fn set_threshold(&self, addr: SocketAddr, threshold: LatencyThreshold) {
        let mut connections = self.connections.write().await;
        if let Some(entry) = connections.get_mut(&addr) {
            entry.threshold = Some(threshold);
        } else {
            let entry = LatencyEntry {
                samples: Vec::new(),
                last_latency: Duration::ZERO,
                total_samples: 0,
                threshold: Some(threshold),
                violation_count: 0,
                last_updated: Instant::now(),
                start_time: Instant::now(),
            };
            connections.insert(addr, entry);
        }
    }

    /// Record a latency measurement
    pub async fn record_latency(&self, addr: SocketAddr, latency: Duration) {
        let now = Instant::now();
        let mut connections = self.connections.write().await;

        let entry = connections.entry(addr).or_insert_with(|| LatencyEntry {
            samples: Vec::new(),
            last_latency: Duration::ZERO,
            total_samples: 0,
            threshold: None,
            violation_count: 0,
            last_updated: now,
            start_time: now,
        });

        entry.samples.push(latency);
        entry.last_latency = latency;
        entry.total_samples += 1;
        entry.last_updated = now;

        // Cleanup old samples
        self.cleanup_old_samples(entry, now);

        // Enforce max samples limit
        if entry.samples.len() > self.max_samples {
            entry
                .samples
                .drain(0..entry.samples.len() - self.max_samples);
        }

        // Check thresholds
        if let Some(alert) = self.check_threshold(addr, entry, latency).await {
            if let Some(ref callback) = *self.alert_callback.read().await {
                callback(alert);
            }
        }
    }

    /// Clean up samples outside the monitoring window
    fn cleanup_old_samples(&self, entry: &mut LatencyEntry, now: Instant) {
        let _window_start = now - self.window_size;
        let start_time = entry.start_time;

        // Calculate how many samples to keep based on time window
        // This is an approximation since we don't store timestamps with samples
        let total_duration = now.duration_since(start_time);
        if total_duration > self.window_size {
            let samples_to_keep = (self.window_size.as_secs_f64() / total_duration.as_secs_f64()
                * entry.samples.len() as f64) as usize;

            if samples_to_keep < entry.samples.len() {
                entry
                    .samples
                    .drain(0..entry.samples.len() - samples_to_keep);
            }
        }
    }

    /// Check latency thresholds and generate alerts
    async fn check_threshold(
        &self,
        addr: SocketAddr,
        entry: &mut LatencyEntry,
        latency: Duration,
    ) -> Option<LatencyAlert> {
        let threshold = entry.threshold.as_ref()?;

        if !threshold.enabled {
            return None;
        }

        let latency_ms = latency.as_millis() as f64;

        // Check critical threshold first
        if let Some(critical_ms) = threshold.critical_ms {
            if latency_ms > critical_ms {
                entry.violation_count += 1;
                if entry.violation_count >= threshold.consecutive_violations {
                    return Some(LatencyAlert {
                        address: addr,
                        level: AlertLevel::Critical,
                        current_latency_ms: latency_ms,
                        threshold_ms: critical_ms,
                        consecutive_violations: entry.violation_count,
                        timestamp: Instant::now(),
                    });
                }
            } else {
                entry.violation_count = 0;
            }
        }

        // Check warning threshold
        if let Some(warning_ms) = threshold.warning_ms {
            if latency_ms > warning_ms {
                return Some(LatencyAlert {
                    address: addr,
                    level: AlertLevel::Warning,
                    current_latency_ms: latency_ms,
                    threshold_ms: warning_ms,
                    consecutive_violations: 1,
                    timestamp: Instant::now(),
                });
            }
        }

        None
    }

    /// Get latency statistics for a connection
    pub async fn get_stats(&self, addr: SocketAddr) -> Option<LatencyStats> {
        let connections = self.connections.read().await;
        let entry = connections.get(&addr)?;

        if entry.samples.is_empty() {
            return None;
        }

        let mut sorted_samples: Vec<Duration> = entry.samples.clone();
        sorted_samples.sort();

        let len = sorted_samples.len();
        let sum_ms: f64 = sorted_samples.iter().map(|d| d.as_millis() as f64).sum();

        let avg_latency_ms = sum_ms / len as f64;
        let min_latency_ms = sorted_samples.first().unwrap().as_millis() as f64;
        let max_latency_ms = sorted_samples.last().unwrap().as_millis() as f64;

        let p50_idx = len / 2;
        let p95_idx = (len as f64 * 0.95) as usize;
        let p99_idx = (len as f64 * 0.99) as usize;

        let p50_latency_ms = sorted_samples[p50_idx.min(len - 1)].as_millis() as f64;
        let p95_latency_ms = sorted_samples[p95_idx.min(len - 1)].as_millis() as f64;
        let p99_latency_ms = sorted_samples[p99_idx.min(len - 1)].as_millis() as f64;

        Some(LatencyStats {
            address: addr,
            sample_count: entry.total_samples,
            avg_latency_ms,
            min_latency_ms,
            max_latency_ms,
            p50_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            current_latency_ms: entry.last_latency.as_millis() as f64,
            last_updated: entry.last_updated.elapsed().as_secs(),
            window_size: self.window_size,
        })
    }

    /// Get latency statistics for all connections
    pub async fn get_all_stats(&self) -> Vec<LatencyStats> {
        let connections = self.connections.read().await;
        let mut stats = Vec::new();

        for (&addr, _) in connections.iter() {
            if let Some(stat) = self.get_stats(addr).await {
                stats.push(stat);
            }
        }

        stats
    }

    /// Get connections with high latency
    pub async fn get_high_latency_connections(&self, threshold_ms: f64) -> Vec<LatencyStats> {
        let all_stats = self.get_all_stats().await;
        all_stats
            .into_iter()
            .filter(|stats| stats.avg_latency_ms > threshold_ms)
            .collect()
    }

    /// Get latency distribution
    pub async fn get_latency_distribution(
        &self,
        addr: SocketAddr,
        buckets: &[f64],
    ) -> Option<Vec<(f64, u64)>> {
        let connections = self.connections.read().await;
        let entry = connections.get(&addr)?;

        if entry.samples.is_empty() {
            return None;
        }

        let mut distribution = vec![0u64; buckets.len()];

        for sample in &entry.samples {
            let latency_ms = sample.as_millis() as f64;

            for (i, &bucket_max) in buckets.iter().enumerate() {
                if latency_ms <= bucket_max {
                    distribution[i] += 1;
                    break;
                }
            }
        }

        Some(
            buckets
                .iter()
                .zip(distribution.iter())
                .map(|(&bucket, &count)| (bucket, count))
                .collect(),
        )
    }

    /// Remove a connection from monitoring
    pub async fn remove_connection(&self, addr: SocketAddr) -> bool {
        self.connections.write().await.remove(&addr).is_some()
    }

    /// Clear all latency statistics
    pub async fn clear_stats(&self) {
        self.connections.write().await.clear();
    }

    /// Start periodic monitoring and cleanup
    pub async fn start_monitoring(&self) {
        *self.is_monitoring.write().await = true;

        let connections = Arc::clone(&self.connections);
        let is_monitoring = Arc::clone(&self.is_monitoring);
        let _window_size = self.window_size;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));

            while *is_monitoring.read().await {
                interval.tick().await;

                let now = Instant::now();
                let mut connections_guard = connections.write().await;

                // Remove stale connections (no activity for 5 minutes)
                let stale_cutoff = now - Duration::from_secs(300);
                connections_guard.retain(|_, entry| entry.last_updated >= stale_cutoff);
            }
        });
    }

    /// Stop monitoring
    pub async fn stop_monitoring(&self) {
        *self.is_monitoring.write().await = false;
    }

    /// Get overall latency summary
    pub async fn get_summary(&self) -> LatencySummary {
        let all_stats = self.get_all_stats().await;

        if all_stats.is_empty() {
            return LatencySummary::default();
        }

        let total_connections = all_stats.len();
        let total_samples: u64 = all_stats.iter().map(|s| s.sample_count).sum();
        let avg_latency: f64 =
            all_stats.iter().map(|s| s.avg_latency_ms).sum::<f64>() / total_connections as f64;
        let max_latency = all_stats
            .iter()
            .map(|s| s.max_latency_ms)
            .fold(0.0, f64::max);
        let min_latency = all_stats
            .iter()
            .map(|s| s.min_latency_ms)
            .fold(f64::INFINITY, f64::min);

        let high_latency_count = all_stats
            .iter()
            .filter(|s| s.avg_latency_ms > 100.0) // 100ms threshold
            .count();

        LatencySummary {
            total_connections,
            total_samples,
            avg_latency_ms: avg_latency,
            min_latency_ms: if min_latency == f64::INFINITY {
                0.0
            } else {
                min_latency
            },
            max_latency_ms: max_latency,
            high_latency_connections: high_latency_count,
        }
    }
}

/// Latency monitoring summary
#[derive(Debug, Clone)]
pub struct LatencySummary {
    pub total_connections: usize,
    pub total_samples: u64,
    pub avg_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub high_latency_connections: usize,
}

impl Default for LatencySummary {
    fn default() -> Self {
        Self {
            total_connections: 0,
            total_samples: 0,
            avg_latency_ms: 0.0,
            min_latency_ms: 0.0,
            max_latency_ms: 0.0,
            high_latency_connections: 0,
        }
    }
}

impl Default for LatencyMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_latency_monitoring() {
        let monitor = LatencyMonitor::new();
        let addr: SocketAddr = "127.0.0.1:6000".parse().unwrap();

        monitor
            .record_latency(addr, Duration::from_millis(50))
            .await;
        monitor
            .record_latency(addr, Duration::from_millis(75))
            .await;
        monitor
            .record_latency(addr, Duration::from_millis(25))
            .await;

        let stats = monitor.get_stats(addr).await.unwrap();
        assert_eq!(stats.sample_count, 3);
        assert_eq!(stats.min_latency_ms, 25.0);
        assert_eq!(stats.max_latency_ms, 75.0);
        assert!((stats.avg_latency_ms - 50.0).abs() < 1.0);
    }

    #[tokio::test]
    async fn test_latency_thresholds() {
        let monitor = LatencyMonitor::new();
        let addr: SocketAddr = "127.0.0.1:6000".parse().unwrap();

        let threshold = LatencyThreshold {
            warning_ms: Some(50.0),
            critical_ms: Some(100.0),
            consecutive_violations: 2,
            enabled: true,
        };

        monitor.set_threshold(addr, threshold).await;

        // This should trigger a warning
        monitor
            .record_latency(addr, Duration::from_millis(75))
            .await;

        let stats = monitor.get_stats(addr).await.unwrap();
        assert_eq!(stats.current_latency_ms, 75.0);
    }

    #[tokio::test]
    async fn test_latency_distribution() {
        let monitor = LatencyMonitor::new();
        let addr: SocketAddr = "127.0.0.1:6000".parse().unwrap();

        monitor
            .record_latency(addr, Duration::from_millis(10))
            .await;
        monitor
            .record_latency(addr, Duration::from_millis(50))
            .await;
        monitor
            .record_latency(addr, Duration::from_millis(150))
            .await;

        let buckets = vec![25.0, 75.0, 200.0];
        let distribution = monitor
            .get_latency_distribution(addr, &buckets)
            .await
            .unwrap();

        assert_eq!(distribution.len(), 3);
        assert_eq!(distribution[0].1, 1); // 10ms falls in first bucket
        assert_eq!(distribution[1].1, 1); // 50ms falls in second bucket
        assert_eq!(distribution[2].1, 1); // 150ms falls in third bucket
    }
}
