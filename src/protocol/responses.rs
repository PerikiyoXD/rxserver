//! X11 Protocol Responses
//!
//! This module handles the generation of X11 protocol responses, replies, and errors.
//! All responses are type-safe and follow the X11 protocol specification.

pub use crate::protocol::events::Event;
use crate::protocol::types::*;
use crate::{todo_high, todo_medium};
use bytes::{BufMut, BytesMut};
use log::debug;
use std::fmt;

/// Generic X11 response type
#[derive(Debug, Clone)]
pub enum Response {
    Reply(Reply),
    Event(Event),
    Error(ErrorResponse),
}

/// X11 reply to a request
#[derive(Debug, Clone)]
pub enum Reply {
    GetWindowAttributes(GetWindowAttributesReply),
    GetGeometry(GetGeometryReply),
    GetProperty(GetPropertyReply),
    QueryTree(QueryTreeReply),
    InternAtom(InternAtomReply),
    GetAtomName(GetAtomNameReply),
    // Add more replies as needed
}

/// X11 error response
#[derive(Debug, Clone)]
pub struct ErrorResponse {
    pub error_code: X11Error,
    pub sequence_number: u16,
    pub bad_value: u32,
    pub minor_opcode: u16,
    pub major_opcode: u8,
}

/// GetWindowAttributes reply
#[derive(Debug, Clone)]
pub struct GetWindowAttributesReply {
    pub backing_store: u8,
    pub visual: VisualId,
    pub class: WindowClass,
    pub bit_gravity: Gravity,
    pub win_gravity: Gravity,
    pub backing_planes: u32,
    pub backing_pixel: u32,
    pub save_under: bool,
    pub map_is_installed: bool,
    pub map_state: u8,
    pub override_redirect: bool,
    pub colormap: Colormap,
    pub all_event_masks: EventMask,
    pub your_event_mask: EventMask,
    pub do_not_propagate_mask: EventMask,
}

/// GetGeometry reply
#[derive(Debug, Clone)]
pub struct GetGeometryReply {
    pub depth: u8,
    pub root: Window,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
}

/// GetProperty reply
#[derive(Debug, Clone)]
pub struct GetPropertyReply {
    pub format: u8,
    pub property_type: Atom,
    pub bytes_after: u32,
    pub data: Vec<u8>,
}

/// QueryTree reply
#[derive(Debug, Clone)]
pub struct QueryTreeReply {
    pub root: Window,
    pub parent: Window,
    pub children: Vec<Window>,
}

/// InternAtom reply
#[derive(Debug, Clone)]
pub struct InternAtomReply {
    pub atom: Atom,
}

/// GetAtomName reply
#[derive(Debug, Clone)]
pub struct GetAtomNameReply {
    pub name: String,
}

/// Expose event
#[derive(Debug, Clone)]
pub struct ExposeEvent {
    pub window: Window,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub count: u16,
}

/// ConfigureNotify event
#[derive(Debug, Clone)]
pub struct ConfigureNotifyEvent {
    pub event: Window,
    pub window: Window,
    pub above_sibling: Window,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub override_redirect: bool,
}

/// MapNotify event
#[derive(Debug, Clone)]
pub struct MapNotifyEvent {
    pub event: Window,
    pub window: Window,
    pub override_redirect: bool,
}

/// UnmapNotify event
#[derive(Debug, Clone)]
pub struct UnmapNotifyEvent {
    pub event: Window,
    pub window: Window,
    pub from_configure: bool,
}

/// DestroyNotify event
#[derive(Debug, Clone)]
pub struct DestroyNotifyEvent {
    pub event: Window,
    pub window: Window,
}

/// KeyPress event
#[derive(Debug, Clone)]
pub struct KeyPressEvent {
    pub detail: KeyCode,
    pub time: Timestamp,
    pub root: Window,
    pub event: Window,
    pub child: Window,
    pub root_x: i16,
    pub root_y: i16,
    pub event_x: i16,
    pub event_y: i16,
    pub state: ModifierMask,
    pub same_screen: bool,
}

/// KeyRelease event (same structure as KeyPress)
pub type KeyReleaseEvent = KeyPressEvent;

/// ButtonPress event
#[derive(Debug, Clone)]
pub struct ButtonPressEvent {
    pub detail: Button,
    pub time: Timestamp,
    pub root: Window,
    pub event: Window,
    pub child: Window,
    pub root_x: i16,
    pub root_y: i16,
    pub event_x: i16,
    pub event_y: i16,
    pub state: ModifierMask,
    pub same_screen: bool,
}

/// ButtonRelease event (same structure as ButtonPress)
pub type ButtonReleaseEvent = ButtonPressEvent;

/// MotionNotify event
#[derive(Debug, Clone)]
pub struct MotionNotifyEvent {
    pub detail: u8,
    pub time: Timestamp,
    pub root: Window,
    pub event: Window,
    pub child: Window,
    pub root_x: i16,
    pub root_y: i16,
    pub event_x: i16,
    pub event_y: i16,
    pub state: ModifierMask,
    pub same_screen: bool,
}

