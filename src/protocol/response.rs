use super::types::*;
use std::convert::TryFrom;

// ==================== SINGLE MARKER TRAIT ====================

/// Marker trait for X11 protocol responses.
/// This is purely for type identification - no compile-time guarantees.
pub trait ProtocolResponse {}

// ==================== CONNECTION SETUP (existing complex types) ====================

/// Raw connection setup request from client (matches X11 protocol exactly)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionSetupRequestRaw {
    pub byte_order: u8, // #x42 = MSB first, #x6C = LSB first
    pub unused1: u8,
    pub protocol_major_version: u16,
    pub protocol_minor_version: u16,
    pub authorization_protocol_name_length: u16,
    pub authorization_protocol_data_length: u16,
    pub unused2: u16,
    // Variable length data follows: auth_name, padding, auth_data, padding
}

/// Complete connection setup request with variable-length data
#[derive(Debug, Clone)]
pub struct ConnectionSetupRequest<'a> {
    pub raw: &'a ConnectionSetupRequestRaw,
    pub authorization_protocol_name: &'a str,
    pub authorization_protocol_data: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VisualClass {
    StaticGray = 0,
    GrayScale = 1,
    StaticColor = 2,
    PseudoColor = 3,
    TrueColor = 4,
    DirectColor = 5,
}

impl TryFrom<u8> for VisualClass {
    type Error = u8;
    fn try_from(v: u8) -> std::result::Result<Self, Self::Error> {
        use VisualClass::*;
        match v {
            0 => Ok(StaticGray),
            1 => Ok(GrayScale),
            2 => Ok(StaticColor),
            3 => Ok(PseudoColor),
            4 => Ok(TrueColor),
            5 => Ok(DirectColor),
            e => Err(e),
        }
    }
}

impl From<VisualClass> for u8 {
    #[inline(always)]
    fn from(v: VisualClass) -> u8 {
        v as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenDepthVisualRaw {
    pub id: VisualId,
    pub class: u8, // VisualClass as u8
    pub bits_per_rgb_value: u8,
    pub colormap_entries: u16,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
}

impl ScreenDepthVisualRaw {
    pub fn class(&self) -> std::result::Result<VisualClass, u8> {
        VisualClass::try_from(self.class)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ScreenDepth<'a> {
    pub depth: u8,
    pub visuals: &'a [ScreenDepthVisualRaw],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRaw {
    pub root: WindowId,
    pub default_colormap: ColormapId,
    pub white_pixel: u32,
    pub black_pixel: u32,
    pub current_input_masks: u32,
    pub width_in_pixels: u16,
    pub height_in_pixels: u16,
    pub width_in_millimeters: u16,
    pub height_in_millimeters: u16,
    pub min_installed_maps: u16,
    pub max_installed_maps: u16,
    pub root_visual: u32,
    pub backing_stores: u8, // BackingStore as u8
    pub save_unders: u8,
    pub root_depth: u8,
    pub allowed_depths_len: u8, // Number of allowed depths
}

#[derive(Debug)]
pub struct Screen<'a> {
    pub raw: &'a ScreenRaw,
    pub allowed_depths: &'a [ScreenDepth<'a>],
}

#[derive(Debug, Clone)]
pub enum ConnectionSetupResponse<'a> {
    Accepted(ConnectionSetupAccepted<'a>),
    Refused(ConnectionSetupRefused<'a>),
    AuthRequired(ConnectionSetupAuthRequired<'a>),
    Unknown {
        major_version: u16,
        minor_version: u16,
        data: &'a [u8],
    },
}

#[derive(Debug, Clone)]
pub struct ConnectionSetupAccepted<'a> {
    pub protocol_major_version: u16,
    pub protocol_minor_version: u16,
    pub release_number: u32,
    pub resource_id_base: XId,
    pub resource_id_mask: XId,
    pub motion_buffer_size: u32,
    pub vendor: &'a str,
    pub maximum_request_length: u16,
    pub screens: &'a [Screen<'a>],
    pub pixmap_formats: &'a [PixmapFormatRaw],
    pub image_byte_order: u8,
    pub bitmap_format_bit_order: u8,
    pub bitmap_format_scanline_unit: u8,
    pub bitmap_format_scanline_pad: u8,
    pub min_keycode: u8,
    pub max_keycode: u8,
    pub extra_fields: &'a [u8], // for future extensibility
}

#[derive(Debug, Clone)]
pub struct ConnectionSetupRefused<'a> {
    pub protocol_major_version: u16,
    pub protocol_minor_version: u16,
    pub reason: &'a str,
    pub extra_fields: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct ConnectionSetupAuthRequired<'a> {
    pub protocol_major_version: u16,
    pub protocol_minor_version: u16,
    pub reason: &'a str,
    pub extra_fields: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixmapFormatRaw {
    pub depth: u8,
    pub bits_per_pixel: u8,
    pub scanline_pad: u8,
}

// ==================== GET GEOMETRY (the new functionality) ====================

/// GetGeometry response structure matching X11 protocol layout exactly
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetGeometryResponse {
    pub response_type: u8,    // Always 1 for replies
    pub depth: u8,            // Depth of drawable
    pub sequence_number: u16, // Sequence number
    pub length: u32,          // Response length (always 0 for GetGeometry)
    pub root: WindowId,       // Root window
    pub x: i16,               // X coordinate relative to parent
    pub y: i16,               // Y coordinate relative to parent
    pub width: u16,           // Width in pixels
    pub height: u16,          // Height in pixels
    pub border_width: u16,    // Border width in pixels
    pub unused: [u8; 10],     // Padding to 32 bytes total
}

impl GetGeometryResponse {
    /// Create a new GetGeometry response
    pub fn new(
        sequence_number: u16,
        depth: u8,
        root: WindowId,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
    ) -> Self {
        Self {
            response_type: 1, // Reply
            depth,
            sequence_number,
            length: 0, // No additional data
            root,
            x,
            y,
            width,
            height,
            border_width,
            unused: [0; 10],
        }
    }
}

// ==================== RESPONSE ENVELOPE ====================

#[derive(Debug, Clone)]
pub struct RawResponse<'a> {
    pub data: &'a [u8],
}

