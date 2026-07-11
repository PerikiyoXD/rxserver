// events.rs
//! X11 event structures and handling

use crate::protocol::{ByteOrder, endianness::ByteOrderWriter, types::*};

/// X11 event types
#[derive(Debug, Clone)]
pub enum Event {
    Expose(ExposeEvent),
    XIRawMotion(XIRawMotionEvent),
    // Add other events as needed
}

/// Expose event - sent when a window needs to be redrawn
#[derive(Debug, Clone, Copy)]
pub struct ExposeEvent {
    pub window: WindowId,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub count: u16, // Number of remaining expose events
}

impl ExposeEvent {
    pub fn new(window: WindowId, x: i16, y: i16, width: u16, height: u16, count: u16) -> Self {
        Self {
            window,
            x,
            y,
            width,
            height,
            count,
        }
    }

    /// Serialize the expose event to bytes. `sequence_number` must be the
    /// server's current sequence number for this client (the request this
    /// event is a side effect of) - 0 is never valid here, unlike a fresh
    /// client's initial request sequence state; Xlib's event/reply demuxer
    /// relies on this field to stay consistent with what it's already
    /// tracking.
    pub fn serialize(&self, byte_order: ByteOrder, sequence_number: u16) -> Vec<u8> {
        let mut writer = ByteOrderWriter::new(byte_order);
        writer.write_u8(12); // Event code for Expose
        writer.write_u8(0); // Unused
        writer.write_u16(sequence_number);
        writer.write_u32(self.window);
        writer.write_u16(self.x as u16); // Cast to unsigned for transmission
        writer.write_u16(self.y as u16);
        writer.write_u16(self.width);
        writer.write_u16(self.height);
        writer.write_u16(self.count);
        writer.write_padding(14); // Padding to 32 bytes
        writer.into_vec()
    }
}

/// XI2's own event type constants (from X11/extensions/XI2.h), distinct
/// from the core X11 event codes above - these are the `evtype` field
/// inside a GenericEvent's XI2 payload, not the wire event code itself
/// (which is always 35, GenericEvent, for every XI2 event).
pub mod xi2_event_type {
    pub const RAW_MOTION: u16 = 17;
}

/// XI2's device-id pseudo-constants (from X11/extensions/XI2.h) - not real
/// device ids, used only in requests/events that mean "all devices of this
/// kind" rather than one specific device.
pub mod xi2_device {
    pub const ALL_DEVICES: u16 = 0;
    pub const ALL_MASTER_DEVICES: u16 = 1;
}

/// XI2 RawMotion event - a GenericEvent (core event code 35) wrapping XI2's
/// own `evtype`. Sent to clients that selected `XI_RawMotionMask` via
/// XISelectEvents (see `Window::xi_event_masks`). This server reports no
/// per-axis valuator data (`valuators_len=0`) - just the fact that the
/// pointer moved, which is enough for a client like xeyes that only wants
/// to know "did something move" rather than raw per-axis deltas.
#[derive(Debug, Clone, Copy)]
pub struct XIRawMotionEvent {
    /// The XInputExtension major opcode assigned this session - GenericEvent
    /// carries it in the `extension` field so a client with multiple
    /// GenericEvent-using extensions can tell which one an event belongs to.
    pub xinput_major_opcode: u8,
    pub deviceid: u16,
    pub sourceid: u16,
    pub time: u32,
}

impl XIRawMotionEvent {
    pub fn new(xinput_major_opcode: u8, deviceid: u16, sourceid: u16, time: u32) -> Self {
        Self {
            xinput_major_opcode,
            deviceid,
            sourceid,
            time,
        }
    }

    /// Serialize per xXIRawEvent's wire layout (XI2proto.h): a 32-byte
    /// fixed header, no trailing valuator mask/axis data since
    /// `valuators_len=0`. `sequence_number` has the same non-zero
    /// requirement as `ExposeEvent::serialize` - see its doc comment.
    pub fn serialize(&self, byte_order: ByteOrder, sequence_number: u16) -> Vec<u8> {
        let mut writer = ByteOrderWriter::new(byte_order);
        writer.write_u8(35); // GenericEvent
        writer.write_u8(self.xinput_major_opcode); // extension
        writer.write_u16(sequence_number);
        writer.write_u32(0); // length: no trailing valuator data
        writer.write_u16(xi2_event_type::RAW_MOTION); // evtype
        writer.write_u16(self.deviceid);
        writer.write_u32(self.time);
        writer.write_u32(0); // detail: unused for RawMotion
        writer.write_u16(self.sourceid);
        writer.write_u16(0); // valuators_len: no axis data reported
        writer.write_u32(0); // flags
        writer.write_u32(0); // pad2
        writer.into_vec()
    }
}

/// Event mask constants (from X11/X.h)
pub mod event_mask {
    pub const NO_EVENT_MASK: u32 = 0;
    pub const KEY_PRESS_MASK: u32 = 1 << 0;
    pub const KEY_RELEASE_MASK: u32 = 1 << 1;
    pub const BUTTON_PRESS_MASK: u32 = 1 << 2;
    pub const BUTTON_RELEASE_MASK: u32 = 1 << 3;
    pub const ENTER_WINDOW_MASK: u32 = 1 << 4;
    pub const LEAVE_WINDOW_MASK: u32 = 1 << 5;
    pub const POINTER_MOTION_MASK: u32 = 1 << 6;
    pub const POINTER_MOTION_HINT_MASK: u32 = 1 << 7;
    pub const BUTTON1_MOTION_MASK: u32 = 1 << 8;
    pub const BUTTON2_MOTION_MASK: u32 = 1 << 9;
    pub const BUTTON3_MOTION_MASK: u32 = 1 << 10;
    pub const BUTTON4_MOTION_MASK: u32 = 1 << 11;
    pub const BUTTON5_MOTION_MASK: u32 = 1 << 12;
    pub const BUTTON_MOTION_MASK: u32 = 1 << 13;
    pub const KEYMAP_STATE_MASK: u32 = 1 << 14;
    pub const EXPOSURE_MASK: u32 = 1 << 15;
    pub const VISIBILITY_CHANGE_MASK: u32 = 1 << 16;
    pub const STRUCTURE_NOTIFY_MASK: u32 = 1 << 17;
    pub const RESIZE_REDIRECT_MASK: u32 = 1 << 18;
    pub const SUBSTRUCTURE_NOTIFY_MASK: u32 = 1 << 19;
    pub const SUBSTRUCTURE_REDIRECT_MASK: u32 = 1 << 20;
    pub const FOCUS_CHANGE_MASK: u32 = 1 << 21;
    pub const PROPERTY_CHANGE_MASK: u32 = 1 << 22;
    pub const COLORMAP_CHANGE_MASK: u32 = 1 << 23;
    pub const OWNER_GRAB_BUTTON_MASK: u32 = 1 << 24;
}