/// Response serializer for converting responses to wire format
pub struct ResponseSerializer;

impl ResponseSerializer {
    /// Serialize a response to bytes
    pub fn serialize(response: &Response, sequence: u16) -> Vec<u8> {
        let mut buf = BytesMut::new();
        match response {
            Response::Reply(reply) => Self::serialize_reply(reply, sequence, &mut buf),
            Response::Event(event) => Self::serialize_event(event, &mut buf),
            Response::Error(error) => Self::serialize_error(error, &mut buf),
        }

        buf.to_vec()
    }

    fn serialize_reply(reply: &Reply, sequence: u16, buf: &mut BytesMut) {
        buf.put_u8(1); // Reply type

        match reply {
            Reply::GetWindowAttributes(reply) => {
                buf.put_u8(reply.backing_store);
                buf.put_u16(sequence);
                buf.put_u32(3); // Length in 4-byte units
                buf.put_u32(reply.visual);
                buf.put_u16(reply.class as u16);
                buf.put_u8(reply.bit_gravity as u8);
                buf.put_u8(reply.win_gravity as u8);
                buf.put_u32(reply.backing_planes);
                buf.put_u32(reply.backing_pixel);
                buf.put_u8(if reply.save_under { 1 } else { 0 });
                buf.put_u8(if reply.map_is_installed { 1 } else { 0 });
                buf.put_u8(reply.map_state);
                buf.put_u8(if reply.override_redirect { 1 } else { 0 });
                buf.put_u32(reply.colormap);
                buf.put_u32(reply.all_event_masks.bits());
                buf.put_u32(reply.your_event_mask.bits());
                buf.put_u32(reply.do_not_propagate_mask.bits());
                buf.put_u32(0); // Padding
            }
            Reply::InternAtom(reply) => {
                buf.put_u8(0); // Unused byte
                buf.put_u16(sequence);
                buf.put_u32(0); // Reply length (0 for fixed-length replies)
                buf.put_u32(reply.atom);
                // Pad to 32 bytes total (20 bytes of padding needed)
                for _ in 0..20 {
                    buf.put_u8(0);
                }
            }
            _ => {
                todo_medium!("protocol_responses", "Most reply types not implemented yet");
                // TODO: Implement other reply types
                buf.put_u8(0);
                buf.put_u16(sequence);
                buf.put_u32(0);
            }
        }
    }

    fn serialize_event(event: &Event, buf: &mut BytesMut) {
        match event {
            Event::Expose {
                window,
                x,
                y,
                width,
                height,
                count,
            } => {
                buf.put_u8(EventType::Expose as u8);
                buf.put_u8(0); // Padding
                buf.put_u16(0); // Sequence number (filled by connection)
                buf.put_u32(*window);
                buf.put_u16(*x);
                buf.put_u16(*y);
                buf.put_u16(*width);
                buf.put_u16(*height);
                buf.put_u16(*count);
                buf.put_u16(0); // Padding
                                // Pad to 32 bytes
                for _ in 0..14 {
                    buf.put_u8(0);
                }
            }
            Event::ConfigureNotify {
                event,
                window,
                above_sibling,
                x,
                y,
                width,
                height,
                border_width,
                override_redirect,
            } => {
                buf.put_u8(EventType::ConfigureNotify as u8);
                buf.put_u8(0); // Padding
                buf.put_u16(0); // Sequence number
                buf.put_u32(*event);
                buf.put_u32(*window);
                buf.put_u32(*above_sibling);
                buf.put_i16(*x);
                buf.put_i16(*y);
                buf.put_u16(*width);
                buf.put_u16(*height);
                buf.put_u16(*border_width);
                buf.put_u8(if *override_redirect { 1 } else { 0 });
                buf.put_u8(0); // Padding
                buf.put_u32(0); // Padding
            }
            _ => {
                debug!("Unhandled event type: {:?}", event);
                todo_high!(
                    "protocol_responses",
                    "Most event types not implemented - only basic events work"
                );
                // TODO: Implement other event types
                buf.put_u8(0);
                for _ in 0..31 {
                    buf.put_u8(0);
                }
            }
        }
    }

    fn serialize_error(error: &ErrorResponse, buf: &mut BytesMut) {
        buf.put_u8(0); // Error type
        buf.put_u8(error.error_code as u8);
        buf.put_u16(error.sequence_number);
        buf.put_u32(error.bad_value);
        buf.put_u16(error.minor_opcode);
        buf.put_u8(error.major_opcode);
        // Pad to 32 bytes
        for _ in 0..21 {
            buf.put_u8(0);
        }
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::Reply(reply) => write!(f, "Reply({:?})", reply),
            Response::Event(event) => write!(f, "Event({:?})", event),
            Response::Error(error) => {
                write!(f, "Error({:?}: {})", error.error_code, error.bad_value)
            }
        }
    }
}
