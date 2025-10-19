//! Standard X11 request handlers
//!
//! This module contains concrete implementations of common X11 request handlers
//! that were previously hardcoded in the connection handler.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use crate::{
    protocol::{
        ByteOrder, ByteOrderWriter, HandlerResult, Request, RequestHandler, RequestKind, X11Error,
        randr::*,
    },
    server::{
        GrabResult, PointerGrab, Server,
        client_system::ClientId,
        graphics::{draw_arc, draw_line, fill_arc, fill_rectangle},
        window_system::WindowClass,
    },
};

/// Handler for GetGeometry requests (opcode 14)
pub struct GetGeometryHandler;

#[async_trait]
impl RequestHandler for GetGeometryHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let get_geometry_request = match &request.kind {
            RequestKind::GetGeometry(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for GetGeometry: {:?}",
                    request.kind
                )));
            }
        };

        let server = server.lock().await;

        // Get the drawable (window)
        let drawable_id = get_geometry_request.drawable;

        // Check if drawable exists
        let window = server.get_window(drawable_id).ok_or_else(|| {
            X11Error::Protocol(format!(
                "GetGeometry: drawable {} does not exist",
                drawable_id
            ))
        })?;

        // Get the byte order from the client
        let client = server
            .get_client(client_id)
            .ok_or_else(|| X11Error::Protocol(format!("Client {} not found", client_id)))?;
        let byte_order = client.lock().await.byte_order();

        // Create response
        let mut writer = ByteOrderWriter::new(byte_order);
        writer.write_u8(1); // Reply
        writer.write_u8(window.depth); // Depth
        writer.write_u16(request.sequence_number); // Sequence number
        writer.write_u32(0); // Reply length
        writer.write_u32(server.get_root_window().id); // Root window
        writer.write_u16(window.x as u16); // X coordinate
        writer.write_u16(window.y as u16); // Y coordinate
        writer.write_u16(window.width); // Width
        writer.write_u16(window.height); // Height
        writer.write_u16(window.border_width); // Border width
        writer.write_padding(10); // Padding to 32 bytes

        Ok(Some(writer.into_vec()))
    }

    fn opcode(&self) -> u8 {
        14
    }

    fn name(&self) -> &'static str {
        "GetGeometry"
    }
}

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

/// Handler for QueryExtension requests (opcode 98)
pub struct QueryExtensionHandler;

#[async_trait]
impl RequestHandler for QueryExtensionHandler {
    async fn handle_request(
        &self,
        _client_id: ClientId,
        request: &Request,
        _server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let query_extension_request = match &request.kind {
            RequestKind::QueryExtension(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for QueryExtension: {:?}",
                    request.kind
                )));
            }
        };

        // Check if the extension name matches (trim null terminators)
        let name_trimmed = query_extension_request.name.trim_end_matches('\0');
        debug!(
            "QueryExtension: client asked for extension '{}'",
            name_trimmed
        );
        let is_supported = matches!(
            name_trimmed,
            "RANDR" | "SHAPE" | "MIT-SHM" | "XINERAMA" | "RENDER" | "BIG-REQUESTS"
        );

        if !is_supported {
            debug!(
                "QueryExtension: client asked for unsupported extension '{}'",
                name_trimmed
            );
        }

        // For now, no extensions are supported, so always return present=0
        let mut writer = ByteOrderWriter::new(ByteOrder::LittleEndian);
        writer.write_u8(1); // Reply
        writer.write_u8(0); // Unused
        writer.write_u16(request.sequence_number); // Sequence number
        writer.write_u32(0); // Reply length
        writer.write_u8(if is_supported { 1 } else { 0 }); // Present (1 = present)
        let major_opcode = match name_trimmed {
            "RANDR" => 200,
            "SHAPE" => 129,
            "MIT-SHM" => 130,
            "XINERAMA" => 131,
            "RENDER" => 139,
            "BIG-REQUESTS" => 134,
            _ => 0,
        };
        writer.write_u8(major_opcode); // Major opcode
        writer.write_u8(0); // First event (unused)
        writer.write_u8(0); // First error (unused)
        writer.write_padding(20); // Padding to 32 bytes

        Ok(Some(writer.into_vec()))
    }

    fn opcode(&self) -> u8 {
        98
    }

    fn name(&self) -> &'static str {
        "QueryExtension"
    }
}

/// Handler for BigRequests requests (opcode 134)
pub struct BigRequestsHandler;

