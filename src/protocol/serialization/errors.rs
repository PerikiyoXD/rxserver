//! Error Serialization
//!
//! This module handles serialization of X11 errors to wire format.

use crate::protocol::message::ErrorResponse;
use bytes::{BufMut, BytesMut};

/// Serialize an error to wire format
pub fn serialize_error(error: &ErrorResponse, buf: &mut BytesMut) {
    buf.put_u8(0); // Error type
    buf.put_u8(error.error_code as u8);
    buf.put_u16(error.sequence_number);
    buf.put_u32(error.bad_value);
    buf.put_u16(error.minor_opcode);
    buf.put_u8(error.major_opcode);
    
    // Pad to 32 bytes (21 bytes padding needed)
    for _ in 0..21 {
        buf.put_u8(0);
    }
}
