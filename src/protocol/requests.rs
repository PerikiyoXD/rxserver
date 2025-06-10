//! X11 Protocol Request Handling
//!
//! This module defines all X11 protocol requests in a clean, type-safe manner.
//! Each request is represented as a Rust struct with proper validation.

use crate::protocol::opcodes;
use crate::protocol::types::*;
use crate::{todo_high, todo_medium, Result};
use bytes::{Buf, Bytes};
use std::fmt;

/// X11 protocol request header
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestHeader {
    pub opcode: u8,
    pub minor_opcode: u8,
    pub length: u16,
}

/// All possible X11 requests
#[derive(Debug, Clone)]
pub enum Request {
    CreateWindow(CreateWindowRequest),
    ChangeWindowAttributes(ChangeWindowAttributesRequest),
    GetWindowAttributes(GetWindowAttributesRequest),
    DestroyWindow(DestroyWindowRequest),
    MapWindow(MapWindowRequest),
    UnmapWindow(UnmapWindowRequest),
    ConfigureWindow(ConfigureWindowRequest),
    CreateGC(CreateGCRequest),
    ClearArea(ClearAreaRequest),
    CopyArea(CopyAreaRequest),
    // Unknown request for unimplemented opcodes
    Unknown { opcode: u8, data: bytes::Bytes },
}

/// Create Window request
#[derive(Debug, Clone)]
pub struct CreateWindowRequest {
    pub depth: u8,
    pub wid: Window,
    pub parent: Window,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub class: WindowClass,
    pub visual: VisualId,
    pub value_mask: WindowAttributesMask,
    pub value_list: WindowAttributes,
}

/// Window attributes for CreateWindow
#[derive(Debug, Clone, Default)]
pub struct WindowAttributes {
    pub background_pixmap: Option<Pixmap>,
    pub background_pixel: Option<u32>,
    pub border_pixmap: Option<Pixmap>,
    pub border_pixel: Option<u32>,
    pub bit_gravity: Option<Gravity>,
    pub win_gravity: Option<Gravity>,
    pub backing_store: Option<u8>,
    pub backing_planes: Option<u32>,
    pub backing_pixel: Option<u32>,
    pub override_redirect: Option<bool>,
    pub save_under: Option<bool>,
    pub event_mask: Option<EventMask>,
    pub do_not_propagate_mask: Option<EventMask>,
    pub colormap: Option<Colormap>,
    pub cursor: Option<Cursor>,
}

/// Change Window Attributes request
#[derive(Debug, Clone)]
pub struct ChangeWindowAttributesRequest {
    pub window: Window,
    pub value_mask: WindowAttributesMask,
    pub value_list: WindowAttributes,
}

/// Get Window Attributes request
#[derive(Debug, Clone)]
pub struct GetWindowAttributesRequest {
    pub window: Window,
}

/// Destroy Window request
#[derive(Debug, Clone)]
pub struct DestroyWindowRequest {
    pub window: Window,
}

/// Map Window request
#[derive(Debug, Clone)]
pub struct MapWindowRequest {
    pub window: Window,
}

/// Unmap Window request
#[derive(Debug, Clone)]
pub struct UnmapWindowRequest {
    pub window: Window,
}

/// Configure Window request
#[derive(Debug, Clone)]
pub struct ConfigureWindowRequest {
    pub window: Window,
    pub value_mask: u16,
    pub x: Option<i16>,
    pub y: Option<i16>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub border_width: Option<u16>,
    pub sibling: Option<Window>,
    pub stack_mode: Option<u8>,
}

/// Create Graphics Context request
#[derive(Debug, Clone)]
pub struct CreateGCRequest {
    pub cid: GContext,
    pub drawable: Drawable,
    pub value_mask: u32,
    pub value_list: Vec<u32>,
}

