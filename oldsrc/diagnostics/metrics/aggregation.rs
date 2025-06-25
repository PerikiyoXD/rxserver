//! Metrics aggregation functionality.
//!
//! This module handles aggregation of metrics over time windows and statistical calculations.

use super::MetricsSnapshot;
use crate::types::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Aggregates metrics over time windows.
#[derive(Debug)]
pub struct MetricsAggregator {
    window_size: Duration,
    snapshots: Vec<MetricsSnapshot>,
    max_snapshots: usize,
}

impl MetricsAggregator {
    /// Creates a new metrics aggregator.
    pub fn new() -> Self {
        Self {
            window_size: Duration::from_secs(300), // 5 minutes
            snapshots: Vec::new(),
            max_snapshots: 100,
        }
    }

    /// Aggregates a metrics snapshot.
    pub async fn aggregate(&mut self, snapshot: &MetricsSnapshot) -> Result<AggregatedMetrics> {
        // Add snapshot to history
        self.snapshots.push(snapshot.clone());

        // Maintain max snapshots
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }

        // Calculate aggregations
        let window_start = Instant::now() - self.window_size;
        let recent_snapshots: Vec<_> = self
            .snapshots
            .iter()
            .filter(|s| s.timestamp >= window_start)
            .collect();

        let aggregated = self.calculate_aggregations(&recent_snapshots)?;
        Ok(aggregated)
    }

    /// Sets the aggregation window size.
    pub fn set_window_size(&mut self, window_size: Duration) {
        self.window_size = window_size;
    }

    /// Sets the maximum number of snapshots to keep.
    pub fn set_max_snapshots(&mut self, max_snapshots: usize) {
        self.max_snapshots = max_snapshots;
        if self.snapshots.len() > max_snapshots {
            self.snapshots.truncate(max_snapshots);
        }
    }

    fn calculate_aggregations(&self, snapshots: &[&MetricsSnapshot]) -> Result<AggregatedMetrics> {
        let mut counter_rates = HashMap::new();
        let mut gauge_stats = HashMap::new();
        let mut histogram_stats = HashMap::new();
        let mut timer_stats = HashMap::new();

        if snapshots.is_empty() {
            return Ok(AggregatedMetrics {
                window_start: Instant::now() - self.window_size,
                window_end: Instant::now(),
                counter_rates,
                gauge_stats,
                histogram_stats,
                timer_stats,
            });
        }

        let window_start = snapshots[0].timestamp;
        let window_end = snapshots[snapshots.len() - 1].timestamp;
        let window_duration = window_end.duration_since(window_start);

        // Aggregate counters (calculate rates)
        for metric_name in self.get_all_counter_names(snapshots) {
            let values: Vec<_> = snapshots
                .iter()
                .filter_map(|s| s.counters.get(&metric_name).copied())
                .collect();

            if let (Some(&first), Some(&last)) = (values.first(), values.last()) {
                let rate = if window_duration.as_secs() > 0 {
                    (last - first) as f64 / window_duration.as_secs_f64()
                } else {
                    0.0
                };
                counter_rates.insert(
                    metric_name,
                    CounterRate {
                        rate,
                        total_change: last - first,
                        start_value: first,
                        end_value: last,
                    },
                );
            }
        }

        // Aggregate gauges (calculate statistics)
        for metric_name in self.get_all_gauge_names(snapshots) {
            let values: Vec<_> = snapshots
                .iter()
                .filter_map(|s| s.gauges.get(&metric_name).copied())
                .collect();

            if !values.is_empty() {
                let sum = values.iter().sum::<f64>();
                let mean = sum / values.len() as f64;
                let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

                // Calculate standard deviation
                let variance =
                    values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
                let std_dev = variance.sqrt();

                gauge_stats.insert(
                    metric_name,
                    GaugeStats {
                        mean,
                        min,
                        max,
                        std_dev,
                        count: values.len(),
                    },
                );
            }
        }

        // Aggregate histograms
        for metric_name in self.get_all_histogram_names(snapshots) {
            let histograms: Vec<_> = snapshots
                .iter()
                .filter_map(|s| s.histograms.get(&metric_name))
                .collect();

            if !histograms.is_empty() {
                let total_count = histograms.iter().map(|h| h.count).sum();
                let total_sum = histograms.iter().map(|h| h.sum).sum();
                let min = histograms
                    .iter()
                    .map(|h| h.min)
                    .fold(f64::INFINITY, f64::min);
                let max = histograms
                    .iter()
                    .map(|h| h.max)
                    .fold(f64::NEG_INFINITY, f64::max);
                let mean = if total_count > 0 {
                    total_sum / total_count as f64
                } else {
                    0.0
                };

                histogram_stats.insert(
                    metric_name,
                    HistogramStats {
                        mean,
                        min,
                        max,
                        total_count,
                        total_sum,
                    },
                );
            }
        }

        // Aggregate timers
        for metric_name in self.get_all_timer_names(snapshots) {
            let timers: Vec<_> = snapshots
                .iter()
                .filter_map(|s| s.timers.get(&metric_name))
                .collect();

            if !timers.is_empty() {
                let total_count = timers.iter().map(|t| t.count).sum();
                let total_duration = timers.iter().map(|t| t.total_duration).sum();
                let min_duration = timers
                    .iter()
                    .map(|t| t.min_duration)
                    .min()
                    .unwrap_or(Duration::ZERO);
                let max_duration = timers
                    .iter()
                    .map(|t| t.max_duration)
                    .max()
                    .unwrap_or(Duration::ZERO);
                let avg_duration = if total_count > 0 {
                    total_duration / total_count as u32
                } else {
                    Duration::ZERO
                };

                timer_stats.insert(
                    metric_name,
                    TimerStats {
                        avg_duration,
                        min_duration,
                        max_duration,
                        total_count,
                        total_duration,
                    },
                );
            }
        }

        Ok(AggregatedMetrics {
            window_start,
            window_end,
            counter_rates,
            gauge_stats,
            histogram_stats,
            timer_stats,
        })
    }

    fn get_all_counter_names(
        &self,
        snapshots: &[&MetricsSnapshot],
    ) -> std::collections::HashSet<String> {
        snapshots
            .iter()
            .flat_map(|s| s.counters.keys())
            .cloned()
            .collect()
    }

    fn get_all_gauge_names(
        &self,
        snapshots: &[&MetricsSnapshot],
    ) -> std::collections::HashSet<String> {
        snapshots
            .iter()
            .flat_map(|s| s.gauges.keys())
            .cloned()
            .collect()
    }

    fn get_all_histogram_names(
        &self,
        snapshots: &[&MetricsSnapshot],
    ) -> std::collections::HashSet<String> {
        snapshots
            .iter()
            .flat_map(|s| s.histograms.keys())
            .cloned()
            .collect()
    }

    fn get_all_timer_names(
        &self,
        snapshots: &[&MetricsSnapshot],
    ) -> std::collections::HashSet<String> {
        snapshots
            .iter()
            .flat_map(|s| s.timers.keys())
            .cloned()
            .collect()
    }
}

