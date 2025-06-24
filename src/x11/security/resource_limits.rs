//! Per-client resource limits for X11 server
//!
//! This module implements resource limiting mechanisms to prevent clients
//! from consuming excessive server resources.

use crate::x11::protocol::types::ClientId;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Resource limit manager
#[derive(Debug)]
pub struct LimitManager {
    /// Per-client resource limits
    client_limits: HashMap<ClientId, ResourceLimits>,
    /// Global default limits
    default_limits: ResourceLimits,
    /// Current resource usage tracking
    usage_tracking: HashMap<ClientId, ResourceUsage>,
    /// Rate limiting for requests
    rate_limits: HashMap<ClientId, RateLimit>,
}

/// Resource limits for a client
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum number of windows
    pub max_windows: Option<u32>,
    /// Maximum number of pixmaps
    pub max_pixmaps: Option<u32>,
    /// Maximum number of graphics contexts
    pub max_graphics_contexts: Option<u32>,
    /// Maximum number of fonts
    pub max_fonts: Option<u32>,
    /// Maximum total memory usage (bytes)
    pub max_memory: Option<u64>,
    /// Maximum number of concurrent grabs
    pub max_grabs: Option<u32>,
    /// Maximum request rate (requests per second)
    pub max_request_rate: Option<u32>,
    /// Maximum connection idle time
    pub max_idle_time: Option<Duration>,
}

/// Current resource usage for a client
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// Current number of windows
    pub windows: u32,
    /// Current number of pixmaps
    pub pixmaps: u32,
    /// Current number of graphics contexts
    pub graphics_contexts: u32,
    /// Current number of fonts
    pub fonts: u32,
    /// Current memory usage (bytes)
    pub memory: u64,
    /// Current number of grabs
    pub grabs: u32,
    /// Last activity timestamp
    pub last_activity: Instant,
}

/// Rate limiting information
#[derive(Debug)]
struct RateLimit {
    /// Request count in current window
    request_count: u32,
    /// Window start time
    window_start: Instant,
    /// Window duration
    window_duration: Duration,
    /// Maximum requests per window
    max_requests: u32,
}

/// Resource limit violation types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LimitViolation {
    /// Too many windows
    TooManyWindows { current: u32, limit: u32 },
    /// Too many pixmaps
    TooManyPixmaps { current: u32, limit: u32 },
    /// Too many graphics contexts
    TooManyGraphicsContexts { current: u32, limit: u32 },
    /// Too many fonts
    TooManyFonts { current: u32, limit: u32 },
    /// Memory limit exceeded
    MemoryLimitExceeded { current: u64, limit: u64 },
    /// Too many concurrent grabs
    TooManyGrabs { current: u32, limit: u32 },
    /// Request rate limit exceeded
    RateLimitExceeded { rate: u32, limit: u32 },
    /// Connection idle timeout
    IdleTimeout {
        idle_time: Duration,
        limit: Duration,
    },
}

impl LimitManager {
    /// Create a new limit manager with default limits
    pub fn new() -> Self {
        Self {
            client_limits: HashMap::new(),
            default_limits: ResourceLimits::default(),
            usage_tracking: HashMap::new(),
            rate_limits: HashMap::new(),
        }
    }

    /// Set resource limits for a specific client
    pub fn set_client_limits(&mut self, client_id: ClientId, limits: ResourceLimits) {
        self.client_limits.insert(client_id, limits);
    }

    /// Get resource limits for a client
    pub fn get_client_limits(&self, client_id: ClientId) -> &ResourceLimits {
        self.client_limits
            .get(&client_id)
            .unwrap_or(&self.default_limits)
    }

    /// Set default resource limits
    pub fn set_default_limits(&mut self, limits: ResourceLimits) {
        self.default_limits = limits;
    }

    /// Initialize tracking for a new client
    pub fn add_client(&mut self, client_id: ClientId) {
        self.usage_tracking.insert(
            client_id,
            ResourceUsage {
                windows: 0,
                pixmaps: 0,
                graphics_contexts: 0,
                fonts: 0,
                memory: 0,
                grabs: 0,
                last_activity: Instant::now(),
            },
        );

        // Initialize rate limiting
        if let Some(rate_limit) = self.get_client_limits(client_id).max_request_rate {
            self.rate_limits.insert(
                client_id,
                RateLimit {
                    request_count: 0,
                    window_start: Instant::now(),
                    window_duration: Duration::from_secs(1),
                    max_requests: rate_limit,
                },
            );
        }
    }

    /// Remove tracking for a client
    pub fn remove_client(&mut self, client_id: ClientId) {
        self.usage_tracking.remove(&client_id);
        self.rate_limits.remove(&client_id);
        self.client_limits.remove(&client_id);
    }