#[async_trait]
impl RequestHandler for BigRequestsHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let _big_requests_request = match &request.kind {
            RequestKind::BigRequests(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for BigRequests: {:?}",
                    request.kind
                )));
            }
        };

        debug!(
            "BigRequests: client {} enabled big requests support",
            client_id
        );

        // Mark the client as supporting big requests
        let server_guard = server.lock().await;
        if let Some(client) = server_guard.get_client(client_id) {
            client.lock().await.set_big_requests_enabled(true);
        }

        // Send reply with maximum request length
        let mut writer = ByteOrderWriter::new(ByteOrder::LittleEndian);
        writer.write_u8(1); // Reply
        writer.write_u8(0); // Data (unused)
        writer.write_u16(request.sequence_number); // Sequence number
        writer.write_u32(1); // Reply length (1 additional 4-byte unit)
        writer.write_u32(0x4000000); // Maximum request length (64MB in 4-byte units)
        writer.write_padding(20); // Padding to 32 bytes total (8 + 4 + 20 = 32)

        let reply_bytes = writer.into_vec();
        debug!(
            "BigRequests: created reply with {} bytes: {:?}",
            reply_bytes.len(),
            &reply_bytes[..16]
        );
        Ok(Some(reply_bytes))
    }

    fn opcode(&self) -> u8 {
        134
    }

    fn name(&self) -> &'static str {
        "BigRequests"
    }
}

/// Handler for CreateWindow requests (opcode 1)
pub struct CreateWindowHandler;

#[async_trait]
impl RequestHandler for CreateWindowHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let create_window_request = match &request.kind {
            RequestKind::CreateWindow(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for CreateWindow: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;

        // Convert window class
        let window_class = match create_window_request.class {
            0 => WindowClass::CopyFromParent,
            1 => WindowClass::InputOutput,
            2 => WindowClass::InputOnly,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid window class: {}",
                    create_window_request.class
                )));
            }
        };

        // Create the window
        server
            .create_window(
                create_window_request.wid,
                create_window_request.parent,
                create_window_request.x,
                create_window_request.y,
                create_window_request.width,
                create_window_request.height,
                create_window_request.border_width,
                create_window_request.depth,
                window_class,
                client_id,
            )
            .await
            .map_err(|e| X11Error::Protocol(format!("Failed to create window: {}", e)))?;

        // CreateWindow doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> u8 {
        1
    }

    fn name(&self) -> &'static str {
        "CreateWindow"
    }
}

/// Handler for DestroyWindow requests (opcode 4)
pub struct DestroyWindowHandler;

#[async_trait]
impl RequestHandler for DestroyWindowHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let destroy_window_request = match &request.kind {
            RequestKind::DestroyWindow(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for DestroyWindow: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;

        // Check if client owns the window
        if let Some(window) = server.get_window(destroy_window_request.window) {
            if window.owner != Some(client_id) {
                return Err(X11Error::Protocol(format!(
                    "Client {} does not own window {}",
                    client_id, destroy_window_request.window
                )));
            }
        } else {
            return Err(X11Error::Protocol(format!(
                "Window {} does not exist",
                destroy_window_request.window
            )));
        }

        // Destroy the window
        server
            .destroy_window(destroy_window_request.window)
            .await
            .map_err(|e| X11Error::Protocol(format!("Failed to destroy window: {}", e)))?;

        // DestroyWindow doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> u8 {
        4
    }

    fn name(&self) -> &'static str {
        "DestroyWindow"
    }
}

/// Handler for MapWindow requests (opcode 8)
pub struct MapWindowHandler;

#[async_trait]
impl RequestHandler for MapWindowHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let map_window_request = match &request.kind {
            RequestKind::MapWindow(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for MapWindow: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;

        // Check if client owns the window
        if let Some(window) = server.get_window(map_window_request.window) {
            if window.owner != Some(client_id) {
                return Err(X11Error::Protocol(format!(
                    "Client {} does not own window {}",
                    client_id, map_window_request.window
                )));
            }
        } else {
            return Err(X11Error::Protocol(format!(
                "Window {} does not exist",
                map_window_request.window
            )));
        }

        // Map the window
        server
            .map_window(map_window_request.window)
            .await
            .map_err(|e| X11Error::Protocol(format!("Failed to map window: {}", e)))?;

        // Send expose event to notify client that window needs redrawing
        // Get window info before sending event to avoid borrowing issues
        let (window_id, width, height) = {
            let window = server.get_window(map_window_request.window).unwrap();
            (window.id, window.width, window.height)
        };
        server
            .send_expose_event(client_id, window_id, 0, 0, width, height, 0)
            .await;

        // MapWindow doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> u8 {
        8
    }

