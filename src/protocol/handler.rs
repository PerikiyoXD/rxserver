// SPDX-License-Identifier: Apache-2.0

//! Protocol Request Handler Traits
//!
//! This module defines the core traits for handling X11 protocol requests
//! and generating appropriate responses using EndianWriter for spec compliance.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::server::{Server, client_system::ClientId};

use super::{Request, X11Error};

/// Result type for request handlers
pub type HandlerResult<T> = Result<T, X11Error>;

/// Core trait for handling X11 protocol requests
#[async_trait]
pub trait RequestHandler: Send + Sync {
    /// Handle a specific X11 request and generate a response using EndianWriter
    ///
    /// # Arguments
    /// * `client_id` - The ID of the client making the request
    /// * `request` - The parsed X11 request
    /// * `server_state` - Shared server state
    /// * `client_state` - Client-specific state
    /// * `byte_order` - Client's preferred byte order for EndianWriter
    ///
    /// # Returns
    /// * `Ok(Some(response_bytes))` - Response data generated with EndianWriter
    /// * `Ok(None)` - No response needed for this request type
    /// * `Err(error)` - Error processing the request
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>>;

    /// The (major, minor) opcode this handler supports. `minor` is `None`
    /// for core protocol requests, and `Some(_)` for extension sub-requests
    /// that share a major opcode (e.g. every RANDR request handler holds the
    /// session's RANDR major opcode, assigned by `ExtensionRegistry` and
    /// passed into the handler's constructor, distinguished by minor
    /// opcode).
    fn opcode(&self) -> (u8, Option<u8>);

    /// Get a human-readable name for this handler
    fn name(&self) -> &'static str;
}

/// Registry for managing request handlers by (major, minor) opcode
pub struct RequestHandlerRegistry {
    handlers: HashMap<(u8, Option<u8>), Arc<dyn RequestHandler>>,
}

impl RequestHandlerRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler for its (major, minor) opcode
    pub fn register_handler<T: RequestHandler + 'static>(&mut self, handler: T) {
        let key = handler.opcode();
        self.handlers.insert(key, Arc::new(handler));
    }

    /// Handle a request by routing it to the appropriate handler
    pub async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let key = (request.opcode, request.minor_opcode);

        if let Some(handler) = self.handlers.get(&key) {
            handler.handle_request(client_id, request, server).await
        } else {
            Ok(None)
        }
    }

    /// Get the number of registered handlers
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    /// Get all registered (major, minor) opcodes
    pub fn registered_opcodes(&self) -> Vec<(u8, Option<u8>)> {
        self.handlers.keys().copied().collect()
    }
}

impl Default for RequestHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
