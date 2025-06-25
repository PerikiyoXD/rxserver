//! X11 Protocol Error Definitions
//!
//! This module defines the X11 protocol error types and error response generation.

use crate::{
    RequestKind,
    x11::protocol::types::{SequenceNumber, XId},
};

/// X11 Protocol Error Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum X11Error {
    /// Request: request type or datum out-of-range
    Request = 1,
    /// Value: integer parameter out of range
    Value = 2,
    /// Window: parameter not a Window
    Window = 3,
    /// Pixmap: parameter not a Pixmap
    Pixmap = 4,
    /// Atom: parameter not an Atom
    Atom = 5,
    /// Cursor: parameter not a Cursor
    Cursor = 6,
    /// Font: parameter not a Font
    Font = 7,
    /// Match: parameter mismatch
    Match = 8,
    /// Drawable: parameter not a Pixmap or Window
    Drawable = 9,
    /// Access: attempt to access private resource of another client
    Access = 10,
    /// Alloc: insufficient resources
    Alloc = 11,
    /// Colormap: parameter not a Colormap
    Colormap = 12,
    /// GContext: parameter not a GContext
    GContext = 13,
    /// IDChoice: choice not in range or already used
    IDChoice = 14,
    /// Name: font or color name doesn't exist
    Name = 15,
    /// Length: Request length incorrect; can't be corrected
    Length = 16,
    /// Implementation: server is defective
    Implementation = 17,
}

impl X11Error {
    /// Get the numeric error code
    pub fn code(self) -> u8 {
        self as u8
    }

    /// Get a human-readable description of the error
    pub fn description(self) -> &'static str {
        match self {
            X11Error::Request => "Bad request code or invalid parameters",
            X11Error::Value => "Integer parameter out of range",
            X11Error::Window => "Parameter is not a Window",
            X11Error::Pixmap => "Parameter is not a Pixmap",
            X11Error::Atom => "Parameter is not an Atom",
            X11Error::Cursor => "Parameter is not a Cursor",
            X11Error::Font => "Parameter is not a Font",
            X11Error::Match => "Parameter mismatch",
            X11Error::Drawable => "Parameter is not a Pixmap or Window",
            X11Error::Access => "Access denied to private resource",
            X11Error::Alloc => "Insufficient resources",
            X11Error::Colormap => "Parameter is not a Colormap",
            X11Error::GContext => "Parameter is not a Graphics Context",
            X11Error::IDChoice => "Choice not in range or already used",
            X11Error::Name => "Font or color name doesn't exist",
            X11Error::Length => "Request length incorrect",
            X11Error::Implementation => "Server implementation error",
        }
    }
}

/// Protocol-level error for parsing and validation
#[derive(Debug, Clone)]
pub enum ProtocolError {
    // Parsing/Format errors
    InvalidFormat,
    InvalidPadding,
    InvalidByteOrder(u8),
    InsufficientData,
    Serialization(String),
    MessageTooShort { expected: usize, actual: usize },
    MessageTooLong { max: usize, actual: usize },

    // Protocol/Request errors
    BadRequest,
    InvalidOpcode(u8),
    UnsupportedVersion { major: u16, minor: u16 },
    UnimplementedOpcode { opcode: u8 },
    UnimplementedRequestHandler(RequestKind),
    InvalidRequestType { expected: String, actual: String },
    InvalidSequence,

