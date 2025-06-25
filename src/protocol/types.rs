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
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, X11Error>;
