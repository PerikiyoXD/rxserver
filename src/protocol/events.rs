//! X11 event handling
//!
//! This module defines X11 events that are sent from the server to clients.

use crate::protocol::types::*;
use crate::{todo_high, todo_medium};

/// X11 event types
#[derive(Debug, Clone)]
pub enum Event {
    KeyPress {
        detail: KeyCode,
        time: Timestamp,
        root: Window,
        event: Window,
        child: Window,
        root_x: i16,
        root_y: i16,
        event_x: i16,
        event_y: i16,
        state: u16,
        same_screen: bool,
    },
    KeyRelease {
        detail: KeyCode,
        time: Timestamp,
        root: Window,
        event: Window,
        child: Window,
        root_x: i16,
        root_y: i16,
        event_x: i16,
        event_y: i16,
        state: u16,
        same_screen: bool,
    },
    ButtonPress {
        detail: u8,
        time: Timestamp,
        root: Window,
        event: Window,
        child: Window,
        root_x: i16,
        root_y: i16,
        event_x: i16,
        event_y: i16,
        state: u16,
        same_screen: bool,
    },
    ButtonRelease {
        detail: u8,
        time: Timestamp,
        root: Window,
        event: Window,
        child: Window,
        root_x: i16,
        root_y: i16,
        event_x: i16,
        event_y: i16,
        state: u16,
        same_screen: bool,
    },
    MotionNotify {
        detail: u8,
        time: Timestamp,
        root: Window,
        event: Window,
        child: Window,
        root_x: i16,
        root_y: i16,
        event_x: i16,
        event_y: i16,
        state: u16,
        same_screen: bool,
    },
    EnterNotify {
        detail: u8,
        time: Timestamp,
        root: Window,
        event: Window,
        child: Window,
        root_x: i16,
        root_y: i16,
        event_x: i16,
        event_y: i16,
        state: u16,
        mode: u8,
        same_screen_focus: u8,
    },
    LeaveNotify {
        detail: u8,
        time: Timestamp,
        root: Window,
        event: Window,
        child: Window,
        root_x: i16,
        root_y: i16,
        event_x: i16,
        event_y: i16,
        state: u16,
        mode: u8,
        same_screen_focus: u8,
    },
    Expose {
        window: Window,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        count: u16,
    },
    MapNotify {
        event: Window,
        window: Window,
        override_redirect: bool,
    },
    UnmapNotify {
        event: Window,
        window: Window,
        from_configure: bool,
    },
    ConfigureNotify {
        event: Window,
        window: Window,
        above_sibling: Window,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        override_redirect: bool,
    },
    CreateNotify {
        parent: Window,
        window: Window,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        override_redirect: bool,
    },
    DestroyNotify {
        event: Window,
        window: Window,
    },
}

impl Event {
    /// Get the event code for this event type
    pub fn event_code(&self) -> u8 {
        match self {
            Event::KeyPress { .. } => 2,
            Event::KeyRelease { .. } => 3,
            Event::ButtonPress { .. } => 4,
            Event::ButtonRelease { .. } => 5,
            Event::MotionNotify { .. } => 6,
            Event::EnterNotify { .. } => 7,
            Event::LeaveNotify { .. } => 8,
            Event::Expose { .. } => 12,
            Event::MapNotify { .. } => 19,
            Event::UnmapNotify { .. } => 18,
            Event::ConfigureNotify { .. } => 22,
            Event::CreateNotify { .. } => 16,
            Event::DestroyNotify { .. } => 17,
        }
    }

    /// Serialize event to bytes for sending to client
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = vec![0u8; 32]; // X11 events are always 32 bytes
        data[0] = self.event_code();
        todo_medium!("event_serialization", "Implement event serialization");
        match self {
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
                data[1] = *detail;
                data[2..6].copy_from_slice(&time.to_ne_bytes());
                data[6..10].copy_from_slice(&root.to_ne_bytes());
                data[10..14].copy_from_slice(&event.to_ne_bytes());
                data[14..18].copy_from_slice(&child.to_ne_bytes());
                data[18..20].copy_from_slice(&root_x.to_ne_bytes());
                data[20..22].copy_from_slice(&root_y.to_ne_bytes());
                data[22..24].copy_from_slice(&event_x.to_ne_bytes());
                data[24..26].copy_from_slice(&event_y.to_ne_bytes());
                data[26..28].copy_from_slice(&state.to_ne_bytes());
                data[28] = if *same_screen { 1 } else { 0 };
            } // Handle other event types similarly...
            _ => {
                todo_high!(
                    "event_serialization",
                    "Handle serialization for other event types"
                );
                // Return empty data for now
                data
            }
        }
        data
    }
}
