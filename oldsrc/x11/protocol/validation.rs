//! X11 Protocol Message Validation
//!
//! This module provides validation for X11 protocol messages and parameters.

use tracing::warn;

use crate::x11::protocol::{
    errors::{ProtocolError, X11Error},
    opcodes::Opcode,
    types::*,
};

/// Protocol message validator
pub struct X11ProtocolValidator;

/// Shared zero-check for all XId types
macro_rules! xid_zero_check {
    ($name:ident, $typ:ty, $err:expr) => {
        pub fn $name(id: $typ) -> Result<(), X11Error> {
            if id == 0 {
                warn!("Invalid {}: 0", stringify!($typ));
                return Err($err);
            }
            Ok(())
        }
    };
}

impl X11ProtocolValidator {
    /// Validate a request message
    pub fn validate_request(request: &Request) -> Result<(), X11Error> {
        match request.kind {
            // Setup request validation
            RequestKind::ConnectionSetup(connection_setup) => todo!(),
            // Opcode request validation
            RequestKind::CreateWindow(req) => Self::validate_create_window(req),
            RequestKind::DestroyWindow(req) => Self::validate(req),
            RequestKind::MapWindow(req) => Self::validate(req),
            RequestKind::UnmapWindow(req) => Self::validate(req),
            RequestKind::InternAtom(req) => Self::validate(req),
            RequestKind::CreateGlyphCursor(create_glyph_cursor) => todo!(),
            RequestKind::OpenFont(open_font) => todo!(),
            RequestKind::NoOperation(no_operation) => todo!(),
            RequestKind::GetGeometry(get_geometry) => todo!(),
        }
    }

    /// Validate CreateWindow parameters
    fn validate(req: &CreateWindow) -> Result<(), X11Error> {
        // Width and height must be non-zero
        if req.width == 0 || req.height == 0 {
            tracing::error!(
                "Invalid CreateWindow: width={}, height={}",
                req.width,
                req.height
            );
            return Err(X11Error::Value);
        }

        // Depth must be non-zero
        if req.depth == 0 {
            tracing::error!("Invalid CreateWindow: depth={}", req.depth);
            return Err(X11Error::Value);
        }

        // Class must be within valid range (1-2 for X11, 3 for X11R6)
        if req.class > 2 {
            tracing::error!("Invalid CreateWindow: class={}", req.class);
            return Err(X11Error::Value);
        }

        // Border width must be within valid range (0-1000)
        if req.border_width > 1000 {
            tracing::error!("Invalid CreateWindow: border_width={}", req.border_width);
            return Err(X11Error::Value)?;
        }

        Ok(())
    }

    /// Validate DestroyWindow parameters
    fn validate(req: &DestroyWindow) -> Result<(), X11Error> {
        Self::validate_window_id(req.window)
    }

    xid_zero_check!(validate_window_id, WindowId, X11Error::Window);
    xid_zero_check!(validate_pixmap_id, PixmapId, X11Error::Pixmap);
    xid_zero_check!(validate_gc_id, GContextId, X11Error::GContext);
    xid_zero_check!(validate_font_id, FontId, X11Error::Font);
    xid_zero_check!(validate_cursor_id, CursorId, X11Error::Cursor);
    xid_zero_check!(validate_colormap_id, ColormapId, X11Error::Colormap);
    xid_zero_check!(validate_atom_id, Atom, X11Error::Atom);
    xid_zero_check!(validate_drawable_id, XId, X11Error::Drawable);

    /// Validate atom name
    fn validate_atom_name(name: &str) -> Result<(), X11Error> {
        Self::validate_name_len(name, 255, X11Error::Atom)
    }
    /// Validate font name
    pub fn validate_font_name(name: &str) -> Result<(), X11Error> {
        Self::validate_name_len(name, 255, X11Error::Font)
    }
    /// Helper for atom/font names
    fn validate_name_len(name: &str, max: usize, err: X11Error) -> Result<(), X11Error> {
        if name.is_empty() {
            warn!("Name is empty");
            return Err(err);
        }
        if name.len() > max {
            warn!("Name exceeds max length: {}", name.len());
            return Err(err);
        }
        Ok(())
    }

