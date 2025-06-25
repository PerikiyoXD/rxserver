//! Network metrics collection and analysis
//!
//! Provides comprehensive metrics collection for network performance monitoring.

use super::DataDirection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Network metrics data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub timestamp: u64,
    pub connections: ConnectionMetrics,
    pub bandwidth: BandwidthMetrics,
    pub latency: LatencyMetrics,
    pub errors: ErrorMetrics,
    pub protocol: ProtocolMetrics,
}

/// Connection-related metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetrics {
    pub total_connections: u64,
    pub active_connections: u64,
    pub connections_per_second: f64,
    pub connection_duration_avg: f64,
    pub connection_duration_max: f64,
    pub failed_connections: u64,
}

/// Bandwidth-related metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub throughput_bps: f64,
    pub peak_throughput_bps: f64,
}

/// Latency-related metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    pub avg_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
}

/// Error-related metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub connection_errors: u64,
    pub timeout_errors: u64,
    pub protocol_errors: u64,
    pub io_errors: u64,
    pub authentication_errors: u64,
}

/// Protocol-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMetrics {
    pub x11_requests: u64,
    pub x11_responses: u64,
    pub x11_events: u64,
    pub x11_errors: u64,
    pub request_rate: f64,
}

/// Types of metrics that can be collected
#[derive(Debug, Clone, PartialEq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Timer,
}

/// Individual metric entry
#[derive(Debug, Clone)]
struct MetricEntry {
    name: String,
    metric_type: MetricType,
    value: f64,
    timestamp: Instant,
    labels: HashMap<String, String>,
}

/// Metrics collector implementation
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, MetricEntry>>>,
    connection_stats: Arc<RwLock<HashMap<SocketAddr, ConnectionStats>>>,
    start_time: Instant,
}

/// Per-connection statistics
#[derive(Debug, Clone)]
struct ConnectionStats {
    established_at: Instant,
    bytes_sent: u64,
    bytes_received: u64,
    packets_sent: u64,
    packets_received: u64,
    latencies: Vec<Duration>,
    last_activity: Instant,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            connection_stats: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Record a counter metric
    pub async fn record_counter(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        let metric = MetricEntry {
            name: name.to_string(),
            metric_type: MetricType::Counter,
            value,
            timestamp: Instant::now(),
            labels,
        };

        self.metrics.write().await.insert(name.to_string(), metric);
    }

    /// Record a gauge metric
    pub async fn record_gauge(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        let metric = MetricEntry {
            name: name.to_string(),
            metric_type: MetricType::Gauge,
            value,
            timestamp: Instant::now(),
            labels,
        };

        self.metrics.write().await.insert(name.to_string(), metric);
    }

    /// Record a connection establishment
    pub async fn record_connection_established(&self, addr: SocketAddr) {
        let stats = ConnectionStats {
            established_at: Instant::now(),
            bytes_sent: 0,
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
            latencies: Vec::new(),
            last_activity: Instant::now(),
        };

        self.connection_stats.write().await.insert(addr, stats);

        let mut labels = HashMap::new();
        labels.insert("address".to_string(), addr.to_string());
        self.record_counter("connections_established", 1.0, labels)
            .await;
    }

    /// Record a connection closure
    pub async fn record_connection_closed(&self, addr: SocketAddr) {
        self.connection_stats.write().await.remove(&addr);

        let mut labels = HashMap::new();
        labels.insert("address".to_string(), addr.to_string());
        self.record_counter("connections_closed", 1.0, labels).await;
    }

    /// Record data transfer
    pub async fn record_data_transfer(
        &self,
        addr: SocketAddr,
        bytes: u64,
        direction: DataDirection,
    ) {
        if let Some(stats) = self.connection_stats.write().await.get_mut(&addr) {
            match direction {
                DataDirection::Sent => {
                    stats.bytes_sent += bytes;
                    stats.packets_sent += 1;
                }
                DataDirection::Received => {
                    stats.bytes_received += bytes;
                    stats.packets_received += 1;
                }
            }
            stats.last_activity = Instant::now();
        }

        let mut labels = HashMap::new();
        labels.insert("address".to_string(), addr.to_string());
        labels.insert(
            "direction".to_string(),
            format!("{:?}", direction).to_lowercase(),
        );

        self.record_counter("bytes_transferred", bytes as f64, labels)
            .await;
    }

    /// Record latency measurement
    pub async fn record_latency(&self, addr: SocketAddr, latency: Duration) {
        if let Some(stats) = self.connection_stats.write().await.get_mut(&addr) {
            stats.latencies.push(latency);

            // Keep only the last 1000 latency measurements to prevent unbounded growth
            if stats.latencies.len() > 1000 {
                stats.latencies.drain(0..stats.latencies.len() - 1000);
            }
        }

        let mut labels = HashMap::new();
        labels.insert("address".to_string(), addr.to_string());
        self.record_gauge("latency_ms", latency.as_millis() as f64, labels)
            .await;
    }

    /// Record an error
    pub async fn record_error(&self, error_type: &str, addr: Option<SocketAddr>) {
        let mut labels = HashMap::new();
        labels.insert("error_type".to_string(), error_type.to_string());

        if let Some(addr) = addr {
            labels.insert("address".to_string(), addr.to_string());
        }

        self.record_counter("errors", 1.0, labels).await;
    }

