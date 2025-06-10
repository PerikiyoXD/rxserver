//! X11 response generation
//!
//! This module handles generating X11 responses to send back to clients.

use super::types::*;
use crate::{Error, Result};
use bytes::{BufMut, BytesMut};

/// X11 response types
#[derive(Debug, Clone)]
pub enum Response {
    /// Success response
    Success,
    /// Error response
    Error {
        error_code: u8,
        sequence_number: u16,
        bad_value: u32,
        minor_opcode: u16,
        major_opcode: u8,
    },
    /// Reply with data
    Reply {
        data: u8,
        sequence_number: u16,
        length: u32,
        body: Vec<u8>,
    },
}

/// X11 response builder
pub struct ResponseBuilder;

impl ResponseBuilder {
    /// Build a success response
    pub fn success() -> Response {
        Response::Success
    }

    /// Build an error response
    pub fn error(
        error_code: u8,
        sequence_number: u16,
        bad_value: u32,
        minor_opcode: u16,
        major_opcode: u8,
    ) -> Response {
        Response::Error {
            error_code,
            sequence_number,
            bad_value,
            minor_opcode,
            major_opcode,
        }
    }

    /// Build a reply response
    pub fn reply(data: u8, sequence_number: u16, body: Vec<u8>) -> Response {
        let length = (body.len() / 4) as u32; // Length in 4-byte units
        Response::Reply {
            data,
            sequence_number,
            length,
            body,
        }
    }

    /// Serialize a response to bytes
    pub fn serialize(response: &Response) -> Result<Vec<u8>> {
        let mut buf = BytesMut::new();

        match response {
            Response::Success => {
                buf.put_u8(1); // Success indicator
                buf.put_u8(0); // Unused
                buf.put_u16(0); // Sequence number
                buf.put_u32(0); // Length
                buf.resize(32, 0); // Pad to 32 bytes
            }
            Response::Error {
                error_code,
                sequence_number,
                bad_value,
                minor_opcode,
                major_opcode,
            } => {
                buf.put_u8(0); // Error indicator
                buf.put_u8(*error_code);
                buf.put_u16(*sequence_number);
                buf.put_u32(*bad_value);
                buf.put_u16(*minor_opcode);
                buf.put_u8(*major_opcode);
                // Pad to 32 bytes
                buf.resize(32, 0);
            }
            Response::Reply {
                data,
                sequence_number,
                length,
                body,
            } => {
                buf.put_u8(1); // Reply indicator
                buf.put_u8(*data);
                buf.put_u16(*sequence_number);
                buf.put_u32(*length);
                // Pad header to 32 bytes
                buf.resize(32, 0);
                // Add body
                buf.extend_from_slice(body);
                // Pad body to 4-byte boundary
                while buf.len() % 4 != 0 {
                    buf.put_u8(0);
                }
            }
        }

        Ok(buf.to_vec())
    }
}
