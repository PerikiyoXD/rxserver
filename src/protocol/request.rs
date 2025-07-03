use anyhow::Result;
use crate::protocol::types::ByteOrder;

use crate::EndianReader;

use super::types::*;

/// X11 protocol opcodes
pub mod opcodes {
    pub const CONNECTION_SETUP: u8 = 0;
    pub const CREATE_WINDOW: u8 = 1;
    pub const DESTROY_WINDOW: u8 = 4;
    pub const MAP_WINDOW: u8 = 8;
    pub const UNMAP_WINDOW: u8 = 10;
    pub const GET_GEOMETRY: u8 = 14;
    pub const INTERN_ATOM: u8 = 16;
    pub const OPEN_FONT: u8 = 45;
    pub const CREATE_GLYPH_CURSOR: u8 = 94;
    pub const NO_OPERATION: u8 = 127;
}

#[derive(Debug, Clone)]
pub enum RequestKind {
    ConnectionSetup,
    GetGeometry(GetGeometryRequest),
    InternAtom(InternAtomRequest),
    CreateWindow(CreateWindowRequest),
    DestroyWindow(DestroyWindowRequest),
    MapWindow(MapWindowRequest),
    UnmapWindow(UnmapWindowRequest),
    CreateGlyphCursor(CreateGlyphCursorRequest),
    OpenFont(OpenFontRequest),
    NoOperation(NoOperationRequest),
}

#[derive(Debug, Clone)]
pub struct Request {
    pub kind: RequestKind,
    pub sequence_number: SequenceNumber,
    pub opcode: u8,
}

/// GetGeometry request structure matching X11 protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetGeometryRequest {
    pub opcode: u8,  // Should be 14
    pub unused: u8,  // Padding
    pub length: u16, // Request length in 4-byte units (always 2)
    pub drawable: DrawableId,
}

/// InternAtom request structure matching X11 protocol
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternAtomRequest {
    pub opcode: u8,         // Should be 16
    pub only_if_exists: u8, // BOOL
    pub length: u16,        // Request length in 4-byte units
    pub name_len: u16,      // Length of name
    pub unused: u16,        // Padding
    pub atom_name: String,  // Atom name
}

/// CreateWindow request structure matching X11 protocol
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateWindowRequest {
    pub opcode: u8,           // 1: opcode (1)
    pub depth: u8,            // 1: depth
    pub length: u16,          // 2: request length in 4-byte units (8+n)
    pub wid: WindowId,        // 4: window id
    pub parent: WindowId,     // 4: parent window id
    pub x: i16,               // 2: x position
    pub y: i16,               // 2: y position
    pub width: u16,           // 2: width
    pub height: u16,          // 2: height
    pub border_width: u16,    // 2: border width
    pub class: u16,           // 2: window class (see WindowClass enum)
    pub visual: VisualId,     // 4: visual id (0 = CopyFromParent)
    pub value_mask: u32,      // 4: value mask
    pub value_list: Vec<u32>, // 4n: variable length value list
}

/// DestroyWindow request structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DestroyWindowRequest {
    pub opcode: u8,       // opcode (4)
    pub unused: u8,       // unused
    pub length: u16,      // request length (2)
    pub window: WindowId, // window to destroy
}

/// MapWindow request structure  
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapWindowRequest {
    pub opcode: u8,       // opcode (8)
    pub unused: u8,       // unused
    pub length: u16,      // request length (2)
    pub window: WindowId, // window to map
}

/// UnmapWindow request structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnmapWindowRequest {
    pub opcode: u8,       // opcode (10)
    pub unused: u8,       // unused
    pub length: u16,      // request length (2)
    pub window: WindowId, // window to unmap
}

/// CreateGlyphCursor request structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateGlyphCursorRequest {
    pub opcode: u8,          // opcode (94)
    pub unused: u8,          // unused
    pub length: u16,         // request length
    pub cid: CursorId,       // cursor id
    pub source_font: FontId, // source font
    pub mask_font: FontId,   // mask font (0 = None)
    pub source_char: u16,    // source character
    pub mask_char: u16,      // mask character
    pub fore_red: u16,       // foreground red
    pub fore_green: u16,     // foreground green
    pub fore_blue: u16,      // foreground blue
    pub back_red: u16,       // background red
    pub back_green: u16,     // background green
    pub back_blue: u16,      // background blue
}

/// OpenFont request structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenFontRequest {
    pub opcode: u8,    // opcode (45)
    pub unused: u8,    // unused
    pub length: u16,   // request length
    pub fid: FontId,   // font id
    pub name_len: u16, // length of name
    pub unused2: u16,  // padding
    pub name: String,  // font name
}

/// NoOperation request structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoOperationRequest {
    pub opcode: u8,  // opcode (127)
    pub unused: u8,  // unused
    pub length: u16, // request length
}

pub trait RequestParser {
    /// Parse a byte slice into a Request
    fn parse(bytes: &[u8]) -> Result<Request>;

    /// Get the opcode for this request type
    fn opcode(&self) -> u8;

    /// Validate the request structure
    fn validate(&self) -> Result<()>;
}

pub struct X11RequestParser;

impl RequestParser for X11RequestParser {
    fn parse(bytes: &[u8]) -> Result<Request> {
        let _reader = EndianReader::new(bytes, ByteOrder::LittleEndian);
        // TODO: Implement actual parsing logic
        todo!("Implement request parsing")
    }

    fn opcode(&self) -> u8 {
        // Return the opcode for this request type
        0 // TODO: Implement proper opcode logic
    }

    fn validate(&self) -> Result<()> {
        // Implement request validation logic here
        Ok(())
    }
}
