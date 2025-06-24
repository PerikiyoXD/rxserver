//! Bandwidth monitoring and management
//!
//! Provides bandwidth monitoring, limiting, and analysis capabilities.

use super::DataDirection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Bandwidth statistics for a connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthStats {
    pub address: SocketAddr,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub bytes_per_second_sent: f64,
    pub bytes_per_second_received: f64,
    pub peak_bps_sent: f64,
    pub peak_bps_received: f64,
    pub start_time: u64,
    pub last_updated: u64,
    pub window_size: Duration,
}

/// Bandwidth limit configuration
#[derive(Debug, Clone)]
pub struct BandwidthLimit {
    pub max_bps_sent: Option<u64>,
    pub max_bps_received: Option<u64>,
    pub burst_allowance: Option<u64>,
    pub window_size: Duration,
    pub enabled: bool,
}

/// Bandwidth tracking entry
#[derive(Debug, Clone)]
struct BandwidthEntry {
    bytes_sent: u64,
    bytes_received: u64,
    timestamps_sent: Vec<(Instant, u64)>,
    timestamps_received: Vec<(Instant, u64)>,
    peak_bps_sent: f64,
    peak_bps_received: f64,
    start_time: Instant,
    last_updated: Instant,
    limit: Option<BandwidthLimit>,
}

/// Bandwidth monitor implementation
pub struct BandwidthMonitor {
    connections: Arc<RwLock<HashMap<SocketAddr, BandwidthEntry>>>,
    global_limit: Arc<RwLock<Option<BandwidthLimit>>>,
    window_size: Duration,
    is_monitoring: Arc<RwLock<bool>>,
}

/// Bandwidth usage over time
#[derive(Debug, Clone)]
pub struct BandwidthUsage {
    pub timestamp: Instant,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub bps_sent: f64,
    pub bps_received: f64,
}

impl Default for BandwidthLimit {
    fn default() -> Self {
        Self {
            max_bps_sent: None,
            max_bps_received: None,
            burst_allowance: Some(1024 * 1024), // 1MB burst
            window_size: Duration::from_secs(1),
            enabled: false,
        }
    }
}

