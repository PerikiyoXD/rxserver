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
pub struct ProtocolValidator;

/// Shared zero-check for all XID types
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

impl ProtocolValidator {
    /// Validate a request message
    pub fn validate_request(request: &Request) -> Result<(), X11Error> {
        match request {
            Request::CreateWindow {
                depth,
                width,
                height,
                border_width,
                class,
                ..
            } => Self::validate_create_window(*depth, *width, *height, *border_width, *class),
            Request::MapWindow { window }
            | Request::UnmapWindow { window }
            | Request::DestroyWindow { window } => Self::validate_window_id(*window),
            Request::InternAtom { name, .. } => Self::validate_atom_name(name),
            Request::OpenFont { font_id, name } => {
                Self::validate_font_id(*font_id)?;
                Self::validate_font_name(name)
            }
            Request::NoOperation => Ok(()),
        }
    }

    /// Validate CreateWindow parameters
    fn validate_create_window(
        depth: u8,
        width: u16,
        height: u16,
        border_width: u16,
        class: u16,
    ) -> Result<(), X11Error> {
        // Width and height must be non-zero
        if width == 0 || height == 0 {
            return Err(X11Error::Value);
        }
        if depth == 0 {
            return Err(X11Error::Value);
        }
        if class > 2 {
            return Err(X11Error::Value);
        }
        if border_width > 1000 {
            return Err(X11Error::Value);
        }
        Ok(())
    }

    xid_zero_check!(validate_window_id, WindowId, X11Error::Window);
    xid_zero_check!(validate_pixmap_id, PixmapId, X11Error::Pixmap);
    xid_zero_check!(validate_gc_id, GContextId, X11Error::GContext);
    xid_zero_check!(validate_font_id, FontId, X11Error::Font);
    xid_zero_check!(validate_cursor_id, CursorId, X11Error::Cursor);
    xid_zero_check!(validate_colormap_id, ColormapId, X11Error::Colormap);
    xid_zero_check!(validate_atom_id, Atom, X11Error::Atom);
    xid_zero_check!(validate_drawable_id, XID, X11Error::Drawable);

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_validate_create_window() {
        assert!(ProtocolValidator::validate_create_window(24, 100, 200, 1, 1).is_ok());
        assert!(matches!(
            ProtocolValidator::validate_create_window(24, 0, 200, 1, 1),
            Err(X11Error::Value)
        ));
        assert!(matches!(
            ProtocolValidator::validate_create_window(24, 100, 0, 1, 1),
            Err(X11Error::Value)
        ));
        assert!(matches!(
            ProtocolValidator::validate_create_window(24, 100, 200, 1, 5),
            Err(X11Error::Value)
        ));
    }
    #[test]
    fn test_validate_window_id() {
        assert!(ProtocolValidator::validate_window_id(123).is_ok());
        assert!(matches!(
            ProtocolValidator::validate_window_id(0),
            Err(X11Error::Window)
        ));
    }
    #[test]
    fn test_validate_protocol_version() {
        assert!(ProtocolValidator::validate_protocol_version(11, 0).is_ok());
        assert!(matches!(
            ProtocolValidator::validate_protocol_version(10, 0),
            Err(ProtocolError::UnsupportedVersion { .. })
        ));
    }
}