    // Resource/Parameter errors
    InvalidWindowId { window_id: u32 },
    InvalidWindowDimensions { width: u16, height: u16 },
    InvalidDepth { depth: u8 },
    InvalidDrawableId { drawable_id: u32 },
    InvalidCursorId { cursor_id: u32 },
    InvalidFontId { font_id: u32 },
    InvalidFontName { name: String },
    FontNameTooLong { name: String, length: usize },
    InvalidAtomName { name: String },
    AtomNameTooLong { name: String, length: usize },
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::InvalidFormat => write!(f, "Invalid message format"),
            ProtocolError::UnsupportedVersion { major, minor } => {
                write!(f, "Unsupported protocol version {}.{}", major, minor)
            }
            ProtocolError::InvalidOpcode(code) => write!(f, "Invalid opcode: {}", code),
            ProtocolError::MessageTooShort { expected, actual } => {
                write!(
                    f,
                    "Message too short: expected {}, got {}",
                    expected, actual
                )
            }
            ProtocolError::MessageTooLong { max, actual } => {
                write!(f, "Message too long: max {}, got {}", max, actual)
            }
            ProtocolError::InvalidByteOrder(order) => {
                write!(f, "Invalid byte order marker: {}", order)
            }
            ProtocolError::InvalidPadding => write!(f, "Invalid padding"),
            ProtocolError::InvalidSequence => write!(f, "Invalid sequence number"),
            ProtocolError::InsufficientData => write!(f, "Insufficient data for parsing"),
            ProtocolError::BadRequest => write!(f, "Bad request (unimplemented or invalid)"),
            ProtocolError::UnimplementedOpcode { opcode, .. } => {
                write!(f, "Opcode not implemented: {}", opcode)
            }
            ProtocolError::UnimplementedRequestHandler(request_kind) => {
                write!(f, "Unimplemented request handler: {:?}", request_kind)
            }
            ProtocolError::Serialization(string) => write!(f, "Serialization error: {}", string),

            // New error variants
            ProtocolError::InvalidRequestType { expected, actual } => {
                write!(
                    f,
                    "Invalid request type: expected {}, got {}",
                    expected, actual
                )
            }
            ProtocolError::InvalidWindowDimensions { width, height } => {
                write!(f, "Invalid window dimensions: {}x{}", width, height)
            }
            ProtocolError::InvalidDepth { depth } => {
                write!(f, "Invalid depth: {}", depth)
            }
            ProtocolError::InvalidWindowId { window_id } => {
                write!(f, "Invalid window ID: {}", window_id)
            }
            ProtocolError::InvalidAtomName { name } => {
                write!(f, "Invalid atom name: '{}'", name)
            }
            ProtocolError::AtomNameTooLong { name, length } => {
                write!(f, "Atom name too long: '{}' ({} chars)", name, length)
            }
            ProtocolError::InvalidCursorId { cursor_id } => {
                write!(f, "Invalid cursor ID: {}", cursor_id)
            }
            ProtocolError::InvalidFontId { font_id } => {
                write!(f, "Invalid font ID: {}", font_id)
            }
            ProtocolError::InvalidFontName { name } => {
                write!(f, "Invalid font name: '{}'", name)
            }
            ProtocolError::FontNameTooLong { name, length } => {
                write!(f, "Font name too long: '{}' ({} chars)", name, length)
            }
            ProtocolError::InvalidDrawableId { drawable_id } => {
                write!(f, "Invalid drawable ID: {}", drawable_id)
            }
        }
    }
}

impl std::error::Error for ProtocolError {}

/// Error response message
#[derive(Debug, Clone)]
pub struct ErrorMessage {
    pub error_code: u8,
    pub sequence_number: SequenceNumber,
    pub resource_id: XId,
    pub minor_opcode: u16,
    pub major_opcode: u8,
}

impl ErrorMessage {
    /// Create a new error message
    pub fn new(
        error: X11Error,
        sequence_number: SequenceNumber,
        resource_id: XId,
        major_opcode: u8,
        minor_opcode: u16,
    ) -> Self {
        Self {
            error_code: error.code(),
            sequence_number,
            resource_id,
            minor_opcode,
            major_opcode,
        }
    }

    /// Serialize this error message to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(32);
        bytes.push(0); // Error type
        bytes.push(self.error_code);
        bytes.extend_from_slice(&self.sequence_number.to_le_bytes());
        bytes.extend_from_slice(&self.resource_id.to_le_bytes());
        bytes.extend_from_slice(&self.minor_opcode.to_le_bytes());
        bytes.push(self.major_opcode);
        bytes.resize(32, 0); // Pad to 32 bytes
        bytes
    }
}