    fn name(&self) -> &'static str {
        "MapWindow"
    }
}

/// Handler for UnmapWindow requests (opcode 10)
pub struct UnmapWindowHandler;

#[async_trait]
impl RequestHandler for UnmapWindowHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let unmap_window_request = match &request.kind {
            RequestKind::UnmapWindow(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for UnmapWindow: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;

        // Check if client owns the window
        if let Some(window) = server.get_window(unmap_window_request.window) {
            if window.owner != Some(client_id) {
                return Err(X11Error::Protocol(format!(
                    "Client {} does not own window {}",
                    client_id, unmap_window_request.window
                )));
            }
        } else {
            return Err(X11Error::Protocol(format!(
                "Window {} does not exist",
                unmap_window_request.window
            )));
        }

        // Unmap the window
        server
            .unmap_window(unmap_window_request.window)
            .await
            .map_err(|e| X11Error::Protocol(format!("Failed to unmap window: {}", e)))?;

        // UnmapWindow doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> u8 {
        10
    }

    fn name(&self) -> &'static str {
        "UnmapWindow"
    }
}

/// Handler for CreateGC requests (opcode 55)
pub struct CreateGCHandler;

#[async_trait]
impl RequestHandler for CreateGCHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let create_gc_request = match &request.kind {
            RequestKind::CreateGC(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for CreateGC: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;

        // Create the graphics context
        server
            .create_gc(create_gc_request.gc, create_gc_request.drawable, client_id)
            .map_err(|e| X11Error::Protocol(format!("Failed to create graphics context: {}", e)))?;

        // CreateGC doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> u8 {
        55
    }

    fn name(&self) -> &'static str {
        "CreateGC"
    }
}

/// Handler for PolyArc requests (opcode 59)
pub struct PolyArcHandler;

#[async_trait]
impl RequestHandler for PolyArcHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let poly_arc_request = match &request.kind {
            RequestKind::PolyArc(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for PolyArc: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;

        // Get the drawable (window)
        let window_id = poly_arc_request.drawable;
        let gc_id = poly_arc_request.gc;

        // Check if window exists and client owns it
        {
            let window = server.get_window(window_id).ok_or_else(|| {
                X11Error::Protocol(format!("PolyArc: window {} does not exist", window_id))
            })?;

            if window.owner != Some(client_id) {
                return Err(X11Error::Protocol(format!(
                    "PolyArc: client {} does not own window {}",
                    client_id, window_id
                )));
            }
        } // immutable borrow ends here

        // Get graphics context
        let gc_foreground = server
            .get_gc(gc_id)
            .ok_or_else(|| {
                X11Error::Protocol(format!(
                    "PolyArc: graphics context {} does not exist",
                    gc_id
                ))
            })?
            .foreground;

        // Now get mutable window reference
        let window = server.get_window_mut(window_id).unwrap();

        // Draw arcs
        for arc in &poly_arc_request.arcs {
            draw_arc(window, arc, gc_foreground);
        }

        // PolyArc doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> u8 {
        59
    }

    fn name(&self) -> &'static str {
        "PolyArc"
    }
}

/// Handler for FillArc requests (opcode 61)
pub struct FillArcHandler;

#[async_trait]
impl RequestHandler for FillArcHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let fill_arc_request = match &request.kind {
            RequestKind::FillArc(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for FillArc: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;

        // Get the drawable (window)
        let window_id = fill_arc_request.drawable;
        let gc_id = fill_arc_request.gc;

        // Check if window exists and client owns it
        {
            let window = server.get_window(window_id).ok_or_else(|| {
                X11Error::Protocol(format!("FillArc: window {} does not exist", window_id))
            })?;

            if window.owner != Some(client_id) {
                return Err(X11Error::Protocol(format!(
                    "FillArc: client {} does not own window {}",
                    client_id, window_id
                )));
            }
        } // immutable borrow ends here

        // Get graphics context
        let gc_foreground = server
            .get_gc(gc_id)
            .ok_or_else(|| {
                X11Error::Protocol(format!(
                    "FillArc: graphics context {} does not exist",
                    gc_id
                ))
            })?
            .foreground;

        // Now get mutable window reference
        let window = server.get_window_mut(window_id).unwrap();

        // Fill arcs
        for arc in &fill_arc_request.arcs {
            fill_arc(window, arc, gc_foreground);
        }

        // FillArc doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> u8 {
        61
    }

