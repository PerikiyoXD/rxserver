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
        types::{Opcode, PixmapValue, RenderOpcode, value_mask},
    },
    server::{
        GrabResult, PointerGrab, Server,
        client_system::ClientId,
        gcontext_system::GraphicsContext,
        graphics::{clear_area, draw_arc, draw_line, fill_arc, fill_rectangle},
        window_system::{Background, WindowClass},
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

    fn opcode(&self) -> (u8, Option<u8>) {
        (14, None)
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

        // With only_if_exists=true, a name that isn't already interned is not
        // an error: the spec requires replying with atom None (0), not
        // failing the request.
        let atom = server
            .intern_atom(
                &intern_atom_request.atom_name,
                intern_atom_request.only_if_exists != 0,
            )
            .unwrap_or_else(|| {
                debug!(
                    "InternAtom: '{}' does not exist (only_if_exists=true), replying with None",
                    intern_atom_request.atom_name
                );
                0
            });

        let mut writer = ByteOrderWriter::new(byte_order);
        writer.write_u8(1); // Reply
        writer.write_u8(0); // Unused
        writer.write_u16(request.sequence_number); // Sequence number
        writer.write_u32(0); // Reply length
        writer.write_u32(atom); // Atom ID
        writer.write_padding(20); // Padding to 32 bytes

        Ok(Some(writer.into_vec()))
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (16, None)
    }

    fn name(&self) -> &'static str {
        "InternAtom"
    }
}

/// Handler for ChangeProperty requests (opcode 18)
pub struct ChangePropertyHandler;

#[async_trait]
impl RequestHandler for ChangePropertyHandler {
    async fn handle_request(
        &self,
        _client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let change_property_request = match &request.kind {
            RequestKind::ChangeProperty(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for ChangeProperty: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;

        let window = server
            .get_window_mut(change_property_request.window)
            .ok_or_else(|| {
                X11Error::Protocol(format!(
                    "ChangeProperty: window {} does not exist",
                    change_property_request.window
                ))
            })?;

        window.change_property(
            change_property_request.property,
            change_property_request.r#type,
            change_property_request.format,
            change_property_request.mode,
            change_property_request.data.clone(),
        );

        // ChangeProperty does not generate a response
        Ok(None)
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (18, None)
    }

    fn name(&self) -> &'static str {
        "ChangeProperty"
    }
}

/// Handler for GetProperty requests (opcode 20)
pub struct GetPropertyHandler;

#[async_trait]
impl RequestHandler for GetPropertyHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let get_property_request = match &request.kind {
            RequestKind::GetProperty(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for GetProperty: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;

        // Get the byte order from the client
        let client = server
            .get_client(client_id)
            .ok_or_else(|| X11Error::Protocol(format!("Client {} not found", client_id)))?;
        let byte_order = client.lock().await.byte_order();

        let window = server
            .get_window_mut(get_property_request.window)
            .ok_or_else(|| {
                X11Error::Protocol(format!(
                    "GetProperty: window {} does not exist",
                    get_property_request.window
                ))
            })?;

        let mut writer = ByteOrderWriter::new(byte_order);

        match window.get_property(get_property_request.property) {
            Some(property)
                if get_property_request.r#type == 0
                    || get_property_request.r#type == property.r#type =>
            {
                let unit_bytes = match property.format {
                    8 => 1,
                    16 => 2,
                    _ => 4,
                };
                let offset_bytes = (get_property_request.long_offset as usize) * 4;
                let requested_bytes = (get_property_request.long_length as usize) * 4;

                let start = offset_bytes.min(property.data.len());
                let end = (start + requested_bytes).min(property.data.len());
                let value = property.data[start..end].to_vec();
                let bytes_after = property.data.len() - end;
                let value_units = value.len() / unit_bytes;

                let r#type = property.r#type;
                let format = property.format;

                writer.write_u8(1); // Reply
                writer.write_u8(format);
                writer.write_u16(request.sequence_number);
                writer.write_u32((value.len() as u32 + 3) / 4); // Reply length in 4-byte units
                writer.write_u32(r#type);
                writer.write_u32(bytes_after as u32);
                writer.write_u32(value_units as u32);
                writer.write_padding(12);
                writer.write_bytes(&value);
                let pad = (4 - (value.len() % 4)) % 4;
                writer.write_padding(pad);

                if get_property_request.delete != 0 && bytes_after == 0 {
                    window.delete_property(get_property_request.property);
                }
            }
            _ => {
                // Property doesn't exist, or exists with a mismatched requested type
                writer.write_u8(1); // Reply
                writer.write_u8(0); // Format (0 = property doesn't exist)
                writer.write_u16(request.sequence_number);
                writer.write_u32(0); // Reply length
                writer.write_u32(0); // Type (None)
                writer.write_u32(0); // Bytes after
                writer.write_u32(0); // Value length
                writer.write_padding(12);
            }
        }

        Ok(Some(writer.into_vec()))
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (20, None)
    }

    fn name(&self) -> &'static str {
        "GetProperty"
    }
}

/// Handler for QueryColors requests (opcode 91)
pub struct QueryColorsHandler;

#[async_trait]
impl RequestHandler for QueryColorsHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let query_colors_request = match &request.kind {
            RequestKind::QueryColors(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for QueryColors: {:?}",
                    request.kind
                )));
            }
        };

        let server = server.lock().await;

        let client = server
            .get_client(client_id)
            .ok_or_else(|| X11Error::Protocol(format!("Client {} not found", client_id)))?;
        let byte_order = client.lock().await.byte_order();

        let colormap = server
            .get_colormap(query_colors_request.colormap)
            .ok_or_else(|| {
                X11Error::Protocol(format!(
                    "QueryColors: colormap {} does not exist",
                    query_colors_request.colormap
                ))
            })?;

        let n = query_colors_request.pixels.len() as u32;

        let mut writer = ByteOrderWriter::new(byte_order);
        writer.write_u8(1); // Reply
        writer.write_u8(0); // Unused
        writer.write_u16(request.sequence_number);
        writer.write_u32(n * 2); // Reply length in 4-byte units (n * 8 bytes / 4)
        writer.write_u16(n as u16); // Number of RGB entries
        writer.write_padding(22);

        for &pixel in &query_colors_request.pixels {
            let color =
                colormap
                    .get_color(pixel)
                    .unwrap_or(crate::server::colormap_system::ColorEntry {
                        red: 0,
                        green: 0,
                        blue: 0,
                    });
            writer.write_u16(color.red);
            writer.write_u16(color.green);
            writer.write_u16(color.blue);
            writer.write_u16(0); // Unused
        }

        Ok(Some(writer.into_vec()))
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (91, None)
    }

    fn name(&self) -> &'static str {
        "QueryColors"
    }
}

