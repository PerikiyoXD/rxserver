//! Connection pool implementation
//!
//! Provides connection pooling and reuse functionality.

use crate::network::transport::TransportType;
use crate::network::{ConnectionId, NetworkError};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of pooled connections per transport type
    pub max_pooled_per_transport: usize,
    /// Minimum number of connections to keep alive
    pub min_connections: usize,
    /// Maximum idle time before connection is closed (seconds)
    pub max_idle_time: u64,
    /// Connection validation interval (seconds)
    pub validation_interval: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_pooled_per_transport: 10,
            min_connections: 2,
            max_idle_time: 300,
            validation_interval: 60,
        }
    }
}

/// Pooled connection information
#[derive(Debug, Clone)]
pub struct PooledConnection {
    /// Connection ID
    pub connection_id: ConnectionId,
    /// Transport type
    pub transport_type: TransportType,
    /// Time when connection was added to pool
    pub pooled_at: std::time::SystemTime,
    /// Last activity timestamp
    pub last_activity: std::time::SystemTime,
    /// Number of times this connection has been reused
    pub reuse_count: u32,
}

/// Connection pool
pub struct ConnectionPool {
    /// Pool configuration
    config: PoolConfig,
    /// Pooled connections by transport type
    pools: Arc<RwLock<HashMap<TransportType, VecDeque<PooledConnection>>>>,
    /// Pool statistics
    stats: Arc<RwLock<PoolStatistics>>,
    /// Running state
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStatistics {
    /// Total connections created
    pub connections_created: u64,
    /// Total connections pooled
    pub connections_pooled: u64,
    /// Total connections reused
    pub connections_reused: u64,
    /// Total connections expired
    pub connections_expired: u64,
    /// Current pooled connections by transport
    pub pooled_by_transport: HashMap<TransportType, usize>,
}

impl Default for PoolStatistics {
    fn default() -> Self {
        Self {
            connections_created: 0,
            connections_pooled: 0,
            connections_reused: 0,
            connections_expired: 0,
            pooled_by_transport: HashMap::new(),
        }
    }
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            pools: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(PoolStatistics::default())),
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Start the connection pool
    pub async fn start(&self) -> Result<(), NetworkError> {
        if self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        info!("Starting connection pool");
        self.is_running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        // Start cleanup task
        self.start_cleanup_task().await;

        Ok(())
    }

    /// Stop the connection pool
    pub async fn stop(&self) -> Result<(), NetworkError> {
        if !self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        info!("Stopping connection pool");
        self.is_running
            .store(false, std::sync::atomic::Ordering::SeqCst);

        // Clear all pooled connections
        {
            let mut pools = self.pools.write().await;
            pools.clear();
        }

        info!("Connection pool stopped");
        Ok(())
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self, transport_type: TransportType) -> Option<PooledConnection> {
        let mut pools = self.pools.write().await;

        if let Some(transport_pool) = pools.get_mut(&transport_type) {
            while let Some(mut connection) = transport_pool.pop_front() {
                // Check if connection is still valid (not too old)
                let now = std::time::SystemTime::now();
                let idle_duration = now
                    .duration_since(connection.last_activity)
                    .unwrap_or(std::time::Duration::from_secs(0));

                if idle_duration.as_secs() <= self.config.max_idle_time {
                    // Connection is still valid
                    connection.reuse_count += 1;
                    connection.last_activity = now;

                    debug!(
                        "Reusing pooled connection {} (reuse count: {})",
                        connection.connection_id, connection.reuse_count
                    );

                    // Update statistics
                    {
                        let mut stats = self.stats.write().await;
                        stats.connections_reused += 1;
                    }

                    return Some(connection);
                } else {
                    // Connection expired
                    debug!(
                        "Pooled connection {} expired after {} seconds",
                        connection.connection_id,
                        idle_duration.as_secs()
                    );

                    // Update statistics
                    {
                        let mut stats = self.stats.write().await;
                        stats.connections_expired += 1;
                    }
                    // Continue to next connection in pool
                }
            }
        }

        None
    }

    /// Return a connection to the pool
    pub async fn return_connection(
        &self,
        connection: PooledConnection,
    ) -> Result<(), NetworkError> {
        let transport_type = connection.transport_type;

        {
            let mut pools = self.pools.write().await;
            let transport_pool = pools.entry(transport_type).or_insert_with(VecDeque::new);

            // Check pool size limit
            if transport_pool.len() >= self.config.max_pooled_per_transport {
                debug!(
                    "Pool for {:?} transport is full, not pooling connection {}",
                    transport_type, connection.connection_id
                );
                return Ok(());
            }

            // Add to pool
            transport_pool.push_back(connection.clone());

            debug!(
                "Returned connection {} to pool (transport: {:?})",
                connection.connection_id, transport_type
            );
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.connections_pooled += 1;
            *stats.pooled_by_transport.entry(transport_type).or_insert(0) += 1;
        }

        Ok(())
    }

    /// Remove a connection from the pool
    pub async fn remove_connection(&self, connection_id: ConnectionId) -> bool {
        let mut pools = self.pools.write().await;

        for (transport_type, transport_pool) in pools.iter_mut() {
            if let Some(pos) = transport_pool
                .iter()
                .position(|c| c.connection_id == connection_id)
            {
                transport_pool.remove(pos);
                let transport_type_copy = *transport_type;

                drop(pools);

                debug!(
                    "Removed connection {} from pool (transport: {:?})",
                    connection_id, transport_type_copy
                );

                // Update statistics
                let mut stats = self.stats.write().await;
                let count = stats
                    .pooled_by_transport
                    .entry(transport_type_copy)
                    .or_insert(0);
                *count = count.saturating_sub(1);

                return true;
            }
        }

        false
    }

    /// Get pool statistics
    pub async fn get_statistics(&self) -> PoolStatistics {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Get current pool sizes
    pub async fn get_pool_sizes(&self) -> HashMap<TransportType, usize> {
        let pools = self.pools.read().await;
        pools
            .iter()
            .map(|(transport, pool)| (*transport, pool.len()))
            .collect()
    }

    /// Start cleanup task to remove expired connections
    async fn start_cleanup_task(&self) {
        let pools = self.pools.clone();
        let stats = self.stats.clone();
        let config = self.config.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(config.validation_interval));

            while is_running.load(std::sync::atomic::Ordering::SeqCst) {
                interval.tick().await;

                // Clean up expired connections
                let mut expired_count = 0;
                let now = std::time::SystemTime::now();

                {
                    let mut pools_guard = pools.write().await;

                    for (transport_type, transport_pool) in pools_guard.iter_mut() {
                        let original_len = transport_pool.len();

                        // Remove expired connections
                        transport_pool.retain(|connection| {
                            let idle_duration = now
                                .duration_since(connection.last_activity)
                                .unwrap_or(std::time::Duration::from_secs(0));

                            idle_duration.as_secs() <= config.max_idle_time
                        });

                        let removed = original_len - transport_pool.len();
                        expired_count += removed;

                        if removed > 0 {
                            debug!(
                                "Removed {} expired connections from {:?} pool",
                                removed, transport_type
                            );
                        }
                    }
                }

                // Update statistics
                if expired_count > 0 {
                    let mut stats_guard = stats.write().await;
                    stats_guard.connections_expired += expired_count as u64;

                    // Update pooled counts
                    let pools_guard = pools.read().await;
                    for (transport_type, transport_pool) in pools_guard.iter() {
                        stats_guard
                            .pooled_by_transport
                            .insert(*transport_type, transport_pool.len());
                    }
                }
            }
        });
    }
}
