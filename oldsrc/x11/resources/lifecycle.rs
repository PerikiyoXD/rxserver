//! Resource lifecycle management
//!
//! This module handles the lifecycle of resources from creation to destruction,
//! including state transitions and validation.

use crate::x11::protocol::types::{ClientId, XId};
use std::collections::HashMap;

/// Errors that can occur during lifecycle operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleError {
    /// Invalid state transition
    InvalidTransition {
        from: ResourceState,
        to: ResourceState,
    },
    /// Resource is in the wrong state for the operation
    WrongState {
        current: ResourceState,
        expected: ResourceState,
    },
    /// Resource lifecycle has already ended
    LifecycleEnded(XId),
    /// Resource initialization failed
    InitializationFailed(String),
    /// Resource finalization failed
    FinalizationFailed(String),
}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecycleError::InvalidTransition { from, to } => {
                write!(f, "Invalid state transition from {:?} to {:?}", from, to)
            }
            LifecycleError::WrongState { current, expected } => {
                write!(
                    f,
                    "Wrong state: expected {:?}, found {:?}",
                    expected, current
                )
            }
            LifecycleError::LifecycleEnded(xid) => {
                write!(f, "Resource {} lifecycle has ended", xid)
            }
            LifecycleError::InitializationFailed(msg) => {
                write!(f, "Resource initialization failed: {}", msg)
            }
            LifecycleError::FinalizationFailed(msg) => {
                write!(f, "Resource finalization failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for LifecycleError {}

/// States in the resource lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceState {
    /// Resource is being created
    Creating,
    /// Resource is active and usable
    Active,
    /// Resource is being modified
    Modifying,
    /// Resource is suspended (temporarily unusable)
    Suspended,
    /// Resource is being destroyed
    Destroying,
    /// Resource has been destroyed
    Destroyed,
}

impl ResourceState {
    /// Check if a transition to another state is valid
    pub fn can_transition_to(&self, to: ResourceState) -> bool {
        match (self, to) {
            // Creating can go to Active or Destroying (if creation fails)
            (ResourceState::Creating, ResourceState::Active) => true,
            (ResourceState::Creating, ResourceState::Destroying) => true,

            // Active can go to Modifying, Suspended, or Destroying
            (ResourceState::Active, ResourceState::Modifying) => true,
            (ResourceState::Active, ResourceState::Suspended) => true,
            (ResourceState::Active, ResourceState::Destroying) => true,

            // Modifying can go back to Active or to Destroying
            (ResourceState::Modifying, ResourceState::Active) => true,
            (ResourceState::Modifying, ResourceState::Destroying) => true,

            // Suspended can go to Active or Destroying
            (ResourceState::Suspended, ResourceState::Active) => true,
            (ResourceState::Suspended, ResourceState::Destroying) => true,

            // Destroying can only go to Destroyed
            (ResourceState::Destroying, ResourceState::Destroyed) => true,

            // Destroyed is final
            (ResourceState::Destroyed, _) => false,

            // All other transitions are invalid
            _ => false,
        }
    }

    /// Check if the resource is usable in this state
    pub fn is_usable(&self) -> bool {
        matches!(self, ResourceState::Active | ResourceState::Modifying)
    }

    /// Check if the resource is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, ResourceState::Destroyed)
    }
}

/// Lifecycle information for a resource
#[derive(Debug, Clone)]
pub struct ResourceLifecycle {
    /// Current state of the resource
    state: ResourceState,
    /// XId of the resource
    xid: XId,
    /// Client that owns the resource
    owner: ClientId,
    /// When the resource was created
    created_at: std::time::Instant,
    /// State transition history
    transitions: Vec<(ResourceState, std::time::Instant)>,
    /// Error that occurred during lifecycle (if any)
    error: Option<String>,
}

impl ResourceLifecycle {
    /// Create a new lifecycle tracker for a resource
    pub fn new(xid: XId, owner: ClientId) -> Self {
        let now = std::time::Instant::now();
        Self {
            state: ResourceState::Creating,
            xid,
            owner,
            created_at: now,
            transitions: vec![(ResourceState::Creating, now)],
            error: None,
        }
    }

    /// Get the current state
    pub fn state(&self) -> ResourceState {
        self.state
    }

    /// Get the XId
    pub fn xid(&self) -> XId {
        self.xid
    }

    /// Get the owner
    pub fn owner(&self) -> ClientId {
        self.owner
    }

    /// Get creation time
    pub fn created_at(&self) -> std::time::Instant {
        self.created_at
    }

    /// Get age of the resource
    pub fn age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Get time spent in current state
    pub fn time_in_current_state(&self) -> std::time::Duration {
        self.transitions
            .last()
            .map(|(_, time)| time.elapsed())
            .unwrap_or_default()
    }

    /// Transition to a new state
    pub fn transition_to(&mut self, new_state: ResourceState) -> Result<(), LifecycleError> {
        if self.state.is_terminal() {
            return Err(LifecycleError::LifecycleEnded(self.xid));
        }

        if !self.state.can_transition_to(new_state) {
            return Err(LifecycleError::InvalidTransition {
                from: self.state,
                to: new_state,
            });
        }

        self.state = new_state;
        self.transitions
            .push((new_state, std::time::Instant::now()));
        Ok(())
    }