/// Handler for CreatePixmap requests (opcode 53)
pub struct CreatePixmapHandler;

#[async_trait]
impl RequestHandler for CreatePixmapHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let create_pixmap_request = match &request.kind {
            RequestKind::CreatePixmap(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for CreatePixmap: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;

        // Check if the drawable exists (used to determine depth)
        let drawable_id = create_pixmap_request.drawable;
        if !server.get_window(drawable_id).is_some() {
            return Err(X11Error::Protocol(format!(
                "CreatePixmap: drawable {} does not exist",
                drawable_id
            )));
        }

        // Check if the pixmap ID is within the client's resource range
        let client = server
            .get_client(client_id)
            .ok_or_else(|| X11Error::Protocol(format!("Client {} not found", client_id)))?;
        if !client.lock().await.owns_resource(create_pixmap_request.pid) {
            return Err(X11Error::Protocol(format!(
                "CreatePixmap: pixmap ID {} is not within client's resource range",
                create_pixmap_request.pid
            )));
        }

        // Create the pixmap
        server
            .create_pixmap(
                create_pixmap_request.pid,
                create_pixmap_request.width,
                create_pixmap_request.height,
                create_pixmap_request.depth,
                client_id,
            )
            .map_err(|e| X11Error::Protocol(format!("Failed to create pixmap: {}", e)))?;

        // CreatePixmap doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (53, None)
    }

    fn name(&self) -> &'static str {
        "CreatePixmap"
    }
}

/// Handler for FreePixmap requests (opcode 54)
pub struct FreePixmapHandler;

