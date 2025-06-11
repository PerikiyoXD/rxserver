//! X11 Protocol Request Handling
//!
//! This module defines all X11 protocol requests in a clean, type-safe manner.
//! Each request is represented as a Rust struct with proper validation.

use crate::protocol::opcodes;
use crate::protocol::types::*;
use crate::{todo_high, Result};
use bytes::{Buf, Bytes};
use std::fmt;
use tracing::debug;

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
    InternAtom(InternAtomRequest),
    OpenFont(OpenFontRequest),
    CreateGlyphCursor(CreateGlyphCursorRequest),
    GrabPointer(GrabPointerRequest),
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

/// InternAtom request
#[derive(Debug, Clone)]
pub struct InternAtomRequest {
    pub only_if_exists: bool,
    pub name: String,
}

/// OpenFont request
#[derive(Debug, Clone)]
pub struct OpenFontRequest {
    pub fid: Font,
    pub name: String,
}

/// CreateGlyphCursor request
#[derive(Debug, Clone)]
pub struct CreateGlyphCursorRequest {
    pub cid: Cursor,
    pub source_font: Font,
    pub mask_font: Font,
    pub source_char: u16,
    pub mask_char: u16,
    pub fore_red: u16,
    pub fore_green: u16,
    pub fore_blue: u16,
    pub back_red: u16,
    pub back_green: u16,
    pub back_blue: u16,
}

/// GrabPointer request
#[derive(Debug, Clone)]
pub struct GrabPointerRequest {
    pub owner_events: bool,
    pub grab_window: Window,
    pub event_mask: u16,
    pub pointer_mode: u8,
    pub keyboard_mode: u8,
    pub confine_to: Window,
    pub cursor: Cursor,
    pub time: u32,
}

/// Request parser for incoming X11 protocol data
pub struct RequestParser;

impl RequestParser {
    /// Parse a request from bytes
    pub fn parse(data: &[u8]) -> Result<Request> {
        debug!("Parsing request from {} bytes", data.len());
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
            opcodes::atom::INTERN_ATOM => Self::parse_intern_atom(data),
            opcodes::text::OPEN_FONT => Self::parse_open_font(data),
            opcodes::cursor::CREATE_GLYPH_CURSOR => Self::parse_create_glyph_cursor(data),
            opcodes::input::GRAB_POINTER => Self::parse_grab_pointer(data),
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
        // UnmapWindow request format:
        // 1     3                    opcode
        // 1                         unused
        // 2     1                   request length
        // 4     WINDOW              window
        if data.len() < 8 {
            return Err(crate::Error::Protocol(
                "UnmapWindow request too short".to_string(),
            ));
        }
        let mut buf = data;
        let window = buf.get_u32();

        Ok(Request::UnmapWindow(UnmapWindowRequest { window }))
    }

    fn parse_clear_area(data: &[u8]) -> Result<Request> {
        // ClearArea request format:
        // 1     14                   opcode
        // 1     BOOL                 exposures
        // 2     4                   request length
        // 4     WINDOW              window
        // 2     CARD16              x
        // 2     CARD16              y
        // 2     CARD16              width
        // 2     CARD16              height

        if data.len() < 20 {
            return Err(crate::Error::Protocol(
                "ClearArea request too short".to_string(),
            ));
        }

        let mut buf = data;
        let _opcode = buf.get_u8(); // opcode (14)
        let exposures = buf.get_u8() != 0; // exposures flag
        let _length = buf.get_u16(); // request length (not used here)
        let window = buf.get_u32(); // window ID
        let x = buf.get_i16(); // x coordinate
        let y = buf.get_i16(); // y coordinate
        let width = buf.get_u16(); // width
        let height = buf.get_u16(); // height

        Ok(Request::ClearArea(ClearAreaRequest {
            exposures,
            window,
            x,
            y,
            width,
            height,
        }))
    }

