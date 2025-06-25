//! Core X11 Protocol Types
//!
//! This module defines the fundamental types used throughout the X11 protocol.

use std::fmt;

/// X11 Resource Identifier - 29-bit identifier for X11 resources
pub type XId = u32;

/// Client identifier for tracking connections
pub type ClientId = u32;

/// Sequence number for request/response matching
pub type SequenceNumber = u16;

/// X11 Timestamp
pub type Timestamp = u32;

/// X11 Atom identifier
pub type Atom = u32;

/// Window ID type alias
pub type WindowId = XId;

/// Pixmap ID type alias  
pub type PixmapId = XId;

/// Visual ID type alias
pub type VisualId = XId;

/// Drawable ID type alias
pub type DrawableId = XId;

/// Graphics Context ID type alias
pub type GContextId = XId;

/// Font ID type alias
pub type FontId = XId;

/// Cursor ID type alias
pub type CursorId = XId;

/// Colormap ID type alias
pub type ColormapId = XId;

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
use crate::x11::protocol::request;

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
