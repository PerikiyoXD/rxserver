//! Server State Management
//!
//! This module manages the global state of the X server in a thread-safe manner.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Shared server state
pub struct ServerState {
    inner: RwLock<ServerStateInner>,
}

#[derive(Debug)]
struct ServerStateInner {
    /// Display name (e.g., ":0")
    display_name: String,
    /// Server instance ID
    instance_id: Uuid,
    /// When the server was started
    start_time: Instant,
    /// Whether the server is currently running
    running: bool,
    /// Current sequence number for requests
    sequence_number: u16,
    /// Server generation number (incremented on restart)
    generation: u32,
}

impl ServerState {
    /// Create new server state
    pub fn new(display_name: String) -> Self {
        let inner = ServerStateInner {
            display_name,
            instance_id: Uuid::new_v4(),
            start_time: Instant::now(),
            running: false,
            sequence_number: 0,
            generation: 1,
        };

        Self {
            inner: RwLock::new(inner),
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &str {
        // This is safe since display_name is immutable after creation
        // In practice, you'd want to handle this more carefully
        &""  // TODO: Fix this properly
    }

    /// Get server instance ID
    pub async fn instance_id(&self) -> Uuid {
        self.inner.read().await.instance_id
    }

    /// Check if server is running
    pub async fn is_running(&self) -> bool {
        self.inner.read().await.running
    }

    /// Set running state
    pub async fn set_running(&self, running: bool) {
        self.inner.write().await.running = running;
    }

    /// Get server uptime
    pub async fn uptime(&self) -> Duration {
        self.inner.read().await.start_time.elapsed()
    }

    /// Get next sequence number
    pub async fn next_sequence(&self) -> u16 {
        let mut inner = self.inner.write().await;
        inner.sequence_number = inner.sequence_number.wrapping_add(1);
        inner.sequence_number
    }

    /// Get current generation
    pub async fn generation(&self) -> u32 {
        self.inner.read().await.generation
    }

    /// Increment generation (typically on restart)
    pub async fn increment_generation(&self) {
        self.inner.write().await.generation += 1;
    }

    /// Get server start time
    pub async fn start_time(&self) -> Instant {
        self.inner.read().await.start_time
    }
}

impl std::fmt::Debug for ServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerState")
            .field("display_name", &"...")  // We can't easily access the inner state here
            .finish()
    }
}