    fn parse_intern_atom(data: &[u8]) -> Result<Request> {
        // InternAtom request format:
        // 1     16                   opcode
        // 1     BOOL                 only-if-exists
        // 2     2+(n+p)/4           request length
        // 2     n                   length of name
        // 2                         unused
        // n     STRING8             name
        // p                         unused, p=pad(n)

        if data.len() < 8 {
            return Err(crate::Error::Protocol(
                "InternAtom request too short".to_string(),
            ));
        }

        // Use byte array indexing instead of Buf trait to avoid buffer consumption issues
        let _opcode = data[0]; // opcode (16)
        let only_if_exists = data[1] != 0; // only-if-exists flag
        let _length = u16::from_le_bytes([data[2], data[3]]); // request length
        let name_length = u16::from_le_bytes([data[4], data[5]]) as usize; // name length
        let _unused = u16::from_le_bytes([data[6], data[7]]); // unused padding

        // Check if we have enough data for the name
        if data.len() < 8 + name_length {
            return Err(crate::Error::Protocol(format!(
                "InternAtom request name truncated: need {} bytes, have {}",
                8 + name_length,
                data.len()
            )));
        }

        // Extract the name string starting at byte 8
        let name_bytes = &data[8..8 + name_length];
        let name = String::from_utf8(name_bytes.to_vec())
            .map_err(|_| crate::Error::Protocol("Invalid UTF-8 in atom name".to_string()))?;

        debug!(
            "Parsed InternAtom: name='{}', only_if_exists={}",
            name, only_if_exists
        );

        Ok(Request::InternAtom(InternAtomRequest {
            only_if_exists,
            name,
        }))
    }

    fn parse_open_font(data: &[u8]) -> Result<Request> {
        // OpenFont request format:
        // 1     45                   opcode
        // 1                         unused
        // 2     3+(n+p)/4           request length
        // 4     FONT                fid
        // 2     n                   length of name
        // 2                         unused
        // n     STRING8             name
        // p                         unused, p=pad(n)

        if data.len() < 12 {
            return Err(crate::Error::Protocol(
                "OpenFont request too short".to_string(),
            ));
        }

        // Use byte array indexing for proper parsing
        let _opcode = data[0]; // opcode (45)
        let _unused1 = data[1]; // unused
        let _length = u16::from_le_bytes([data[2], data[3]]); // request length
        let fid = u32::from_le_bytes([data[4], data[5], data[6], data[7]]); // font ID
        let name_length = u16::from_le_bytes([data[8], data[9]]) as usize; // name length
        let _unused2 = u16::from_le_bytes([data[10], data[11]]); // unused padding

        // Check if we have enough data for the name
        if data.len() < 12 + name_length {
            return Err(crate::Error::Protocol(format!(
                "OpenFont request name truncated: need {} bytes, have {}",
                12 + name_length,
                data.len()
            )));
        }

        // Extract the name string starting at byte 12
        let name_bytes = &data[12..12 + name_length];
        let name = String::from_utf8(name_bytes.to_vec())
            .map_err(|_| crate::Error::Protocol("Invalid UTF-8 in font name".to_string()))?;

        debug!("Parsed OpenFont: fid={}, name='{}'", fid, name);

        Ok(Request::OpenFont(OpenFontRequest { fid, name }))
    }

