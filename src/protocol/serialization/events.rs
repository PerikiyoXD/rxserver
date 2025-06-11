//! Event Serialization
//!
//! This module handles serialization of X11 events to wire format.

use crate::protocol::message::Event;
use crate::protocol::types::EventType;
use crate::todo_high;
use bytes::{BufMut, BytesMut};
use log::debug;

/// Serialize an event to wire format
pub fn serialize_event(event: &Event, buf: &mut BytesMut) {
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
        Event::KeyPress {
            detail,
            time,
            root,
            event,
            child,
            root_x,
            root_y,
            event_x,
            event_y,
            state,
            same_screen,
        } => {
            buf.put_u8(EventType::KeyPress as u8);
            buf.put_u8(*detail);
            buf.put_u16(0); // Sequence number
            buf.put_u32(*time);
            buf.put_u32(*root);
            buf.put_u32(*event);
            buf.put_u32(*child);
            buf.put_i16(*root_x);
            buf.put_i16(*root_y);
            buf.put_i16(*event_x);
            buf.put_i16(*event_y);
            buf.put_u16(*state);
            buf.put_u8(if *same_screen { 1 } else { 0 });
            buf.put_u8(0); // Padding
        }
        Event::KeyRelease {
            detail,
            time,
            root,
            event,
            child,
            root_x,
            root_y,
            event_x,
            event_y,
            state,
            same_screen,
        } => {
            buf.put_u8(EventType::KeyRelease as u8);
            buf.put_u8(*detail);
            buf.put_u16(0); // Sequence number
            buf.put_u32(*time);
            buf.put_u32(*root);
            buf.put_u32(*event);
            buf.put_u32(*child);
            buf.put_i16(*root_x);
            buf.put_i16(*root_y);
            buf.put_i16(*event_x);
            buf.put_i16(*event_y);
            buf.put_u16(*state);
            buf.put_u8(if *same_screen { 1 } else { 0 });
            buf.put_u8(0); // Padding
        }
        Event::ButtonPress {
            detail,
            time,
            root,
            event,
            child,
            root_x,
            root_y,
            event_x,
            event_y,
            state,
            same_screen,
        } => {
            buf.put_u8(EventType::ButtonPress as u8);
            buf.put_u8(*detail);
            buf.put_u16(0); // Sequence number
            buf.put_u32(*time);
            buf.put_u32(*root);
            buf.put_u32(*event);
            buf.put_u32(*child);
            buf.put_i16(*root_x);
            buf.put_i16(*root_y);
            buf.put_i16(*event_x);
            buf.put_i16(*event_y);
            buf.put_u16(*state);
            buf.put_u8(if *same_screen { 1 } else { 0 });
            buf.put_u8(0); // Padding
        }
        Event::ButtonRelease {
            detail,
            time,
            root,
            event,
            child,
            root_x,
            root_y,
            event_x,
            event_y,
            state,
            same_screen,
        } => {
            buf.put_u8(EventType::ButtonRelease as u8);
            buf.put_u8(*detail);
            buf.put_u16(0); // Sequence number
            buf.put_u32(*time);
            buf.put_u32(*root);
            buf.put_u32(*event);
            buf.put_u32(*child);
            buf.put_i16(*root_x);
            buf.put_i16(*root_y);
            buf.put_i16(*event_x);
            buf.put_i16(*event_y);
            buf.put_u16(*state);
            buf.put_u8(if *same_screen { 1 } else { 0 });
            buf.put_u8(0); // Padding
        }
        _ => {
            debug!("Unhandled event type: {:?}", event);
            todo_high!(
                "event_serialization",
                "Most event types not implemented - only basic events work"
            );
            // Default serialization for unimplemented events
            buf.put_u8(0);
            for _ in 0..31 {
                buf.put_u8(0);
            }
        }
    }
}
