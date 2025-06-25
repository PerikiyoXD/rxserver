use super::types::*;
use std::convert::TryFrom;
use std::ops::Deref;

// Forward-thinking: Try to enforce protocol safety, zero-copy capability, extensibility, and data-oriented principles

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VisualClass {
    StaticGray   = 0,
    GrayScale    = 1,
    StaticColor  = 2,
    PseudoColor  = 3,
    TrueColor    = 4,
    DirectColor  = 5,
}

impl TryFrom<u8> for VisualClass {
    type Error = u8;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        use VisualClass::*;
        match v {
            0 => Ok(StaticGray),
            1 => Ok(GrayScale),
            2 => Ok(StaticColor),
            3 => Ok(PseudoColor),
            4 => Ok(TrueColor),
            5 => Ok(DirectColor),
            e => Err(e),
        }
    }
}

impl From<VisualClass> for u8 {
    #[inline(always)]
    fn from(v: VisualClass) -> u8 {
        v as u8
    }
}

impl ByteOrderConversion for VisualClass {
    #[inline(always)]
    fn from_byte_order(self, _order: ByteOrder) -> Self { self }
    #[inline(always)]
    fn to_byte_order(self, _order: ByteOrder) -> Box<[u8]> { Box::new([self as u8]) }
}

// -- Zero-copy slice-of-struct pattern for data-oriented access

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenDepthVisualRaw {
    pub id: VisualId,
    pub class: u8, // VisualClass as u8
    pub bits_per_rgb_value: u8,
    pub colormap_entries: u16,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
}

impl ScreenDepthVisualRaw {
    pub fn class(&self) -> Result<VisualClass, u8> {
        VisualClass::try_from(self.class)
    }
}

// Data-oriented: Use slices, not Vec
#[derive(Debug, PartialEq, Eq)]
pub struct ScreenDepth<'a> {
    pub depth: u8,
    pub visuals: &'a [ScreenDepthVisualRaw],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRaw {
    pub root: WindowId,
    pub default_colormap: ColormapId,
    pub white_pixel: u32,
    pub black_pixel: u32,
    pub current_input_masks: u32,
    pub width_in_pixels: u16,
    pub height_in_pixels: u16,
    pub width_in_millimeters: u16,
    pub height_in_millimeters: u16,
    pub min_installed_maps: u16,
    pub max_installed_maps: u16,
    pub root_visual: u32,
    pub backing_stores: u8, // BackingStore as u8
    pub save_unders: u8,
    pub root_depth: u8,
    pub allowed_depths_len: u8, // Number of allowed depths
    // allowed_depths follow in memory, not in struct
}

// The 'Screen' view overlays the underlying buffer, and can yield zero-copy subviews
pub struct Screen<'a> {
    pub raw: &'a ScreenRaw,
    pub allowed_depths: &'a [ScreenDepth<'a>],
}

// Protocol response: Use enums for extensibility, with version/extra extension fields
#[derive(Debug, Clone)]
pub enum ConnectionSetupResponse<'a> {
    Accepted(ConnectionSetupAccepted<'a>),
    Refused(ConnectionSetupRefused<'a>),
    AuthRequired(ConnectionSetupAuthRequired<'a>),
    Unknown {
        major_version: u16,
        minor_version: u16,
        data: &'a [u8],
    },
}

// New: Versioned, extensible, data-oriented accepted response
#[derive(Debug, Clone)]
pub struct ConnectionSetupAccepted<'a> {
    pub protocol_major_version: u16,
    pub protocol_minor_version: u16,
    pub release_number: u32,
    pub resource_id_base: XId,
    pub resource_id_mask: XId,
    pub motion_buffer_size: u32,
    pub vendor: &'a str,
    pub maximum_request_length: u16,
    pub screens: &'a [Screen<'a>],
    pub pixmap_formats: &'a [PixmapFormatRaw],
    pub image_byte_order: u8,
    pub bitmap_format_bit_order: u8,
    pub bitmap_format_scanline_unit: u8,
    pub bitmap_format_scanline_pad: u8,
    pub min_keycode: u8,
    pub max_keycode: u8,
    pub extra_fields: &'a [u8], // for future extensibility
}

#[derive(Debug, Clone)]
pub struct ConnectionSetupRefused<'a> {
    pub protocol_major_version: u16,
    pub protocol_minor_version: u16,
    pub reason: &'a str,
    pub extra_fields: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct ConnectionSetupAuthRequired<'a> {
    pub protocol_major_version: u16,
    pub protocol_minor_version: u16,
    pub reason: &'a str,
    pub extra_fields: &'a [u8],
}

// Example: Pixmap format as a POD struct
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixmapFormatRaw {
    pub depth: u8,
    pub bits_per_pixel: u8,
    pub scanline_pad: u8,
}

// For fallback/unknown responses, or to support raw data overlays
#[derive(Debug, Clone)]
pub struct RawResponse<'a> {
    pub data: &'a [u8],
}

#[derive(Debug, Clone)]
pub enum ResponseKind<'a> {
    Reply, // Generic
    ConnectionSetup(ConnectionSetupResponse<'a>),
    Raw(RawResponse<'a>),
}

#[derive(Debug, Clone)]
pub struct Response<'a> {
    pub kind: ResponseKind<'a>,
    pub sequence_number: SequenceNumber,
    pub byte_order: ByteOrder,
}

impl<'a> Default for Response<'a> {
    fn default() -> Self {
        Self {
            kind: ResponseKind::Reply,
            sequence_number: 0,
            byte_order: ByteOrder::LittleEndian,
        }
    }
}

// Note: Parsing/overlaying these zero-copy views would require careful lifetime management.
//        For a production parser, consider using 'bytemuck', 'zerocopy', or hand-rolled byte overlay code.
