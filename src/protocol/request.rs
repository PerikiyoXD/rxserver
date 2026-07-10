use crate::protocol::{types::ByteOrder, ByteOrderReader};
use anyhow::Result;
use tracing::trace;

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
    pub const GET_PROPERTY: u8 = 20;
    pub const OPEN_FONT: u8 = 45;
    pub const CREATE_PIXMAP: u8 = 53;
    pub const CREATE_GC: u8 = 55;
    pub const POLY_ARC: u8 = 59;
    pub const COPY_AREA: u8 = 60;
    pub const FILL_ARC: u8 = 61;
    pub const POLY_LINE: u8 = 65;
    pub const POLY_FILL_RECTANGLE: u8 = 70;
    pub const PUT_IMAGE: u8 = 72;
    pub const CREATE_GLYPH_CURSOR: u8 = 94;
    pub const QUERY_EXTENSION: u8 = 98;
    pub const NO_OPERATION: u8 = 127;
    pub const BIG_REQUESTS: u8 = 134;

    // RANDR extension opcodes (major opcode will be assigned dynamically)
    pub const RANDR_QUERY_VERSION: u8 = 0;
    pub const RANDR_GET_SCREEN_RESOURCES: u8 = 1;
    pub const RANDR_GET_OUTPUT_INFO: u8 = 2;
    pub const RANDR_LIST_OUTPUT_PROPERTIES: u8 = 3;
    pub const RANDR_QUERY_OUTPUT_PROPERTY: u8 = 4;
    pub const RANDR_CONFIGURE_OUTPUT_PROPERTY: u8 = 5;
    pub const RANDR_CHANGE_OUTPUT_PROPERTY: u8 = 6;
    pub const RANDR_DELETE_OUTPUT_PROPERTY: u8 = 7;
    pub const RANDR_GET_OUTPUT_PROPERTY: u8 = 8;
    pub const RANDR_CREATE_MODE: u8 = 9;
    pub const RANDR_DESTROY_MODE: u8 = 10;
    pub const RANDR_ADD_OUTPUT_MODE: u8 = 11;
    pub const RANDR_DELETE_OUTPUT_MODE: u8 = 12;
    pub const RANDR_GET_CRTC_INFO: u8 = 13;
    pub const RANDR_SET_CRTC_CONFIG: u8 = 14;
    pub const RANDR_GET_CRTC_GAMMA_SIZE: u8 = 15;
    pub const RANDR_GET_CRTC_GAMMA: u8 = 16;
    pub const RANDR_SET_CRTC_GAMMA: u8 = 17;
    pub const RANDR_GET_SCREEN_SIZE_RANGE: u8 = 18;
    pub const RANDR_SET_SCREEN_SIZE: u8 = 19;
    pub const RANDR_GET_SCREEN_RESOURCES_CURRENT: u8 = 20;
    pub const RANDR_SET_CRTC_TRANSFORM: u8 = 21;
    pub const RANDR_GET_CRTC_TRANSFORM: u8 = 22;
    pub const RANDR_GET_PANNING: u8 = 23;
    pub const RANDR_SET_PANNING: u8 = 24;
    pub const RANDR_SET_OUTPUT_PRIMARY: u8 = 25;
    pub const RANDR_GET_OUTPUT_PRIMARY: u8 = 26;
}

#[derive(Debug, Clone)]
pub enum RequestKind {
    ConnectionSetup,
    GetGeometry(GetGeometryRequest),
    InternAtom(InternAtomRequest),
    GetProperty(GetPropertyRequest),
    CreatePixmap(CreatePixmapRequest),
    PutImage(PutImageRequest),
    CreateWindow(CreateWindowRequest),
    DestroyWindow(DestroyWindowRequest),
    MapWindow(MapWindowRequest),
    UnmapWindow(UnmapWindowRequest),
    CreateGC(CreateGCRequest),
    PolyArc(PolyArcRequest),
    CopyArea(CopyAreaRequest),
    FillArc(FillArcRequest),
    PolyLine(PolyLineRequest),
    PolyFillRectangle(PolyFillRectangleRequest),
    CreateGlyphCursor(CreateGlyphCursorRequest),
    OpenFont(OpenFontRequest),
    GrabPointer(GrabPointerRequest),
    NoOperation(NoOperationRequest),
    QueryExtension(QueryExtensionRequest),
    BigRequests(BigRequestsRequest),
    // RANDR extension requests
    RandrQueryVersion(RandrQueryVersionRequest),
    RandrGetScreenResources(RandrGetScreenResourcesRequest),
    RandrGetOutputInfo(RandrGetOutputInfoRequest),
    RandrGetCrtcInfo(RandrGetCrtcInfoRequest),
    RandrGetScreenSizeRange(RandrGetScreenSizeRangeRequest),
}

