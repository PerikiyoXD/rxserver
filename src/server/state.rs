//! Server State Management
//!
//! This module manages the global state of the X server in a thread-safe manner.

use crate::core::{AtomManager, CursorManager, FontManager, PointerManager};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Shared server state
pub struct ServerState {
    /// Display name (e.g., ":0") - immutable after creation
    display_name: String,
    inner: RwLock<ServerStateInner>,
    /// Atom manager for X11 atoms
    atom_manager: AtomManager,
    /// Font manager for X11 fonts
    font_manager: FontManager,
    /// Cursor manager for X11 cursors
    cursor_manager: CursorManager,
    /// Pointer manager for X11 input
    pointer_manager: PointerManager,
}

#[derive(Debug)]
struct ServerStateInner {
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
            instance_id: Uuid::new_v4(),
            start_time: Instant::now(),
            running: false,
            sequence_number: 0,
            generation: 1,
        };
        Self {
            display_name,
            inner: RwLock::new(inner),
            atom_manager: AtomManager::new(),
            font_manager: FontManager::new(),
            cursor_manager: CursorManager::new(),
            pointer_manager: PointerManager::new(),
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &str {
        &self.display_name
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

    /// Get atom manager
    pub fn atom_manager(&self) -> &AtomManager {
        &self.atom_manager
    }

    /// Get font manager
    pub fn font_manager(&self) -> &FontManager {
        &self.font_manager
    }

    /// Get cursor manager
    pub fn cursor_manager(&self) -> &CursorManager {
        &self.cursor_manager
    }

    /// Get pointer manager
    pub fn pointer_manager(&self) -> &PointerManager {
        &self.pointer_manager
    }
}

impl std::fmt::Debug for ServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerState")
            .field("display_name", &self.display_name)
            .field(
                "atom_manager",
                &format!("AtomManager({} atoms)", self.atom_manager.atom_count()),
            )
            .field(
                "font_manager",
                &format!("FontManager({} fonts)", self.font_manager.font_count()),
            )
            .field(
                "cursor_manager",
                &format!(
                    "CursorManager({} cursors)",
                    self.cursor_manager.cursor_count()
                ),
            )
            .field(
                "pointer_manager",
                &format!(
                    "PointerManager(grabbed: {})",
                    self.pointer_manager.is_grabbed()
                ),
            )
            .field("inner", &"<ServerStateInner>")
            .finish()
    }
}
