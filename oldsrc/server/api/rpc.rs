//! RPC Interface
//!
//! This module provides RPC interface for server management.

/// RPC handler
pub struct RpcHandler {
    _placeholder: (),
}

impl RpcHandler {
    /// Create new RPC handler
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for RpcHandler {
    fn default() -> Self {
        Self::new()
    }
}
