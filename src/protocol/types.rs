#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

pub type XId = u32;
pub type VisualId = u32;
pub type WindowId = u32;
pub type PixmapId = u32;
pub type GContextId = u32;
pub type FontId = u32;
pub type CursorId = u32;
pub type ColormapId = u32;
pub type Atom = u32;
pub type DrawableId = u32;

pub type SequenceNumber = u16;

#[derive(Debug, thiserror::Error)]
pub enum X11Error {
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Detailed value error: {0}")]
    DetailedValue(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, X11Error>;

// X11 Protocol Constants
pub mod constants {
    // Window classes
    pub const COPY_FROM_PARENT: u16 = 0;
    pub const INPUT_OUTPUT: u16 = 1;
    pub const INPUT_ONLY: u16 = 2;

    // Visual ID constants
    pub const COPY_FROM_PARENT_VISUAL: u32 = 0;

    // Pixmap constants
    pub const NONE: u32 = 0;
    pub const PARENT_RELATIVE: u32 = 1;

    // Cursor constants
    pub const CURSOR_NONE: u32 = 0;

    // Colormap constants
    pub const COPY_FROM_PARENT_COLORMAP: u32 = 0;
}

// Window class enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum WindowClass {
    CopyFromParent = 0,
    InputOutput = 1,
    InputOnly = 2,
}

// Bit gravity enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BitGravity {
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

// Window gravity enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WinGravity {
    Unmap = 0,
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

// Backing store enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BackingStore {
    NotUseful = 0,
    WhenMapped = 1,
    Always = 2,
}

// Value mask bits for CreateWindow
pub mod value_mask {
    pub const BACKGROUND_PIXMAP: u32 = 0x00000001;
    pub const BACKGROUND_PIXEL: u32 = 0x00000002;
    pub const BORDER_PIXMAP: u32 = 0x00000004;
    pub const BORDER_PIXEL: u32 = 0x00000008;
    pub const BIT_GRAVITY: u32 = 0x00000010;
    pub const WIN_GRAVITY: u32 = 0x00000020;
    pub const BACKING_STORE: u32 = 0x00000040;
    pub const BACKING_PLANES: u32 = 0x00000080;
    pub const BACKING_PIXEL: u32 = 0x00000100;
    pub const OVERRIDE_REDIRECT: u32 = 0x00000200;
    pub const SAVE_UNDER: u32 = 0x00000400;
    pub const EVENT_MASK: u32 = 0x00000800;
    pub const DO_NOT_PROPAGATE_MASK: u32 = 0x00001000;
    pub const COLORMAP: u32 = 0x00002000;
    pub const CURSOR: u32 = 0x00004000;
}