    fn name(&self) -> &'static str {
        "FillArc"
    }
}

/// Handler for PolyLine requests (opcode 65)
pub struct PolyLineHandler;

#[async_trait]
impl RequestHandler for PolyLineHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let poly_line_request = match &request.kind {
            RequestKind::PolyLine(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for PolyLine: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;

        // Get the drawable (window)
        let window_id = poly_line_request.drawable;
        let gc_id = poly_line_request.gc;

        // Check if window exists and client owns it
        {
            let window = server.get_window(window_id).ok_or_else(|| {
                X11Error::Protocol(format!("PolyLine: window {} does not exist", window_id))
            })?;

            if window.owner != Some(client_id) {
                return Err(X11Error::Protocol(format!(
                    "PolyLine: client {} does not own window {}",
                    client_id, window_id
                )));
            }
        } // immutable borrow ends here

        // Get graphics context
        let gc_foreground = server
            .get_gc(gc_id)
            .ok_or_else(|| {
                X11Error::Protocol(format!(
                    "PolyLine: graphics context {} does not exist",
                    gc_id
                ))
            })?
            .foreground;

        // Now get mutable window reference
        let window = server.get_window_mut(window_id).unwrap();

        // Draw lines
        draw_line(window, &poly_line_request.points, gc_foreground);

        // PolyLine doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> u8 {
        65
    }

    fn name(&self) -> &'static str {
        "PolyLine"
    }
}

/// Handler for PolyFillRectangle requests (opcode 70)
pub struct PolyFillRectangleHandler;

#[async_trait]
impl RequestHandler for PolyFillRectangleHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let poly_fill_rect_request = match &request.kind {
            RequestKind::PolyFillRectangle(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for PolyFillRectangle: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;

        // Get the drawable (window)
        let window_id = poly_fill_rect_request.drawable;
        let gc_id = poly_fill_rect_request.gc;

        // Check if window exists and client owns it
        {
            let window = server.get_window(window_id).ok_or_else(|| {
                X11Error::Protocol(format!(
                    "PolyFillRectangle: window {} does not exist",
                    window_id
                ))
            })?;

            if window.owner != Some(client_id) {
                return Err(X11Error::Protocol(format!(
                    "PolyFillRectangle: client {} does not own window {}",
                    client_id, window_id
                )));
            }
        } // immutable borrow ends here

        // Get graphics context
        let gc_foreground = server
            .get_gc(gc_id)
            .ok_or_else(|| {
                X11Error::Protocol(format!(
                    "PolyFillRectangle: graphics context {} does not exist",
                    gc_id
                ))
            })?
            .foreground;

        // Now get mutable window reference
        let window = server.get_window_mut(window_id).unwrap();

        // Fill rectangles
        for rect in &poly_fill_rect_request.rectangles {
            fill_rectangle(window, rect, gc_foreground);
        }

        // PolyFillRectangle doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> u8 {
        70
    }

    fn name(&self) -> &'static str {
        "PolyFillRectangle"
    }
}

/// Convenience function to create a registry with standard handlers
pub fn create_standard_handler_registry() -> crate::protocol::RequestHandlerRegistry {
    let mut registry = crate::protocol::RequestHandlerRegistry::new();

    // Window management handlers
    registry.register_handler(CreateWindowHandler);
    registry.register_handler(DestroyWindowHandler);
    registry.register_handler(MapWindowHandler);
    registry.register_handler(UnmapWindowHandler);
    registry.register_handler(GetGeometryHandler);
    registry.register_handler(CreateGCHandler);

    registry.register_handler(PolyArcHandler);
    registry.register_handler(FillArcHandler);
    registry.register_handler(PolyLineHandler);
    registry.register_handler(PolyFillRectangleHandler);

    registry.register_handler(InternAtomHandler);
    registry.register_handler(OpenFontHandler);
    registry.register_handler(CreateGlyphCursorHandler);
    registry.register_handler(GrabPointerHandler);
    registry.register_handler(QueryExtensionHandler);
    registry.register_handler(BigRequestsHandler);

    // RANDR extension handlers (using major opcode 200 + minor opcode)
    registry.register_handler(RandrQueryVersionHandler);
    registry.register_handler(RandrGetScreenResourcesHandler);
    registry.register_handler(RandrGetOutputInfoHandler);
    registry.register_handler(RandrGetCrtcInfoHandler);
    registry.register_handler(RandrGetScreenSizeRangeHandler);

    registry
}