#[derive(Debug, Clone)]
pub enum ResponseKind<'a> {
    Reply, // Generic
    ConnectionSetup(ConnectionSetupResponse<'a>),
    GetGeometry(GetGeometryResponse),
    Raw(RawResponse<'a>),
}

#[derive(Debug, Clone)]
pub struct Response<'a> {
    pub kind: ResponseKind<'a>,
    pub sequence_number: SequenceNumber,
    pub byte_order: ByteOrder,
}

impl<'a> Default for Response<'a> {
    fn default() -> Self {
        Self {
            kind: ResponseKind::Reply,
            sequence_number: 0,
            byte_order: ByteOrder::LittleEndian,
        }
    }
}

// ==================== PROTOCOL RESPONSE IMPLEMENTATIONS ====================

// Only implement ProtocolResponse for types that are actual protocol responses
impl ProtocolResponse for GetGeometryResponse {}
impl<'a> ProtocolResponse for ConnectionSetupResponse<'a> {}
impl<'a> ProtocolResponse for Response<'a> {}

// ==================== PROTOCOL CONSTANTS ====================

pub mod byte_order {
    pub const MSB_FIRST: u8 = 0x42;
    pub const LSB_FIRST: u8 = 0x6C;
}

pub mod connection_status {
    pub const FAILED: u8 = 0;
    pub const SUCCESS: u8 = 1;
    pub const AUTHENTICATE: u8 = 2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_geometry_response_creation() {
        let response = GetGeometryResponse::new(
            42,   // sequence_number
            32,   // depth
            1,    // root window
            100,  // x
            200,  // y
            1024, // width
            768,  // height
            5,    // border_width
        );

        assert_eq!(response.response_type, 1);
        assert_eq!(response.depth, 32);
        assert_eq!(response.sequence_number, 42);
        assert_eq!(response.length, 0);
        assert_eq!(response.root, 1);
        assert_eq!(response.x, 100);
        assert_eq!(response.y, 200);
        assert_eq!(response.width, 1024);
        assert_eq!(response.height, 768);
        assert_eq!(response.border_width, 5);
        assert_eq!(response.unused, [0; 10]);
    }

    #[test]
    fn test_get_geometry_response_size() {
        use std::mem;
        // Verify the struct size matches X11 protocol
        assert_eq!(mem::size_of::<GetGeometryResponse>(), 32);
    }

    #[test]
    fn test_protocol_response_trait() {
        // Test that our types implement the ProtocolResponse trait
        fn assert_protocol_response<T: ProtocolResponse>() {}

        assert_protocol_response::<GetGeometryResponse>();
    }
}