impl Default for MetricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregated metrics over a time window.
#[derive(Debug, Clone)]
pub struct AggregatedMetrics {
    /// Start of the aggregation window.
    pub window_start: Instant,
    /// End of the aggregation window.
    pub window_end: Instant,
    /// Counter rate statistics.
    pub counter_rates: HashMap<String, CounterRate>,
    /// Gauge statistics.
    pub gauge_stats: HashMap<String, GaugeStats>,
    /// Histogram statistics.
    pub histogram_stats: HashMap<String, HistogramStats>,
    /// Timer statistics.
    pub timer_stats: HashMap<String, TimerStats>,
}

/// Counter rate information.
#[derive(Debug, Clone)]
pub struct CounterRate {
    /// Rate per second.
    pub rate: f64,
    /// Total change over the window.
    pub total_change: u64,
    /// Value at start of window.
    pub start_value: u64,
    /// Value at end of window.
    pub end_value: u64,
}

/// Gauge statistics.
#[derive(Debug, Clone)]
pub struct GaugeStats {
    /// Mean value.
    pub mean: f64,
    /// Minimum value.
    pub min: f64,
    /// Maximum value.
    pub max: f64,
    /// Standard deviation.
    pub std_dev: f64,
    /// Number of data points.
    pub count: usize,
}

/// Histogram statistics.
#[derive(Debug, Clone)]
pub struct HistogramStats {
    /// Mean value.
    pub mean: f64,
    /// Minimum value.
    pub min: f64,
    /// Maximum value.
    pub max: f64,
    /// Total count.
    pub total_count: u64,
    /// Total sum.
    pub total_sum: f64,
}

/// Timer statistics.
#[derive(Debug, Clone)]
pub struct TimerStats {
    /// Average duration.
    pub avg_duration: Duration,
    /// Minimum duration.
    pub min_duration: Duration,
    /// Maximum duration.
    pub max_duration: Duration,
    /// Total count.
    pub total_count: u64,
    /// Total duration.
    pub total_duration: Duration,
}
