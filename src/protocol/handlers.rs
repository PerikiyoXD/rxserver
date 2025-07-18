//! Standard X11 request handlers
//!
//! This module contains concrete implementations of common X11 request handlers
//! that were previously hardcoded in the connection handler.

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::protocol::{
    ByteOrder, ByteOrderWriter, HandlerResult, Request, RequestHandler, RequestKind, X11Error,
};
use crate::server::state::{ClientId, ClientState, ServerState};

/// Handler for InternAtom requests (opcode 16)
pub struct InternAtomHandler;

#[async_trait]
impl RequestHandler for InternAtomHandler {
    async fn handle_request(
        &self,
        _client_id: ClientId,
        request: &Request,
        _server_state: Arc<Mutex<ServerState>>,
        _client_state: Arc<Mutex<ClientState>>,
        byte_order: ByteOrder,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let ref _atom_registry = _server_state.lock().unwrap().atom_registry;

        // Get request kind
        let _kind = match &request.kind {
            RequestKind::InternAtom(_atom) => _atom,
            _ => {
                return Err(X11Error::Protocol(
                    "Invalid request type for InternAtomHandler".to_string(),
                ));
            }
        };

        // Create a simple response - in a real implementation, this would look up atoms
        let mut writer = ByteOrderWriter::new(byte_order);
        writer.write_u8(1); // Reply
        writer.write_u8(0); // Unused
        writer.write_u16(request.sequence_number); // Sequence number
        writer.write_u32(0); // Reply length
        writer.write_u32(100); // Atom ID (dummy value)
        writer.write_padding(20); // Padding to 32 bytes

        Ok(Some(writer.into_vec()))
    }

    fn opcode(&self) -> u8 {
        16
    }

    fn name(&self) -> &'static str {
        "InternAtom"
    }
}

/// Handler for OpenFont requests (opcode 45)
pub struct OpenFontHandler;

#[async_trait]
impl RequestHandler for OpenFontHandler {
    async fn handle_request(
        &self,
        _client_id: ClientId,
        _request: &Request,
        _server_state: Arc<Mutex<ServerState>>,
        _client_state: Arc<Mutex<ClientState>>,
        _byte_order: ByteOrder,
    ) -> HandlerResult<Option<Vec<u8>>> {
        // OpenFont doesn't generate a response, just return None
        Ok(None)
    }

    fn opcode(&self) -> u8 {
        45
    }

    fn name(&self) -> &'static str {
        "OpenFont"
    }
}

/// Handler for CreateGlyphCursor requests (opcode 94)
pub struct CreateGlyphCursorHandler;

#[async_trait]
impl RequestHandler for CreateGlyphCursorHandler {
    async fn handle_request(
        &self,
        _client_id: ClientId,
        _request: &Request,
        _server_state: Arc<Mutex<ServerState>>,
        _client_state: Arc<Mutex<ClientState>>,
        _byte_order: ByteOrder,
    ) -> HandlerResult<Option<Vec<u8>>> {
        // CreateGlyphCursor doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> u8 {
        94
    }

    fn name(&self) -> &'static str {
        "CreateGlyphCursor"
    }
}

/// Handler for GrabPointer requests (opcode 26)
pub struct GrabPointerHandler;

#[async_trait]
impl RequestHandler for GrabPointerHandler {
    async fn handle_request(
        &self,
        _client_id: ClientId,
        request: &Request,
        _server_state: Arc<Mutex<ServerState>>,
        _client_state: Arc<Mutex<ClientState>>,
        byte_order: ByteOrder,
    ) -> HandlerResult<Option<Vec<u8>>> {
        // GrabPointer generates a reply
        let mut response = ByteOrderWriter::new(byte_order);

        response.write_u8(1); // Reply
        response.write_u8(0); // Success status
        response.write_u16(request.sequence_number); // Sequence number
        response.write_u32(0); // Reply length
        response.write_padding(20); // Padding to 32 bytes  

        Ok(Some(response.into_vec()))
    }

    fn opcode(&self) -> u8 {
        26
    }

    fn name(&self) -> &'static str {
        "GrabPointer"
    }
}

/// Convenience function to create a registry with standard handlers
pub fn create_standard_handler_registry() -> crate::protocol::RequestHandlerRegistry {
    let mut registry = crate::protocol::RequestHandlerRegistry::new();

    registry.register_handler(InternAtomHandler);
    registry.register_handler(OpenFontHandler);
    registry.register_handler(CreateGlyphCursorHandler);
    registry.register_handler(GrabPointerHandler);

    registry
}
