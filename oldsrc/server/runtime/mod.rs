//! Runtime management interface
//!
//! This module provides runtime management capabilities including task scheduling,
//! execution, thread pool management, and resource management.

pub mod executor;
pub mod resource_manager;
pub mod scheduler;
pub mod thread_pool;

pub use self::executor::*;
pub use self::resource_manager::*;
pub use self::scheduler::*;
pub use self::thread_pool::*;

/// Runtime error types
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Scheduler error: {0}")]
    Scheduler(String),
    #[error("Executor error: {0}")]
    Executor(String),
    #[error("Thread pool error: {0}")]
    ThreadPool(String),
    #[error("Resource management error: {0}")]
    Resource(String),
    #[error("Task execution failed: {0}")]
    TaskExecution(String),
}

/// Result type for runtime operations
pub type RuntimeResult<T> = Result<T, RuntimeError>;