    /// Set an error state
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// Get the current error (if any)
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Check if the resource is usable
    pub fn is_usable(&self) -> bool {
        self.state.is_usable() && self.error.is_none()
    }

    /// Get the complete transition history
    pub fn transition_history(&self) -> &[(ResourceState, std::time::Instant)] {
        &self.transitions
    }

    /// Get the total time spent in a specific state
    pub fn time_in_state(&self, state: ResourceState) -> std::time::Duration {
        let mut total = std::time::Duration::default();

        for window in self.transitions.windows(2) {
            if window[0].0 == state {
                total += window[1].1.duration_since(window[0].1);
            }
        }

        // Add current state time if it matches
        if self.state == state {
            if let Some((_, start_time)) = self.transitions.last() {
                total += start_time.elapsed();
            }
        }

        total
    }
}

/// Manager for resource lifecycles
#[derive(Debug, Default)]
pub struct ResourceLifecycleManager {
    /// Lifecycle tracking for each resource
    lifecycles: HashMap<XId, ResourceLifecycle>,
    /// Statistics tracking
    stats: LifecycleStats,
}

impl ResourceLifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new resource and start tracking its lifecycle
    pub fn register_resource(&mut self, xid: XId, owner: ClientId) -> Result<(), LifecycleError> {
        if self.lifecycles.contains_key(&xid) {
            return Err(LifecycleError::InitializationFailed(format!(
                "Resource {} already registered",
                xid
            )));
        }

        let lifecycle = ResourceLifecycle::new(xid, owner);
        self.lifecycles.insert(xid, lifecycle);
        self.stats.resources_created += 1;
        Ok(())
    }

    /// Transition a resource to a new state
    pub fn transition_resource(
        &mut self,
        xid: XId,
        new_state: ResourceState,
    ) -> Result<(), LifecycleError> {
        let lifecycle = self.lifecycles.get_mut(&xid).ok_or_else(|| {
            LifecycleError::InitializationFailed(format!("Resource {} not found", xid))
        })?;

        lifecycle.transition_to(new_state)?;

        // Update statistics
        match new_state {
            ResourceState::Active => self.stats.activations += 1,
            ResourceState::Destroyed => {
                self.stats.resources_destroyed += 1;
                self.stats.total_lifetime += lifecycle.age();
            }
            _ => {}
        }

        Ok(())
    }

    /// Mark a resource as having an error
    pub fn set_resource_error(&mut self, xid: XId, error: String) -> Result<(), LifecycleError> {
        let lifecycle = self.lifecycles.get_mut(&xid).ok_or_else(|| {
            LifecycleError::InitializationFailed(format!("Resource {} not found", xid))
        })?;

        lifecycle.set_error(error);
        self.stats.errors += 1;
        Ok(())
    }

    /// Get the lifecycle for a resource
    pub fn get_lifecycle(&self, xid: XId) -> Option<&ResourceLifecycle> {
        self.lifecycles.get(&xid)
    }

    /// Remove a resource from lifecycle tracking
    pub fn unregister_resource(&mut self, xid: XId) -> Option<ResourceLifecycle> {
        self.lifecycles.remove(&xid)
    }

    /// Get all resources in a specific state
    pub fn resources_in_state(&self, state: ResourceState) -> Vec<XId> {
        self.lifecycles
            .iter()
            .filter(|(_, lifecycle)| lifecycle.state() == state)
            .map(|(&xid, _)| xid)
            .collect()
    }

    /// Get all resources owned by a client
    pub fn client_resources(&self, client: ClientId) -> Vec<XId> {
        self.lifecycles
            .iter()
            .filter(|(_, lifecycle)| lifecycle.owner() == client)
            .map(|(&xid, _)| xid)
            .collect()
    }

    /// Get lifecycle statistics
    pub fn stats(&self) -> &LifecycleStats {
        &self.stats
    }

    /// Clean up resources that have been destroyed
    pub fn cleanup_destroyed(&mut self) -> Vec<XId> {
        let destroyed: Vec<XId> = self.resources_in_state(ResourceState::Destroyed);
        for xid in &destroyed {
            self.lifecycles.remove(xid);
        }
        destroyed
    }
}

/// Statistics for resource lifecycle management
#[derive(Debug, Default)]
pub struct LifecycleStats {
    /// Total resources created
    pub resources_created: u64,
    /// Total resources destroyed
    pub resources_destroyed: u64,
    /// Total activations
    pub activations: u64,
    /// Total errors
    pub errors: u64,
    /// Total lifetime of all destroyed resources
    pub total_lifetime: std::time::Duration,
}

impl LifecycleStats {
    /// Get the average lifetime of destroyed resources
    pub fn average_lifetime(&self) -> std::time::Duration {
        if self.resources_destroyed > 0 {
            self.total_lifetime / self.resources_destroyed as u32
        } else {
            std::time::Duration::default()
        }
    }

    /// Get the number of currently active resources
    pub fn active_resources(&self) -> u64 {
        self.resources_created - self.resources_destroyed
    }

    /// Get the error rate
    pub fn error_rate(&self) -> f64 {
        if self.resources_created > 0 {
            self.errors as f64 / self.resources_created as f64
        } else {
            0.0
        }
    }
}
