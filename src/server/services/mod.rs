//! Server services interface
//!
//! This module provides the interface for managing server services including
//! service management, registry, discovery, and health monitoring.

pub mod discovery;
pub mod health;
pub mod manager;
pub mod registry;

pub use self::discovery::*;
pub use self::health::*;
pub use self::manager::*;
pub use self::registry::*;

/// Service error types
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Service not found: {0}")]
    NotFound(String),
    #[error("Service registration failed: {0}")]
    Registration(String),
    #[error("Service discovery failed: {0}")]
    Discovery(String),
    #[error("Service health check failed: {0}")]
    HealthCheck(String),
    #[error("Service management error: {0}")]
    Management(String),
}

/// Result type for service operations
pub type ServiceResult<T> = Result<T, ServiceError>;
