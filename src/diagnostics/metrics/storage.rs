//! Metrics storage infrastructure.
//!
//! This module handles persistent storage of metrics data for historical analysis and reporting.

use super::{AggregatedMetrics, MetricsSnapshot};
use crate::types::Result;
use std::time::Instant;

/// Stores metrics data with configurable retention policies.
#[derive(Debug)]
pub struct MetricsStorage {
    snapshots: Vec<MetricsSnapshot>,
    aggregated_metrics: Vec<AggregatedMetrics>,
    max_snapshots: usize,
    max_aggregated: usize,
}

impl MetricsStorage {
    /// Creates a new metrics storage.
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            aggregated_metrics: Vec::new(),
            max_snapshots: 1000,
            max_aggregated: 500,
        }
    }

    /// Stores a metrics snapshot.
    pub async fn store_snapshot(&mut self, snapshot: &MetricsSnapshot) -> Result<()> {
        self.snapshots.push(snapshot.clone());

        // Maintain maximum snapshots
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }

        Ok(())
    }

    /// Stores aggregated metrics.
    pub async fn store_aggregated(&mut self, aggregated: &AggregatedMetrics) -> Result<()> {
        self.aggregated_metrics.push(aggregated.clone());

        // Maintain maximum aggregated metrics
        if self.aggregated_metrics.len() > self.max_aggregated {
            self.aggregated_metrics.remove(0);
        }

        Ok(())
    }

    /// Gets the latest metrics snapshot.
    pub async fn get_latest_snapshot(&self) -> Result<Option<MetricsSnapshot>> {
        Ok(self.snapshots.last().cloned())
    }

    /// Gets metrics snapshots in a time range.
    pub async fn get_snapshots_range(
        &self,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<MetricsSnapshot>> {
        let snapshots = self
            .snapshots
            .iter()
            .filter(|s| s.timestamp >= start && s.timestamp <= end)
            .cloned()
            .collect();
        Ok(snapshots)
    }

    /// Gets aggregated metrics in a time range.
    pub async fn get_aggregated_range(
        &self,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<AggregatedMetrics>> {
        let aggregated = self
            .aggregated_metrics
            .iter()
            .filter(|a| a.window_start >= start && a.window_end <= end)
            .cloned()
            .collect();
        Ok(aggregated)
    }

    /// Gets storage statistics.
    pub fn get_storage_stats(&self) -> StorageStats {
        let total_snapshots = self.snapshots.len();
        let total_aggregated = self.aggregated_metrics.len();

        let oldest_snapshot = self.snapshots.first().map(|s| s.timestamp);
        let newest_snapshot = self.snapshots.last().map(|s| s.timestamp);

        let storage_duration = match (oldest_snapshot, newest_snapshot) {
            (Some(oldest), Some(newest)) => Some(newest.duration_since(oldest)),
            _ => None,
        };

        // Estimate memory usage (rough calculation)
        let snapshot_size = size_of::<MetricsSnapshot>()
            + self
                .snapshots
                .iter()
                .map(|s| {
                    s.counters.len() * 32
                        + s.gauges.len() * 32
                        + s.histograms.len() * 64
                        + s.timers.len() * 64
                })
                .sum::<usize>();

        let aggregated_size = size_of::<AggregatedMetrics>() * total_aggregated
            + self
                .aggregated_metrics
                .iter()
                .map(|a| {
                    a.counter_rates.len() * 64
                        + a.gauge_stats.len() * 64
                        + a.histogram_stats.len() * 64
                        + a.timer_stats.len() * 64
                })
                .sum::<usize>();

        StorageStats {
            total_snapshots,
            total_aggregated,
            estimated_memory_bytes: snapshot_size + aggregated_size,
            storage_duration,
            oldest_snapshot,
            newest_snapshot,
        }
    }

    /// Purges old data based on retention policy.
    pub async fn purge_old_data(
        &mut self,
        retention_period: std::time::Duration,
    ) -> Result<PurgeStats> {
        let cutoff_time = Instant::now() - retention_period;

        let initial_snapshots = self.snapshots.len();
        let initial_aggregated = self.aggregated_metrics.len();

        // Remove old snapshots
        self.snapshots.retain(|s| s.timestamp > cutoff_time);

        // Remove old aggregated metrics
        self.aggregated_metrics
            .retain(|a| a.window_end > cutoff_time);

        let purged_snapshots = initial_snapshots - self.snapshots.len();
        let purged_aggregated = initial_aggregated - self.aggregated_metrics.len();

        Ok(PurgeStats {
            purged_snapshots,
            purged_aggregated,
            remaining_snapshots: self.snapshots.len(),
            remaining_aggregated: self.aggregated_metrics.len(),
        })
    }

    /// Sets the maximum number of snapshots to store.
    pub fn set_max_snapshots(&mut self, max_snapshots: usize) {
        self.max_snapshots = max_snapshots;
        if self.snapshots.len() > max_snapshots {
            let excess = self.snapshots.len() - max_snapshots;
            self.snapshots.drain(0..excess);
        }
    }

    /// Sets the maximum number of aggregated metrics to store.
    pub fn set_max_aggregated(&mut self, max_aggregated: usize) {
        self.max_aggregated = max_aggregated;
        if self.aggregated_metrics.len() > max_aggregated {
            let excess = self.aggregated_metrics.len() - max_aggregated;
            self.aggregated_metrics.drain(0..excess);
        }
    }

    /// Exports all data for backup or migration.
    pub async fn export_all_data(&self) -> Result<ExportData> {
        Ok(ExportData {
            snapshots: self.snapshots.clone(),
            aggregated_metrics: self.aggregated_metrics.clone(),
            export_timestamp: Instant::now(),
        })
    }

    /// Imports data from backup or migration.
    pub async fn import_data(&mut self, data: ExportData) -> Result<ImportStats> {
        let initial_snapshots = self.snapshots.len();
        let initial_aggregated = self.aggregated_metrics.len();

        // Merge imported snapshots, maintaining chronological order
        for snapshot in data.snapshots {
            if !self
                .snapshots
                .iter()
                .any(|s| s.timestamp == snapshot.timestamp)
            {
                self.insert_snapshot_ordered(snapshot);
            }
        }

        // Merge imported aggregated metrics
        for aggregated in data.aggregated_metrics {
            if !self
                .aggregated_metrics
                .iter()
                .any(|a| a.window_start == aggregated.window_start)
            {
                self.insert_aggregated_ordered(aggregated);
            }
        }

        // Maintain size limits
        if self.snapshots.len() > self.max_snapshots {
            let excess = self.snapshots.len() - self.max_snapshots;
            self.snapshots.drain(0..excess);
        }

        if self.aggregated_metrics.len() > self.max_aggregated {
            let excess = self.aggregated_metrics.len() - self.max_aggregated;
            self.aggregated_metrics.drain(0..excess);
        }

        Ok(ImportStats {
            imported_snapshots: self.snapshots.len() - initial_snapshots,
            imported_aggregated: self.aggregated_metrics.len() - initial_aggregated,
            total_snapshots: self.snapshots.len(),
            total_aggregated: self.aggregated_metrics.len(),
        })
    }

    fn insert_snapshot_ordered(&mut self, snapshot: MetricsSnapshot) {
        let insert_pos = self
            .snapshots
            .binary_search_by(|s| s.timestamp.cmp(&snapshot.timestamp))
            .unwrap_or_else(|pos| pos);
        self.snapshots.insert(insert_pos, snapshot);
    }

    fn insert_aggregated_ordered(&mut self, aggregated: AggregatedMetrics) {
        let insert_pos = self
            .aggregated_metrics
            .binary_search_by(|a| a.window_start.cmp(&aggregated.window_start))
            .unwrap_or_else(|pos| pos);
        self.aggregated_metrics.insert(insert_pos, aggregated);
    }
}

impl Default for MetricsStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage statistics.
#[derive(Debug, Clone)]
pub struct StorageStats {
    /// Total number of snapshots stored.
    pub total_snapshots: usize,
    /// Total number of aggregated metrics stored.
    pub total_aggregated: usize,
    /// Estimated memory usage in bytes.
    pub estimated_memory_bytes: usize,
    /// Duration of stored data.
    pub storage_duration: Option<std::time::Duration>,
    /// Timestamp of oldest snapshot.
    pub oldest_snapshot: Option<Instant>,
    /// Timestamp of newest snapshot.
    pub newest_snapshot: Option<Instant>,
}

/// Purge operation statistics.
#[derive(Debug, Clone)]
pub struct PurgeStats {
    /// Number of snapshots purged.
    pub purged_snapshots: usize,
    /// Number of aggregated metrics purged.
    pub purged_aggregated: usize,
    /// Remaining snapshots after purge.
    pub remaining_snapshots: usize,
    /// Remaining aggregated metrics after purge.
    pub remaining_aggregated: usize,
}

/// Export data structure.
#[derive(Debug, Clone)]
pub struct ExportData {
    /// Exported snapshots.
    pub snapshots: Vec<MetricsSnapshot>,
    /// Exported aggregated metrics.
    pub aggregated_metrics: Vec<AggregatedMetrics>,
    /// When the export was created.
    pub export_timestamp: Instant,
}

/// Import operation statistics.
#[derive(Debug, Clone)]
pub struct ImportStats {
    /// Number of snapshots imported.
    pub imported_snapshots: usize,
    /// Number of aggregated metrics imported.
    pub imported_aggregated: usize,
    /// Total snapshots after import.
    pub total_snapshots: usize,
    /// Total aggregated metrics after import.
    pub total_aggregated: usize,
}
