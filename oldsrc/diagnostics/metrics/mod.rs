//! Metrics interface.
//!
//! This module provides comprehensive metrics collection, aggregation, storage, and export
//! capabilities for monitoring the X11 server's performance and behavior.

use crate::types::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub mod aggregation;
pub mod collection;
pub mod export;
pub mod storage;

pub use aggregation::*;
pub use collection::*;
pub use export::*;
pub use storage::*;

/// Central metrics manager that coordinates all metrics functionality.
#[derive(Debug)]
pub struct MetricsManager {
    collector: MetricsCollector,
    aggregator: MetricsAggregator,
    storage: MetricsStorage,
    exporters: Vec<Box<dyn MetricsExporter>>,
    collection_interval: Duration,
    last_collection: Option<Instant>,
}

impl MetricsManager {
    /// Creates a new metrics manager.
    pub fn new() -> Self {
        Self {
            collector: MetricsCollector::new(),
            aggregator: MetricsAggregator::new(),
            storage: MetricsStorage::new(),
            exporters: Vec::new(),
            collection_interval: Duration::from_secs(60),
            last_collection: None,
        }
    }

    /// Adds a metrics exporter.
    pub fn add_exporter(&mut self, exporter: Box<dyn MetricsExporter>) {
        self.exporters.push(exporter);
    }

    /// Collects metrics from all sources.
    pub async fn collect_metrics(&mut self) -> Result<MetricsSnapshot> {
        let snapshot = self.collector.collect_all().await?;

        // Store the raw metrics
        self.storage.store_snapshot(&snapshot).await?;

        // Aggregate metrics
        let aggregated = self.aggregator.aggregate(&snapshot).await?;
        self.storage.store_aggregated(&aggregated).await?;

        // Export metrics
        for exporter in &mut self.exporters {
            if let Err(e) = exporter.export(&snapshot, &aggregated).await {
                log::warn!("Failed to export metrics: {}", e);
            }
        }

        self.last_collection = Some(Instant::now());
        Ok(snapshot)
    }

    /// Checks if metrics should be collected based on interval.
    pub fn should_collect(&self) -> bool {
        match self.last_collection {
            Some(last) => last.elapsed() >= self.collection_interval,
            None => true,
        }
    }

    /// Sets the collection interval.
    pub fn set_collection_interval(&mut self, interval: Duration) {
        self.collection_interval = interval;
    }

    /// Gets current metrics snapshot.
    pub async fn get_current_metrics(&self) -> Result<Option<MetricsSnapshot>> {
        self.storage.get_latest_snapshot().await
    }

    /// Gets aggregated metrics for a time range.
    pub async fn get_aggregated_metrics(
        &self,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<AggregatedMetrics>> {
        self.storage.get_aggregated_range(start, end).await
    }

    /// Registers a custom metric source.
    pub fn register_metric_source(&mut self, source: Box<dyn MetricSource>) {
        self.collector.add_source(source);
    }
}

impl Default for MetricsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of metrics at a specific point in time.
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    /// Timestamp when metrics were collected.
    pub timestamp: Instant,
    /// Counter metrics (monotonically increasing values).
    pub counters: HashMap<String, u64>,
    /// Gauge metrics (point-in-time values).
    pub gauges: HashMap<String, f64>,
    /// Histogram metrics (distribution of values).
    pub histograms: HashMap<String, HistogramData>,
    /// Timer metrics (duration measurements).
    pub timers: HashMap<String, TimerData>,
}

impl MetricsSnapshot {
    /// Creates a new metrics snapshot.
    pub fn new() -> Self {
        Self {
            timestamp: Instant::now(),
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
            timers: HashMap::new(),
        }
    }

    /// Sets the current timestamp to now.
    pub fn set_instant_now(&mut self) {
        self.timestamp = Instant::now();
    }
}

/// Histogram data for distribution metrics.
#[derive(Debug, Clone)]
pub struct HistogramData {
    /// Number of samples.
    pub count: u64,
    /// Sum of all values.
    pub sum: f64,
    /// Minimum value.
    pub min: f64,
    /// Maximum value.
    pub max: f64,
    /// Histogram buckets.
    pub buckets: Vec<HistogramBucket>,
}

/// Histogram bucket.
#[derive(Debug, Clone)]
pub struct HistogramBucket {
    /// Upper bound of the bucket.
    pub upper_bound: f64,
    /// Count of values in this bucket.
    pub count: u64,
}

/// Timer data for duration metrics.
#[derive(Debug, Clone)]
pub struct TimerData {
    /// Number of timing measurements.
    pub count: u64,
    /// Total duration of all measurements.
    pub total_duration: Duration,
    /// Minimum duration.
    pub min_duration: Duration,
    /// Maximum duration.
    pub max_duration: Duration,
    /// Average duration.
    pub avg_duration: Duration,
}

/// Trait for metric sources.
#[async_trait]
pub trait MetricSource: Send + Sync + std::fmt::Debug {
    /// Collects metrics from this source.
    async fn collect(&mut self) -> Result<HashMap<String, MetricValue>>;

    /// Gets the name of this metric source.
    fn name(&self) -> &str;
}

/// Trait for metrics exporters.
#[async_trait]
pub trait MetricsExporter: Send + Sync + std::fmt::Debug {
    /// Exports metrics data.
    async fn export(
        &mut self,
        snapshot: &MetricsSnapshot,
        aggregated: &AggregatedMetrics,
    ) -> Result<()>;

    /// Gets the name of this exporter.
    fn name(&self) -> &str;
}

/// Individual metric value.
#[derive(Debug, Clone)]
pub enum MetricValue {
    /// Counter value.
    Counter(u64),
    /// Gauge value.
    Gauge(f64),
    /// Histogram value.
    Histogram(f64),
    /// Timer value.
    Timer(Duration),
}

/// Metric types for classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    /// Counter metric (monotonically increasing).
    Counter,
    /// Gauge metric (point-in-time value).
    Gauge,
    /// Histogram metric (distribution).
    Histogram,
    /// Timer metric (duration).
    Timer,
}