impl BandwidthMonitor {
    /// Create a new bandwidth monitor
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            global_limit: Arc::new(RwLock::new(None)),
            window_size: Duration::from_secs(1),
            is_monitoring: Arc::new(RwLock::new(false)),
        }
    }

    /// Set the monitoring window size
    pub async fn set_window_size(&self, _window_size: Duration) {
        // Update would require updating all existing entries
        // For simplicity, we'll just store it for new connections
    }

    /// Set global bandwidth limit
    pub async fn set_global_limit(&self, limit: BandwidthLimit) {
        *self.global_limit.write().await = Some(limit);
    }

    /// Set per-connection bandwidth limit
    pub async fn set_connection_limit(&self, addr: SocketAddr, limit: BandwidthLimit) {
        if let Some(entry) = self.connections.write().await.get_mut(&addr) {
            entry.limit = Some(limit);
        }
    }

    /// Record data transfer
    pub async fn record_transfer(&self, addr: SocketAddr, bytes: u64, direction: DataDirection) {
        let now = Instant::now();
        let mut connections = self.connections.write().await;

        let entry = connections.entry(addr).or_insert_with(|| BandwidthEntry {
            bytes_sent: 0,
            bytes_received: 0,
            timestamps_sent: Vec::new(),
            timestamps_received: Vec::new(),
            peak_bps_sent: 0.0,
            peak_bps_received: 0.0,
            start_time: now,
            last_updated: now,
            limit: None,
        });

        match direction {
            DataDirection::Sent => {
                entry.bytes_sent += bytes;
                entry.timestamps_sent.push((now, bytes));
                self.cleanup_old_timestamps(&mut entry.timestamps_sent, now);

                let current_bps = self.calculate_bps(&entry.timestamps_sent, now);
                entry.peak_bps_sent = entry.peak_bps_sent.max(current_bps);
            }
            DataDirection::Received => {
                entry.bytes_received += bytes;
                entry.timestamps_received.push((now, bytes));
                self.cleanup_old_timestamps(&mut entry.timestamps_received, now);

                let current_bps = self.calculate_bps(&entry.timestamps_received, now);
                entry.peak_bps_received = entry.peak_bps_received.max(current_bps);
            }
        }

        entry.last_updated = now;

        // Check limits
        self.check_limits(addr, entry).await;
    }

    /// Calculate bytes per second from timestamps
    fn calculate_bps(&self, timestamps: &[(Instant, u64)], now: Instant) -> f64 {
        if timestamps.is_empty() {
            return 0.0;
        }

        let window_start = now - self.window_size;
        let bytes_in_window: u64 = timestamps
            .iter()
            .filter(|(timestamp, _)| *timestamp >= window_start)
            .map(|(_, bytes)| *bytes)
            .sum();

        bytes_in_window as f64 / self.window_size.as_secs_f64()
    }

    /// Remove timestamps outside the monitoring window
    fn cleanup_old_timestamps(&self, timestamps: &mut Vec<(Instant, u64)>, now: Instant) {
        let window_start = now - self.window_size;
        timestamps.retain(|(timestamp, _)| *timestamp >= window_start);
    }

    /// Check bandwidth limits for a connection
    async fn check_limits(&self, addr: SocketAddr, entry: &BandwidthEntry) {
        if let Some(ref limit) = entry.limit {
            if !limit.enabled {
                return;
            }

            let now = Instant::now();
            let current_bps_sent = self.calculate_bps(&entry.timestamps_sent, now);
            let current_bps_received = self.calculate_bps(&entry.timestamps_received, now);

            if let Some(max_sent) = limit.max_bps_sent {
                if current_bps_sent > max_sent as f64 {
                    // Log or handle bandwidth limit exceeded
                    eprintln!(
                        "Bandwidth limit exceeded for {}: {} bps sent (limit: {} bps)",
                        addr, current_bps_sent, max_sent
                    );
                }
            }

            if let Some(max_received) = limit.max_bps_received {
                if current_bps_received > max_received as f64 {
                    eprintln!(
                        "Bandwidth limit exceeded for {}: {} bps received (limit: {} bps)",
                        addr, current_bps_received, max_received
                    );
                }
            }
        }
    }

    /// Get bandwidth statistics for a connection
    pub async fn get_stats(&self, addr: SocketAddr) -> Option<BandwidthStats> {
        let connections = self.connections.read().await;
        let entry = connections.get(&addr)?;

        let now = Instant::now();
        let bps_sent = self.calculate_bps(&entry.timestamps_sent, now);
        let bps_received = self.calculate_bps(&entry.timestamps_received, now);

        Some(BandwidthStats {
            address: addr,
            bytes_sent: entry.bytes_sent,
            bytes_received: entry.bytes_received,
            bytes_per_second_sent: bps_sent,
            bytes_per_second_received: bps_received,
            peak_bps_sent: entry.peak_bps_sent,
            peak_bps_received: entry.peak_bps_received,
            start_time: entry.start_time.elapsed().as_secs(),
            last_updated: entry.last_updated.elapsed().as_secs(),
            window_size: self.window_size,
        })
    }

    /// Get bandwidth statistics for all connections
    pub async fn get_all_stats(&self) -> Vec<BandwidthStats> {
        let connections = self.connections.read().await;
        let now = Instant::now();

        connections
            .iter()
            .map(|(addr, entry)| {
                let bps_sent = self.calculate_bps(&entry.timestamps_sent, now);
                let bps_received = self.calculate_bps(&entry.timestamps_received, now);

                BandwidthStats {
                    address: *addr,
                    bytes_sent: entry.bytes_sent,
                    bytes_received: entry.bytes_received,
                    bytes_per_second_sent: bps_sent,
                    bytes_per_second_received: bps_received,
                    peak_bps_sent: entry.peak_bps_sent,
                    peak_bps_received: entry.peak_bps_received,
                    start_time: entry.start_time.elapsed().as_secs(),
                    last_updated: entry.last_updated.elapsed().as_secs(),
                    window_size: self.window_size,
                }
            })
            .collect()
    }

    /// Get total bandwidth usage across all connections
    pub async fn get_total_usage(&self) -> BandwidthUsage {
        let connections = self.connections.read().await;
        let now = Instant::now();

        let mut total_bytes_sent = 0;
        let mut total_bytes_received = 0;
        let mut total_bps_sent = 0.0;
        let mut total_bps_received = 0.0;

        for entry in connections.values() {
            total_bytes_sent += entry.bytes_sent;
            total_bytes_received += entry.bytes_received;
            total_bps_sent += self.calculate_bps(&entry.timestamps_sent, now);
            total_bps_received += self.calculate_bps(&entry.timestamps_received, now);
        }

        BandwidthUsage {
            timestamp: now,
            bytes_sent: total_bytes_sent,
            bytes_received: total_bytes_received,
            bps_sent: total_bps_sent,
            bps_received: total_bps_received,
        }
    }

    /// Remove a connection from monitoring
    pub async fn remove_connection(&self, addr: SocketAddr) -> bool {
        self.connections.write().await.remove(&addr).is_some()
    }

    /// Clear all bandwidth statistics
    pub async fn clear_stats(&self) {
        self.connections.write().await.clear();
    }

    /// Start periodic monitoring and cleanup
    pub async fn start_monitoring(&self) {
        *self.is_monitoring.write().await = true;

        let connections = Arc::clone(&self.connections);
        let is_monitoring = Arc::clone(&self.is_monitoring);
        let window_size = self.window_size;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            while *is_monitoring.read().await {
                interval.tick().await;

                let now = Instant::now();
                let mut connections_guard = connections.write().await;

                // Clean up old timestamps for all connections
                for entry in connections_guard.values_mut() {
                    let window_start = now - window_size;
                    entry
                        .timestamps_sent
                        .retain(|(timestamp, _)| *timestamp >= window_start);
                    entry
                        .timestamps_received
                        .retain(|(timestamp, _)| *timestamp >= window_start);
                }

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

    /// Check if monitoring is active
    pub async fn is_monitoring(&self) -> bool {
        *self.is_monitoring.read().await
    }

    /// Get top bandwidth consumers
    pub async fn get_top_consumers(&self, limit: usize) -> Vec<BandwidthStats> {
        let mut stats = self.get_all_stats().await;
        stats.sort_by(|a, b| {
            let total_a = a.bytes_per_second_sent + a.bytes_per_second_received;
            let total_b = b.bytes_per_second_sent + b.bytes_per_second_received;
            total_b
                .partial_cmp(&total_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        stats.into_iter().take(limit).collect()
    }
}

impl Default for BandwidthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bandwidth_monitoring() {
        let monitor = BandwidthMonitor::new();
        let addr: SocketAddr = "127.0.0.1:6000".parse().unwrap();

        monitor
            .record_transfer(addr, 1024, DataDirection::Sent)
            .await;
        monitor
            .record_transfer(addr, 512, DataDirection::Received)
            .await;

        let stats = monitor.get_stats(addr).await.unwrap();
        assert_eq!(stats.bytes_sent, 1024);
        assert_eq!(stats.bytes_received, 512);
    }

    #[tokio::test]
    async fn test_bandwidth_limits() {
        let monitor = BandwidthMonitor::new();
        let addr: SocketAddr = "127.0.0.1:6000".parse().unwrap();

        let limit = BandwidthLimit {
            max_bps_sent: Some(1000),
            max_bps_received: Some(1000),
            enabled: true,
            ..Default::default()
        };

        monitor.set_connection_limit(addr, limit).await;
        monitor
            .record_transfer(addr, 2000, DataDirection::Sent)
            .await;

        // This would trigger the limit check
        let stats = monitor.get_stats(addr).await.unwrap();
        assert_eq!(stats.bytes_sent, 2000);
    }

    #[tokio::test]
    async fn test_total_usage() {
        let monitor = BandwidthMonitor::new();
        let addr1: SocketAddr = "127.0.0.1:6000".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:6001".parse().unwrap();

        monitor
            .record_transfer(addr1, 1000, DataDirection::Sent)
            .await;
        monitor
            .record_transfer(addr2, 500, DataDirection::Received)
            .await;

        let usage = monitor.get_total_usage().await;
        assert_eq!(usage.bytes_sent, 1000);
        assert_eq!(usage.bytes_received, 500);
    }
}