    fn parse_create_glyph_cursor(data: &[u8]) -> Result<Request> {
        // CreateGlyphCursor request format:
        // 1     94                   opcode
        // 1                         unused
        // 2     8                   request length
        // 4     CURSOR              cid
        // 4     FONT                source-font
        // 4     FONT                mask-font or None
        // 2     CARD16              source-char
        // 2     CARD16              mask-char
        // 2     CARD16              fore-red
        // 2     CARD16              fore-green
        // 2     CARD16              fore-blue
        // 2     CARD16              back-red
        // 2     CARD16              back-green
        // 2     CARD16              back-blue

        if data.len() < 32 {
            return Err(crate::Error::Protocol(
                "CreateGlyphCursor request too short".to_string(),
            ));
        }

        // Use byte array indexing for proper parsing
        let _opcode = data[0]; // opcode (94)
        let _unused = data[1]; // unused
        let _length = u16::from_le_bytes([data[2], data[3]]); // request length
        let cid = u32::from_le_bytes([data[4], data[5], data[6], data[7]]); // cursor ID
        let source_font = u32::from_le_bytes([data[8], data[9], data[10], data[11]]); // source font
        let mask_font = u32::from_le_bytes([data[12], data[13], data[14], data[15]]); // mask font
        let source_char = u16::from_le_bytes([data[16], data[17]]); // source character
        let mask_char = u16::from_le_bytes([data[18], data[19]]); // mask character
        let fore_red = u16::from_le_bytes([data[20], data[21]]); // foreground red
        let fore_green = u16::from_le_bytes([data[22], data[23]]); // foreground green
        let fore_blue = u16::from_le_bytes([data[24], data[25]]); // foreground blue
        let back_red = u16::from_le_bytes([data[26], data[27]]); // background red
        let back_green = u16::from_le_bytes([data[28], data[29]]); // background green
        let back_blue = u16::from_le_bytes([data[30], data[31]]); // background blue

        debug!(
            "Parsed CreateGlyphCursor: cid={}, source_font={}, mask_font={}, source_char={}, mask_char={}",
            cid, source_font, mask_font, source_char, mask_char
        );

        Ok(Request::CreateGlyphCursor(CreateGlyphCursorRequest {
            cid,
            source_font,
            mask_font,
            source_char,
            mask_char,
            fore_red,
            fore_green,
            fore_blue,
            back_red,
            back_green,
            back_blue,
        }))
    }

    fn parse_grab_pointer(data: &[u8]) -> Result<Request> {
        // GrabPointer request format:
        // 1     26                   opcode
        // 1     BOOL                 owner-events
        // 2     6                   request length
        // 4     WINDOW              grab-window
        // 2     SETofPOINTEREVENT   event-mask
        // 1                        pointer-mode (0=Synchronous, 1=Asynchronous)
        // 1                        keyboard-mode (0=Synchronous, 1=Asynchronous)
        // 4     WINDOW              confine-to (0=None)
        // 4     CURSOR              cursor (0=None)
        // 4     TIMESTAMP           time (0=CurrentTime)

        if data.len() < 24 {
            return Err(crate::Error::Protocol(
                "GrabPointer request too short".to_string(),
            ));
        }

        // Use byte array indexing for proper parsing
        let _opcode = data[0]; // opcode (26)
        let owner_events = data[1] != 0; // owner-events flag
        let _length = u16::from_le_bytes([data[2], data[3]]); // request length
        let grab_window = u32::from_le_bytes([data[4], data[5], data[6], data[7]]); // grab window
        let event_mask = u16::from_le_bytes([data[8], data[9]]); // event mask
        let pointer_mode = data[10]; // pointer mode
        let keyboard_mode = data[11]; // keyboard mode
        let confine_to = u32::from_le_bytes([data[12], data[13], data[14], data[15]]); // confine-to window
        let cursor = u32::from_le_bytes([data[16], data[17], data[18], data[19]]); // cursor
        let time = u32::from_le_bytes([data[20], data[21], data[22], data[23]]); // timestamp

        debug!(
            "Parsed GrabPointer: grab_window={}, owner_events={}, event_mask={}, pointer_mode={}, keyboard_mode={}",
            grab_window, owner_events, event_mask, pointer_mode, keyboard_mode
        );

        Ok(Request::GrabPointer(GrabPointerRequest {
            owner_events,
            grab_window,
            event_mask,
            pointer_mode,
            keyboard_mode,
            confine_to,
            cursor,
            time,
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
            Request::InternAtom(req) => write!(
                f,
                "InternAtom(name='{}', only_if_exists={})",
                req.name, req.only_if_exists
            ),
            Request::OpenFont(req) => write!(f, "OpenFont(fid={}, name='{}')", req.fid, req.name),
            Request::CreateGlyphCursor(req) => write!(
                f,
                "CreateGlyphCursor(cid={}, source_font={}, mask_font={}, source_char={}, mask_char={})",
                req.cid, req.source_font, req.mask_font, req.source_char, req.mask_char
            ),
            _ => write!(f, "{:?}", self),
        }
    }
}
