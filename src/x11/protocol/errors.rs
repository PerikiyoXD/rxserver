//! X11 Protocol Error Definitions
//!
//! This module defines the X11 protocol error types and error response generation.

use crate::x11::protocol::types::{SequenceNumber, XID};

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
    /// Invalid message format
    InvalidFormat,
    /// Unsupported protocol version
    UnsupportedVersion { major: u16, minor: u16 },
    /// Invalid opcode
    InvalidOpcode(u8),
    /// Message too short
    MessageTooShort { expected: usize, actual: usize },
    /// Message too long
    MessageTooLong { max: usize, actual: usize },
    /// Invalid byte order marker
    InvalidByteOrder(u8),
    /// Invalid padding
    InvalidPadding,
    /// Invalid sequence number
    InvalidSequence,
    /// Insufficient data for parsing
    InsufficientData,
    /// Bad request (unimplemented or invalid)
    BadRequest,
    /// Unsupported Opcode
    UnimplementedOpcode { opcode: u8 },
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
                write!(f, "Unsupported opcode: {}", opcode)
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
    pub resource_id: XID,
    pub minor_opcode: u16,
    pub major_opcode: u8,
}

impl ErrorMessage {
    /// Create a new error message
    pub fn new(
        error: X11Error,
        sequence_number: SequenceNumber,
        resource_id: XID,
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