#[async_trait]
impl RequestHandler for FreePixmapHandler {
    async fn handle_request(
        &self,
        _client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let free_pixmap_request = match &request.kind {
            RequestKind::FreePixmap(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for FreePixmap: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;

        if !server.remove_pixmap(free_pixmap_request.pixmap) {
            return Err(X11Error::Protocol(format!(
                "FreePixmap: pixmap {} does not exist",
                free_pixmap_request.pixmap
            )));
        }

        // FreePixmap doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (54, None)
    }

    fn name(&self) -> &'static str {
        "FreePixmap"
    }
}

/// Handler for GetInputFocus requests (opcode 43)
pub struct GetInputFocusHandler;

#[async_trait]
impl RequestHandler for GetInputFocusHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        match &request.kind {
            RequestKind::GetInputFocus(_) => {}
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for GetInputFocus: {:?}",
                    request.kind
                )));
            }
        };

        let server = server.lock().await;

        let client = server
            .get_client(client_id)
            .ok_or_else(|| X11Error::Protocol(format!("Client {} not found", client_id)))?;
        let byte_order = client.lock().await.byte_order();

        // No SetInputFocus support yet, so report the server's startup
        // default: focus follows the pointer, currently over the root window.
        let root_window = server.get_root_window().id;

        let mut writer = ByteOrderWriter::new(byte_order);
        writer.write_u8(1); // Reply
        writer.write_u8(1); // revert_to = PointerRoot
        writer.write_u16(request.sequence_number);
        writer.write_u32(0); // Reply length
        writer.write_u32(root_window); // focus
        writer.write_padding(20);

        Ok(Some(writer.into_vec()))
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (43, None)
    }

    fn name(&self) -> &'static str {
        "GetInputFocus"
    }
}

/// Handler for PutImage requests (opcode 72)
pub struct PutImageHandler;

#[async_trait]
impl RequestHandler for PutImageHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let put_image_request = match &request.kind {
            RequestKind::PutImage(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for PutImage: {:?}",
                    request.kind
                )));
            }
        };

        let server = server.lock().await;

        // Check if the drawable exists (window or pixmap)
        let drawable_id = put_image_request.drawable;
        let drawable_exists =
            server.get_window(drawable_id).is_some() || server.get_pixmap(drawable_id).is_some();
        if !drawable_exists {
            return Err(X11Error::Protocol(format!(
                "PutImage: drawable {} does not exist",
                drawable_id
            )));
        }

        // Check if the GC exists and is owned by the client
        let gc_id = put_image_request.gc;
        if let Some(gc) = server.get_gc(gc_id) {
            if gc.owner != client_id {
                return Err(X11Error::Protocol(format!(
                    "PutImage: client {} does not own graphics context {}",
                    client_id, gc_id
                )));
            }
        } else {
            return Err(X11Error::Protocol(format!(
                "PutImage: graphics context {} does not exist",
                gc_id
            )));
        }

        // PutImage doesn't generate a response
        // In a real implementation, this would render the image data to the drawable
        Ok(None)
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (72, None)
    }

    fn name(&self) -> &'static str {
        "PutImage"
    }
}

/// Handler for CopyArea requests (opcode 62)
pub struct CopyAreaHandler;

#[async_trait]
impl RequestHandler for CopyAreaHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let copy_area_request = match &request.kind {
            RequestKind::CopyArea(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for CopyArea: {:?}",
                    request.kind
                )));
            }
        };

        let server = server.lock().await;

        // Check if the source drawable exists (window or pixmap)
        let src_drawable_id = copy_area_request.src_drawable;
        let src_drawable_exists = server.get_window(src_drawable_id).is_some()
            || server.get_pixmap(src_drawable_id).is_some();
        if !src_drawable_exists {
            return Err(X11Error::Protocol(format!(
                "CopyArea: source drawable {} does not exist",
                src_drawable_id
            )));
        }

        // Check if the destination drawable exists (window or pixmap)
        let dst_drawable_id = copy_area_request.dst_drawable;
        let dst_drawable_exists = server.get_window(dst_drawable_id).is_some()
            || server.get_pixmap(dst_drawable_id).is_some();
        if !dst_drawable_exists {
            return Err(X11Error::Protocol(format!(
                "CopyArea: destination drawable {} does not exist",
                dst_drawable_id
            )));
        }

        // Check if the GC exists and is owned by the client
        let gc_id = copy_area_request.gc;
        if let Some(gc) = server.get_gc(gc_id) {
            if gc.owner != client_id {
                return Err(X11Error::Protocol(format!(
                    "CopyArea: client {} does not own graphics context {}",
                    client_id, gc_id
                )));
            }
        } else {
            return Err(X11Error::Protocol(format!(
                "CopyArea: graphics context {} does not exist",
                gc_id
            )));
        }

        // CopyArea doesn't generate a response
        // In a real implementation, this would copy pixels from src to dst drawable
        Ok(None)
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (62, None)
    }

    fn name(&self) -> &'static str {
        "CopyArea"
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

    fn opcode(&self) -> (u8, Option<u8>) {
        (45, None)
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

    fn opcode(&self) -> (u8, Option<u8>) {
        (94, None)
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

    fn opcode(&self) -> (u8, Option<u8>) {
        (26, None)
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
        server: Arc<Mutex<Server>>,
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

        let server = server.lock().await;
        let major_opcode = server.extensions().major_opcode(name_trimmed);
        let is_supported = major_opcode.is_some();

        if !is_supported {
            debug!(
                "QueryExtension: client asked for unsupported extension '{}'",
                name_trimmed
            );
        } else if !server.extensions().is_implemented(name_trimmed) {
            debug!(
                "QueryExtension: '{}' has a major opcode assigned but no request handler yet",
                name_trimmed
            );
        }

        let mut writer = ByteOrderWriter::new(ByteOrder::LittleEndian);
        writer.write_u8(1); // Reply
        writer.write_u8(0); // Unused
        writer.write_u16(request.sequence_number); // Sequence number
        writer.write_u32(0); // Reply length
        writer.write_u8(if is_supported { 1 } else { 0 }); // Present (1 = present)
        // Per spec, major/first-event/first-error are only meaningful when
        // present=1; major_opcode is None exactly when is_supported is
        // false, so 0 here is a defined "don't care" placeholder, not a
        // swallowed error.
        writer.write_u8(major_opcode.unwrap_or(0)); // Major opcode
        writer.write_u8(0); // First event (unused)
        writer.write_u8(0); // First error (unused)
        writer.write_padding(20); // Padding to 32 bytes

        Ok(Some(writer.into_vec()))
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (98, None)
    }

    fn name(&self) -> &'static str {
        "QueryExtension"
    }
}

