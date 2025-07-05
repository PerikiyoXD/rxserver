use crate::protocol::{ByteOrderReader, types::ByteOrder};
use anyhow::Result;

use super::types::*;

/// Macro to convert ByteOrderReader errors to anyhow::Error
macro_rules! read_or_err {
    ($reader:expr, $method:ident) => {
        $reader.$method().map_err(|e| anyhow::anyhow!(e))?
    };
}

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
    /// The opcode this parser handles
    const OPCODE: u8;

    /// Parse a byte slice into a Request
    fn parse(bytes: &[u8]) -> Result<Request>;

    /// Validate the request structure
    fn validate(request: &Request) -> Result<()>;
}

/// Individual parser implementations for each request type
pub struct GetGeometryParser;
pub struct InternAtomParser;
pub struct CreateWindowParser;
pub struct DestroyWindowParser;
pub struct MapWindowParser;
pub struct UnmapWindowParser;
pub struct CreateGlyphCursorParser;
pub struct OpenFontParser;
pub struct NoOperationParser;

impl RequestParser for GetGeometryParser {
    const OPCODE: u8 = opcodes::GET_GEOMETRY;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 8 {
            return Err(anyhow::anyhow!("GetGeometry request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let drawable = read_or_err!(reader, read_u32);

        let request = GetGeometryRequest {
            opcode,
            unused,
            length,
            drawable,
        };

        Ok(Request {
            kind: RequestKind::GetGeometry(request),
            sequence_number: 0, // Will be set by connection handler
            opcode,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::GetGeometry(req) => {
                if req.drawable == 0 {
                    return Err(anyhow::anyhow!("GetGeometry: drawable must be non-zero"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Invalid request type for GetGeometryParser"
            )),
        }
    }
}

impl RequestParser for InternAtomParser {
    const OPCODE: u8 = opcodes::INTERN_ATOM;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 8 {
            return Err(anyhow::anyhow!("InternAtom request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let only_if_exists = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let name_len = read_or_err!(reader, read_u16);
        let unused = read_or_err!(reader, read_u16);

        let atom_name = if name_len > 0 {
            let name_bytes = reader
                .read_bytes(name_len as usize)
                .map_err(|e| anyhow::anyhow!(e))?;
            String::from_utf8_lossy(name_bytes).to_string()
        } else {
            String::new()
        };

        let request = InternAtomRequest {
            opcode,
            only_if_exists,
            length,
            name_len,
            unused,
            atom_name,
        };

        Ok(Request {
            kind: RequestKind::InternAtom(request),
            sequence_number: 0,
            opcode,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::InternAtom(req) => {
                if req.atom_name.is_empty() {
                    return Err(anyhow::anyhow!("InternAtom: atom_name must not be empty"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid request type for InternAtomParser")),
        }
    }
}

impl RequestParser for CreateWindowParser {
    const OPCODE: u8 = opcodes::CREATE_WINDOW;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 32 {
            return Err(anyhow::anyhow!("CreateWindow request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let depth = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let wid = read_or_err!(reader, read_u32);
        let parent = read_or_err!(reader, read_u32);
        let x = read_or_err!(reader, read_i16);
        let y = read_or_err!(reader, read_i16);
        let width = read_or_err!(reader, read_u16);
        let height = read_or_err!(reader, read_u16);
        let border_width = read_or_err!(reader, read_u16);
        let class = read_or_err!(reader, read_u16);
        let visual = read_or_err!(reader, read_u32);
        let value_mask = read_or_err!(reader, read_u32);

        // Parse variable length value list
        let mut value_list = Vec::new();
        let remaining_bytes = (length as usize * 4).saturating_sub(32);
        if remaining_bytes > 0 {
            let values_count = remaining_bytes / 4;
            for _ in 0..values_count {
                value_list.push(read_or_err!(reader, read_u32));
            }
        }

        let request = CreateWindowRequest {
            opcode,
            depth,
            length,
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
        };

        Ok(Request {
            kind: RequestKind::CreateWindow(request),
            sequence_number: 0,
            opcode,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::CreateWindow(req) => {
                if req.width == 0 || req.height == 0 {
                    return Err(anyhow::anyhow!(
                        "CreateWindow: width and height must be non-zero"
                    ));
                }
                if req.depth == 0 {
                    return Err(anyhow::anyhow!("CreateWindow: depth must be non-zero"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Invalid request type for CreateWindowParser"
            )),
        }
    }
}

impl RequestParser for DestroyWindowParser {
    const OPCODE: u8 = opcodes::DESTROY_WINDOW;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 8 {
            return Err(anyhow::anyhow!("DestroyWindow request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let window = read_or_err!(reader, read_u32);

        let request = DestroyWindowRequest {
            opcode,
            unused,
            length,
            window,
        };

        Ok(Request {
            kind: RequestKind::DestroyWindow(request),
            sequence_number: 0,
            opcode,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::DestroyWindow(req) => {
                if req.window == 0 {
                    return Err(anyhow::anyhow!("DestroyWindow: window id must be non-zero"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Invalid request type for DestroyWindowParser"
            )),
        }
    }
}

impl RequestParser for MapWindowParser {
    const OPCODE: u8 = opcodes::MAP_WINDOW;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 8 {
            return Err(anyhow::anyhow!("MapWindow request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let window = read_or_err!(reader, read_u32);

        let request = MapWindowRequest {
            opcode,
            unused,
            length,
            window,
        };

        Ok(Request {
            kind: RequestKind::MapWindow(request),
            sequence_number: 0,
            opcode,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::MapWindow(req) => {
                if req.window == 0 {
                    return Err(anyhow::anyhow!("MapWindow: window id must be non-zero"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid request type for MapWindowParser")),
        }
    }
}

impl RequestParser for UnmapWindowParser {
    const OPCODE: u8 = opcodes::UNMAP_WINDOW;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 8 {
            return Err(anyhow::anyhow!("UnmapWindow request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let window = read_or_err!(reader, read_u32);

        let request = UnmapWindowRequest {
            opcode,
            unused,
            length,
            window,
        };

        Ok(Request {
            kind: RequestKind::UnmapWindow(request),
            sequence_number: 0,
            opcode,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::UnmapWindow(req) => {
                if req.window == 0 {
                    return Err(anyhow::anyhow!("UnmapWindow: window id must be non-zero"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Invalid request type for UnmapWindowParser"
            )),
        }
    }
}

impl RequestParser for CreateGlyphCursorParser {
    const OPCODE: u8 = opcodes::CREATE_GLYPH_CURSOR;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 32 {
            return Err(anyhow::anyhow!("CreateGlyphCursor request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let cid = read_or_err!(reader, read_u32);
        let source_font = read_or_err!(reader, read_u32);
        let mask_font = read_or_err!(reader, read_u32);
        let source_char = read_or_err!(reader, read_u16);
        let mask_char = read_or_err!(reader, read_u16);
        let fore_red = read_or_err!(reader, read_u16);
        let fore_green = read_or_err!(reader, read_u16);
        let fore_blue = read_or_err!(reader, read_u16);
        let back_red = read_or_err!(reader, read_u16);
        let back_green = read_or_err!(reader, read_u16);
        let back_blue = read_or_err!(reader, read_u16);

        let request = CreateGlyphCursorRequest {
            opcode,
            unused,
            length,
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
        };

        Ok(Request {
            kind: RequestKind::CreateGlyphCursor(request),
            sequence_number: 0,
            opcode,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::CreateGlyphCursor(req) => {
                if req.cid == 0 {
                    return Err(anyhow::anyhow!(
                        "CreateGlyphCursor: cursor id must be non-zero"
                    ));
                }
                if req.source_font == 0 {
                    return Err(anyhow::anyhow!(
                        "CreateGlyphCursor: source font must be non-zero"
                    ));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Invalid request type for CreateGlyphCursorParser"
            )),
        }
    }
}

impl RequestParser for OpenFontParser {
    const OPCODE: u8 = opcodes::OPEN_FONT;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 12 {
            return Err(anyhow::anyhow!("OpenFont request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let fid = read_or_err!(reader, read_u32);
        let name_len = read_or_err!(reader, read_u16);
        let unused2 = read_or_err!(reader, read_u16);

        let name = if name_len > 0 {
            let name_bytes = reader
                .read_bytes(name_len as usize)
                .map_err(|e| anyhow::anyhow!(e))?;
            String::from_utf8_lossy(name_bytes).to_string()
        } else {
            String::new()
        };

        let request = OpenFontRequest {
            opcode,
            unused,
            length,
            fid,
            name_len,
            unused2,
            name,
        };

        Ok(Request {
            kind: RequestKind::OpenFont(request),
            sequence_number: 0,
            opcode,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::OpenFont(req) => {
                if req.fid == 0 {
                    return Err(anyhow::anyhow!("OpenFont: font id must be non-zero"));
                }
                if req.name.is_empty() {
                    return Err(anyhow::anyhow!("OpenFont: font name must not be empty"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid request type for OpenFontParser")),
        }
    }
}

impl RequestParser for NoOperationParser {
    const OPCODE: u8 = opcodes::NO_OPERATION;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 4 {
            return Err(anyhow::anyhow!("NoOperation request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);

        let request = NoOperationRequest {
            opcode,
            unused,
            length,
        };

        Ok(Request {
            kind: RequestKind::NoOperation(request),
            sequence_number: 0,
            opcode,
        })
    }

    fn validate(_request: &Request) -> Result<()> {
        // NoOperation requests are always valid
        Ok(())
    }
}

/// Main dispatcher parser that routes to specific parsers based on opcode
pub struct X11RequestParser;

impl RequestParser for X11RequestParser {
    const OPCODE: u8 = 0; // Dispatcher doesn't have a specific opcode

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.is_empty() {
            return Err(anyhow::anyhow!("Request too short"));
        }

        let opcode = bytes[0];
        match opcode {
            opcodes::GET_GEOMETRY => GetGeometryParser::parse(bytes),
            opcodes::INTERN_ATOM => InternAtomParser::parse(bytes),
            opcodes::CREATE_WINDOW => CreateWindowParser::parse(bytes),
            opcodes::DESTROY_WINDOW => DestroyWindowParser::parse(bytes),
            opcodes::MAP_WINDOW => MapWindowParser::parse(bytes),
            opcodes::UNMAP_WINDOW => UnmapWindowParser::parse(bytes),
            opcodes::CREATE_GLYPH_CURSOR => CreateGlyphCursorParser::parse(bytes),
            opcodes::OPEN_FONT => OpenFontParser::parse(bytes),
            opcodes::NO_OPERATION => NoOperationParser::parse(bytes),
            _ => Err(anyhow::anyhow!("Unknown opcode: {}", opcode)),
        }
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::GetGeometry(_) => GetGeometryParser::validate(request),
            RequestKind::InternAtom(_) => InternAtomParser::validate(request),
            RequestKind::CreateWindow(_) => CreateWindowParser::validate(request),
            RequestKind::DestroyWindow(_) => DestroyWindowParser::validate(request),
            RequestKind::MapWindow(_) => MapWindowParser::validate(request),
            RequestKind::UnmapWindow(_) => UnmapWindowParser::validate(request),
            RequestKind::CreateGlyphCursor(_) => CreateGlyphCursorParser::validate(request),
            RequestKind::OpenFont(_) => OpenFontParser::validate(request),
            RequestKind::NoOperation(_) => NoOperationParser::validate(request),
            RequestKind::ConnectionSetup => Ok(()), // Connection setup is handled separately
        }
    }
}