#[derive(Debug, Clone)]
pub struct Request {
    pub kind: RequestKind,
    pub sequence_number: SequenceNumber,
    pub opcode: u8,
    pub minor_opcode: Option<u8>, // For extension requests
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

/// GetProperty request structure matching X11 protocol
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetPropertyRequest {
    pub opcode: u8,       // Should be 20
    pub delete: u8,       // BOOL: delete property
    pub length: u16,      // Request length in 4-byte units
    pub window: WindowId, // Window
    pub property: Atom,   // Property atom
    pub r#type: Atom,     // Type atom (0 = AnyType)
    pub long_offset: u32, // Long offset
    pub long_length: u32, // Long length
}

/// CreatePixmap request structure matching X11 protocol
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePixmapRequest {
    pub opcode: u8,           // Should be 53
    pub depth: u8,            // Depth
    pub length: u16,          // Request length in 4-byte units
    pub pid: PixmapId,        // Pixmap ID
    pub drawable: DrawableId, // Drawable
    pub width: u16,           // Width
    pub height: u16,          // Height
}

/// PutImage request structure matching X11 protocol
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PutImageRequest {
    pub opcode: u8,           // Should be 72
    pub format: u8,           // Image format (0=Bitmap, 1=XYPixmap, 2=ZPixmap)
    pub length: u16,          // Request length in 4-byte units
    pub drawable: DrawableId, // Drawable
    pub gc: GContextId,       // Graphics context
    pub width: u16,           // Width
    pub height: u16,          // Height
    pub dst_x: i16,           // Destination X
    pub dst_y: i16,           // Destination Y
    pub left_pad: u8,         // Left pad
    pub depth: u8,            // Depth
    pub data: Vec<u8>,        // Image data
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

/// CreateGC request structure matching X11 protocol
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateGCRequest {
    pub opcode: u8,           // 55: opcode
    pub unused: u8,           // unused
    pub length: u16,          // request length in 4-byte units
    pub gc: GContextId,       // graphics context id
    pub drawable: DrawableId, // drawable
    pub value_mask: u32,      // value mask
    pub value_list: Vec<u32>, // variable length value list
}

/// Arc structure for PolyArc and FillArc
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arc {
    pub x: i16,      // X coordinate
    pub y: i16,      // Y coordinate
    pub width: u16,  // Width
    pub height: u16, // Height
    pub angle1: i16, // Start angle
    pub angle2: i16, // End angle
}

/// CopyArea request structure matching X11 protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyAreaRequest {
    pub opcode: u8,               // Should be 60
    pub unused: u8,               // unused
    pub length: u16,              // Request length in 4-byte units
    pub src_drawable: DrawableId, // Source drawable
    pub dst_drawable: DrawableId, // Destination drawable
    pub gc: GContextId,           // Graphics context
    pub src_x: i16,               // Source X
    pub src_y: i16,               // Source Y
    pub dst_x: i16,               // Destination X
    pub dst_y: i16,               // Destination Y
    pub width: u16,               // Width
    pub height: u16,              // Height
}

/// PolyArc request structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolyArcRequest {
    pub opcode: u8,           // 59: opcode
    pub unused: u8,           // unused
    pub length: u16,          // request length in 4-byte units
    pub drawable: DrawableId, // drawable
    pub gc: GContextId,       // graphics context
    pub arcs: Vec<Arc>,       // list of arcs
}

/// FillArc request structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FillArcRequest {
    pub opcode: u8,           // 61: opcode
    pub unused: u8,           // unused
    pub length: u16,          // request length in 4-byte units
    pub drawable: DrawableId, // drawable
    pub gc: GContextId,       // graphics context
    pub arcs: Vec<Arc>,       // list of arcs
}

/// Point structure for PolyLine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i16, // X coordinate
    pub y: i16, // Y coordinate
}

/// PolyLine request structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolyLineRequest {
    pub opcode: u8,           // 65: opcode
    pub coordinate_mode: u8,  // coordinate mode
    pub length: u16,          // request length in 4-byte units
    pub drawable: DrawableId, // drawable
    pub gc: GContextId,       // graphics context
    pub points: Vec<Point>,   // list of points
}

/// Rectangle structure for PolyFillRectangle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle {
    pub x: i16,      // X coordinate
    pub y: i16,      // Y coordinate
    pub width: u16,  // Width
    pub height: u16, // Height
}

/// PolyFillRectangle request structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolyFillRectangleRequest {
    pub opcode: u8,                 // 70: opcode
    pub unused: u8,                 // unused
    pub length: u16,                // request length in 4-byte units
    pub drawable: DrawableId,       // drawable
    pub gc: GContextId,             // graphics context
    pub rectangles: Vec<Rectangle>, // list of rectangles
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