/// Handler for BigRequests requests (opcode 134)
pub struct BigRequestsHandler {
    major_opcode: u8,
}

impl BigRequestsHandler {
    pub fn new(major_opcode: u8) -> Self {
        Self { major_opcode }
    }
}

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
        writer.write_u32(0); // Reply length (0 additional 4-byte units)
        writer.write_u32(0x4000000); // Maximum request length (64MB in 4-byte units)
        writer.write_padding(20); // Padding to 32 bytes total

        let reply_bytes = writer.into_vec();
        debug!(
            "BigRequests: created reply with {} bytes: {:?}",
            reply_bytes.len(),
            &reply_bytes[..16]
        );
        Ok(Some(reply_bytes))
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (self.major_opcode, None)
    }

    fn name(&self) -> &'static str {
        "BigRequests"
    }
}

/// Handler for RenderQueryVersion requests (RENDER minor opcode 0)
pub struct RenderQueryVersionHandler {
    major_opcode: u8,
}

impl RenderQueryVersionHandler {
    pub fn new(major_opcode: u8) -> Self {
        Self { major_opcode }
    }
}

#[async_trait]
impl RequestHandler for RenderQueryVersionHandler {
    async fn handle_request(
        &self,
        _client_id: ClientId,
        request: &Request,
        _server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let _query_request = match &request.kind {
            RequestKind::RenderQueryVersion(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for RenderQueryVersion: {:?}",
                    request.kind
                )));
            }
        };

        // Server-side RENDER support level. Advertising 0.11 (the version
        // that introduced trapezoids/triangles) without more of the
        // extension's requests implemented would let a client rely on
        // features this server can't serve - keep it at the lowest version
        // that QueryVersion + no other requests can honestly support.
        let mut writer = ByteOrderWriter::new(ByteOrder::LittleEndian);
        writer.write_u8(1); // Reply
        writer.write_u8(0); // Unused
        writer.write_u16(request.sequence_number); // Sequence number
        writer.write_u32(0); // Reply length
        writer.write_u32(0); // Server major version
        writer.write_u32(1); // Server minor version
        writer.write_padding(16); // Padding to 32 bytes

        Ok(Some(writer.into_vec()))
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (self.major_opcode, Some(RenderOpcode::QueryVersion.to_u8()))
    }

    fn name(&self) -> &'static str {
        "RenderQueryVersion"
    }
}

/// Handler for RenderCreateSolidFill requests (RENDER minor opcode 33)
pub struct RenderCreateSolidFillHandler {
    major_opcode: u8,
}

impl RenderCreateSolidFillHandler {
    pub fn new(major_opcode: u8) -> Self {
        Self { major_opcode }
    }
}