    /// Update resource usage for a client
    pub fn update_usage<F>(
        &mut self,
        client_id: ClientId,
        update_fn: F,
    ) -> Result<(), LimitViolation>
    where
        F: FnOnce(&mut ResourceUsage),
    {
        // Create a test usage to validate the update
        let mut test_usage = if let Some(usage) = self.usage_tracking.get(&client_id) {
            usage.clone()
        } else {
            return Ok(()); // No usage tracking for this client
        };

        // Apply the update to the test copy
        update_fn(&mut test_usage);

        // Check if the update would violate limits
        self.check_limits(client_id, &test_usage)?;

        // If we get here, the update is valid - apply it
        if let Some(usage) = self.usage_tracking.get_mut(&client_id) {
            *usage = test_usage;
            usage.last_activity = Instant::now();
        }

        Ok(())
    }

    /// Check if current usage violates any limits
    fn check_limits(
        &self,
        client_id: ClientId,
        usage: &ResourceUsage,
    ) -> Result<(), LimitViolation> {
        let limits = self.get_client_limits(client_id);

        // Check window limit
        if let Some(limit) = limits.max_windows {
            if usage.windows > limit {
                return Err(LimitViolation::TooManyWindows {
                    current: usage.windows,
                    limit,
                });
            }
        }

        // Check pixmap limit
        if let Some(limit) = limits.max_pixmaps {
            if usage.pixmaps > limit {
                return Err(LimitViolation::TooManyPixmaps {
                    current: usage.pixmaps,
                    limit,
                });
            }
        }

        // Check graphics context limit
        if let Some(limit) = limits.max_graphics_contexts {
            if usage.graphics_contexts > limit {
                return Err(LimitViolation::TooManyGraphicsContexts {
                    current: usage.graphics_contexts,
                    limit,
                });
            }
        }

        // Check font limit
        if let Some(limit) = limits.max_fonts {
            if usage.fonts > limit {
                return Err(LimitViolation::TooManyFonts {
                    current: usage.fonts,
                    limit,
                });
            }
        }

        // Check memory limit
        if let Some(limit) = limits.max_memory {
            if usage.memory > limit {
                return Err(LimitViolation::MemoryLimitExceeded {
                    current: usage.memory,
                    limit,
                });
            }
        }

        // Check grab limit
        if let Some(limit) = limits.max_grabs {
            if usage.grabs > limit {
                return Err(LimitViolation::TooManyGrabs {
                    current: usage.grabs,
                    limit,
                });
            }
        }

        Ok(())
    }

    /// Check rate limit for a request
    pub fn check_rate_limit(&mut self, client_id: ClientId) -> Result<(), LimitViolation> {
        if let Some(rate_limit) = self.rate_limits.get_mut(&client_id) {
            let now = Instant::now();

            // Reset window if needed
            if now.duration_since(rate_limit.window_start) >= rate_limit.window_duration {
                rate_limit.request_count = 0;
                rate_limit.window_start = now;
            }

            // Check if adding this request would exceed the limit
            if rate_limit.request_count >= rate_limit.max_requests {
                return Err(LimitViolation::RateLimitExceeded {
                    rate: rate_limit.request_count,
                    limit: rate_limit.max_requests,
                });
            }

            // Count this request
            rate_limit.request_count += 1;
        }

        // Update last activity
        if let Some(usage) = self.usage_tracking.get_mut(&client_id) {
            usage.last_activity = Instant::now();
        }

        Ok(())
    }

    /// Check for idle timeout violations
    pub fn check_idle_timeouts(&self) -> Vec<(ClientId, Duration)> {
        let mut violations = Vec::new();
        let now = Instant::now();

        for (&client_id, usage) in &self.usage_tracking {
            let limits = self.get_client_limits(client_id);
            if let Some(max_idle) = limits.max_idle_time {
                let idle_time = now.duration_since(usage.last_activity);
                if idle_time > max_idle {
                    violations.push((client_id, idle_time));
                }
            }
        }

        violations
    }

    /// Get current usage for a client
    pub fn get_usage(&self, client_id: ClientId) -> Option<&ResourceUsage> {
        self.usage_tracking.get(&client_id)
    }

    /// Get statistics for all clients
    pub fn get_stats(&self) -> LimitStats {
        let total_clients = self.usage_tracking.len();
        let total_windows: u32 = self.usage_tracking.values().map(|u| u.windows).sum();
        let total_pixmaps: u32 = self.usage_tracking.values().map(|u| u.pixmaps).sum();
        let total_memory: u64 = self.usage_tracking.values().map(|u| u.memory).sum();

        LimitStats {
            total_clients,
            total_windows,
            total_pixmaps,
            total_memory,
            clients_with_custom_limits: self.client_limits.len(),
        }
    }
}