/// GrabCursor request structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrabPointerRequest {
    pub opcode: u8,            // opcode (26)
    pub owner_events: u8,      // BOOL
    pub length: u16,           // request length
    pub grab_window: WindowId, // grab window id
    pub event_mask: u32,       // event mask
    pub pointer_mode: u8,      // pointer mode (see PointerMode enum)
    pub keyboard_mode: u8,     // keyboard mode (see KeyboardMode enum)
    pub confine_to: WindowId,  // confine to window (0 = None)
    pub cursor: CursorId,      // cursor id (0 = None)
    pub time: u32,             // timestamp (0 = CurrentTime)
}

/// NoOperation request structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoOperationRequest {
    pub opcode: u8,  // opcode (127)
    pub unused: u8,  // unused
    pub length: u16, // request length
}

#[derive(Debug, Clone)]
pub struct QueryExtensionRequest {
    pub opcode: u8,    // Should be 98
    pub unused: u8,    // unused
    pub length: u16,   // Request length in 4-byte units
    pub name_len: u16, // Length of extension name
    pub unused2: u16,  // Padding
    pub name: String,  // Extension name
}

/// BigRequests request structure for BIG-REQUESTS extension
#[derive(Debug, Clone)]
pub struct BigRequestsRequest {
    pub opcode: u8,            // Should be 134
    pub unused: u8,            // unused
    pub length: u32,           // 32-bit length in 4-byte units
    pub request_data: Vec<u8>, // The wrapped request data
}

// RANDR extension request structures
#[derive(Debug, Clone)]
pub struct RandrQueryVersionRequest {
    pub major_version: u32,
    pub minor_version: u32,
}

#[derive(Debug, Clone)]
pub struct RandrGetScreenResourcesRequest {
    pub window: u32, // Window ID
}

#[derive(Debug, Clone)]
pub struct RandrGetOutputInfoRequest {
    pub output: u32,           // Output ID
    pub config_timestamp: u32, // Timestamp
}

#[derive(Debug, Clone)]
pub struct RandrGetCrtcInfoRequest {
    pub crtc: u32,             // CRTC ID
    pub config_timestamp: u32, // Timestamp
}

