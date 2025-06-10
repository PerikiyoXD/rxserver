//! X11 data types and constants
//!
//! This module defines the fundamental data types used in the X11 protocol,
//! including resource IDs, coordinates, and protocol constants.

use serde::{Deserialize, Serialize};

/// X11 resource identifier
pub type ResourceId = u32;

/// Window identifier
pub type Window = ResourceId;

/// Pixmap identifier
pub type Pixmap = ResourceId;

/// Graphics context identifier
pub type GContext = ResourceId;

/// Font identifier
pub type Font = ResourceId;

/// Cursor identifier
pub type Cursor = ResourceId;

/// Colormap identifier
pub type Colormap = ResourceId;

/// Atom identifier
pub type Atom = u32;

/// Timestamp value
pub type Timestamp = u32;

/// Keysym value
pub type Keysym = u32;

/// Keycode value
pub type Keycode = u8;

/// X11 coordinate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

/// X11 rectangle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rectangle {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

/// X11 color specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

/// X11 window class
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowClass {
    CopyFromParent = 0,
    InputOutput = 1,
    InputOnly = 2,
}

/// X11 event mask flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventMask(pub u32);

impl EventMask {
    pub const KEY_PRESS: u32 = 1;
    pub const KEY_RELEASE: u32 = 2;
    pub const BUTTON_PRESS: u32 = 4;
    pub const BUTTON_RELEASE: u32 = 8;
    pub const ENTER_WINDOW: u32 = 16;
    pub const LEAVE_WINDOW: u32 = 32;
    pub const POINTER_MOTION: u32 = 64;
    pub const EXPOSURE: u32 = 32768;
    
    pub fn new(mask: u32) -> Self {
        Self(mask)
    }
    
    pub fn has(&self, flag: u32) -> bool {
        (self.0 & flag) != 0
    }
}

/// X11 protocol opcodes
pub mod opcodes {
    pub const CREATE_WINDOW: u8 = 1;
    pub const CHANGE_WINDOW_ATTRIBUTES: u8 = 2;
    pub const GET_WINDOW_ATTRIBUTES: u8 = 3;
    pub const DESTROY_WINDOW: u8 = 4;
    pub const DESTROY_SUBWINDOWS: u8 = 5;
    pub const MAP_WINDOW: u8 = 8;
    pub const MAP_SUBWINDOWS: u8 = 9;
    pub const UNMAP_WINDOW: u8 = 10;
    pub const CONFIGURE_WINDOW: u8 = 12;
    pub const CLEAR_AREA: u8 = 61;
    pub const COPY_AREA: u8 = 62;
}

/// X11 protocol constants
pub mod constants {
    pub const PROTOCOL_MAJOR_VERSION: u16 = 11;
    pub const PROTOCOL_MINOR_VERSION: u16 = 0;
    
    /// None resource ID
    pub const NONE: u32 = 0;
    
    /// Parent relative
    pub const PARENT_RELATIVE: u32 = 1;
    
    /// Copy from parent
    pub const COPY_FROM_PARENT: u32 = 0;
}