/// Resource limit statistics
#[derive(Debug, Clone)]
pub struct LimitStats {
    /// Total number of tracked clients
    pub total_clients: usize,
    /// Total windows across all clients
    pub total_windows: u32,
    /// Total pixmaps across all clients
    pub total_pixmaps: u32,
    /// Total memory usage across all clients
    pub total_memory: u64,
    /// Number of clients with custom limits
    pub clients_with_custom_limits: usize,
}

impl ResourceLimits {
    /// Create unlimited resource limits
    pub fn unlimited() -> Self {
        Self {
            max_windows: None,
            max_pixmaps: None,
            max_graphics_contexts: None,
            max_fonts: None,
            max_memory: None,
            max_grabs: None,
            max_request_rate: None,
            max_idle_time: None,
        }
    }

    /// Create conservative resource limits
    pub fn conservative() -> Self {
        Self {
            max_windows: Some(100),
            max_pixmaps: Some(50),
            max_graphics_contexts: Some(20),
            max_fonts: Some(10),
            max_memory: Some(100 * 1024 * 1024), // 100 MB
            max_grabs: Some(1),
            max_request_rate: Some(1000), // 1000 requests per second
            max_idle_time: Some(Duration::from_secs(3600)), // 1 hour
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::conservative()
    }
}

impl Default for LimitManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for LimitViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LimitViolation::TooManyWindows { current, limit } => {
                write!(f, "Too many windows: {} > {}", current, limit)
            }
            LimitViolation::TooManyPixmaps { current, limit } => {
                write!(f, "Too many pixmaps: {} > {}", current, limit)
            }
            LimitViolation::TooManyGraphicsContexts { current, limit } => {
                write!(f, "Too many graphics contexts: {} > {}", current, limit)
            }
            LimitViolation::TooManyFonts { current, limit } => {
                write!(f, "Too many fonts: {} > {}", current, limit)
            }
            LimitViolation::MemoryLimitExceeded { current, limit } => {
                write!(f, "Memory limit exceeded: {} > {}", current, limit)
            }
            LimitViolation::TooManyGrabs { current, limit } => {
                write!(f, "Too many grabs: {} > {}", current, limit)
            }
            LimitViolation::RateLimitExceeded { rate, limit } => {
                write!(f, "Rate limit exceeded: {} > {}", rate, limit)
            }
            LimitViolation::IdleTimeout { idle_time, limit } => {
                write!(f, "Idle timeout: {:?} > {:?}", idle_time, limit)
            }
        }
    }
}

impl std::error::Error for LimitViolation {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_limits() {
        let mut limit_mgr = LimitManager::new();
        let client_id = 1;

        // Set conservative limits
        limit_mgr.set_client_limits(client_id, ResourceLimits::conservative());
        limit_mgr.add_client(client_id);

        // Should succeed within limits
        let result = limit_mgr.update_usage(client_id, |usage| {
            usage.windows = 50;
        });
        assert!(result.is_ok());

        // Should fail when exceeding limits
        let result = limit_mgr.update_usage(client_id, |usage| {
            usage.windows = 150;
        });
        assert!(matches!(result, Err(LimitViolation::TooManyWindows { .. })));
    }

    #[test]
    fn test_rate_limiting() {
        let mut limit_mgr = LimitManager::new();
        let client_id = 1;

        let mut limits = ResourceLimits::default();
        limits.max_request_rate = Some(2); // Very low limit for testing

        limit_mgr.set_client_limits(client_id, limits);
        limit_mgr.add_client(client_id);

        // First two requests should succeed
        assert!(limit_mgr.check_rate_limit(client_id).is_ok());
        assert!(limit_mgr.check_rate_limit(client_id).is_ok());

        // Third request should fail
        assert!(matches!(
            limit_mgr.check_rate_limit(client_id),
            Err(LimitViolation::RateLimitExceeded { .. })
        ));
    }

    #[test]
    fn test_unlimited_limits() {
        let mut limit_mgr = LimitManager::new();
        let client_id = 1;

        limit_mgr.set_client_limits(client_id, ResourceLimits::unlimited());
        limit_mgr.add_client(client_id);

        // Should succeed with very high values
        let result = limit_mgr.update_usage(client_id, |usage| {
            usage.windows = 10000;
            usage.memory = 1024 * 1024 * 1024; // 1 GB
        });
        assert!(result.is_ok());
    }
}
