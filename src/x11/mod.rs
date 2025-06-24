//! X11 protocol implementation following CLEAN architecture principles
//!
//! This module contains the core X11 protocol handling, including wire format parsing,
//! request processing, event generation, and state management.

pub mod events;
pub mod extensions;
pub mod geometry;
pub mod protocol;
pub mod requests;
pub mod resources;
pub mod security;
pub mod state;
pub mod visuals;

use std::fmt::Display;

/// X11 protocol version constants
pub const X11_PROTOCOL_MAJOR_VERSION: u16 = 11;
pub const X11_PROTOCOL_MINOR_VERSION: u16 = 0;

/// X11 protocol constants
pub mod constants {
    /// Maximum request length in 4-byte units
    pub const MAX_REQUEST_LENGTH: u16 = 65535;

    /// Connection setup constants
    pub const CONNECTION_SETUP_SUCCESS: u8 = 1;
    pub const CONNECTION_SETUP_FAILED: u8 = 0;
    pub const CONNECTION_SETUP_AUTHENTICATE: u8 = 2;

    /// Window class constants
    pub const WINDOW_CLASS_COPY_FROM_PARENT: u16 = 0;
    pub const WINDOW_CLASS_INPUT_OUTPUT: u16 = 1;
    pub const WINDOW_CLASS_INPUT_ONLY: u16 = 2;

    /// Event masks
    pub const EVENT_MASK_NO_EVENT: u32 = 0;
    pub const EVENT_MASK_KEY_PRESS: u32 = 1 << 0;
    pub const EVENT_MASK_KEY_RELEASE: u32 = 1 << 1;
    pub const EVENT_MASK_BUTTON_PRESS: u32 = 1 << 2;
    pub const EVENT_MASK_BUTTON_RELEASE: u32 = 1 << 3;
    pub const EVENT_MASK_ENTER_WINDOW: u32 = 1 << 4;
    pub const EVENT_MASK_LEAVE_WINDOW: u32 = 1 << 5;
    pub const EVENT_MASK_POINTER_MOTION: u32 = 1 << 6;
    pub const EVENT_MASK_POINTER_MOTION_HINT: u32 = 1 << 7;
    pub const EVENT_MASK_BUTTON_1_MOTION: u32 = 1 << 8;
    pub const EVENT_MASK_BUTTON_2_MOTION: u32 = 1 << 9;
    pub const EVENT_MASK_BUTTON_3_MOTION: u32 = 1 << 10;
    pub const EVENT_MASK_BUTTON_4_MOTION: u32 = 1 << 11;
    pub const EVENT_MASK_BUTTON_5_MOTION: u32 = 1 << 12;
    pub const EVENT_MASK_BUTTON_MOTION: u32 = 1 << 13;
    pub const EVENT_MASK_KEYMAP_STATE: u32 = 1 << 14;
    pub const EVENT_MASK_EXPOSURE: u32 = 1 << 15;
    pub const EVENT_MASK_VISIBILITY_CHANGE: u32 = 1 << 16;
    pub const EVENT_MASK_STRUCTURE_NOTIFY: u32 = 1 << 17;
    pub const EVENT_MASK_RESIZE_REDIRECT: u32 = 1 << 18;
    pub const EVENT_MASK_SUBSTRUCTURE_NOTIFY: u32 = 1 << 19;
    pub const EVENT_MASK_SUBSTRUCTURE_REDIRECT: u32 = 1 << 20;
    pub const EVENT_MASK_FOCUS_CHANGE: u32 = 1 << 21;
    pub const EVENT_MASK_PROPERTY_CHANGE: u32 = 1 << 22;
    pub const EVENT_MASK_COLORMAP_CHANGE: u32 = 1 << 23;
    pub const EVENT_MASK_OWNER_GRAB_BUTTON: u32 = 1 << 24;

    /// Backing store constants
    pub const BACKING_STORE_NOT_USEFUL: u8 = 0;
    pub const BACKING_STORE_WHEN_MAPPED: u8 = 1;
    pub const BACKING_STORE_ALWAYS: u8 = 2;

    /// Gravity constants
    pub const GRAVITY_FORGET: u8 = 0;
    pub const GRAVITY_NORTH_WEST: u8 = 1;
    pub const GRAVITY_NORTH: u8 = 2;
    pub const GRAVITY_NORTH_EAST: u8 = 3;
    pub const GRAVITY_WEST: u8 = 4;
    pub const GRAVITY_CENTER: u8 = 5;
    pub const GRAVITY_EAST: u8 = 6;
    pub const GRAVITY_SOUTH_WEST: u8 = 7;
    pub const GRAVITY_SOUTH: u8 = 8;
    pub const GRAVITY_SOUTH_EAST: u8 = 9;
    pub const GRAVITY_STATIC: u8 = 10;
}

// Re-export ByteOrder from protocol module
pub use protocol::endianness::ByteOrder;

/// X11 Visual ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VisualId(pub u32);

impl VisualId {
    /// Create a new visual ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get raw visual ID
    pub fn raw(&self) -> u32 {
        self.0
    }
}

impl Display for VisualId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VisualId({})", self.0)
    }
}

/// X11 Colormap ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColormapId(pub u32);

impl ColormapId {
    /// Create a new colormap ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get raw colormap ID
    pub fn raw(&self) -> u32 {
        self.0
    }
}

// Re-export core types from protocol module
pub use protocol::types::{CursorId, FontId, GContextId as GraphicsContextId, PixmapId, WindowId};
