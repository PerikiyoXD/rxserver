//! Network monitoring module
//!
//! Provides comprehensive monitoring capabilities for network connections and performance.

pub mod bandwidth;
pub mod health;
pub mod latency;
pub mod metrics;

// Re-export commonly used items
pub use bandwidth::{BandwidthLimit, BandwidthMonitor, BandwidthStats};
pub use health::{HealthCheck, HealthMonitor, HealthStatus};
pub use latency::{LatencyMonitor, LatencyStats, LatencyThreshold};
pub use metrics::{MetricType, MetricsCollector, NetworkMetrics};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Comprehensive network monitor that combines all monitoring capabilities
pub struct NetworkMonitor {
    metrics_collector: Arc<MetricsCollector>,
    health_monitor: Arc<HealthMonitor>,
    bandwidth_monitor: Arc<BandwidthMonitor>,
    latency_monitor: Arc<LatencyMonitor>,
    connections: Arc<RwLock<HashMap<SocketAddr, ConnectionInfo>>>,
}

/// Information about a monitored connection
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub address: SocketAddr,
    pub connected_at: Instant,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub last_activity: Instant,
    pub connection_type: ConnectionType,
}

/// Types of network connections
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionType {
    Client,
    Server,
    Proxy,
    Internal,
}

/// Monitoring events
#[derive(Debug, Clone)]
pub enum MonitoringEvent {
    ConnectionEstablished(SocketAddr),
    ConnectionClosed(SocketAddr),
    DataTransferred {
        addr: SocketAddr,
        bytes: u64,
        direction: DataDirection,
    },
    LatencyThresholdExceeded {
        addr: SocketAddr,
        latency: Duration,
    },
    BandwidthLimitExceeded {
        addr: SocketAddr,
        bandwidth: u64,
    },
    HealthCheckFailed(SocketAddr),
}

/// Data transfer direction
#[derive(Debug, Clone, PartialEq)]
pub enum DataDirection {
    Sent,
    Received,
}

impl NetworkMonitor {
    /// Create a new network monitor
    pub fn new() -> Self {
        Self {
            metrics_collector: Arc::new(MetricsCollector::new()),
            health_monitor: Arc::new(HealthMonitor::new()),
            bandwidth_monitor: Arc::new(BandwidthMonitor::new()),
            latency_monitor: Arc::new(LatencyMonitor::new()),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new connection for monitoring
    pub async fn register_connection(&self, addr: SocketAddr, conn_type: ConnectionType) {
        let info = ConnectionInfo {
            address: addr,
            connected_at: Instant::now(),
            bytes_sent: 0,
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
            last_activity: Instant::now(),
            connection_type: conn_type,
        };

        self.connections.write().await.insert(addr, info);
        self.metrics_collector
            .record_connection_established(addr)
            .await;
    }

    /// Unregister a connection
    pub async fn unregister_connection(&self, addr: SocketAddr) {
        self.connections.write().await.remove(&addr);
        self.metrics_collector.record_connection_closed(addr).await;
    }
    /// Record data transfer
    pub async fn record_data_transfer(
        &self,
        addr: SocketAddr,
        bytes: u64,
        direction: DataDirection,
    ) {
        if let Some(conn) = self.connections.write().await.get_mut(&addr) {
            match direction {
                DataDirection::Sent => {
                    conn.bytes_sent += bytes;
                    conn.packets_sent += 1;
                }
                DataDirection::Received => {
                    conn.bytes_received += bytes;
                    conn.packets_received += 1;
                }
            }
            conn.last_activity = Instant::now();
        }
        self.bandwidth_monitor
            .record_transfer(addr, bytes, direction.clone())
            .await;
        self.metrics_collector
            .record_data_transfer(addr, bytes, direction)
            .await;
    }

    /// Record latency measurement
    pub async fn record_latency(&self, addr: SocketAddr, latency: Duration) {
        self.latency_monitor.record_latency(addr, latency).await;
        self.metrics_collector.record_latency(addr, latency).await;
    }

    /// Get connection information
    pub async fn get_connection_info(&self, addr: SocketAddr) -> Option<ConnectionInfo> {
        self.connections.read().await.get(&addr).cloned()
    }

    /// Get all connections
    pub async fn get_all_connections(&self) -> Vec<ConnectionInfo> {
        self.connections.read().await.values().cloned().collect()
    }

    /// Get network metrics
    pub async fn get_metrics(&self) -> NetworkMetrics {
        self.metrics_collector.get_metrics().await
    }

    /// Get health status
    pub async fn get_health_status(&self) -> HealthStatus {
        self.health_monitor.get_status().await
    }

    /// Get bandwidth statistics
    pub async fn get_bandwidth_stats(&self, addr: SocketAddr) -> Option<BandwidthStats> {
        self.bandwidth_monitor.get_stats(addr).await
    }

    /// Get latency statistics
    pub async fn get_latency_stats(&self, addr: SocketAddr) -> Option<LatencyStats> {
        self.latency_monitor.get_stats(addr).await
    }

    /// Start monitoring all components
    pub async fn start_monitoring(&self) {
        self.health_monitor.start_monitoring().await;
        self.bandwidth_monitor.start_monitoring().await;
        self.latency_monitor.start_monitoring().await;
    }

    /// Stop monitoring all components
    pub async fn stop_monitoring(&self) {
        self.health_monitor.stop_monitoring().await;
        self.bandwidth_monitor.stop_monitoring().await;
        self.latency_monitor.stop_monitoring().await;
    }

    /// Generate monitoring report
    pub async fn generate_report(&self) -> MonitoringReport {
        let connections = self.get_all_connections().await;
        let metrics = self.get_metrics().await;
        let health = self.get_health_status().await;

        MonitoringReport {
            timestamp: Instant::now(),
            total_connections: connections.len(),
            active_connections: connections
                .iter()
                .filter(|c| c.last_activity.elapsed() < Duration::from_secs(30))
                .count(),
            total_bytes_transferred: connections
                .iter()
                .map(|c| c.bytes_sent + c.bytes_received)
                .sum(),
            metrics,
            health,
        }
    }
}

/// Comprehensive monitoring report
#[derive(Debug, Clone)]
pub struct MonitoringReport {
    pub timestamp: Instant,
    pub total_connections: usize,
    pub active_connections: usize,
    pub total_bytes_transferred: u64,
    pub metrics: NetworkMetrics,
    pub health: HealthStatus,
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_monitor_creation() {
        let monitor = NetworkMonitor::new();
        let addr = "127.0.0.1:6000".parse().unwrap();

        monitor
            .register_connection(addr, ConnectionType::Client)
            .await;
        let info = monitor.get_connection_info(addr).await;

        assert!(info.is_some());
        assert_eq!(info.unwrap().connection_type, ConnectionType::Client);
    }

    #[tokio::test]
    async fn test_data_transfer_recording() {
        let monitor = NetworkMonitor::new();
        let addr = "127.0.0.1:6000".parse().unwrap();

        monitor
            .register_connection(addr, ConnectionType::Client)
            .await;
        monitor
            .record_data_transfer(addr, 1024, DataDirection::Sent)
            .await;

        let info = monitor.get_connection_info(addr).await.unwrap();
        assert_eq!(info.bytes_sent, 1024);
        assert_eq!(info.packets_sent, 1);
    }
}
