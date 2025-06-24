//! Session management for X11 connections
//!
//! Manages X11 client sessions, their state, and resources.

use crate::network::ConnectionId;
use crate::x11::protocol::types::{ColormapId, CursorId, FontId, GContextId, PixmapId, WindowId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// X11 session state
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    /// Session is being initialized
    Initializing,
    /// Session is active
    Active,
    /// Session is suspended
    Suspended,
    /// Session is being terminated
    Terminating,
    /// Session is terminated
    Terminated,
}

/// X11 session information
#[derive(Debug, Clone)]
pub struct Session {
    /// Unique session identifier
    pub id: SessionId,
    /// Associated connection ID
    pub connection_id: ConnectionId,
    /// Session state
    pub state: SessionState,
    /// Client information
    pub client_info: ClientInfo,
    /// X11 protocol information
    pub protocol_info: ProtocolInfo,
    /// Resource allocations
    pub resources: ResourceTracker,
    /// Session statistics
    pub stats: SessionStats,
    /// Session created timestamp
    pub created_at: std::time::SystemTime,
    /// Last activity timestamp
    pub last_activity: std::time::SystemTime,
}

/// Session identifier
pub type SessionId = u64;

/// Client information
#[derive(Debug, Clone)]
pub struct ClientInfo {
    /// Client application name
    pub name: String,
    /// Client class
    pub class: String,
    /// Process ID (if available)
    pub pid: Option<u32>,
    /// Client machine/host
    pub machine: Option<String>,
    /// Authorization information
    pub auth_info: Option<AuthInfo>,
}

/// Authorization information
#[derive(Debug, Clone)]
pub struct AuthInfo {
    /// Authorization protocol name
    pub protocol_name: String,
    /// Authorization data
    pub protocol_data: Vec<u8>,
}

/// X11 protocol information
#[derive(Debug, Clone)]
pub struct ProtocolInfo {
    /// Protocol major version
    pub major_version: u16,
    /// Protocol minor version
    pub minor_version: u16,
    /// Byte order (true for MSB first, false for LSB first)
    pub byte_order: bool,
    /// Resource ID base
    pub resource_id_base: u32,
    /// Resource ID mask
    pub resource_id_mask: u32,
    /// Maximum request length
    pub max_request_length: u16,
    /// Supported extensions
    pub extensions: Vec<String>,
}

/// Resource tracker for X11 resources
#[derive(Debug, Clone)]
pub struct ResourceTracker {
    /// Allocated windows
    pub windows: HashSet<WindowId>,
    /// Allocated pixmaps
    pub pixmaps: HashSet<PixmapId>,
    /// Allocated fonts
    pub fonts: HashSet<FontId>,
    /// Allocated colormaps
    pub colormaps: HashSet<ColormapId>,
    /// Allocated cursors
    pub cursors: HashSet<CursorId>,
    /// Allocated graphics contexts
    pub graphics_contexts: HashSet<GContextId>,
    /// Next available resource ID
    next_resource_id: u32,
    /// Resource ID base
    resource_id_base: u32,
    /// Resource ID mask
    resource_id_mask: u32,
}

impl ResourceTracker {
    /// Create a new resource tracker
    pub fn new(resource_id_base: u32, resource_id_mask: u32) -> Self {
        Self {
            windows: HashSet::new(),
            pixmaps: HashSet::new(),
            fonts: HashSet::new(),
            colormaps: HashSet::new(),
            cursors: HashSet::new(),
            graphics_contexts: HashSet::new(),
            next_resource_id: 1,
            resource_id_base,
            resource_id_mask,
        }
    }

    /// Allocate a new resource ID
    pub fn allocate_resource_id(&mut self) -> u32 {
        let id = self.resource_id_base | (self.next_resource_id & self.resource_id_mask);
        self.next_resource_id += 1;
        id
    }
    /// Add a window resource
    pub fn add_window(&mut self, window: WindowId) {
        self.windows.insert(window);
    }

    /// Remove a window resource
    pub fn remove_window(&mut self, window: WindowId) -> bool {
        self.windows.remove(&window)
    }

    /// Add a pixmap resource
    pub fn add_pixmap(&mut self, pixmap: PixmapId) {
        self.pixmaps.insert(pixmap);
    }

    /// Remove a pixmap resource
    pub fn remove_pixmap(&mut self, pixmap: PixmapId) -> bool {
        self.pixmaps.remove(&pixmap)
    }

    /// Get total resource count
    pub fn total_resources(&self) -> usize {
        self.windows.len()
            + self.pixmaps.len()
            + self.fonts.len()
            + self.colormaps.len()
            + self.cursors.len()
            + self.graphics_contexts.len()
    }

    /// Clear all resources
    pub fn clear(&mut self) {
        self.windows.clear();
        self.pixmaps.clear();
        self.fonts.clear();
        self.colormaps.clear();
        self.cursors.clear();
        self.graphics_contexts.clear();
    }
}

/// Session statistics
#[derive(Debug, Clone)]
pub struct SessionStats {
    /// Total requests processed
    pub requests_processed: u64,
    /// Total responses sent
    pub responses_sent: u64,
    /// Total events sent
    pub events_sent: u64,
    /// Total errors sent
    pub errors_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Average request processing time (microseconds)
    pub avg_request_time_us: u64,
    /// Peak resource count
    pub peak_resource_count: usize,
}

impl Default for SessionStats {
    fn default() -> Self {
        Self {
            requests_processed: 0,
            responses_sent: 0,
            events_sent: 0,
            errors_sent: 0,
            bytes_received: 0,
            bytes_sent: 0,
            avg_request_time_us: 0,
            peak_resource_count: 0,
        }
    }
}

