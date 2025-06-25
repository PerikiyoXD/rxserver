//! Protocol Request Handler Traits
//!
//! This module defines the core traits for handling X11 protocol requests
//! and generating appropriate responses using EndianWriter for spec compliance.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::{ByteOrder, EndianWriter, Request, X11Error};
use crate::server::state::{ClientId, ClientState, ServerState};

/// Result type for request handlers
pub type HandlerResult<T> = Result<T, X11Error>;

/// Core trait for handling X11 protocol requests
/// All implementations must use EndianWriter for response generation to ensure protocol compliance
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
        server_state: Arc<Mutex<ServerState>>,
        client_state: Arc<Mutex<ClientState>>,
        byte_order: ByteOrder,
    ) -> HandlerResult<Option<Vec<u8>>>;

    /// Get the opcode this handler supports
    fn opcode(&self) -> u8;

    /// Get a human-readable name for this handler
    fn name(&self) -> &'static str;
}

/// Registry for managing request handlers by opcode
pub struct RequestHandlerRegistry {
    handlers: HashMap<u8, Arc<dyn RequestHandler>>,
}

impl RequestHandlerRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler for a specific opcode
    pub fn register_handler<T: RequestHandler + 'static>(&mut self, handler: T) {
        let opcode = handler.opcode();
        self.handlers.insert(opcode, Arc::new(handler));
    }

    /// Handle a request by routing it to the appropriate handler
    pub async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server_state: Arc<Mutex<ServerState>>,
        client_state: Arc<Mutex<ClientState>>,
        byte_order: ByteOrder,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let opcode = request.opcode();

        if let Some(handler) = self.handlers.get(&opcode) {
            handler
                .handle_request(client_id, request, server_state, client_state, byte_order)
                .await
        } else {
            Ok(None)
        }
    }

    /// Get the number of registered handlers
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    /// Get all registered opcodes
    pub fn registered_opcodes(&self) -> Vec<u8> {
        self.handlers.keys().copied().collect()
    }
}

impl Default for RequestHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