    /// Validate sequence number range
    pub fn validate_sequence_number(seq: SequenceNumber) -> Result<(), ProtocolError> {
        let _ = seq;
        todo!(
            "Sequence number validation is not strictly necessary in X11, but can be added for consistency"
        );
    }

    /// Validate coordinate values
    pub fn validate_coordinates(x: i16, y: i16) -> Result<(), X11Error> {
        let _ = x;
        let _ = y;
        todo!(
            "Coordinate validation is not strictly necessary in X11, but can be added for consistency"
        );
    }

    /// Validate dimension values
    pub fn validate_dimensions(width: u16, height: u16) -> Result<(), X11Error> {
        if width == 0 || height == 0 {
            return Err(X11Error::Value);
        }
        Ok(())
    }

    /// Validate rectangle bounds
    pub fn validate_rectangle(rect: &Rectangle) -> Result<(), X11Error> {
        Self::validate_coordinates(rect.x, rect.y)?;
        Self::validate_dimensions(rect.width, rect.height)?;
        Ok(())
    }

    /// Validate point coordinates
    pub fn validate_point(point: &Point) -> Result<(), X11Error> {
        Self::validate_coordinates(point.x, point.y)
    }

    /// Validate timestamp
    pub fn validate_timestamp(timestamp: Timestamp) -> Result<(), X11Error> {
        let _ = timestamp;
        todo!(
            "Timestamp validation is not strictly necessary in X11, but can be added for consistency"
        );
    }

    /// Validate message length
    pub fn validate_message_length(
        length: u16,
        min_length: u16,
        max_length: Option<u16>,
    ) -> Result<(), ProtocolError> {
        if length < min_length {
            return Err(ProtocolError::MessageTooShort {
                expected: min_length as usize,
                actual: length as usize,
            });
        }
        if let Some(max) = max_length {
            if length > max {
                return Err(ProtocolError::MessageTooLong {
                    max: max as usize,
                    actual: length as usize,
                });
            }
        }
        Ok(())
    }

    /// Validate opcode
    pub fn validate_opcode(opcode: u8) -> Result<Opcode, ProtocolError> {
        Opcode::from_u8(opcode).ok_or(ProtocolError::InvalidOpcode(opcode))
    }

    /// Validate protocol version
    pub fn validate_protocol_version(major: u16, minor: u16) -> Result<(), ProtocolError> {
        if major != 11 {
            return Err(ProtocolError::UnsupportedVersion { major, minor });
        }
        if minor > 0 {
            // Accept any minor version for now
        }
        Ok(())
    }

    /// Validate padding in messages
    pub fn validate_padding(bytes: &[u8], start: usize, count: usize) -> Result<(), ProtocolError> {
        for i in start..start + count {
            if i < bytes.len() && bytes[i] != 0 {
                return Err(ProtocolError::InvalidPadding);
            }
        }
        Ok(())
    }
}

/// Range validation utilities
pub mod ranges {
    use super::*;
    pub fn validate_range<T: PartialOrd>(value: T, min: T, max: T) -> Result<(), X11Error> {
        if value < min || value > max {
            Err(X11Error::Value)
        } else {
            Ok(())
        }
    }
    pub fn validate_u32_range(value: u32, min: u32, max: u32) -> Result<(), X11Error> {
        validate_range(value, min, max)
    }
    pub fn validate_i32_range(value: i32, min: i32, max: i32) -> Result<(), X11Error> {
        validate_range(value, min, max)
    }
    pub fn validate_u16_range(value: u16, min: u16, max: u16) -> Result<(), X11Error> {
        validate_range(value, min, max)
    }
    pub fn validate_i16_range(value: i16, min: i16, max: i16) -> Result<(), X11Error> {
        validate_range(value, min, max)
    }
    pub fn validate_u8_range(value: u8, min: u8, max: u8) -> Result<(), X11Error> {
        validate_range(value, min, max)
    }
}