#[derive(Debug, Clone)]
pub struct RandrGetScreenSizeRangeRequest {
    pub window: u32, // Window ID
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
pub struct GetPropertyParser;
pub struct CreatePixmapParser;
pub struct PutImageParser;
pub struct CreateWindowParser;
pub struct DestroyWindowParser;
pub struct MapWindowParser;
pub struct UnmapWindowParser;
pub struct CreateGCParser;
pub struct PolyArcParser;
pub struct CopyAreaParser;
pub struct FillArcParser;
pub struct PolyLineParser;
pub struct PolyFillRectangleParser;
pub struct CreateGlyphCursorParser;
pub struct OpenFontParser;
pub struct NoOperationParser;
pub struct QueryExtensionParser;
pub struct BigRequestsParser;
// RANDR parsers
pub struct RandrQueryVersionParser;
pub struct RandrGetScreenResourcesParser;
pub struct RandrGetOutputInfoParser;
pub struct RandrGetCrtcInfoParser;
pub struct RandrGetScreenSizeRangeParser;

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
            minor_opcode: None,
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
            minor_opcode: None,
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

impl RequestParser for GetPropertyParser {
    const OPCODE: u8 = opcodes::GET_PROPERTY;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 24 {
            return Err(anyhow::anyhow!("GetProperty request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let delete = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let window = read_or_err!(reader, read_u32);
        let property = read_or_err!(reader, read_u32);
        let r#type = read_or_err!(reader, read_u32);
        let long_offset = read_or_err!(reader, read_u32);
        let long_length = read_or_err!(reader, read_u32);

        let request = GetPropertyRequest {
            opcode,
            delete,
            length,
            window,
            property,
            r#type,
            long_offset,
            long_length,
        };

        Ok(Request {
            kind: RequestKind::GetProperty(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::GetProperty(req) => {
                if req.window == 0 {
                    return Err(anyhow::anyhow!("GetProperty: window must be non-zero"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Invalid request type for GetPropertyParser"
            )),
        }
    }
}

impl RequestParser for CreatePixmapParser {
    const OPCODE: u8 = opcodes::CREATE_PIXMAP;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 16 {
            return Err(anyhow::anyhow!("CreatePixmap request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let depth = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let pid = read_or_err!(reader, read_u32);
        let drawable = read_or_err!(reader, read_u32);
        let width = read_or_err!(reader, read_u16);
        let height = read_or_err!(reader, read_u16);

        let request = CreatePixmapRequest {
            opcode,
            depth,
            length,
            pid,
            drawable,
            width,
            height,
        };

        Ok(Request {
            kind: RequestKind::CreatePixmap(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::CreatePixmap(req) => {
                if req.pid == 0 {
                    return Err(anyhow::anyhow!("CreatePixmap: pixmap id must be non-zero"));
                }
                if req.drawable == 0 {
                    return Err(anyhow::anyhow!(
                        "CreatePixmap: drawable id must be non-zero"
                    ));
                }
                if req.width == 0 || req.height == 0 {
                    return Err(anyhow::anyhow!(
                        "CreatePixmap: width and height must be non-zero"
                    ));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Invalid request type for CreatePixmapParser"
            )),
        }
    }
}

impl RequestParser for PutImageParser {
    const OPCODE: u8 = opcodes::PUT_IMAGE;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 24 {
            return Err(anyhow::anyhow!("PutImage request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let format = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let drawable = read_or_err!(reader, read_u32);
        let gc = read_or_err!(reader, read_u32);
        let width = read_or_err!(reader, read_u16);
        let height = read_or_err!(reader, read_u16);
        let dst_x = read_or_err!(reader, read_i16);
        let dst_y = read_or_err!(reader, read_i16);
        let left_pad = read_or_err!(reader, read_u8);
        let depth = read_or_err!(reader, read_u8);
        let _unused = read_or_err!(reader, read_u16);

        // Read the remaining data as image data
        let data_start = 24; // Fixed header size
        let total_length = (length as usize) * 4;
        if bytes.len() < total_length {
            return Err(anyhow::anyhow!("PutImage request data incomplete"));
        }
        let data = bytes[data_start..total_length].to_vec();

        let request = PutImageRequest {
            opcode,
            format,
            length,
            drawable,
            gc,
            width,
            height,
            dst_x,
            dst_y,
            left_pad,
            depth,
            data,
        };

        Ok(Request {
            kind: RequestKind::PutImage(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::PutImage(req) => {
                if req.drawable == 0 {
                    return Err(anyhow::anyhow!("PutImage: drawable id must be non-zero"));
                }
                if req.gc == 0 {
                    return Err(anyhow::anyhow!(
                        "PutImage: graphics context id must be non-zero"
                    ));
                }
                if req.width == 0 || req.height == 0 {
                    return Err(anyhow::anyhow!(
                        "PutImage: width and height must be non-zero"
                    ));
                }
                if req.format > 2 {
                    return Err(anyhow::anyhow!("PutImage: invalid format"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid request type for PutImageParser")),
        }
    }
}

impl RequestParser for CopyAreaParser {
    const OPCODE: u8 = opcodes::COPY_AREA;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 28 {
            return Err(anyhow::anyhow!("CopyArea request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let src_drawable = read_or_err!(reader, read_u32);
        let dst_drawable = read_or_err!(reader, read_u32);
        let gc = read_or_err!(reader, read_u32);
        let src_x = read_or_err!(reader, read_i16);
        let src_y = read_or_err!(reader, read_i16);
        let dst_x = read_or_err!(reader, read_i16);
        let dst_y = read_or_err!(reader, read_i16);
        let width = read_or_err!(reader, read_u16);
        let height = read_or_err!(reader, read_u16);

        let request = CopyAreaRequest {
            opcode,
            unused,
            length,
            src_drawable,
            dst_drawable,
            gc,
            src_x,
            src_y,
            dst_x,
            dst_y,
            width,
            height,
        };

        Ok(Request {
            kind: RequestKind::CopyArea(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::CopyArea(req) => {
                if req.src_drawable == 0 {
                    return Err(anyhow::anyhow!("CopyArea: src_drawable id must be non-zero"));
                }
                if req.dst_drawable == 0 {
                    return Err(anyhow::anyhow!("CopyArea: dst_drawable id must be non-zero"));
                }
                if req.gc == 0 {
                    return Err(anyhow::anyhow!("CopyArea: graphics context id must be non-zero"));
                }
                if req.width == 0 || req.height == 0 {
                    return Err(anyhow::anyhow!("CopyArea: width and height must be non-zero"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid request type for CopyAreaParser")),
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
            minor_opcode: None,
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
            minor_opcode: None,
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
            minor_opcode: None,
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
            minor_opcode: None,
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

impl RequestParser for CreateGCParser {
    const OPCODE: u8 = opcodes::CREATE_GC;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 16 {
            return Err(anyhow::anyhow!("CreateGC request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let gc = read_or_err!(reader, read_u32);
        let drawable = read_or_err!(reader, read_u32);
        let value_mask = read_or_err!(reader, read_u32);

        // Parse variable length value list
        let mut value_list = Vec::new();
        let remaining_bytes = (length as usize * 4).saturating_sub(16);
        if remaining_bytes > 0 {
            let values_count = remaining_bytes / 4;
            for _ in 0..values_count {
                value_list.push(read_or_err!(reader, read_u32));
            }
        }

        let request = CreateGCRequest {
            opcode,
            unused,
            length,
            gc,
            drawable,
            value_mask,
            value_list,
        };

        Ok(Request {
            kind: RequestKind::CreateGC(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::CreateGC(req) => {
                if req.gc == 0 {
                    return Err(anyhow::anyhow!(
                        "CreateGC: graphics context id must be non-zero"
                    ));
                }
                if req.drawable == 0 {
                    return Err(anyhow::anyhow!("CreateGC: drawable id must be non-zero"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid request type for CreateGCParser")),
        }
    }
}

impl RequestParser for PolyArcParser {
    const OPCODE: u8 = opcodes::POLY_ARC;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 12 {
            return Err(anyhow::anyhow!("PolyArc request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let drawable = read_or_err!(reader, read_u32);
        let gc = read_or_err!(reader, read_u32);

        // Parse arcs
        let mut arcs = Vec::new();
        let remaining_bytes = (length as usize * 4).saturating_sub(12);
        if remaining_bytes > 0 {
            let arc_count = remaining_bytes / 12; // Each arc is 12 bytes
            for _ in 0..arc_count {
                let x = read_or_err!(reader, read_i16);
                let y = read_or_err!(reader, read_i16);
                let width = read_or_err!(reader, read_u16);
                let height = read_or_err!(reader, read_u16);
                let angle1 = read_or_err!(reader, read_i16);
                let angle2 = read_or_err!(reader, read_i16);
                arcs.push(Arc {
                    x,
                    y,
                    width,
                    height,
                    angle1,
                    angle2,
                });
            }
        }

        let request = PolyArcRequest {
            opcode,
            unused,
            length,
            drawable,
            gc,
            arcs,
        };

        Ok(Request {
            kind: RequestKind::PolyArc(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::PolyArc(req) => {
                if req.drawable == 0 {
                    return Err(anyhow::anyhow!("PolyArc: drawable id must be non-zero"));
                }
                if req.gc == 0 {
                    return Err(anyhow::anyhow!(
                        "PolyArc: graphics context id must be non-zero"
                    ));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid request type for PolyArcParser")),
        }
    }
}

impl RequestParser for FillArcParser {
    const OPCODE: u8 = opcodes::FILL_ARC;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 12 {
            return Err(anyhow::anyhow!("FillArc request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let drawable = read_or_err!(reader, read_u32);
        let gc = read_or_err!(reader, read_u32);

        // Parse arcs
        let mut arcs = Vec::new();
        let remaining_bytes = (length as usize * 4).saturating_sub(12);
        if remaining_bytes > 0 {
            let arc_count = remaining_bytes / 12; // Each arc is 12 bytes
            for _ in 0..arc_count {
                let x = read_or_err!(reader, read_i16);
                let y = read_or_err!(reader, read_i16);
                let width = read_or_err!(reader, read_u16);
                let height = read_or_err!(reader, read_u16);
                let angle1 = read_or_err!(reader, read_i16);
                let angle2 = read_or_err!(reader, read_i16);
                arcs.push(Arc {
                    x,
                    y,
                    width,
                    height,
                    angle1,
                    angle2,
                });
            }
        }

        let request = FillArcRequest {
            opcode,
            unused,
            length,
            drawable,
            gc,
            arcs,
        };

        Ok(Request {
            kind: RequestKind::FillArc(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::FillArc(req) => {
                if req.drawable == 0 {
                    return Err(anyhow::anyhow!("FillArc: drawable id must be non-zero"));
                }
                if req.gc == 0 {
                    return Err(anyhow::anyhow!(
                        "FillArc: graphics context id must be non-zero"
                    ));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid request type for FillArcParser")),
        }
    }
}

impl RequestParser for PolyLineParser {
    const OPCODE: u8 = opcodes::POLY_LINE;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 12 {
            return Err(anyhow::anyhow!("PolyLine request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let coordinate_mode = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let drawable = read_or_err!(reader, read_u32);
        let gc = read_or_err!(reader, read_u32);

        // Parse points
        let mut points = Vec::new();
        let remaining_bytes = (length as usize * 4).saturating_sub(12);
        if remaining_bytes > 0 {
            let point_count = remaining_bytes / 4; // Each point is 4 bytes
            for _ in 0..point_count {
                let x = read_or_err!(reader, read_i16);
                let y = read_or_err!(reader, read_i16);
                points.push(Point { x, y });
            }
        }

        let request = PolyLineRequest {
            opcode,
            coordinate_mode,
            length,
            drawable,
            gc,
            points,
        };

        Ok(Request {
            kind: RequestKind::PolyLine(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::PolyLine(req) => {
                if req.drawable == 0 {
                    return Err(anyhow::anyhow!("PolyLine: drawable id must be non-zero"));
                }
                if req.gc == 0 {
                    return Err(anyhow::anyhow!(
                        "PolyLine: graphics context id must be non-zero"
                    ));
                }
                if req.points.len() < 2 {
                    return Err(anyhow::anyhow!("PolyLine: must have at least 2 points"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid request type for PolyLineParser")),
        }
    }
}

impl RequestParser for PolyFillRectangleParser {
    const OPCODE: u8 = opcodes::POLY_FILL_RECTANGLE;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 12 {
            return Err(anyhow::anyhow!("PolyFillRectangle request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let drawable = read_or_err!(reader, read_u32);
        let gc = read_or_err!(reader, read_u32);

        // Parse rectangles
        let mut rectangles = Vec::new();
        let remaining_bytes = (length as usize * 4).saturating_sub(12);
        if remaining_bytes > 0 {
            let rect_count = remaining_bytes / 8; // Each rectangle is 8 bytes
            for _ in 0..rect_count {
                let x = read_or_err!(reader, read_i16);
                let y = read_or_err!(reader, read_i16);
                let width = read_or_err!(reader, read_u16);
                let height = read_or_err!(reader, read_u16);
                rectangles.push(Rectangle {
                    x,
                    y,
                    width,
                    height,
                });
            }
        }

        let request = PolyFillRectangleRequest {
            opcode,
            unused,
            length,
            drawable,
            gc,
            rectangles,
        };

        Ok(Request {
            kind: RequestKind::PolyFillRectangle(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::PolyFillRectangle(req) => {
                if req.drawable == 0 {
                    return Err(anyhow::anyhow!(
                        "PolyFillRectangle: drawable id must be non-zero"
                    ));
                }
                if req.gc == 0 {
                    return Err(anyhow::anyhow!(
                        "PolyFillRectangle: graphics context id must be non-zero"
                    ));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Invalid request type for PolyFillRectangleParser"
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
            minor_opcode: None,
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
            minor_opcode: None,
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
            minor_opcode: None,
        })
    }

    fn validate(_request: &Request) -> Result<()> {
        // NoOperation requests are always valid
        Ok(())
    }
}

impl RequestParser for BigRequestsParser {
    const OPCODE: u8 = opcodes::BIG_REQUESTS;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 4 {
            return Err(anyhow::anyhow!("BigRequests request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);

        let request = BigRequestsRequest {
            opcode,
            unused,
            length: length as u32,
            request_data: Vec::new(), // This request doesn't wrap other data
        };

        Ok(Request {
            kind: RequestKind::BigRequests(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(_request: &Request) -> Result<()> {
        // BigRequests requests are always valid
        Ok(())
    }
}

impl RequestParser for QueryExtensionParser {
    const OPCODE: u8 = opcodes::QUERY_EXTENSION;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 8 {
            return Err(anyhow::anyhow!("QueryExtension request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
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

        let request = QueryExtensionRequest {
            opcode,
            unused,
            length,
            name_len,
            unused2,
            name,
        };

        Ok(Request {
            kind: RequestKind::QueryExtension(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::QueryExtension(req) => {
                if req.name.is_empty() {
                    return Err(anyhow::anyhow!("QueryExtension: name must not be empty"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Invalid request type for QueryExtensionParser"
            )),
        }
    }
}

// RANDR extension parsers
impl RequestParser for RandrQueryVersionParser {
    const OPCODE: u8 = opcodes::RANDR_QUERY_VERSION;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 12 {
            return Err(anyhow::anyhow!("RandrQueryVersion request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let major_opcode = read_or_err!(reader, read_u8);
        let minor_opcode = read_or_err!(reader, read_u8);
        let _length = read_or_err!(reader, read_u16);
        let major_version = read_or_err!(reader, read_u32);
        let minor_version = read_or_err!(reader, read_u32);

        let request = RandrQueryVersionRequest {
            major_version,
            minor_version,
        };

        Ok(Request {
            kind: RequestKind::RandrQueryVersion(request),
            sequence_number: 0,
            opcode: major_opcode,
            minor_opcode: Some(minor_opcode),
        })
    }

    fn validate(_request: &Request) -> Result<()> {
        Ok(())
    }
}

impl RequestParser for RandrGetScreenResourcesParser {
    const OPCODE: u8 = opcodes::RANDR_GET_SCREEN_RESOURCES;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 8 {
            return Err(anyhow::anyhow!("RandrGetScreenResources request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let major_opcode = read_or_err!(reader, read_u8);
        let _minor_opcode = read_or_err!(reader, read_u8);
        let _length = read_or_err!(reader, read_u16);
        let window = read_or_err!(reader, read_u32);

        let request = RandrGetScreenResourcesRequest { window };

        Ok(Request {
            kind: RequestKind::RandrGetScreenResources(request),
            sequence_number: 0,
            opcode: major_opcode,
            minor_opcode: Some(opcodes::RANDR_GET_SCREEN_RESOURCES),
        })
    }

    fn validate(_request: &Request) -> Result<()> {
        Ok(())
    }
}

impl RequestParser for RandrGetOutputInfoParser {
    const OPCODE: u8 = opcodes::RANDR_GET_OUTPUT_INFO;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 12 {
            return Err(anyhow::anyhow!("RandrGetOutputInfo request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let major_opcode = read_or_err!(reader, read_u8);
        let _minor_opcode = read_or_err!(reader, read_u8);
        let _length = read_or_err!(reader, read_u16);
        let output = read_or_err!(reader, read_u32);
        let config_timestamp = read_or_err!(reader, read_u32);

        let request = RandrGetOutputInfoRequest {
            output,
            config_timestamp,
        };

        Ok(Request {
            kind: RequestKind::RandrGetOutputInfo(request),
            sequence_number: 0,
            opcode: major_opcode,
            minor_opcode: Some(opcodes::RANDR_GET_OUTPUT_INFO),
        })
    }

    fn validate(_request: &Request) -> Result<()> {
        Ok(())
    }
}

impl RequestParser for RandrGetCrtcInfoParser {
    const OPCODE: u8 = opcodes::RANDR_GET_CRTC_INFO;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 12 {
            return Err(anyhow::anyhow!("RandrGetCrtcInfo request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let major_opcode = read_or_err!(reader, read_u8);
        let _minor_opcode = read_or_err!(reader, read_u8);
        let _length = read_or_err!(reader, read_u16);
        let crtc = read_or_err!(reader, read_u32);
        let config_timestamp = read_or_err!(reader, read_u32);

        let request = RandrGetCrtcInfoRequest {
            crtc,
            config_timestamp,
        };

        Ok(Request {
            kind: RequestKind::RandrGetCrtcInfo(request),
            sequence_number: 0,
            opcode: major_opcode,
            minor_opcode: Some(opcodes::RANDR_GET_CRTC_INFO),
        })
    }

    fn validate(_request: &Request) -> Result<()> {
        Ok(())
    }
}

impl RequestParser for RandrGetScreenSizeRangeParser {
    const OPCODE: u8 = opcodes::RANDR_GET_SCREEN_SIZE_RANGE;

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 8 {
            return Err(anyhow::anyhow!("RandrGetScreenSizeRange request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let major_opcode = read_or_err!(reader, read_u8);
        let _minor_opcode = read_or_err!(reader, read_u8);
        let _length = read_or_err!(reader, read_u16);
        let window = read_or_err!(reader, read_u32);

        let request = RandrGetScreenSizeRangeRequest { window };

        Ok(Request {
            kind: RequestKind::RandrGetScreenSizeRange(request),
            sequence_number: 0,
            opcode: major_opcode,
            minor_opcode: Some(opcodes::RANDR_GET_SCREEN_SIZE_RANGE),
        })
    }

    fn validate(_request: &Request) -> Result<()> {
        Ok(())
    }
}

/// Main dispatcher parser that routes to specific parsers based on opcode
pub struct X11RequestParser;

impl X11RequestParser {
    fn opcode_name(opcode: u8) -> &'static str {
        match opcode {
            opcodes::CONNECTION_SETUP => "ConnectionSetup",
            opcodes::CREATE_WINDOW => "CreateWindow",
            opcodes::DESTROY_WINDOW => "DestroyWindow",
            opcodes::MAP_WINDOW => "MapWindow",
            opcodes::UNMAP_WINDOW => "UnmapWindow",
            opcodes::GET_GEOMETRY => "GetGeometry",
            opcodes::INTERN_ATOM => "InternAtom",
            opcodes::GET_PROPERTY => "GetProperty",
            opcodes::CREATE_PIXMAP => "CreatePixmap",
            opcodes::PUT_IMAGE => "PutImage",
            opcodes::CREATE_GC => "CreateGC",
            opcodes::POLY_ARC => "PolyArc",
            opcodes::COPY_AREA => "CopyArea",
            opcodes::FILL_ARC => "FillArc",
            opcodes::POLY_LINE => "PolyLine",
            opcodes::POLY_FILL_RECTANGLE => "PolyFillRectangle",
            opcodes::OPEN_FONT => "OpenFont",
            opcodes::CREATE_GLYPH_CURSOR => "CreateGlyphCursor",
            opcodes::NO_OPERATION => "NoOperation",
            opcodes::QUERY_EXTENSION => "QueryExtension",
            opcodes::BIG_REQUESTS => "BigRequests",
            _ => "Unknown",
        }
    }
}

impl RequestParser for X11RequestParser {
    const OPCODE: u8 = 0; // Dispatcher doesn't have a specific opcode

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.is_empty() {
            return Err(anyhow::anyhow!("Request too short"));
        }

        let opcode = bytes[0];
        let opcode_name = X11RequestParser::opcode_name(opcode);
        trace!(
            "Dispatching request with opcode: {} ({})",
            opcode, opcode_name
        );
        match opcode {
            opcodes::GET_GEOMETRY => GetGeometryParser::parse(bytes),
            opcodes::INTERN_ATOM => InternAtomParser::parse(bytes),
            opcodes::GET_PROPERTY => GetPropertyParser::parse(bytes),
            opcodes::CREATE_PIXMAP => CreatePixmapParser::parse(bytes),
            opcodes::PUT_IMAGE => PutImageParser::parse(bytes),
            opcodes::CREATE_WINDOW => CreateWindowParser::parse(bytes),
            opcodes::DESTROY_WINDOW => DestroyWindowParser::parse(bytes),
            opcodes::MAP_WINDOW => MapWindowParser::parse(bytes),
            opcodes::UNMAP_WINDOW => UnmapWindowParser::parse(bytes),
            opcodes::CREATE_GC => CreateGCParser::parse(bytes),
            opcodes::POLY_ARC => PolyArcParser::parse(bytes),
            opcodes::COPY_AREA => CopyAreaParser::parse(bytes),
            opcodes::FILL_ARC => FillArcParser::parse(bytes),
            opcodes::POLY_LINE => PolyLineParser::parse(bytes),
            opcodes::POLY_FILL_RECTANGLE => PolyFillRectangleParser::parse(bytes),
            opcodes::CREATE_GLYPH_CURSOR => CreateGlyphCursorParser::parse(bytes),
            opcodes::OPEN_FONT => OpenFontParser::parse(bytes),
            opcodes::NO_OPERATION => NoOperationParser::parse(bytes),
            opcodes::QUERY_EXTENSION => QueryExtensionParser::parse(bytes),
            opcodes::BIG_REQUESTS => BigRequestsParser::parse(bytes),
            _ => Err(anyhow::anyhow!("Unknown opcode: {}", opcode)),
        }
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::GetGeometry(_) => GetGeometryParser::validate(request),
            RequestKind::InternAtom(_) => InternAtomParser::validate(request),
            RequestKind::GetProperty(_) => GetPropertyParser::validate(request),
            RequestKind::CreatePixmap(_) => CreatePixmapParser::validate(request),
            RequestKind::PutImage(_) => PutImageParser::validate(request),
            RequestKind::CreateWindow(_) => CreateWindowParser::validate(request),
            RequestKind::DestroyWindow(_) => DestroyWindowParser::validate(request),
            RequestKind::MapWindow(_) => MapWindowParser::validate(request),
            RequestKind::UnmapWindow(_) => UnmapWindowParser::validate(request),
            RequestKind::CreateGC(_) => CreateGCParser::validate(request),
            RequestKind::PolyArc(_) => PolyArcParser::validate(request),
            RequestKind::CopyArea(_) => CopyAreaParser::validate(request),
            RequestKind::FillArc(_) => FillArcParser::validate(request),
            RequestKind::PolyLine(_) => PolyLineParser::validate(request),
            RequestKind::PolyFillRectangle(_) => PolyFillRectangleParser::validate(request),
            RequestKind::CreateGlyphCursor(_) => CreateGlyphCursorParser::validate(request),
            RequestKind::OpenFont(_) => OpenFontParser::validate(request),
            RequestKind::NoOperation(_) => NoOperationParser::validate(request),
            RequestKind::ConnectionSetup => Ok(()),
            RequestKind::QueryExtension(_) => QueryExtensionParser::validate(request),
            RequestKind::BigRequests(_) => BigRequestsParser::validate(request),
            RequestKind::RandrQueryVersion(_) => RandrQueryVersionParser::validate(request),
            RequestKind::RandrGetScreenResources(_) => {
                RandrGetScreenResourcesParser::validate(request)
            }
            RequestKind::RandrGetOutputInfo(_) => RandrGetOutputInfoParser::validate(request),
            RequestKind::RandrGetCrtcInfo(_) => RandrGetCrtcInfoParser::validate(request),
            RequestKind::RandrGetScreenSizeRange(_) => {
                RandrGetScreenSizeRangeParser::validate(request)
            }
            RequestKind::GrabPointer(_) => {
                // GrabPointer requests have their own validation logic
                Ok(())
            }
        }
    }
}