    /// Get comprehensive network metrics
    pub async fn get_metrics(&self) -> NetworkMetrics {
        let connection_stats = self.connection_stats.read().await;
        let uptime = self.start_time.elapsed();

        // Calculate connection metrics
        let total_connections = connection_stats.len() as u64;
        let active_connections = connection_stats
            .values()
            .filter(|s| s.last_activity.elapsed() < Duration::from_secs(30))
            .count() as u64;

        let connections_per_second = if uptime.as_secs() > 0 {
            total_connections as f64 / uptime.as_secs() as f64
        } else {
            0.0
        };

        let connection_durations: Vec<f64> = connection_stats
            .values()
            .map(|s| s.established_at.elapsed().as_secs_f64())
            .collect();

        let connection_duration_avg = if !connection_durations.is_empty() {
            connection_durations.iter().sum::<f64>() / connection_durations.len() as f64
        } else {
            0.0
        };
        let connection_duration_max = connection_durations
            .iter()
            .fold(0.0f64, |max, &duration| max.max(duration));

        // Calculate bandwidth metrics
        let total_bytes_sent: u64 = connection_stats.values().map(|s| s.bytes_sent).sum();
        let total_bytes_received: u64 = connection_stats.values().map(|s| s.bytes_received).sum();
        let total_packets_sent: u64 = connection_stats.values().map(|s| s.packets_sent).sum();
        let total_packets_received: u64 =
            connection_stats.values().map(|s| s.packets_received).sum();

        let throughput_bps = if uptime.as_secs() > 0 {
            ((total_bytes_sent + total_bytes_received) * 8) as f64 / uptime.as_secs() as f64
        } else {
            0.0
        };

        // Calculate latency metrics
        let all_latencies: Vec<Duration> = connection_stats
            .values()
            .flat_map(|s| s.latencies.iter())
            .cloned()
            .collect();

        let (avg_latency_ms, min_latency_ms, max_latency_ms, p95_latency_ms, p99_latency_ms) =
            if !all_latencies.is_empty() {
                let mut sorted_latencies = all_latencies.clone();
                sorted_latencies.sort();

                let avg = all_latencies
                    .iter()
                    .map(|d| d.as_millis() as f64)
                    .sum::<f64>()
                    / all_latencies.len() as f64;
                let min = sorted_latencies.first().unwrap().as_millis() as f64;
                let max = sorted_latencies.last().unwrap().as_millis() as f64;

                let p95_idx = (sorted_latencies.len() as f64 * 0.95) as usize;
                let p99_idx = (sorted_latencies.len() as f64 * 0.99) as usize;

                let p95 = sorted_latencies
                    .get(p95_idx.min(sorted_latencies.len() - 1))
                    .unwrap_or(&Duration::ZERO)
                    .as_millis() as f64;
                let p99 = sorted_latencies
                    .get(p99_idx.min(sorted_latencies.len() - 1))
                    .unwrap_or(&Duration::ZERO)
                    .as_millis() as f64;

                (avg, min, max, p95, p99)
            } else {
                (0.0, 0.0, 0.0, 0.0, 0.0)
            };

        NetworkMetrics {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            connections: ConnectionMetrics {
                total_connections,
                active_connections,
                connections_per_second,
                connection_duration_avg,
                connection_duration_max,
                failed_connections: 0, // TODO: Track failed connections
            },
            bandwidth: BandwidthMetrics {
                bytes_sent: total_bytes_sent,
                bytes_received: total_bytes_received,
                packets_sent: total_packets_sent,
                packets_received: total_packets_received,
                throughput_bps,
                peak_throughput_bps: throughput_bps, // TODO: Track peak throughput
            },
            latency: LatencyMetrics {
                avg_latency_ms,
                min_latency_ms,
                max_latency_ms,
                p95_latency_ms,
                p99_latency_ms,
            },
            errors: ErrorMetrics {
                connection_errors: 0,
                timeout_errors: 0,
                protocol_errors: 0,
                io_errors: 0,
                authentication_errors: 0,
            },
            protocol: ProtocolMetrics {
                x11_requests: 0,
                x11_responses: 0,
                x11_events: 0,
                x11_errors: 0,
                request_rate: 0.0,
            },
        }
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let metrics = self.metrics.read().await;
        let mut output = String::new();

        for (name, metric) in metrics.iter() {
            let labels = metric
                .labels
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                .collect::<Vec<_>>()
                .join(",");

            if labels.is_empty() {
                output.push_str(&format!("{} {}\n", name, metric.value));
            } else {
                output.push_str(&format!("{}{{{}}} {}\n", name, labels, metric.value));
            }
        }

        output
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        let addr: SocketAddr = "127.0.0.1:6000".parse().unwrap();

        collector.record_connection_established(addr).await;
        collector
            .record_data_transfer(addr, 1024, DataDirection::Sent)
            .await;
        collector
            .record_latency(addr, Duration::from_millis(10))
            .await;

        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.connections.total_connections, 1);
        assert_eq!(metrics.bandwidth.bytes_sent, 1024);
        assert!(metrics.latency.avg_latency_ms > 0.0);
    }

    #[tokio::test]
    async fn test_prometheus_export() {
        let collector = MetricsCollector::new();
        let mut labels = HashMap::new();
        labels.insert("test".to_string(), "value".to_string());

        collector.record_counter("test_counter", 42.0, labels).await;

        let output = collector.export_prometheus().await;
        assert!(output.contains("test_counter"));
        assert!(output.contains("42"));
    }
}
