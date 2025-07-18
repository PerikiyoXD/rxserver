//! Standard X11 request handlers
//!
//! This module contains concrete implementations of common X11 request handlers
//! that were previously hardcoded in the connection handler.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    protocol::{ByteOrderWriter, HandlerResult, Request, RequestHandler, RequestKind, X11Error},
    server::{GrabResult, PointerGrab, Server, client_system::ClientId},
};

/// Handler for InternAtom requests (opcode 16)
pub struct InternAtomHandler;

#[async_trait]
impl RequestHandler for InternAtomHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        // Get the byte order from the server
        let mut server = server.lock().await;
        let client = server
            .get_client(client_id)
            .ok_or_else(|| X11Error::Protocol(format!("Client {} not found", client_id)))?;
        let byte_order = client.lock().await.byte_order();

        let intern_atom_request = match &request.kind {
            RequestKind::InternAtom(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for InternAtom: {:?}",
                    request.kind
                )));
            }
        };

        let atom = server
            .intern_atom(
                &intern_atom_request.atom_name,
                intern_atom_request.only_if_exists != 0,
            )
            .ok_or_else(|| {
                X11Error::Protocol(format!(
                    "Failed to intern atom {}",
                    intern_atom_request.atom_name
                ))
            })?;

        // Create a simple response - in a real implementation, this would look up atoms
        let mut writer = ByteOrderWriter::new(byte_order);
        writer.write_u8(1); // Reply
        writer.write_u8(0); // Unused
        writer.write_u16(request.sequence_number); // Sequence number
        writer.write_u32(0); // Reply length
        writer.write_u32(atom); // Atom ID
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
        _server: Arc<Mutex<Server>>,
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
        _server: Arc<Mutex<Server>>,
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
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let mut server = server.lock().await;
        let client = server
            .get_client(client_id)
            .ok_or_else(|| X11Error::Protocol(format!("Client {} not found", client_id)))?;
        let byte_order = client.lock().await.byte_order();

        let grab_request = match &request.kind {
            RequestKind::GrabPointer(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for GrabPointer: {:?}",
                    request.kind
                )));
            }
        };

        // Convert request to PointerGrab
        let pointer_grab = PointerGrab {
            grab_window: grab_request.grab_window,
            grabbing_client: client_id,
            owner_events: grab_request.owner_events != 0,
            event_mask: grab_request.event_mask,
            confine_to: if grab_request.confine_to == 0 {
                None
            } else {
                Some(grab_request.confine_to)
            },
            cursor: if grab_request.cursor == 0 {
                None
            } else {
                Some(grab_request.cursor)
            },
            time: grab_request.time,
        };

        // Validate window exists and is viewable
        if !server.window_exists(grab_request.grab_window) {
            let mut response = ByteOrderWriter::new(byte_order);
            response.write_u8(1); // Reply
            response.write_u8(GrabResult::BadWindow.to_x11_status());
            response.write_u16(request.sequence_number);
            response.write_u32(0); // Reply length
            response.write_padding(20);
            return Ok(Some(response.into_vec()));
        }

        // Check if the grab window is viewable
        if !server.is_window_viewable(grab_request.grab_window) {
            let mut response = ByteOrderWriter::new(byte_order);
            response.write_u8(1); // Reply
            response.write_u8(GrabResult::NotViewable.to_x11_status());
            response.write_u16(request.sequence_number);
            response.write_u32(0); // Reply length
            response.write_padding(20);
            return Ok(Some(response.into_vec()));
        }

        // Attempt to establish the grab
        let grab_result = server.establish_pointer_grab(pointer_grab);

        // Send reply with status
        let mut response = ByteOrderWriter::new(byte_order);
        response.write_u8(1); // Reply
        response.write_u8(grab_result.to_x11_status());
        response.write_u16(request.sequence_number);
        response.write_u32(0); // Reply length
        response.write_padding(20);

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