#[async_trait]
impl RequestHandler for RenderCreateSolidFillHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let create_request = match &request.kind {
            RequestKind::RenderCreateSolidFill(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for RenderCreateSolidFill: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;

        let client = server
            .get_client(client_id)
            .ok_or_else(|| X11Error::Protocol(format!("Client {} not found", client_id)))?;
        if !client.lock().await.owns_resource(create_request.pid) {
            return Err(X11Error::Protocol(format!(
                "RenderCreateSolidFill: picture id {} is not within client's resource range",
                create_request.pid
            )));
        }

        server
            .create_solid_fill_picture(
                create_request.pid,
                crate::server::picture_system::RenderColor {
                    red: create_request.red,
                    green: create_request.green,
                    blue: create_request.blue,
                    alpha: create_request.alpha,
                },
                client_id,
            )
            .map_err(|e| X11Error::Protocol(format!("Failed to create solid fill: {}", e)))?;

        // RenderCreateSolidFill doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (
            self.major_opcode,
            Some(RenderOpcode::CreateSolidFill.to_u8()),
        )
    }

    fn name(&self) -> &'static str {
        "RenderCreateSolidFill"
    }
}

/// Handler for CreateWindow requests (opcode 1)
pub struct CreateWindowHandler;

/// Look up the value-list slot for a single value-mask bit, per xproto's
/// CreateWindow encoding: each set bit in the mask consumes exactly one
/// CARD32 slot in `value_list`, in ascending bit order. Returns `None` if
/// `bit` isn't set in `value_mask`.
fn value_list_entry(value_mask: u32, value_list: &[u32], bit: u32) -> Option<u32> {
    if value_mask & bit == 0 {
        return None;
    }
    let index = (value_mask & (bit - 1)).count_ones() as usize;
    value_list.get(index).copied()
}

/// Decode the `background-pixmap`/`background-pixel` value-list entries into
/// a `Background`, per xproto's CreateWindow VALUEs table. `background-pixel`
/// takes precedence when both bits are set, matching real X servers - the
/// spec doesn't define an order between them, but only one is ever sent in
/// practice.
fn decode_background(value_mask: u32, value_list: &[u32]) -> Background {
    if let Some(pixel) = value_list_entry(value_mask, value_list, value_mask::BACKGROUND_PIXEL) {
        return Background::Pixel(pixel);
    }
    if let Some(pixmap) = value_list_entry(value_mask, value_list, value_mask::BACKGROUND_PIXMAP) {
        return match PixmapValue::from_u32(pixmap) {
            PixmapValue::None => Background::None,
            PixmapValue::ParentRelative => Background::ParentRelative,
            PixmapValue::Id(id) => Background::Pixmap(id),
        };
    }
    // Neither bit set: spec default for CreateWindow is background None.
    Background::None
}

mod gc_value_mask {
    pub const FUNCTION: u32 = 0x00000001;
    pub const PLANE_MASK: u32 = 0x00000002;
    pub const FOREGROUND: u32 = 0x00000004;
    pub const BACKGROUND: u32 = 0x00000008;
    pub const LINE_WIDTH: u32 = 0x00000010;
    pub const LINE_STYLE: u32 = 0x00000020;
    pub const CAP_STYLE: u32 = 0x00000040;
    pub const JOIN_STYLE: u32 = 0x00000080;
    pub const FILL_STYLE: u32 = 0x00000100;
    pub const FILL_RULE: u32 = 0x00000200;
    pub const TILE: u32 = 0x00000400;
    pub const STIPPLE: u32 = 0x00000800;
    pub const TILE_STIPPLE_X_ORIGIN: u32 = 0x00001000;
    pub const TILE_STIPPLE_Y_ORIGIN: u32 = 0x00002000;
    pub const FONT: u32 = 0x00004000;
    pub const SUBWINDOW_MODE: u32 = 0x00008000;
    pub const GRAPHICS_EXPOSURES: u32 = 0x00010000;
    pub const CLIP_X_ORIGIN: u32 = 0x00020000;
    pub const CLIP_Y_ORIGIN: u32 = 0x00040000;
    pub const CLIP_MASK: u32 = 0x00080000;
    pub const DASH_OFFSET: u32 = 0x00100000;
    pub const DASHES: u32 = 0x00200000;
    pub const ARC_MODE: u32 = 0x00400000;
}