/// Session manager
pub struct SessionManager {
    /// Active sessions
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    /// Connection to session mapping
    connection_sessions: Arc<RwLock<HashMap<ConnectionId, SessionId>>>,
    /// Next session ID
    next_session_id: Arc<std::sync::atomic::AtomicU64>,
    /// Session statistics
    global_stats: Arc<RwLock<SessionManagerStats>>,
}

/// Session manager statistics
#[derive(Debug, Clone)]
pub struct SessionManagerStats {
    /// Total sessions created
    pub sessions_created: u64,
    /// Currently active sessions
    pub active_sessions: u64,
    /// Total sessions terminated
    pub sessions_terminated: u64,
    /// Average session duration (seconds)
    pub avg_session_duration_secs: u64,
}

impl Default for SessionManagerStats {
    fn default() -> Self {
        Self {
            sessions_created: 0,
            active_sessions: 0,
            sessions_terminated: 0,
            avg_session_duration_secs: 0,
        }
    }
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            connection_sessions: Arc::new(RwLock::new(HashMap::new())),
            next_session_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            global_stats: Arc::new(RwLock::new(SessionManagerStats::default())),
        }
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        connection_id: ConnectionId,
        client_info: ClientInfo,
        protocol_info: ProtocolInfo,
    ) -> SessionId {
        let session_id = self
            .next_session_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let now = std::time::SystemTime::now();
        let session = Session {
            id: session_id,
            connection_id,
            state: SessionState::Initializing,
            client_info,
            protocol_info: protocol_info.clone(),
            resources: ResourceTracker::new(
                protocol_info.resource_id_base,
                protocol_info.resource_id_mask,
            ),
            stats: SessionStats::default(),
            created_at: now,
            last_activity: now,
        };
        let client_name = session.client_info.name.clone();

        // Add to sessions map
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session);
        }

        // Add connection mapping
        {
            let mut connection_sessions = self.connection_sessions.write().await;
            connection_sessions.insert(connection_id, session_id);
        }

        // Update statistics
        {
            let mut stats = self.global_stats.write().await;
            stats.sessions_created += 1;
            stats.active_sessions += 1;
        }
        info!(
            "Created new session {} for connection {} (client: {})",
            session_id, connection_id, client_name
        );

        session_id
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: SessionId) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(&session_id).cloned()
    }
    /// Get session by connection ID
    pub async fn get_session_by_connection(&self, connection_id: ConnectionId) -> Option<Session> {
        let connection_sessions = self.connection_sessions.read().await;
        let session_id = connection_sessions.get(&connection_id).copied()?;
        drop(connection_sessions);
        self.get_session(session_id).await
    }

    /// Update session state
    pub async fn update_session_state(
        &self,
        session_id: SessionId,
        new_state: SessionState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            let old_state = session.state.clone();
            session.state = new_state.clone();
            session.last_activity = std::time::SystemTime::now();

            debug!(
                "Session {} state changed: {:?} -> {:?}",
                session_id, old_state, new_state
            );

            Ok(())
        } else {
            Err(format!("Session {} not found", session_id).into())
        }
    }

    /// Terminate session
    pub async fn terminate_session(
        &self,
        session_id: SessionId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (connection_id, duration) = {
            let mut sessions = self.sessions.write().await;
            if let Some(mut session) = sessions.remove(&session_id) {
                session.state = SessionState::Terminated;
                session.resources.clear();

                let duration = std::time::SystemTime::now()
                    .duration_since(session.created_at)
                    .unwrap_or_default();

                info!(
                    "Terminated session {} (duration: {:?})",
                    session_id, duration
                );

                (session.connection_id, duration)
            } else {
                return Err(format!("Session {} not found", session_id).into());
            }
        };

        // Remove connection mapping
        {
            let mut connection_sessions = self.connection_sessions.write().await;
            connection_sessions.remove(&connection_id);
        }

        // Update statistics
        {
            let mut stats = self.global_stats.write().await;
            stats.active_sessions = stats.active_sessions.saturating_sub(1);
            stats.sessions_terminated += 1;

            // Update average session duration
            if stats.sessions_terminated > 0 {
                let total_duration = stats.avg_session_duration_secs
                    * (stats.sessions_terminated - 1)
                    + duration.as_secs();
                stats.avg_session_duration_secs = total_duration / stats.sessions_terminated;
            }
        }

        Ok(())
    }

    /// Get all active sessions
    pub async fn get_active_sessions(&self) -> Vec<SessionId> {
        let sessions = self.sessions.read().await;
        sessions.keys().cloned().collect()
    }

    /// Get session count
    pub async fn get_session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// Update session statistics
    pub async fn update_session_stats(
        &self,
        session_id: SessionId,
        requests_processed: u64,
        bytes_received: u64,
        bytes_sent: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.stats.requests_processed += requests_processed;
            session.stats.bytes_received += bytes_received;
            session.stats.bytes_sent += bytes_sent;
            session.last_activity = std::time::SystemTime::now();

            // Update peak resource count
            let current_resources = session.resources.total_resources();
            if current_resources > session.stats.peak_resource_count {
                session.stats.peak_resource_count = current_resources;
            }

            Ok(())
        } else {
            Err(format!("Session {} not found", session_id).into())
        }
    }

    /// Get global statistics
    pub async fn get_global_stats(&self) -> SessionManagerStats {
        let stats = self.global_stats.read().await;
        stats.clone()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
