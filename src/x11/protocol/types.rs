//! Core X11 Protocol Types
//!
//! This module defines the fundamental types used throughout the X11 protocol.

use std::fmt;

/// X11 Resource Identifier - 29-bit identifier for X11 resources
pub type XID = u32;

/// Client identifier for tracking connections
pub type ClientId = u32;

/// Sequence number for request/response matching
pub type SequenceNumber = u16;

/// X11 Timestamp
pub type Timestamp = u32;

/// X11 Atom identifier
pub type Atom = u32;

/// Window ID type alias
pub type WindowId = XID;

/// Pixmap ID type alias  
pub type PixmapId = XID;

/// Graphics Context ID type alias
pub type GContextId = XID;

/// Font ID type alias
pub type FontId = XID;

/// Cursor ID type alias
pub type CursorId = XID;

/// Colormap ID type alias
pub type ColormapId = XID;

/// Keysym - symbolic representation of a key
pub type Keysym = u32;

/// Keycode - hardware-specific key identifier
pub type Keycode = u8;

/// Button identifier for mouse buttons
pub type Button = u8;

/// Coordinate value type
pub type Coordinate = i16;

/// Dimension value type
pub type Dimension = u16;

/// Re-export geometric types from geometry module
pub use crate::x11::geometry::types::{Point, Rectangle};
/// Re-export endianness types
pub use crate::x11::protocol::endianness::ByteOrder;
use crate::x11::resources::types::window::BackingStore;

/// X11 Protocol version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolVersion {
    /// Major version number
    pub major: u16,
    /// Minor version number
    pub minor: u16,
}

impl ProtocolVersion {
    /// X11 protocol version 11.0
    pub const X11: Self = Self {
        major: 11,
        minor: 0,
    };

    /// Create a new protocol version
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// X11 Color value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red component (0-65535)
    pub red: u16,
    /// Green component (0-65535)
    pub green: u16,
    /// Blue component (0-65535)
    pub blue: u16,
}

impl Color {
    /// Create a new color
    pub const fn new(red: u16, green: u16, blue: u16) -> Self {
        Self { red, green, blue }
    }

    /// Black color
    pub const BLACK: Self = Self::new(0, 0, 0);

    /// White color
    pub const WHITE: Self = Self::new(65535, 65535, 65535);

    /// Red color
    pub const RED: Self = Self::new(65535, 0, 0);

    /// Green color
    pub const GREEN: Self = Self::new(0, 65535, 0);

    /// Blue color
    pub const BLUE: Self = Self::new(0, 0, 65535);
}