fn apply_gc_values(
    gc: &mut GraphicsContext,
    value_mask: u32,
    value_list: &[u32],
) -> HandlerResult<()> {
    let mut values = value_list.iter().copied();

    macro_rules! apply_if_set {
        ($bit:expr, $body:expr) => {
            if value_mask & $bit != 0 {
                let value = values.next().ok_or_else(|| {
                    X11Error::Protocol("GC value-list ended before value-mask was satisfied".into())
                })?;
                ($body)(value);
            }
        };
    }

    apply_if_set!(gc_value_mask::FUNCTION, |value| gc.function = value as u8);
    apply_if_set!(gc_value_mask::PLANE_MASK, |value| gc.plane_mask = value);
    apply_if_set!(gc_value_mask::FOREGROUND, |value| gc.foreground = value);
    apply_if_set!(gc_value_mask::BACKGROUND, |value| gc.background = value);
    apply_if_set!(gc_value_mask::LINE_WIDTH, |value| gc.line_width =
        value as u16);
    apply_if_set!(gc_value_mask::LINE_STYLE, |value| gc.line_style =
        value as u8);
    apply_if_set!(gc_value_mask::CAP_STYLE, |value| gc.cap_style = value as u8);
    apply_if_set!(gc_value_mask::JOIN_STYLE, |value| gc.join_style =
        value as u8);
    apply_if_set!(gc_value_mask::FILL_STYLE, |value| gc.fill_style =
        value as u8);
    apply_if_set!(gc_value_mask::FILL_RULE, |value| gc.fill_rule = value as u8);
    apply_if_set!(gc_value_mask::TILE, |value| gc.tile = Some(value));
    apply_if_set!(gc_value_mask::STIPPLE, |value| gc.stipple = Some(value));
    apply_if_set!(gc_value_mask::TILE_STIPPLE_X_ORIGIN, |value| {
        gc.tile_stipple_x_origin = value as u16 as i16;
    });
    apply_if_set!(gc_value_mask::TILE_STIPPLE_Y_ORIGIN, |value| {
        gc.tile_stipple_y_origin = value as u16 as i16;
    });
    apply_if_set!(gc_value_mask::FONT, |value| gc.font = Some(value));
    apply_if_set!(gc_value_mask::SUBWINDOW_MODE, |value| gc.subwindow_mode =
        value as u8);
    apply_if_set!(gc_value_mask::GRAPHICS_EXPOSURES, |value| {
        gc.graphics_exposures = value != 0;
    });
    apply_if_set!(gc_value_mask::CLIP_X_ORIGIN, |value| {
        gc.clip_x_origin = value as u16 as i16;
    });
    apply_if_set!(gc_value_mask::CLIP_Y_ORIGIN, |value| {
        gc.clip_y_origin = value as u16 as i16;
    });
    apply_if_set!(gc_value_mask::CLIP_MASK, |value| {
        gc.clip_mask = (value != 0).then_some(value);
    });
    apply_if_set!(gc_value_mask::DASH_OFFSET, |value| gc.dash_offset =
        value as u16);
    apply_if_set!(gc_value_mask::DASHES, |value| gc.dashes = value as u8);
    apply_if_set!(gc_value_mask::ARC_MODE, |value| gc.arc_mode = value as u8);

    Ok(())
}

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

        let background = decode_background(
            create_window_request.value_mask,
            &create_window_request.value_list,
        );

        // If background-pixmap names a real pixmap ID (not None/ParentRelative),
        // it must already exist - CreateWindow errors with Pixmap otherwise.
        if let Background::Pixmap(id) = background {
            if server.get_pixmap(id).is_none() {
                return Err(X11Error::Protocol(format!(
                    "CreateWindow: background pixmap {} does not exist",
                    id
                )));
            }
        }

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
                background,
            )
            .await
            .map_err(|e| X11Error::Protocol(format!("Failed to create window: {}", e)))?;

        // CreateWindow doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (1, None)
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

    fn opcode(&self) -> (u8, Option<u8>) {
        (4, None)
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

    fn opcode(&self) -> (u8, Option<u8>) {
        (8, None)
    }

    fn name(&self) -> &'static str {
        "MapWindow"
    }
}

/// Handler for ClearArea requests (opcode 61)
pub struct ClearAreaHandler;

