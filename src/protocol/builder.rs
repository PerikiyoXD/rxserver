//! X11 Response Builder
//!
//! This module provides utilities for constructing X11 protocol responses.
//! It acts as a high-level interface over the serialization module, providing
//! convenient methods for creating common response types.

use crate::protocol::message::{Response, Reply, Event, ErrorResponse};
use crate::protocol::serialization::ResponseSerializer;
use crate::protocol::types::X11Error;
use crate::{Result};

/// Builder for constructing X11 protocol responses
pub struct ResponseBuilder;

impl ResponseBuilder {
    /// Create a new response builder
    pub fn new() -> Self {
        ResponseBuilder
    }

    /// Create an error response
    pub fn error(
        error_code: u8,
        sequence: u16,
        bad_value: u32,
        minor_opcode: u16,
        major_opcode: u8,
    ) -> Response {
        Response::Error(ErrorResponse {
            error_code: X11Error::from(error_code),
            sequence_number: sequence,
            bad_value,
            minor_opcode,
            major_opcode,
        })
    }

    /// Create a success response (using a minimal geometry reply as placeholder)
    pub fn success() -> Response {
        Response::Reply(Reply::GetGeometry(
            crate::protocol::message::replies::GetGeometryReply {
                depth: 24,
                root: 1,
                x: 0,
                y: 0,
                width: 1024,
                height: 768,
                border_width: 0,
            },
        ))
    }

    /// Serialize a response to wire format
    pub fn serialize(response: &Response) -> Result<Vec<u8>> {
        Ok(ResponseSerializer::serialize(response, 0))
    }

    /// Create an InternAtom reply
    pub fn intern_atom_reply(atom: u32) -> Response {
        Response::Reply(Reply::InternAtom(
            crate::protocol::message::replies::InternAtomReply { atom }
        ))
    }

    /// Create a GrabPointer reply
    pub fn grab_pointer_reply(status: u8) -> Response {
        Response::Reply(Reply::GrabPointer(
            crate::protocol::message::replies::GrabPointerReply { status }
        ))
    }

    /// Create an Expose event
    pub fn expose_event(
        window: u32,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        count: u16,
    ) -> Response {
        Response::Event(Event::Expose {
            window,
            x,
            y,
            width,
            height,
            count,
        })
    }

    /// Create a ConfigureNotify event
    pub fn configure_notify_event(
        event: u32,
        window: u32,
        above_sibling: u32,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        override_redirect: bool,
    ) -> Response {
        Response::Event(Event::ConfigureNotify {
            event,
            window,
            above_sibling,
            x,
            y,
            width,
            height,
            border_width,
            override_redirect,
        })
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}
