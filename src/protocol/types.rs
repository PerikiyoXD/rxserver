//! X11 Protocol Data Types and Constants
//!
//! This module defines all the core X11 protocol data types, constants, and enums
//! in a clean, type-safe manner. Unlike the original X server, we use Rust's
//! type system to prevent many classes of errors.

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

/// X11 Protocol version
pub const X_PROTOCOL: u16 = 11;
pub const X_PROTOCOL_REVISION: u16 = 0;

/// Resource IDs
pub type ResourceId = u32;
pub type Window = ResourceId;
pub type Pixmap = ResourceId;
pub type Cursor = ResourceId;
pub type Font = ResourceId;
pub type GContext = ResourceId;
pub type Colormap = ResourceId;
pub type Drawable = ResourceId;
pub type KeySym = u32;
pub type KeyCode = u8;
pub type Atom = u32;
pub type VisualId = u32;
pub type Timestamp = u32;

/// Special resource values
pub const NONE: ResourceId = 0;
pub const COPY_FROM_PARENT: ResourceId = 0;
pub const CURRENT_TIME: Timestamp = 0;
pub const ANY_PROPERTY_TYPE: Atom = 0;
pub const ANY_KEY: KeyCode = 0;
pub const ANY_BUTTON: u8 = 0;

/// Event types (clean enum instead of constants)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum EventType {
    KeyPress = 2,
    KeyRelease = 3,
    ButtonPress = 4,
    ButtonRelease = 5,
    MotionNotify = 6,
    EnterNotify = 7,
    LeaveNotify = 8,
    FocusIn = 9,
    FocusOut = 10,
    KeymapNotify = 11,
    Expose = 12,
    GraphicsExpose = 13,
    NoExpose = 14,
    VisibilityNotify = 15,
    CreateNotify = 16,
    DestroyNotify = 17,
    UnmapNotify = 18,
    MapNotify = 19,
    MapRequest = 20,
    ReparentNotify = 21,
    ConfigureNotify = 22,
    ConfigureRequest = 23,
    GravityNotify = 24,
    ResizeRequest = 25,
    CirculateNotify = 26,
    CirculateRequest = 27,
    PropertyNotify = 28,
    SelectionClear = 29,
    SelectionRequest = 30,
    SelectionNotify = 31,
    ColormapNotify = 32,
    ClientMessage = 33,
    MappingNotify = 34,
    GenericEvent = 35,
}

/// Input event masks (using bitflags for type safety)
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct EventMask: u32 {
        const NO_EVENT = 0;
        const KEY_PRESS = 1 << 0;
        const KEY_RELEASE = 1 << 1;
        const BUTTON_PRESS = 1 << 2;
        const BUTTON_RELEASE = 1 << 3;
        const ENTER_WINDOW = 1 << 4;
        const LEAVE_WINDOW = 1 << 5;
        const POINTER_MOTION = 1 << 6;
        const POINTER_MOTION_HINT = 1 << 7;
        const BUTTON1_MOTION = 1 << 8;
        const BUTTON2_MOTION = 1 << 9;
        const BUTTON3_MOTION = 1 << 10;
        const BUTTON4_MOTION = 1 << 11;
        const BUTTON5_MOTION = 1 << 12;
        const BUTTON_MOTION = 1 << 13;
        const KEYMAP_STATE = 1 << 14;
        const EXPOSURE = 1 << 15;
        const VISIBILITY_CHANGE = 1 << 16;
        const STRUCTURE_NOTIFY = 1 << 17;
        const RESIZE_REDIRECT = 1 << 18;
        const SUBSTRUCTURE_NOTIFY = 1 << 19;
        const SUBSTRUCTURE_REDIRECT = 1 << 20;
        const FOCUS_CHANGE = 1 << 21;
        const PROPERTY_CHANGE = 1 << 22;
        const COLORMAP_CHANGE = 1 << 23;
        const OWNER_GRAB_BUTTON = 1 << 24;
    }
}

/// Modifier masks (type-safe modifiers)
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ModifierMask: u16 {
        const SHIFT = 1 << 0;
        const LOCK = 1 << 1;
        const CONTROL = 1 << 2;
        const MOD1 = 1 << 3;
        const MOD2 = 1 << 4;
        const MOD3 = 1 << 5;
        const MOD4 = 1 << 6;
        const MOD5 = 1 << 7;
        const ANY_MODIFIER = 1 << 15;
    }
}

/// Button masks
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ButtonMask: u16 {
        const BUTTON1 = 1 << 8;
        const BUTTON2 = 1 << 9;
        const BUTTON3 = 1 << 10;
        const BUTTON4 = 1 << 11;
        const BUTTON5 = 1 << 12;
    }
}

/// Button numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Button {
    Button1 = 1,
    Button2 = 2,
    Button3 = 3,
    Button4 = 4,
    Button5 = 5,
}

/// Window class
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum WindowClass {
    CopyFromParent = 0,
    InputOutput = 1,
    InputOnly = 2,
}

/// Coordinate structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

/// Rectangle structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rectangle {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

/// Color specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

/// Error codes (clean enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum X11Error {
    Success = 0,
    BadRequest = 1,
    BadValue = 2,
    BadWindow = 3,
    BadPixmap = 4,
    BadAtom = 5,
    BadCursor = 6,
    BadFont = 7,
    BadMatch = 8,
    BadDrawable = 9,
    BadAccess = 10,
    BadAlloc = 11,
    BadColor = 12,
    BadGC = 13,
    BadIdChoice = 14,
    BadName = 15,
    BadLength = 16,
    BadImplementation = 17,
}