#[async_trait]
impl RequestHandler for ClearAreaHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let clear_area_request = match &request.kind {
            RequestKind::ClearArea(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for ClearArea: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;

        let window = server
            .get_window(clear_area_request.window)
            .ok_or_else(|| {
                X11Error::Protocol(format!(
                    "ClearArea: window {} does not exist",
                    clear_area_request.window
                ))
            })?;

        if window.class == crate::server::window_system::WindowClass::InputOnly {
            return Err(X11Error::Protocol(format!(
                "ClearArea: window {} is InputOnly (Match error)",
                clear_area_request.window
            )));
        }

        // Resolve ParentRelative up the window tree to a concrete
        // background before clearing - clear_area() itself only knows how
        // to fill None/Pixel/Pixmap.
        let background = server
            .resolve_background(clear_area_request.window)
            .unwrap_or(Background::None);

        let background_pixmap = if let Background::Pixmap(id) = background {
            server.get_pixmap(id).cloned()
        } else {
            None
        };

        let window = server.get_window_mut(clear_area_request.window).unwrap();
        clear_area(
            window,
            clear_area_request.x,
            clear_area_request.y,
            clear_area_request.width,
            clear_area_request.height,
            background,
            background_pixmap.as_ref(),
        );

        if clear_area_request.exposures != 0 {
            let (window_id, width, height) = {
                let window = server.get_window(clear_area_request.window).unwrap();
                (window.id, window.width, window.height)
            };
            server
                .send_expose_event(
                    client_id,
                    window_id,
                    clear_area_request.x,
                    clear_area_request.y,
                    if clear_area_request.width == 0 {
                        width
                    } else {
                        clear_area_request.width
                    },
                    if clear_area_request.height == 0 {
                        height
                    } else {
                        clear_area_request.height
                    },
                    0,
                )
                .await;
        }

        // ClearArea doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (61, None)
    }

    fn name(&self) -> &'static str {
        "ClearArea"
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

    fn opcode(&self) -> (u8, Option<u8>) {
        (10, None)
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

        if let Some(gc) = server.get_gc_mut(create_gc_request.gc) {
            apply_gc_values(
                gc,
                create_gc_request.value_mask,
                &create_gc_request.value_list,
            )?;
        }

        // CreateGC doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (55, None)
    }

    fn name(&self) -> &'static str {
        "CreateGC"
    }
}

/// Handler for ChangeGC requests (opcode 56)
pub struct ChangeGCHandler;

#[async_trait]
impl RequestHandler for ChangeGCHandler {
    async fn handle_request(
        &self,
        client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let change_gc_request = match &request.kind {
            RequestKind::ChangeGC(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for ChangeGC: {:?}",
                    request.kind
                )));
            }
        };

        let mut server = server.lock().await;
        let gc = server.get_gc_mut(change_gc_request.gc).ok_or_else(|| {
            X11Error::Protocol(format!(
                "ChangeGC: graphics context {} does not exist",
                change_gc_request.gc
            ))
        })?;

        if gc.owner != client_id {
            return Err(X11Error::Protocol(format!(
                "ChangeGC: client {} does not own graphics context {}",
                client_id, change_gc_request.gc
            )));
        }

        apply_gc_values(
            gc,
            change_gc_request.value_mask,
            &change_gc_request.value_list,
        )?;

        // ChangeGC doesn't generate a response.
        Ok(None)
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (Opcode::ChangeGC.to_u8(), None)
    }

    fn name(&self) -> &'static str {
        "ChangeGC"
    }
}

/// Handler for PolyArc requests (opcode 68)
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

    fn opcode(&self) -> (u8, Option<u8>) {
        (68, None)
    }

    fn name(&self) -> &'static str {
        "PolyArc"
    }
}