/// Clear Area request
#[derive(Debug, Clone)]
pub struct ClearAreaRequest {
    pub exposures: bool,
    pub window: Window,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

/// Copy Area request
#[derive(Debug, Clone)]
pub struct CopyAreaRequest {
    pub src_drawable: Drawable,
    pub dst_drawable: Drawable,
    pub gc: GContext,
    pub src_x: i16,
    pub src_y: i16,
    pub dst_x: i16,
    pub dst_y: i16,
    pub width: u16,
    pub height: u16,
}

/// Request parser for incoming X11 protocol data
pub struct RequestParser;

impl RequestParser {
    /// Parse a request from bytes
    pub fn parse(data: &[u8]) -> Result<Request> {
        if data.len() < 4 {
            return Err(crate::Error::Protocol("Request too short".to_string()));
        }

        let opcode = data[0];
        let _minor_opcode = data[1];
        let length = u16::from_ne_bytes([data[2], data[3]]);

        // Ensure we have enough data for the full request
        let expected_bytes = (length as usize) * 4;
        if data.len() < expected_bytes {
            return Err(crate::Error::Protocol(format!(
                "Incomplete request: expected {} bytes, got {}",
                expected_bytes,
                data.len()
            )));
        }
        match opcode {
            opcodes::window::CREATE_WINDOW => Self::parse_create_window(&data[4..]),
            opcodes::window::MAP_WINDOW => Self::parse_map_window(&data[4..]),
            opcodes::window::UNMAP_WINDOW => Self::parse_unmap_window(&data[4..]),
            opcodes::graphics::CLEAR_AREA => Self::parse_clear_area(&data[4..]),
            _ => {
                todo_high!(
                    "request_parsing",
                    "Unknown opcode {} not implemented",
                    opcode
                );
                Ok(Request::Unknown {
                    opcode,
                    data: Bytes::copy_from_slice(data),
                })
            }
        }
    }
    fn parse_create_window(data: &[u8]) -> Result<Request> {
        todo_high!(
            "request_parsing",
            "CreateWindow parsing is incomplete - using placeholder values"
        );

        if data.len() < 28 {
            return Err(crate::Error::Protocol(
                "CreateWindow request too short".to_string(),
            ));
        }

        let mut buf = data;
        let depth = buf.get_u8();
        let wid = buf.get_u32();
        let parent = buf.get_u32();
        let x = buf.get_i16();
        let y = buf.get_i16();
        let width = buf.get_u16();
        let height = buf.get_u16();
        let border_width = buf.get_u16();
        let class = match buf.get_u16() {
            0 => WindowClass::CopyFromParent,
            1 => WindowClass::InputOutput,
            2 => WindowClass::InputOnly,
            _ => return Err(crate::Error::Protocol("Invalid window class".to_string())),
        };
        let visual = buf.get_u32();
        let value_mask = WindowAttributesMask::from_bits_truncate(buf.get_u32());

        // TODO: Parse value list based on mask - currently using defaults
        todo_high!(
            "request_parsing",
            "CreateWindow value_list parsing not implemented"
        );
        let value_list = WindowAttributes::default();

        Ok(Request::CreateWindow(CreateWindowRequest {
            depth,
            wid,
            parent,
            x,
            y,
            width,
            height,
            border_width,
            class,
            visual,
            value_mask,
            value_list,
        }))
    }
    fn parse_map_window(data: &[u8]) -> Result<Request> {
        todo_high!(
            "request_parsing",
            "MapWindow parsing is basic - needs validation"
        );

        if data.len() < 4 {
            return Err(crate::Error::Protocol(
                "MapWindow request too short".to_string(),
            ));
        }

        let mut buf = data;
        let window = buf.get_u32();

        Ok(Request::MapWindow(MapWindowRequest { window }))
    }

    fn parse_unmap_window(data: &[u8]) -> Result<Request> {
        todo_high!(
            "request_parsing",
            "UnmapWindow parsing is basic - needs validation"
        );

        if data.len() < 4 {
            return Err(crate::Error::Protocol(
                "UnmapWindow request too short".to_string(),
            ));
        }

        let mut buf = data;
        let window = buf.get_u32();

        Ok(Request::UnmapWindow(UnmapWindowRequest { window }))
    }

    fn parse_clear_area(data: &[u8]) -> Result<Request> {
        todo_high!(
            "request_parsing",
            "ClearArea parsing is basic - needs validation"
        );

        if data.len() < 12 {
            return Err(crate::Error::Protocol(
                "ClearArea request too short".to_string(),
            ));
        }

        let mut buf = data;
        let exposures = buf.get_u8() != 0;
        let window = buf.get_u32();
        let x = buf.get_i16();
        let y = buf.get_i16();
        let width = buf.get_u16();
        let height = buf.get_u16();

        Ok(Request::ClearArea(ClearAreaRequest {
            exposures,
            window,
            x,
            y,
            width,
            height,
        }))
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Request::CreateWindow(req) => write!(
                f,
                "CreateWindow(wid={}, parent={}, {}x{}+{}+{})",
                req.wid, req.parent, req.width, req.height, req.x, req.y
            ),
            Request::MapWindow(req) => write!(f, "MapWindow({})", req.window),
            Request::UnmapWindow(req) => write!(f, "UnmapWindow({})", req.window),
            Request::ClearArea(req) => write!(
                f,
                "ClearArea(window={}, {}x{}+{}+{})",
                req.window, req.width, req.height, req.x, req.y
            ),
            _ => write!(f, "{:?}", self),
        }
    }
}
