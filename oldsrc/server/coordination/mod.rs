//! Component coordination interface
//!
//! This module provides the coordination interface for managing component interactions,
//! dependencies, communication, and synchronization across the server.

pub mod communication;
pub mod dependencies;
pub mod orchestrator;
pub mod synchronization;

pub use self::communication::*;
pub use self::dependencies::*;
pub use self::orchestrator::*;
pub use self::synchronization::*;

/// Component coordination error types
#[derive(Debug, thiserror::Error)]
pub enum CoordinationError {
    #[error("Dependency resolution failed: {0}")]
    DependencyResolution(String),
    #[error("Communication failure: {0}")]
    Communication(String),
    #[error("Synchronization error: {0}")]
    Synchronization(String),
    #[error("Orchestration error: {0}")]
    Orchestration(String),
}

/// Result type for coordination operations
pub type CoordinationResult<T> = Result<T, CoordinationError>;
