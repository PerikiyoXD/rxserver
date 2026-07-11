// events.rs
//! X11 event structures and handling

use crate::protocol::{ByteOrder, endianness::ByteOrderWriter, types::*};

/// X11 event types
#[derive(Debug, Clone)]
pub enum Event {
    Expose(ExposeEvent),
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

    /// Serialize the expose event to bytes
    pub fn serialize(&self, byte_order: ByteOrder) -> Vec<u8> {
        let mut writer = ByteOrderWriter::new(byte_order);
        writer.write_u8(12); // Event code for Expose
        writer.write_u8(0); // Unused
        writer.write_u16(0); // Sequence number (filled by client)
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
