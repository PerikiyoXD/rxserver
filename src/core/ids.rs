//! Type-safe ID wrappers for X11 resources
//!
//! This module provides newtype wrappers for X11 resource IDs to prevent
//! bugs that are common in the original X server due to mixing up different
//! types of IDs.

use std::fmt;

/// Type-safe wrapper for X11 Window IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u32);

/// Type-safe wrapper for X11 Graphics Context IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GContextId(pub u32);

/// Type-safe wrapper for X11 Pixmap IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PixmapId(pub u32);

/// Type-safe wrapper for X11 Colormap IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColormapId(pub u32);

/// Type-safe wrapper for X11 Font IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontId(pub u32);

/// Type-safe wrapper for X11 Cursor IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CursorId(pub u32);

/// Type-safe wrapper for X11 Atom IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtomId(pub u32);

/// Type-safe wrapper for Client IDs (internal to our server)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(pub u32);

/// Trait for X11 resource IDs
pub trait ResourceId: Copy + fmt::Debug + PartialEq + Eq {
    fn as_u32(self) -> u32;
    fn from_u32(id: u32) -> Self;
}

macro_rules! impl_resource_id {
    ($id_type:ty) => {
        impl ResourceId for $id_type {
            fn as_u32(self) -> u32 {
                self.0
            }

            fn from_u32(id: u32) -> Self {
                Self(id)
            }
        }

        impl fmt::Display for $id_type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<u32> for $id_type {
            fn from(id: u32) -> Self {
                Self(id)
            }
        }

        impl From<$id_type> for u32 {
            fn from(id: $id_type) -> u32 {
                id.0
            }
        }
    };
}

// Implement the trait for all ID types
impl_resource_id!(WindowId);
impl_resource_id!(GContextId);
impl_resource_id!(PixmapId);
impl_resource_id!(ColormapId);
impl_resource_id!(FontId);
impl_resource_id!(CursorId);
impl_resource_id!(AtomId);
impl_resource_id!(ClientId);

/// Special window IDs defined by X11
impl WindowId {
    pub const ROOT: WindowId = WindowId(1);
    pub const NONE: WindowId = WindowId(0);
}

/// Special atom IDs defined by X11
impl AtomId {
    pub const PRIMARY: AtomId = AtomId(1);
    pub const SECONDARY: AtomId = AtomId(2);
    pub const ARC: AtomId = AtomId(3);
    pub const ATOM: AtomId = AtomId(4);
    pub const BITMAP: AtomId = AtomId(5);
    pub const CARDINAL: AtomId = AtomId(6);
    pub const COLORMAP: AtomId = AtomId(7);
    pub const CURSOR: AtomId = AtomId(8);
    pub const CUT_BUFFER0: AtomId = AtomId(9);
    pub const CUT_BUFFER1: AtomId = AtomId(10);
    pub const CUT_BUFFER2: AtomId = AtomId(11);
    pub const CUT_BUFFER3: AtomId = AtomId(12);
    pub const CUT_BUFFER4: AtomId = AtomId(13);
    pub const CUT_BUFFER5: AtomId = AtomId(14);
    pub const CUT_BUFFER6: AtomId = AtomId(15);
    pub const CUT_BUFFER7: AtomId = AtomId(16);
    pub const DRAWABLE: AtomId = AtomId(17);
    pub const FONT: AtomId = AtomId(18);
    pub const INTEGER: AtomId = AtomId(19);
    pub const PIXMAP: AtomId = AtomId(20);
    pub const POINT: AtomId = AtomId(21);
    pub const RECTANGLE: AtomId = AtomId(22);
    pub const RESOURCE_MANAGER: AtomId = AtomId(23);
    pub const RGB_COLOR_MAP: AtomId = AtomId(24);
    pub const RGB_BEST_MAP: AtomId = AtomId(25);
    pub const RGB_BLUE_MAP: AtomId = AtomId(26);
    pub const RGB_DEFAULT_MAP: AtomId = AtomId(27);
    pub const RGB_GRAY_MAP: AtomId = AtomId(28);
    pub const RGB_GREEN_MAP: AtomId = AtomId(29);
    pub const RGB_RED_MAP: AtomId = AtomId(30);
    pub const STRING: AtomId = AtomId(31);
    pub const VISUALID: AtomId = AtomId(32);
    pub const WINDOW: AtomId = AtomId(33);
    pub const WM_COMMAND: AtomId = AtomId(34);
    pub const WM_HINTS: AtomId = AtomId(35);
    pub const WM_CLIENT_MACHINE: AtomId = AtomId(36);
    pub const WM_ICON_NAME: AtomId = AtomId(37);
    pub const WM_ICON_SIZE: AtomId = AtomId(38);
    pub const WM_NAME: AtomId = AtomId(39);
    pub const WM_NORMAL_HINTS: AtomId = AtomId(40);
    pub const WM_SIZE_HINTS: AtomId = AtomId(41);
    pub const WM_ZOOM_HINTS: AtomId = AtomId(42);
    pub const MIN_SPACE: AtomId = AtomId(43);
    pub const NORM_SPACE: AtomId = AtomId(44);
    pub const MAX_SPACE: AtomId = AtomId(45);
    pub const END_SPACE: AtomId = AtomId(46);
    pub const SUPERSCRIPT_X: AtomId = AtomId(47);
    pub const SUPERSCRIPT_Y: AtomId = AtomId(48);
    pub const SUBSCRIPT_X: AtomId = AtomId(49);
    pub const SUBSCRIPT_Y: AtomId = AtomId(50);
    pub const UNDERLINE_POSITION: AtomId = AtomId(51);
    pub const UNDERLINE_THICKNESS: AtomId = AtomId(52);
    pub const STRIKEOUT_ASCENT: AtomId = AtomId(53);
    pub const STRIKEOUT_DESCENT: AtomId = AtomId(54);
    pub const ITALIC_ANGLE: AtomId = AtomId(55);
    pub const X_HEIGHT: AtomId = AtomId(56);
    pub const QUAD_WIDTH: AtomId = AtomId(57);
    pub const WEIGHT: AtomId = AtomId(58);
    pub const POINT_SIZE: AtomId = AtomId(59);
    pub const RESOLUTION: AtomId = AtomId(60);
    pub const COPYRIGHT: AtomId = AtomId(61);
    pub const NOTICE: AtomId = AtomId(62);
    pub const FONT_NAME: AtomId = AtomId(63);
    pub const FAMILY_NAME: AtomId = AtomId(64);
    pub const FULL_NAME: AtomId = AtomId(65);
    pub const CAP_HEIGHT: AtomId = AtomId(66);
    pub const WM_CLASS: AtomId = AtomId(67);
    pub const WM_TRANSIENT_FOR: AtomId = AtomId(68);
}