pub mod requests {
    use crate::{CursorId, FontId, WindowId};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CreateWindow {
        pub depth: u8,
        pub wid: WindowId,
        pub parent: WindowId,
        pub x: i16,
        pub y: i16,
        pub width: u16,
        pub height: u16,
        pub border_width: u16,
        pub class: u16,
        pub visual: u32,
        pub value_mask: u32,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CreateGlyphCursor {
        pub cursor_id: CursorId,
        pub source_font: FontId,
        pub mask_font: FontId,
        pub source_char: u16,
        pub mask_char: u16,
        pub fore_red: u16,
        pub fore_green: u16,
        pub fore_blue: u16,
        pub back_red: u16,
        pub back_green: u16,
        pub back_blue: u16,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct InternAtom {
        pub only_if_exists: bool,
        pub name: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct OpenFont {
        pub font_id: FontId,
        pub name: String,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MapWindow {
        pub window: WindowId,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct UnmapWindow {
        pub window: WindowId,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DestroyWindow {
        pub window: WindowId,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NoOperation;
}

/// X11 Request types
#[derive(Debug, Clone)]
pub enum Request {
    CreateWindow(requests::CreateWindow),
    CreateGlyphCursor(requests::CreateGlyphCursor),
    MapWindow(requests::MapWindow),
    UnmapWindow(requests::UnmapWindow),
    DestroyWindow(requests::DestroyWindow),
    InternAtom(requests::InternAtom),
    OpenFont(requests::OpenFont),
    NoOperation(requests::NoOperation),
}

/// X11 Response types
#[derive(Debug, Clone)]
pub enum Response {
    Empty,
    GetGeometry {
        root: WindowId,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        depth: u8,
    },
    QueryTree {
        root: WindowId,
        parent: WindowId,
        children: Vec<WindowId>,
    },
    InternAtom {
        atom: XID, // ATOM is just an XID, 0 means None
    },
    // Add more response types as needed
}

/// X11 Error Response
#[derive(Debug, Clone)]
pub struct ErrorResponse {
    pub error_code: u8,
    pub sequence_number: SequenceNumber,
    pub resource_id: XID,
    pub minor_opcode: u16,
    pub major_opcode: u8,
}

/// Connection setup data
#[derive(Debug, Clone)]
pub struct SetupRequest {
    pub byte_order: ByteOrder,
    pub protocol_major_version: u16,
    pub protocol_minor_version: u16,
    pub authorization_protocol_name: String,
    pub authorization_protocol_data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SetupResponse {
    pub status: u8,
    pub protocol_major_version: u16,
    pub protocol_minor_version: u16,
    pub length: u16,
    pub release_number: u32,
    pub resource_id_base: XID,
    pub resource_id_mask: XID,
    pub motion_buffer_size: u32,
    pub vendor: String,
    pub maximum_request_length: u16,
    pub number_of_screens: u8,
    pub number_of_formats: u8,
    pub image_byte_order: u8,
    pub bitmap_format_bit_order: u8,
    pub bitmap_format_scanline_unit: u8,
    pub bitmap_format_scanline_pad: u8,
    pub min_keycode: u8,
    pub max_keycode: u8,
}

/// Connection setup request from client
#[derive(Debug, Clone)]
pub struct ConnectionSetupRequest {
    pub byte_order: u8,
    pub major_version: u16,
    pub minor_version: u16,
    pub authorization_protocol_name_length: u16,
    pub authorization_protocol_data_length: u16,
    pub authorization_protocol_name: String,
    pub authorization_protocol_data: Vec<u8>,
}

impl ConnectionSetupRequest {
    /// Parse connection setup request from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, crate::x11::protocol::ProtocolError> {
        Self::validate_minimum_length(data, 12)?;

        let byte_order = data[0];
        let major_version = u16::from_le_bytes([data[2], data[3]]);
        let minor_version = u16::from_le_bytes([data[4], data[5]]);
        let auth_name_len = u16::from_le_bytes([data[6], data[7]]) as usize;
        let auth_data_len = u16::from_le_bytes([data[8], data[9]]) as usize;

        let mut offset = 12;

        // Read authorization protocol name with padding
        let (auth_name, new_offset) = Self::read_padded_string(data, offset, auth_name_len)?;
        offset = new_offset;

        // Read authorization protocol data
        let auth_data = Self::read_byte_array(data, offset, auth_data_len)?;

        Ok(Self {
            byte_order,
            major_version,
            minor_version,
            authorization_protocol_name_length: auth_name_len as u16,
            authorization_protocol_data_length: auth_data_len as u16,
            authorization_protocol_name: auth_name,
            authorization_protocol_data: auth_data,
        })
    }

    /// Validate that data has at least the minimum required length
    fn validate_minimum_length(
        data: &[u8],
        min_length: usize,
    ) -> Result<(), crate::x11::protocol::ProtocolError> {
        if data.len() < min_length {
            return Err(crate::x11::protocol::ProtocolError::MessageTooShort {
                expected: min_length,
                actual: data.len(),
            });
        }
        Ok(())
    }

    /// Read a padded string from data buffer
    fn read_padded_string(
        data: &[u8],
        offset: usize,
        length: usize,
    ) -> Result<(String, usize), crate::x11::protocol::ProtocolError> {
        if length == 0 {
            return Ok((String::new(), offset));
        }

        Self::validate_minimum_length(data, offset + length)?;

        let string_data = &data[offset..offset + length];
        let parsed_string = String::from_utf8_lossy(string_data).to_string();

        // Calculate new offset with 4-byte boundary padding
        let new_offset = (offset + length + 3) & !3;

        Ok((parsed_string, new_offset))
    }

    /// Read a byte array from data buffer
    fn read_byte_array(
        data: &[u8],
        offset: usize,
        length: usize,
    ) -> Result<Vec<u8>, crate::x11::protocol::ProtocolError> {
        if length == 0 {
            return Ok(Vec::new());
        }

        Self::validate_minimum_length(data, offset + length)?;

        Ok(data[offset..offset + length].to_vec())
    }
}

/// Connection setup success response
#[derive(Debug, Clone)]
pub struct ConnectionSetupSuccess {
    pub success: u8,
    pub protocol_major_version: u16,
    pub protocol_minor_version: u16,
    pub additional_data_length: u16,
    pub release_number: u32,
    pub resource_id_base: XID,
    pub resource_id_mask: XID,
    pub motion_buffer_size: u32,
    pub vendor_length: u16,
    pub maximum_request_length: u16,
    pub number_of_screens: u8,
    pub number_of_formats: u8,
    pub image_byte_order: u8,
    pub bitmap_format_bit_order: u8,
    pub bitmap_format_scanline_unit: u8,
    pub bitmap_format_scanline_pad: u8,
    pub min_keycode: u8,
    pub max_keycode: u8,
    pub vendor: String,
    pub pixmap_formats: Vec<PixmapFormat>,
    pub screens: Vec<Screen>,
}

impl ConnectionSetupSuccess {
    /// Serialize the setup response to bytes
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        data.push(self.success);
        data.push(0); // unused
        data.extend_from_slice(&self.protocol_major_version.to_le_bytes());
        data.extend_from_slice(&self.protocol_minor_version.to_le_bytes());

        // Calculate additional data length according to X11 spec: 8+2n+(v+p+m)/4
        // where:
        // - 8: fixed overhead (32 bytes of fixed fields after length, in 4-byte units)
        // - 2n: pixmap formats (n formats * 8 bytes each = 2n units of 4 bytes)
        // - (v+p+m)/4: (vendor_length + padding + screens_length) in 4-byte units
        let vendor_padded_len = (self.vendor.len() + 3) & !3;
        let screens_len = self
            .screens
            .iter()
            .map(|s| s.serialized_size())
            .sum::<usize>();
        let additional_data_len =
            8 + 2 * (self.pixmap_formats.len()) + (vendor_padded_len + screens_len) / 4;

        data.extend_from_slice(&(additional_data_len as u16).to_le_bytes());
        data.extend_from_slice(&self.release_number.to_le_bytes());
        data.extend_from_slice(&self.resource_id_base.to_le_bytes());
        data.extend_from_slice(&self.resource_id_mask.to_le_bytes());
        data.extend_from_slice(&self.motion_buffer_size.to_le_bytes());
        data.extend_from_slice(&self.vendor_length.to_le_bytes());
        data.extend_from_slice(&self.maximum_request_length.to_le_bytes());
        data.push(self.number_of_screens);
        data.push(self.number_of_formats);
        data.push(self.image_byte_order);
        data.push(self.bitmap_format_bit_order);
        data.push(self.bitmap_format_scanline_unit);
        data.push(self.bitmap_format_scanline_pad);
        data.push(self.min_keycode);
        data.push(self.max_keycode);
        data.extend_from_slice(&[0; 4]); // padding

        // Vendor string with padding
        data.extend_from_slice(self.vendor.as_bytes());
        while data.len() % 4 != 0 {
            data.push(0);
        }

        // Pixmap formats
        for format in &self.pixmap_formats {
            data.extend_from_slice(&format.serialize());
        }

        // Screens
        for screen in &self.screens {
            data.extend_from_slice(&screen.serialize());
        }

        data
    }
}

/// Pixmap format descriptor
#[derive(Debug, Clone)]
pub struct PixmapFormat {
    pub depth: u8,
    pub bits_per_pixel: u8,
    pub scanline_pad: u8,
}

impl PixmapFormat {
    pub fn serialize(&self) -> [u8; 8] {
        let bytes = [
            self.depth,
            self.bits_per_pixel,
            self.scanline_pad,
            0,
            0,
            0,
            0,
            0, // padding
        ];

        //tracing::trace!("PixmapFormat: \n\n{:?}\n\n{:02X?}", self, bytes);
        bytes
    }
}

/// Screen descriptor
#[derive(Debug, Clone)]
pub struct Screen {
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
    pub backing_stores: BackingStore,
    pub save_unders: u8,
    pub root_depth: u8,
    pub allowed_depths: Vec<Depth>,
}

impl Screen {
    pub fn serialized_size(&self) -> usize {
        40 + self
            .allowed_depths
            .iter()
            .map(|d| d.serialized_size())
            .sum::<usize>()
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        data.extend_from_slice(&self.root.to_le_bytes());
        data.extend_from_slice(&self.default_colormap.to_le_bytes());
        data.extend_from_slice(&self.white_pixel.to_le_bytes());
        data.extend_from_slice(&self.black_pixel.to_le_bytes());
        data.extend_from_slice(&self.current_input_masks.to_le_bytes());
        data.extend_from_slice(&self.width_in_pixels.to_le_bytes());
        data.extend_from_slice(&self.height_in_pixels.to_le_bytes());
        data.extend_from_slice(&self.width_in_millimeters.to_le_bytes());
        data.extend_from_slice(&self.height_in_millimeters.to_le_bytes());
        data.extend_from_slice(&self.min_installed_maps.to_le_bytes());
        data.extend_from_slice(&self.max_installed_maps.to_le_bytes());
        data.extend_from_slice(&self.root_visual.to_le_bytes());
        data.push(self.backing_stores as u8);
        data.push(self.save_unders);
        data.push(self.root_depth);
        data.push(self.allowed_depths.len() as u8);

        for depth in &self.allowed_depths {
            //tracing::trace!("Serializing depth: {:?}", depth);
            data.extend_from_slice(&depth.serialize());
        }

        //tracing::trace!("Screen: \n\n{:?}\n\n{:02X?}", self, data);

        data
    }
}

/// Depth and associated visuals
#[derive(Debug, Clone)]
pub struct Depth {
    pub depth: u8,
    pub visuals: Vec<Visual>,
}

impl Depth {
    pub fn serialized_size(&self) -> usize {
        8 + (self.visuals.len() * Visual::serialized_size())
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(self.depth);
        bytes.push(0); // padding
        bytes.extend_from_slice(&(self.visuals.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&[0; 4]); // 4 bytes padding

        for visual in &self.visuals {
            bytes.extend_from_slice(&visual.serialize());
        }

        //tracing::trace!("Depth: \n\n{:?}\n\n{:02X?}", self, bytes);

        bytes
    }
}

/// Visual descriptor
#[derive(Debug, Clone)]
pub struct Visual {
    pub visual_id: u32,
    pub class: u8,
    pub bits_per_rgb_value: u8,
    pub colormap_entries: u16,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
}

impl Visual {
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(24);
        bytes.extend_from_slice(&self.visual_id.to_le_bytes());
        bytes.push(self.class);
        bytes.push(self.bits_per_rgb_value);
        bytes.extend_from_slice(&self.colormap_entries.to_le_bytes());
        bytes.extend_from_slice(&self.red_mask.to_le_bytes());
        bytes.extend_from_slice(&self.green_mask.to_le_bytes());
        bytes.extend_from_slice(&self.blue_mask.to_le_bytes());
        bytes.extend_from_slice(&[0; 4]);
        //tracing::trace!("Visual: \n\n{:?}\n\n{:02X?}", self, bytes);
        bytes
    }

    fn serialized_size() -> usize {
        24 // 4 + 1 + 1 + 2 + 4 + 4 + 4 + 4 (padding)
    }
}

/// Request header for all X11 requests
#[derive(Debug, Clone)]
pub struct RequestHeader {
    pub opcode: u8,
    pub data: u8,
    pub length: u16,
    pub sequence_number: SequenceNumber,
}

impl RequestHeader {
    /// Parse request header from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, crate::x11::protocol::ProtocolError> {
        if data.len() < 4 {
            return Err(crate::x11::protocol::ProtocolError::InsufficientData);
        }

        Ok(Self {
            opcode: data[0],
            data: data[1],
            length: u16::from_le_bytes([data[2], data[3]]),
            sequence_number: 0, // Set by protocol handler
        })
    }
}

/// Query extension response
#[derive(Debug, Clone)]
pub struct QueryExtensionResponse {
    pub present: u8,
    pub major_opcode: u8,
    pub first_event: u8,
    pub first_error: u8,
}

impl QueryExtensionResponse {
    /// Serialize response with sequence number
    pub fn serialize(&self, sequence_number: SequenceNumber) -> Vec<u8> {
        vec![
            1, // Reply
            self.present,
            sequence_number as u8,
            (sequence_number >> 8) as u8,
            0,
            0,
            0,
            0, // length (always 0 for this response)
            self.major_opcode,
            self.first_event,
            self.first_error,
            0, // padding
            0,
            0,
            0,
            0, // padding
            0,
            0,
            0,
            0, // padding
            0,
            0,
            0,
            0, // padding
            0,
            0,
            0,
            0, // padding
            0,
            0,
            0,
            0, // padding
            0,
            0,
            0,
            0, // padding
            0,
            0,
            0,
            0, // padding
        ]
    }
}

/// Protocol parsing utilities
pub mod parsing {
    /// Common parsing error result type
    pub type ParseResult<T> = Result<T, crate::x11::protocol::ProtocolError>;

    /// Validate buffer has minimum required length
    pub fn validate_buffer_length(data: &[u8], min_length: usize) -> ParseResult<()> {
        if data.len() < min_length {
            return Err(crate::x11::protocol::ProtocolError::MessageTooShort {
                expected: min_length,
                actual: data.len(),
            });
        }
        Ok(())
    }

    /// Read a fixed-length string from buffer without padding
    pub fn read_string(data: &[u8], offset: usize, length: usize) -> ParseResult<String> {
        validate_buffer_length(data, offset + length)?;
        let string_data = &data[offset..offset + length];
        Ok(String::from_utf8_lossy(string_data).to_string())
    }

    /// Read a string with 4-byte boundary padding
    pub fn read_padded_string(
        data: &[u8],
        offset: usize,
        length: usize,
    ) -> ParseResult<(String, usize)> {
        if length == 0 {
            return Ok((String::new(), offset));
        }

        let string = read_string(data, offset, length)?;
        let padded_offset = (offset + length + 3) & !3;

        Ok((string, padded_offset))
    }

    /// Read a byte array from buffer
    pub fn read_bytes(data: &[u8], offset: usize, length: usize) -> ParseResult<Vec<u8>> {
        if length == 0 {
            return Ok(Vec::new());
        }

        validate_buffer_length(data, offset + length)?;
        Ok(data[offset..offset + length].to_vec())
    }

    /// Read a u16 in little-endian format
    pub fn read_u16_le(data: &[u8], offset: usize) -> ParseResult<u16> {
        validate_buffer_length(data, offset + 2)?;
        Ok(u16::from_le_bytes([data[offset], data[offset + 1]]))
    }

    /// Read a u32 in little-endian format  
    pub fn read_u32_le(data: &[u8], offset: usize) -> ParseResult<u32> {
        validate_buffer_length(data, offset + 4)?;
        Ok(u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]))
    }

    /// Calculate padding to align to 4-byte boundary
    pub fn calculate_padding(length: usize) -> usize {
        (4 - (length % 4)) % 4
    }

    /// Calculate padded length for 4-byte alignment
    pub fn padded_length(length: usize) -> usize {
        (length + 3) & !3
    }
}