/// Handler for FillArc requests (opcode 71, PolyFillArc)
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

        // The drawable is a DRAWABLE (xproto encoding.xml types:DRAWABLE):
        // either a window or a pixmap ID - resolve by trying both, same as
        // PolyFillRectangleHandler.
        let drawable_id = fill_arc_request.drawable;
        let gc_id = fill_arc_request.gc;

        enum Target {
            Window,
            Pixmap,
        }

        let target = if let Some(window) = server.get_window(drawable_id) {
            if window.owner != Some(client_id) {
                return Err(X11Error::Protocol(format!(
                    "FillArc: client {} does not own window {}",
                    client_id, drawable_id
                )));
            }
            Target::Window
        } else if let Some(pixmap) = server.get_pixmap(drawable_id) {
            if pixmap.owner != client_id {
                return Err(X11Error::Protocol(format!(
                    "FillArc: client {} does not own pixmap {}",
                    client_id, drawable_id
                )));
            }
            Target::Pixmap
        } else {
            return Err(X11Error::Protocol(format!(
                "FillArc: drawable {} does not exist",
                drawable_id
            )));
        };

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

        // Fill arcs on whichever surface the drawable resolved to.
        match target {
            Target::Window => {
                let window = server.get_window_mut(drawable_id).unwrap();
                for arc in &fill_arc_request.arcs {
                    fill_arc(window, arc, gc_foreground);
                }
            }
            Target::Pixmap => {
                let pixmap = server.get_pixmap_mut(drawable_id).unwrap();
                for arc in &fill_arc_request.arcs {
                    fill_arc(pixmap, arc, gc_foreground);
                }
            }
        }

        // FillArc doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (71, None)
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

    fn opcode(&self) -> (u8, Option<u8>) {
        (65, None)
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

        // The drawable is a DRAWABLE (xproto encoding.xml types:DRAWABLE):
        // either a window or a pixmap ID, and the ID spaces don't overlap,
        // so which one it is has to be resolved by trying both.
        let drawable_id = poly_fill_rect_request.drawable;
        let gc_id = poly_fill_rect_request.gc;

        enum Target {
            Window,
            Pixmap,
        }

        let target = if let Some(window) = server.get_window(drawable_id) {
            if window.owner != Some(client_id) {
                return Err(X11Error::Protocol(format!(
                    "PolyFillRectangle: client {} does not own window {}",
                    client_id, drawable_id
                )));
            }
            Target::Window
        } else if let Some(pixmap) = server.get_pixmap(drawable_id) {
            if pixmap.owner != client_id {
                return Err(X11Error::Protocol(format!(
                    "PolyFillRectangle: client {} does not own pixmap {}",
                    client_id, drawable_id
                )));
            }
            Target::Pixmap
        } else {
            return Err(X11Error::Protocol(format!(
                "PolyFillRectangle: drawable {} does not exist",
                drawable_id
            )));
        };

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

        // Fill rectangles on whichever surface the drawable resolved to.
        match target {
            Target::Window => {
                let window = server.get_window_mut(drawable_id).unwrap();
                for rect in &poly_fill_rect_request.rectangles {
                    fill_rectangle(window, rect, gc_foreground);
                }
            }
            Target::Pixmap => {
                let pixmap = server.get_pixmap_mut(drawable_id).unwrap();
                for rect in &poly_fill_rect_request.rectangles {
                    fill_rectangle(pixmap, rect, gc_foreground);
                }
            }
        }

        // PolyFillRectangle doesn't generate a response
        Ok(None)
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (70, None)
    }

    fn name(&self) -> &'static str {
        "PolyFillRectangle"
    }
}

/// Convenience function to create a registry with standard handlers.
///
/// `extensions` supplies this session's dynamically assigned extension major
/// opcodes (see `ExtensionRegistry`) so extension handlers register under the
/// same opcode `parse_dynamic` will route requests to.
pub fn create_standard_handler_registry(
    extensions: &crate::protocol::ExtensionRegistry,
) -> crate::protocol::RequestHandlerRegistry {
    let mut registry = crate::protocol::RequestHandlerRegistry::new();

    // Window management handlers
    registry.register_handler(CreateWindowHandler);
    registry.register_handler(DestroyWindowHandler);
    registry.register_handler(MapWindowHandler);
    registry.register_handler(ClearAreaHandler);
    registry.register_handler(UnmapWindowHandler);
    registry.register_handler(GetGeometryHandler);
    registry.register_handler(CreateGCHandler);
    registry.register_handler(ChangeGCHandler);

    registry.register_handler(PolyArcHandler);
    registry.register_handler(FillArcHandler);
    registry.register_handler(PolyLineHandler);
    registry.register_handler(PolyFillRectangleHandler);

    registry.register_handler(InternAtomHandler);
    registry.register_handler(ChangePropertyHandler);
    registry.register_handler(GetPropertyHandler);
    registry.register_handler(QueryColorsHandler);
    registry.register_handler(CreatePixmapHandler);
    registry.register_handler(FreePixmapHandler);
    registry.register_handler(GetInputFocusHandler);
    registry.register_handler(PutImageHandler);
    registry.register_handler(CopyAreaHandler);
    registry.register_handler(OpenFontHandler);
    registry.register_handler(CreateGlyphCursorHandler);
    registry.register_handler(GrabPointerHandler);
    registry.register_handler(QueryExtensionHandler);

    if let Some(major) = extensions.major_opcode("BIG-REQUESTS") {
        registry.register_handler(BigRequestsHandler::new(major));
    }

    if let Some(major) = extensions.major_opcode("RANDR") {
        registry.register_handler(RandrQueryVersionHandler::new(major));
        registry.register_handler(RandrGetScreenResourcesHandler::new(major));
        registry.register_handler(RandrGetOutputInfoHandler::new(major));
        registry.register_handler(RandrGetCrtcInfoHandler::new(major));
        registry.register_handler(RandrGetScreenSizeRangeHandler::new(major));
    }

    if let Some(major) = extensions.major_opcode("RENDER") {
        registry.register_handler(RenderQueryVersionHandler::new(major));
        registry.register_handler(RenderCreateSolidFillHandler::new(major));
    }

    registry
}
