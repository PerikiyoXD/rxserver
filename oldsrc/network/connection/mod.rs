//! Connection management module
//!
//! Handles connection lifecycle, pooling, and management for the X11 server.

pub mod auth;
pub mod manager;
pub mod pool;
pub mod session;

// Re-export commonly used items
pub use auth::{AuthenticationError, AuthenticationManager, AuthenticationResult};
pub use manager::{ConnectionManager, ConnectionManagerConfig};
pub use pool::{ConnectionPool, PoolConfig};
pub use session::{Session, SessionManager, SessionState};