/// Window attributes for CreateWindow and ChangeWindowAttributes
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct WindowAttributesMask: u32 {
        const BACK_PIXMAP = 1 << 0;
        const BACK_PIXEL = 1 << 1;
        const BORDER_PIXMAP = 1 << 2;
        const BORDER_PIXEL = 1 << 3;
        const BIT_GRAVITY = 1 << 4;
        const WIN_GRAVITY = 1 << 5;
        const BACKING_STORE = 1 << 6;
        const BACKING_PLANES = 1 << 7;
        const BACKING_PIXEL = 1 << 8;
        const OVERRIDE_REDIRECT = 1 << 9;
        const SAVE_UNDER = 1 << 10;
        const EVENT_MASK = 1 << 11;
        const DONT_PROPAGATE = 1 << 12;
        const COLORMAP = 1 << 13;
        const CURSOR = 1 << 14;
    }
}

/// Window gravity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Gravity {
    Forget = 0,
    NorthWest = 1,
    North = 2,
    NorthEast = 3,
    West = 4,
    Center = 5,
    East = 6,
    SouthWest = 7,
    South = 8,
    SouthEast = 9,
    Static = 10,
}

/// X11 protocol opcodes organized by category
pub mod opcodes {
    /// Window management opcodes
    pub mod window {
        pub const CREATE_WINDOW: u8 = 1;
        pub const CHANGE_WINDOW_ATTRIBUTES: u8 = 2;
        pub const GET_WINDOW_ATTRIBUTES: u8 = 3;
        pub const DESTROY_WINDOW: u8 = 4;
        pub const DESTROY_SUBWINDOWS: u8 = 5;
        pub const CHANGE_SAVE_SET: u8 = 6;
        pub const REPARENT_WINDOW: u8 = 7;
        pub const MAP_WINDOW: u8 = 8;
        pub const MAP_SUBWINDOWS: u8 = 9;
        pub const UNMAP_WINDOW: u8 = 10;
        pub const UNMAP_SUBWINDOWS: u8 = 11;
        pub const CONFIGURE_WINDOW: u8 = 12;
        pub const CIRCULATE_WINDOW: u8 = 13;
        pub const GET_GEOMETRY: u8 = 14;
        pub const QUERY_TREE: u8 = 15;
    }
    
    /// Input and events
    pub mod input {
        pub const GRAB_POINTER: u8 = 26;
        pub const UNGRAB_POINTER: u8 = 27;
        pub const GRAB_BUTTON: u8 = 28;
        pub const UNGRAB_BUTTON: u8 = 29;
        pub const GRAB_KEYBOARD: u8 = 31;
        pub const UNGRAB_KEYBOARD: u8 = 32;
        pub const GRAB_KEY: u8 = 33;
        pub const UNGRAB_KEY: u8 = 34;
        pub const SEND_EVENT: u8 = 25;
    }
    
    /// Graphics operations
    pub mod graphics {
        pub const CREATE_GC: u8 = 55;
        pub const CHANGE_GC: u8 = 56;
        pub const COPY_GC: u8 = 57;
        pub const SET_DASHES: u8 = 58;
        pub const SET_CLIP_RECTANGLES: u8 = 59;
        pub const FREE_GC: u8 = 60;
        pub const CLEAR_AREA: u8 = 61;
        pub const COPY_AREA: u8 = 62;
        pub const COPY_PLANE: u8 = 63;
        pub const POLY_POINT: u8 = 64;
        pub const POLY_LINE: u8 = 65;
        pub const POLY_SEGMENT: u8 = 66;
        pub const POLY_RECTANGLE: u8 = 67;
        pub const POLY_ARC: u8 = 68;
        pub const FILL_POLY: u8 = 69;
        pub const POLY_FILL_RECTANGLE: u8 = 70;
        pub const POLY_FILL_ARC: u8 = 71;
        pub const PUT_IMAGE: u8 = 72;
        pub const GET_IMAGE: u8 = 73;
    }
}

impl Default for Point {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self { x: 0, y: 0, width: 0, height: 0 }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self { red: 0, green: 0, blue: 0 }
    }
}

impl From<u8> for X11Error {
    fn from(value: u8) -> Self {
        match value {
            0 => X11Error::Success,
            1 => X11Error::BadRequest,
            2 => X11Error::BadValue,
            3 => X11Error::BadWindow,
            4 => X11Error::BadPixmap,
            5 => X11Error::BadAtom,
            6 => X11Error::BadCursor,
            7 => X11Error::BadFont,
            8 => X11Error::BadMatch,
            9 => X11Error::BadDrawable,
            10 => X11Error::BadAccess,
            11 => X11Error::BadAlloc,
            12 => X11Error::BadColor,
            13 => X11Error::BadGC,
            14 => X11Error::BadIdChoice,
            15 => X11Error::BadName,
            16 => X11Error::BadLength,
            17 => X11Error::BadImplementation,
            _ => {
                log::warn!("Unknown X11 error code: {}", value);
                X11Error::BadImplementation // Default to BadImplementation for unknown codes
            }
        }
    }
}
