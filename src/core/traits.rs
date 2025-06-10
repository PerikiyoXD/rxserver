//! Core traits for X11 server components
//!
//! This module defines fundamental traits that establish patterns for
//! the X server architecture, promoting consistent interfaces and
//! making the code more maintainable than the original X server.

use crate::{protocol::Response, Result};
use async_trait::async_trait;
use std::fmt::Debug;

/// Trait for X11 protocol message handlers
#[async_trait]
pub trait MessageHandler: Send + Sync + Debug {
    /// Handle an incoming message from a client
    async fn handle_message(
        &self,
        client_id: crate::core::ClientId,
        data: &[u8],
    ) -> Result<Option<Response>>;
}

/// Trait for X11 event dispatchers
#[async_trait]
pub trait EventDispatcher: Send + Sync + Debug {
    /// Dispatch an event to interested clients
    async fn dispatch_event(&self, event: &crate::protocol::Event) -> Result<()>;
}

/// Trait for X11 resource managers
pub trait ResourceManager<T>: Send + Sync + Debug {
    type Id: crate::core::ResourceId;

    /// Create a new resource
    fn create_resource(&mut self, resource: T) -> Result<Self::Id>;

    /// Get a resource by ID
    fn get_resource(&self, id: Self::Id) -> Option<&T>;

    /// Get a mutable reference to a resource by ID
    fn get_resource_mut(&mut self, id: Self::Id) -> Option<&mut T>;

    /// Remove a resource by ID
    fn remove_resource(&mut self, id: Self::Id) -> Option<T>;

    /// Check if a resource exists
    fn has_resource(&self, id: Self::Id) -> bool;

    /// Get all resource IDs
    fn all_ids(&self) -> Vec<Self::Id>;
}

/// Trait for components that can be cleanly shutdown
#[async_trait]
pub trait Shutdown: Send + Sync {
    /// Shutdown the component gracefully
    async fn shutdown(&mut self) -> Result<()>;
}

/// Trait for validating X11 protocol requests
pub trait RequestValidator: Send + Sync + Debug {
    /// Validate that a client can perform a request
    fn validate_request(
        &self,
        client_id: crate::core::ClientId,
        request: &crate::protocol::Request,
    ) -> Result<()>;
}

/// Trait for components that track server statistics
pub trait StatsProvider: Send + Sync + Debug {
    /// Get current statistics as key-value pairs
    fn get_stats(&self) -> std::collections::HashMap<String, String>;
}

/// Trait for configurable components
pub trait Configurable: Send + Sync + Debug {
    type Config: Send + Sync + Debug;

    /// Update the component's configuration
    fn update_config(&mut self, config: Self::Config) -> Result<()>;

    /// Get the current configuration
    fn get_config(&self) -> &Self::Config;
}
