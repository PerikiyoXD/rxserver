use crate::protocol::{
    ByteOrderReader, constants::FIRST_EXTENSION_OPCODE, extension_registry::ExtensionRegistry,
    types::ByteOrder,
};
use anyhow::Result;
use tracing::trace;

use super::types::*;

/// Macro to convert ByteOrderReader errors to anyhow::Error
macro_rules! read_or_err {
    ($reader:expr, $method:ident) => {
        $reader.$method().map_err(|e| anyhow::anyhow!(e))?
    };
}

#[derive(Debug, Clone)]
pub enum RequestKind {
    ConnectionSetup,
    GetGeometry(GetGeometryRequest),
    InternAtom(InternAtomRequest),
    ChangeProperty(ChangePropertyRequest),
    GetProperty(GetPropertyRequest),
    QueryColors(QueryColorsRequest),
    CreatePixmap(CreatePixmapRequest),
    FreePixmap(FreePixmapRequest),
    GetInputFocus(GetInputFocusRequest),
    PutImage(PutImageRequest),
    CreateWindow(CreateWindowRequest),
    DestroyWindow(DestroyWindowRequest),
    MapWindow(MapWindowRequest),
    UnmapWindow(UnmapWindowRequest),
    ClearArea(ClearAreaRequest),
    CreateGC(CreateGCRequest),
    ChangeGC(ChangeGCRequest),
    FreeGC(FreeGCRequest),
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
    // RENDER extension requests
    RenderQueryVersion(RenderQueryVersionRequest),
    RenderCreateSolidFill(RenderCreateSolidFillRequest),
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

/// ChangeProperty request structure matching X11 protocol
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangePropertyRequest {
    pub opcode: u8,       // Should be 18
    pub mode: u8,         // 0 = Replace, 1 = Prepend, 2 = Append
    pub length: u16,      // Request length in 4-byte units
    pub window: WindowId, // Window
    pub property: Atom,   // Property atom
    pub r#type: Atom,     // Type atom
    pub format: u8,       // 8, 16, or 32
    pub data: Vec<u8>,    // Raw property data (format units, unpadded)
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

/// QueryColors request structure matching X11 protocol
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryColorsRequest {
    pub opcode: u8,           // Should be 91
    pub unused: u8,           // Padding
    pub length: u16,          // Request length in 4-byte units
    pub colormap: ColormapId, // Colormap
    pub pixels: Vec<u32>,     // Pixel values to look up
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

/// FreePixmap request structure matching X11 protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreePixmapRequest {
    pub opcode: u8,       // 54: opcode
    pub unused: u8,       // unused
    pub length: u16,      // request length (2)
    pub pixmap: PixmapId, // pixmap id
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

/// ClearArea request structure matching X11 protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClearAreaRequest {
    pub opcode: u8,       // opcode (61)
    pub exposures: u8,    // BOOL: whether to generate Expose events
    pub length: u16,      // request length (4)
    pub window: WindowId, // window to clear
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
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

/// ChangeGC request structure matching X11 protocol
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeGCRequest {
    pub opcode: u8,           // 56: opcode
    pub unused: u8,           // unused
    pub length: u16,          // request length in 4-byte units
    pub gc: GContextId,       // graphics context id
    pub value_mask: u32,      // value mask
    pub value_list: Vec<u32>, // variable length value list
}

/// FreeGC request structure matching X11 protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreeGCRequest {
    pub opcode: u8,     // 60: opcode
    pub unused: u8,     // unused
    pub length: u16,    // request length (2)
    pub gc: GContextId, // graphics context id
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

/// GetInputFocus request structure matching X11 protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetInputFocusRequest {
    pub opcode: u8,  // Should be 43
    pub unused: u8,  // Padding
    pub length: u16, // Request length in 4-byte units (always 1)
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

// RENDER extension request structures
#[derive(Debug, Clone)]
pub struct RenderQueryVersionRequest {
    pub client_major_version: u32,
    pub client_minor_version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderCreateSolidFillRequest {
    pub pid: PictureId,
    pub red: u16,
    pub green: u16,
    pub blue: u16,
    pub alpha: u16,
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
pub struct ChangePropertyParser;
pub struct GetPropertyParser;
pub struct QueryColorsParser;
pub struct CreatePixmapParser;
pub struct FreePixmapParser;
pub struct GetInputFocusParser;
pub struct PutImageParser;
pub struct CreateWindowParser;
pub struct DestroyWindowParser;
pub struct MapWindowParser;
pub struct UnmapWindowParser;
pub struct ClearAreaParser;
pub struct CreateGCParser;
pub struct ChangeGCParser;
pub struct FreeGCParser;
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
// RENDER parsers
pub struct RenderQueryVersionParser;
pub struct RenderCreateSolidFillParser;

impl RequestParser for GetGeometryParser {
    const OPCODE: u8 = Opcode::GetGeometry.to_u8();

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
    const OPCODE: u8 = Opcode::InternAtom.to_u8();

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

impl RequestParser for ChangePropertyParser {
    const OPCODE: u8 = Opcode::ChangeProperty.to_u8();

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 24 {
            return Err(anyhow::anyhow!("ChangeProperty request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let mode = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let window = read_or_err!(reader, read_u32);
        let property = read_or_err!(reader, read_u32);
        let r#type = read_or_err!(reader, read_u32);
        let format = read_or_err!(reader, read_u8);
        reader
            .read_bytes(3) // unused padding
            .map_err(|e| anyhow::anyhow!(e))?;
        let data_len = read_or_err!(reader, read_u32);

        let unit_bytes = match format {
            8 => 1,
            16 => 2,
            32 => 4,
            _ => return Err(anyhow::anyhow!("ChangeProperty: invalid format {}", format)),
        };
        let byte_len = data_len as usize * unit_bytes;
        let data = if byte_len > 0 {
            reader
                .read_bytes(byte_len)
                .map_err(|e| anyhow::anyhow!(e))?
                .to_vec()
        } else {
            Vec::new()
        };

        let request = ChangePropertyRequest {
            opcode,
            mode,
            length,
            window,
            property,
            r#type,
            format,
            data,
        };

        Ok(Request {
            kind: RequestKind::ChangeProperty(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::ChangeProperty(req) => {
                if req.window == 0 {
                    return Err(anyhow::anyhow!("ChangeProperty: window must be non-zero"));
                }
                if req.property == 0 {
                    return Err(anyhow::anyhow!("ChangeProperty: property must be non-zero"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Invalid request type for ChangePropertyParser"
            )),
        }
    }
}

impl RequestParser for GetPropertyParser {
    const OPCODE: u8 = Opcode::GetProperty.to_u8();

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

impl RequestParser for QueryColorsParser {
    const OPCODE: u8 = Opcode::QueryColors.to_u8();

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 8 {
            return Err(anyhow::anyhow!("QueryColors request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let colormap = read_or_err!(reader, read_u32);

        let expected_len = (length as usize) * 4;
        if bytes.len() < expected_len {
            return Err(anyhow::anyhow!("QueryColors request too short"));
        }
        let pixel_count = (expected_len.saturating_sub(8)) / 4;

        let mut pixels = Vec::with_capacity(pixel_count);
        for _ in 0..pixel_count {
            pixels.push(read_or_err!(reader, read_u32));
        }

        let request = QueryColorsRequest {
            opcode,
            unused,
            length,
            colormap,
            pixels,
        };

        Ok(Request {
            kind: RequestKind::QueryColors(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::QueryColors(req) => {
                if req.colormap == 0 {
                    return Err(anyhow::anyhow!("QueryColors: colormap must be non-zero"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Invalid request type for QueryColorsParser"
            )),
        }
    }
}

impl RequestParser for CreatePixmapParser {
    const OPCODE: u8 = Opcode::CreatePixmap.to_u8();

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

impl RequestParser for FreePixmapParser {
    const OPCODE: u8 = Opcode::FreePixmap.to_u8();

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 8 {
            return Err(anyhow::anyhow!("FreePixmap request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let pixmap = read_or_err!(reader, read_u32);

        let request = FreePixmapRequest {
            opcode,
            unused,
            length,
            pixmap,
        };

        Ok(Request {
            kind: RequestKind::FreePixmap(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::FreePixmap(req) => {
                if req.pixmap == 0 {
                    return Err(anyhow::anyhow!("FreePixmap: pixmap id must be non-zero"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid request type for FreePixmapParser")),
        }
    }
}

impl RequestParser for GetInputFocusParser {
    const OPCODE: u8 = Opcode::GetInputFocus.to_u8();

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 4 {
            return Err(anyhow::anyhow!("GetInputFocus request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);

        let request = GetInputFocusRequest {
            opcode,
            unused,
            length,
        };

        Ok(Request {
            kind: RequestKind::GetInputFocus(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(_request: &Request) -> Result<()> {
        // GetInputFocus takes no arguments; always valid once parsed
        Ok(())
    }
}

impl RequestParser for PutImageParser {
    const OPCODE: u8 = Opcode::PutImage.to_u8();

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
    const OPCODE: u8 = Opcode::CopyArea.to_u8();

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
                    return Err(anyhow::anyhow!(
                        "CopyArea: src_drawable id must be non-zero"
                    ));
                }
                if req.dst_drawable == 0 {
                    return Err(anyhow::anyhow!(
                        "CopyArea: dst_drawable id must be non-zero"
                    ));
                }
                if req.gc == 0 {
                    return Err(anyhow::anyhow!(
                        "CopyArea: graphics context id must be non-zero"
                    ));
                }
                if req.width == 0 || req.height == 0 {
                    return Err(anyhow::anyhow!(
                        "CopyArea: width and height must be non-zero"
                    ));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid request type for CopyAreaParser")),
        }
    }
}

impl RequestParser for CreateWindowParser {
    const OPCODE: u8 = Opcode::CreateWindow.to_u8();

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
    const OPCODE: u8 = Opcode::DestroyWindow.to_u8();

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
    const OPCODE: u8 = Opcode::MapWindow.to_u8();

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
    const OPCODE: u8 = Opcode::UnmapWindow.to_u8();

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

impl RequestParser for ClearAreaParser {
    const OPCODE: u8 = Opcode::ClearArea.to_u8();

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 16 {
            return Err(anyhow::anyhow!("ClearArea request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let exposures = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let window = read_or_err!(reader, read_u32);
        let x = read_or_err!(reader, read_i16);
        let y = read_or_err!(reader, read_i16);
        let width = read_or_err!(reader, read_u16);
        let height = read_or_err!(reader, read_u16);

        let request = ClearAreaRequest {
            opcode,
            exposures,
            length,
            window,
            x,
            y,
            width,
            height,
        };

        Ok(Request {
            kind: RequestKind::ClearArea(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::ClearArea(req) => {
                if req.window == 0 {
                    return Err(anyhow::anyhow!("ClearArea: window id must be non-zero"));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid request type for ClearAreaParser")),
        }
    }
}

impl RequestParser for CreateGCParser {
    const OPCODE: u8 = Opcode::CreateGC.to_u8();

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
        const KNOWN_GC_VALUE_MASK: u32 = 0x007F_FFFF;

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
                if req.value_mask & !KNOWN_GC_VALUE_MASK != 0 {
                    return Err(anyhow::anyhow!(
                        "CreateGC: value mask has unsupported bits set: 0x{:08x}",
                        req.value_mask
                    ));
                }
                let expected_values = req.value_mask.count_ones() as usize;
                if req.value_list.len() != expected_values {
                    return Err(anyhow::anyhow!(
                        "CreateGC: value-list length {} does not match value-mask count {}",
                        req.value_list.len(),
                        expected_values
                    ));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid request type for CreateGCParser")),
        }
    }
}

impl RequestParser for ChangeGCParser {
    const OPCODE: u8 = Opcode::ChangeGC.to_u8();

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 12 {
            return Err(anyhow::anyhow!("ChangeGC request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let gc = read_or_err!(reader, read_u32);
        let value_mask = read_or_err!(reader, read_u32);

        let mut value_list = Vec::new();
        let remaining_bytes = (length as usize * 4).saturating_sub(12);
        if remaining_bytes > 0 {
            let values_count = remaining_bytes / 4;
            for _ in 0..values_count {
                value_list.push(read_or_err!(reader, read_u32));
            }
        }

        let request = ChangeGCRequest {
            opcode,
            unused,
            length,
            gc,
            value_mask,
            value_list,
        };

        Ok(Request {
            kind: RequestKind::ChangeGC(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        const KNOWN_GC_VALUE_MASK: u32 = 0x007F_FFFF;

        match &request.kind {
            RequestKind::ChangeGC(req) => {
                if req.gc == 0 {
                    return Err(anyhow::anyhow!(
                        "ChangeGC: graphics context id must be non-zero"
                    ));
                }
                if req.value_mask & !KNOWN_GC_VALUE_MASK != 0 {
                    return Err(anyhow::anyhow!(
                        "ChangeGC: value mask has unsupported bits set: 0x{:08x}",
                        req.value_mask
                    ));
                }
                let expected_values = req.value_mask.count_ones() as usize;
                if req.value_list.len() != expected_values {
                    return Err(anyhow::anyhow!(
                        "ChangeGC: value-list length {} does not match value-mask count {}",
                        req.value_list.len(),
                        expected_values
                    ));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid request type for ChangeGCParser")),
        }
    }
}

impl RequestParser for FreeGCParser {
    const OPCODE: u8 = Opcode::FreeGC.to_u8();

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 8 {
            return Err(anyhow::anyhow!("FreeGC request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let opcode = read_or_err!(reader, read_u8);
        let unused = read_or_err!(reader, read_u8);
        let length = read_or_err!(reader, read_u16);
        let gc = read_or_err!(reader, read_u32);

        let request = FreeGCRequest {
            opcode,
            unused,
            length,
            gc,
        };

        Ok(Request {
            kind: RequestKind::FreeGC(request),
            sequence_number: 0,
            opcode,
            minor_opcode: None,
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::FreeGC(req) => {
                if req.gc == 0 {
                    return Err(anyhow::anyhow!(
                        "FreeGC: graphics context id must be non-zero"
                    ));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Invalid request type for FreeGCParser")),
        }
    }
}

impl RequestParser for PolyArcParser {
    const OPCODE: u8 = Opcode::PolyArc.to_u8();

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
    const OPCODE: u8 = Opcode::PolyFillArc.to_u8();

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
    const OPCODE: u8 = Opcode::PolyLine.to_u8();

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
    const OPCODE: u8 = Opcode::PolyFillRectangle.to_u8();

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
    const OPCODE: u8 = Opcode::CreateGlyphCursor.to_u8();

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
    const OPCODE: u8 = Opcode::OpenFont.to_u8();

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
    const OPCODE: u8 = Opcode::NoOperation.to_u8();

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
    const OPCODE: u8 = BigRequestsOpcode::Enable.to_u8();

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
    const OPCODE: u8 = Opcode::QueryExtension.to_u8();

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
    const OPCODE: u8 = RandrOpcode::QueryVersion.to_u8();

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
    const OPCODE: u8 = RandrOpcode::GetScreenResources.to_u8();

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
            minor_opcode: Some(RandrOpcode::GetScreenResources.to_u8()),
        })
    }

    fn validate(_request: &Request) -> Result<()> {
        Ok(())
    }
}

impl RequestParser for RandrGetOutputInfoParser {
    const OPCODE: u8 = RandrOpcode::GetOutputInfo.to_u8();

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
            minor_opcode: Some(RandrOpcode::GetOutputInfo.to_u8()),
        })
    }

    fn validate(_request: &Request) -> Result<()> {
        Ok(())
    }
}

impl RequestParser for RandrGetCrtcInfoParser {
    const OPCODE: u8 = RandrOpcode::GetCrtcInfo.to_u8();

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
            minor_opcode: Some(RandrOpcode::GetCrtcInfo.to_u8()),
        })
    }

    fn validate(_request: &Request) -> Result<()> {
        Ok(())
    }
}

impl RequestParser for RandrGetScreenSizeRangeParser {
    const OPCODE: u8 = RandrOpcode::GetScreenSizeRange.to_u8();

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
            minor_opcode: Some(RandrOpcode::GetScreenSizeRange.to_u8()),
        })
    }

    fn validate(_request: &Request) -> Result<()> {
        Ok(())
    }
}

// RENDER extension parsers
impl RequestParser for RenderQueryVersionParser {
    const OPCODE: u8 = RenderOpcode::QueryVersion.to_u8();

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 12 {
            return Err(anyhow::anyhow!("RenderQueryVersion request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let major_opcode = read_or_err!(reader, read_u8);
        let minor_opcode = read_or_err!(reader, read_u8);
        let _length = read_or_err!(reader, read_u16);
        let client_major_version = read_or_err!(reader, read_u32);
        let client_minor_version = read_or_err!(reader, read_u32);

        let request = RenderQueryVersionRequest {
            client_major_version,
            client_minor_version,
        };

        Ok(Request {
            kind: RequestKind::RenderQueryVersion(request),
            sequence_number: 0,
            opcode: major_opcode,
            minor_opcode: Some(minor_opcode),
        })
    }

    fn validate(_request: &Request) -> Result<()> {
        Ok(())
    }
}

impl RequestParser for RenderCreateSolidFillParser {
    const OPCODE: u8 = RenderOpcode::CreateSolidFill.to_u8();

    fn parse(bytes: &[u8]) -> Result<Request> {
        if bytes.len() < 16 {
            return Err(anyhow::anyhow!("RenderCreateSolidFill request too short"));
        }

        let mut reader = ByteOrderReader::new(bytes, ByteOrder::LittleEndian);
        let major_opcode = read_or_err!(reader, read_u8);
        let minor_opcode = read_or_err!(reader, read_u8);
        let _length = read_or_err!(reader, read_u16);
        let pid = read_or_err!(reader, read_u32);
        let red = read_or_err!(reader, read_u16);
        let green = read_or_err!(reader, read_u16);
        let blue = read_or_err!(reader, read_u16);
        let alpha = read_or_err!(reader, read_u16);

        let request = RenderCreateSolidFillRequest {
            pid,
            red,
            green,
            blue,
            alpha,
        };

        Ok(Request {
            kind: RequestKind::RenderCreateSolidFill(request),
            sequence_number: 0,
            opcode: major_opcode,
            minor_opcode: Some(minor_opcode),
        })
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::RenderCreateSolidFill(req) => {
                if req.pid == 0 {
                    return Err(anyhow::anyhow!(
                        "RenderCreateSolidFill: picture id must be non-zero"
                    ));
                }
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Not a RenderCreateSolidFill request")),
        }
    }
}

/// Main dispatcher parser that routes to specific parsers based on opcode
pub struct X11RequestParser;

impl X11RequestParser {
    fn opcode_name(opcode: u8) -> String {
        if opcode >= FIRST_EXTENSION_OPCODE {
            Opcode::Extension(ExtensionOpcode::Unknown(opcode, 0)).name()
        } else {
            Opcode::from_u8(opcode).name()
        }
    }

    /// Dispatch a request, resolving extension (opcode >= FIRST_EXTENSION_OPCODE) requests
    /// against `extensions` - this server's per-session, dynamically
    /// assigned major opcode table - rather than fixed constants. Core
    /// protocol opcodes (< 128) dispatch identically either way, since their
    /// numbers are fixed by the spec.
    pub fn parse_dynamic(bytes: &[u8], extensions: &ExtensionRegistry) -> Result<Request> {
        if bytes.is_empty() {
            return Err(anyhow::anyhow!("Request too short"));
        }

        let opcode = bytes[0];
        if opcode < FIRST_EXTENSION_OPCODE {
            return Self::parse(bytes);
        }

        let opcode_name = Self::opcode_name(opcode);
        trace!(
            "Dispatching extension request with opcode: {} ({})",
            opcode, opcode_name
        );

        match extensions.extension_for_opcode(opcode) {
            Some("BIG-REQUESTS") => BigRequestsParser::parse(bytes),
            Some("RANDR") => {
                if bytes.len() < 2 {
                    return Err(anyhow::anyhow!("RANDR request too short"));
                }
                let minor_opcode = bytes[1];
                if minor_opcode == RandrOpcode::QueryVersion.to_u8() {
                    RandrQueryVersionParser::parse(bytes)
                } else if minor_opcode == RandrOpcode::GetScreenResources.to_u8() {
                    RandrGetScreenResourcesParser::parse(bytes)
                } else if minor_opcode == RandrOpcode::GetOutputInfo.to_u8() {
                    RandrGetOutputInfoParser::parse(bytes)
                } else if minor_opcode == RandrOpcode::GetCrtcInfo.to_u8() {
                    RandrGetCrtcInfoParser::parse(bytes)
                } else if minor_opcode == RandrOpcode::GetScreenSizeRange.to_u8() {
                    RandrGetScreenSizeRangeParser::parse(bytes)
                } else {
                    Err(anyhow::anyhow!(
                        "Unknown RANDR minor opcode: {}",
                        minor_opcode
                    ))
                }
            }
            Some("RENDER") => {
                if bytes.len() < 2 {
                    return Err(anyhow::anyhow!("RENDER request too short"));
                }
                let minor_opcode = bytes[1];
                if minor_opcode == RenderOpcode::QueryVersion.to_u8() {
                    RenderQueryVersionParser::parse(bytes)
                } else if minor_opcode == RenderOpcode::CreateSolidFill.to_u8() {
                    RenderCreateSolidFillParser::parse(bytes)
                } else {
                    Err(anyhow::anyhow!(
                        "Unknown RENDER minor opcode: {}",
                        minor_opcode
                    ))
                }
            }
            Some(name) => Err(anyhow::anyhow!(
                "Extension '{}' has an assigned opcode but no request parser yet",
                name
            )),
            None => Err(anyhow::anyhow!("Unknown opcode: {}", opcode)),
        }
    }
}

impl RequestParser for X11RequestParser {
    const OPCODE: u8 = 0; // Dispatcher doesn't have a specific opcode

    /// Dispatches core protocol requests (opcode < 128) only. Extension
    /// requests need a per-session major opcode table to resolve, which this
    /// trait method has no way to receive - use `parse_dynamic` for those.
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
        if opcode == Opcode::GetGeometry.to_u8() {
            GetGeometryParser::parse(bytes)
        } else if opcode == Opcode::InternAtom.to_u8() {
            InternAtomParser::parse(bytes)
        } else if opcode == Opcode::ChangeProperty.to_u8() {
            ChangePropertyParser::parse(bytes)
        } else if opcode == Opcode::GetProperty.to_u8() {
            GetPropertyParser::parse(bytes)
        } else if opcode == Opcode::QueryColors.to_u8() {
            QueryColorsParser::parse(bytes)
        } else if opcode == Opcode::CreatePixmap.to_u8() {
            CreatePixmapParser::parse(bytes)
        } else if opcode == Opcode::FreePixmap.to_u8() {
            FreePixmapParser::parse(bytes)
        } else if opcode == Opcode::GetInputFocus.to_u8() {
            GetInputFocusParser::parse(bytes)
        } else if opcode == Opcode::PutImage.to_u8() {
            PutImageParser::parse(bytes)
        } else if opcode == Opcode::CreateWindow.to_u8() {
            CreateWindowParser::parse(bytes)
        } else if opcode == Opcode::DestroyWindow.to_u8() {
            DestroyWindowParser::parse(bytes)
        } else if opcode == Opcode::MapWindow.to_u8() {
            MapWindowParser::parse(bytes)
        } else if opcode == Opcode::UnmapWindow.to_u8() {
            UnmapWindowParser::parse(bytes)
        } else if opcode == Opcode::ClearArea.to_u8() {
            ClearAreaParser::parse(bytes)
        } else if opcode == Opcode::CreateGC.to_u8() {
            CreateGCParser::parse(bytes)
        } else if opcode == Opcode::ChangeGC.to_u8() {
            ChangeGCParser::parse(bytes)
        } else if opcode == Opcode::FreeGC.to_u8() {
            FreeGCParser::parse(bytes)
        } else if opcode == Opcode::PolyArc.to_u8() {
            PolyArcParser::parse(bytes)
        } else if opcode == Opcode::CopyArea.to_u8() {
            CopyAreaParser::parse(bytes)
        } else if opcode == Opcode::PolyFillArc.to_u8() {
            FillArcParser::parse(bytes)
        } else if opcode == Opcode::PolyLine.to_u8() {
            PolyLineParser::parse(bytes)
        } else if opcode == Opcode::PolyFillRectangle.to_u8() {
            PolyFillRectangleParser::parse(bytes)
        } else if opcode == Opcode::CreateGlyphCursor.to_u8() {
            CreateGlyphCursorParser::parse(bytes)
        } else if opcode == Opcode::OpenFont.to_u8() {
            OpenFontParser::parse(bytes)
        } else if opcode == Opcode::NoOperation.to_u8() {
            NoOperationParser::parse(bytes)
        } else if opcode == Opcode::QueryExtension.to_u8() {
            QueryExtensionParser::parse(bytes)
        } else if opcode >= FIRST_EXTENSION_OPCODE {
            Err(anyhow::anyhow!(
                "Extension opcode {} requires parse_dynamic (per-session major opcode table)",
                opcode
            ))
        } else {
            Err(anyhow::anyhow!("Unknown opcode: {}", opcode))
        }
    }

    fn validate(request: &Request) -> Result<()> {
        match &request.kind {
            RequestKind::GetGeometry(_) => GetGeometryParser::validate(request),
            RequestKind::InternAtom(_) => InternAtomParser::validate(request),
            RequestKind::ChangeProperty(_) => ChangePropertyParser::validate(request),
            RequestKind::GetProperty(_) => GetPropertyParser::validate(request),
            RequestKind::QueryColors(_) => QueryColorsParser::validate(request),
            RequestKind::CreatePixmap(_) => CreatePixmapParser::validate(request),
            RequestKind::FreePixmap(_) => FreePixmapParser::validate(request),
            RequestKind::GetInputFocus(_) => GetInputFocusParser::validate(request),
            RequestKind::PutImage(_) => PutImageParser::validate(request),
            RequestKind::CreateWindow(_) => CreateWindowParser::validate(request),
            RequestKind::DestroyWindow(_) => DestroyWindowParser::validate(request),
            RequestKind::MapWindow(_) => MapWindowParser::validate(request),
            RequestKind::UnmapWindow(_) => UnmapWindowParser::validate(request),
            RequestKind::ClearArea(_) => ClearAreaParser::validate(request),
            RequestKind::CreateGC(_) => CreateGCParser::validate(request),
            RequestKind::ChangeGC(_) => ChangeGCParser::validate(request),
            RequestKind::FreeGC(_) => FreeGCParser::validate(request),
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
            RequestKind::RenderQueryVersion(_) => RenderQueryVersionParser::validate(request),
            RequestKind::RenderCreateSolidFill(_) => RenderCreateSolidFillParser::validate(request),
            RequestKind::GrabPointer(_) => {
                // GrabPointer requests have their own validation logic
                Ok(())
            }
        }
    }
}
