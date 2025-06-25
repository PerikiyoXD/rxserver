use crate::{
    Atom, ByteOrder, ColormapId, VisualId, WindowId, XId,
    x11::protocol::endianness::ByteOrderConversion,
};

// -----------------------------------------------------------------------------
// Utilities
// -----------------------------------------------------------------------------

/// Backing store enumeration for window backing store support.
///
/// Backing store determines when the X server saves window contents
/// when the window is obscured by other windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BackingStore {
    /// Never save window contents - client must handle all exposures
    Never = 0,
    /// Save contents only when window is mapped
    WhenMapped = 1,
    /// Always save window contents when obscured
    Always = 2,
}

/// Pixmap format structure representing a supported pixmap format.
///
/// This contains the depth, bits per pixel, and scanline padding
/// for a specific pixmap format supported by the X server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionSetupAcceptedPixmapFormat {
    /// Color depth in bits (e.g., 1, 8, 16, 24, 32)
    pub depth: u8,
    /// Bits per pixel in memory (must be >= depth)
    pub bits_per_pixel: u8,
    /// Scanline padding in bits (8, 16, or 32)
    pub scanline_pad: u8,
}

/// Visual class for a visual type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VisualClass {
    StaticGray = 0,
    GrayScale = 1,
    StaticColor = 2,
    PseudoColor = 3,
    TrueColor = 4,
    DirectColor = 5,
}

impl From<VisualClass> for u8 {
    fn from(class: VisualClass) -> Self {
        class as u8
    }
}

impl ByteOrderConversion for VisualClass {
    fn from_byte_order(self, _order: ByteOrder) -> Self {
        self // u8-based enum has no byte order
    }

    fn to_byte_order(self, _order: ByteOrder) -> Box<[u8]> {
        Box::new([self as u8])
    }
}

// -----------------------------------------------------------------------------
// Connection Setup response types
// -----------------------------------------------------------------------------

/// Visual structure representing a supported visual type.
///
/// This contains the visual ID, class, bits per RGB value,
/// colormap entries, and RGB masks for a specific visual type
/// supported by the X server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionSetupAcceptedScreenDepthVisual {
    /// Visual ID
    pub id: VisualId,
    /// Visual class
    pub class: VisualClass,
    /// Number of bits per RGB value
    pub bits_per_rgb_value: u8,
    /// Number of entries in the colormap
    pub colormap_entries: u16,
    /// Red mask
    pub red_mask: u32,
    /// Green mask
    pub green_mask: u32,
    /// Blue mask
    pub blue_mask: u32,
}

/// Depth structure representing a supported visual depth.
///
/// This contains the depth in bits and a list of visual IDs
/// that are available for that depth on a particular screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionSetupAcceptedScreenDepth {
    /// Depth in bits (number of bits per pixel)
    pub depth: u8,
    /// List of visual IDs available at this depth
    pub visuals: Vec<ConnectionSetupAcceptedScreenDepthVisual>,
}

/// Screen structure representing a physical or virtual screen in the X server.
///
/// This contains comprehensive information about a screen including
/// its root window, default colormap, pixel values, physical dimensions,
/// and supported visual information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionSetupAcceptedScreen {
    /// Root window ID for this screen
    pub root: WindowId,
    /// Default colormap for this screen
    pub default_colormap: ColormapId,
    /// Pixel value that represents white
    pub white_pixel: u32,
    /// Pixel value that represents black  
    pub black_pixel: u32,
    /// Event mask for events that can be selected on the root window
    pub current_input_masks: u32,
    /// Screen width in pixels
    pub width_in_pixels: u16,
    /// Screen height in pixels
    pub height_in_pixels: u16,
    /// Physical screen width in millimeters
    pub width_in_millimeters: u16,
    /// Physical screen height in millimeters
    pub height_in_millimeters: u16,
    /// Minimum number of colormaps that can be installed simultaneously
    pub min_installed_maps: u16,
    /// Maximum number of colormaps that can be installed simultaneously
    pub max_installed_maps: u16,
    /// Visual ID of the root window's visual
    pub root_visual: u32,
    /// Backing store support level for this screen
    pub backing_stores: BackingStore,
    /// Whether the screen supports save-unders (0=False, 1=True)
    pub save_unders: u8,
    /// Depth of the root window in bits
    pub root_depth: u8,
    /// List of supported depths and their associated visuals
    pub allowed_depths: Vec<ConnectionSetupAcceptedScreenDepth>,
}

/// Successful connection setup response from the X server.
///
/// This indicates that the connection was accepted and provides
/// comprehensive details about the X server's capabilities,
/// supported formats, and available screens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionSetupAcceptedResponse {
    /// Success status (always 1 for accepted connections)
    pub success: u8,
    /// X11 protocol major version number supported by server
    pub protocol_major_version: u16,
    /// X11 protocol minor version number supported by server
    pub protocol_minor_version: u16,
    /// Length of additional data following this structure in 4-byte units
    pub additional_data_length: u16,
    /// X server release number (vendor-specific)
    pub release_number: u32,
    /// Base value for generating resource IDs for this client
    pub resource_id_base: XId,
    /// Mask for generating unique resource IDs (use with resource_id_base)
    pub resource_id_mask: XId,
    /// Size of motion event buffer in bytes (0 if not supported)
    pub motion_buffer_size: u32,
    /// Length of vendor identification string in bytes
    pub vendor_length: u16,
    /// Maximum request length supported by server in 4-byte units
    pub maximum_request_length: u16,
    /// Number of available screens on this display
    pub number_of_screens: u8,
    /// Number of supported pixmap formats
    pub number_of_formats: u8,
    /// Image byte order (0=LSBFirst, 1=MSBFirst)
    pub image_byte_order: u8,
    /// Bitmap bit order within bytes (0=LSBFirst, 1=MSBFirst)
    pub bitmap_format_bit_order: u8,
    /// Bitmap scanline unit in bits (8, 16, or 32)
    pub bitmap_format_scanline_unit: u8,
    /// Bitmap scanline padding in bits (8, 16, or 32)
    pub bitmap_format_scanline_pad: u8,
    /// Minimum keycode value supported by server
    pub min_keycode: u8,
    /// Maximum keycode value supported by server
    pub max_keycode: u8,
    /// X server vendor identification string
    pub vendor: String,
    /// List of supported pixmap formats
    pub pixmap_formats: Vec<ConnectionSetupAcceptedPixmapFormat>,
    /// List of available screens with their properties
    pub screens: Vec<ConnectionSetupAcceptedScreen>,
}

/// Connection setup refused response from the X server.
///
/// This indicates that the connection was refused, typically
/// due to authentication failure, protocol version mismatch,
/// or server policy restrictions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionSetupRefusedResponse {
    /// X11 protocol major version number supported by server
    pub protocol_major_version: u16,
    /// X11 protocol minor version number supported by server
    pub protocol_minor_version: u16,
    /// Length of reason string in 4-byte units
    pub additional_data_length: u16,
    /// Human-readable explanation for connection refusal
    pub reason: String,
}

/// Connection setup requires authentication response from the X server.
///
/// This is sent when the server requires additional authentication
/// data before allowing the client to connect. The client should
/// respond with appropriate authentication information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionSetupAuthRequiredResponse {
    /// X11 protocol major version number supported by server
    pub protocol_major_version: u16,
    /// X11 protocol minor version number supported by server
    pub protocol_minor_version: u16,
    /// Length of reason string in 4-byte units
    pub additional_data_length: u16,
    /// Authentication challenge or instruction string
    pub reason: String,
}

// -----------------------------------------------------------------------------
// CreateWindow request types
// -----------------------------------------------------------------------------

/// Window class types for CreateWindow requests.
///
/// These determine the fundamental capabilities and behavior
/// of the window being created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateWindowClass {
    /// Inherit the parent window's class
    CopyFromParent,
    /// Window can be drawn to and receive input events
    InputOutput,
    /// Window is invisible but can receive input events (no drawing)
    InputOnly,
}

/// Visual specification for CreateWindow requests.
///
/// This determines how the window's pixel data will be
/// interpreted and displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CreateWindowVisual {
    /// Inherit the parent window's visual
    CopyFromParent = 0,
    /// Any other visual ID specified explicitly
    Any(VisualId),
}

/// CreateWindow request to create a new window in the X server.
///
/// This request creates a new window as a child of the specified
/// parent window with the given properties and attributes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateWindow {
    /// Unique window ID for the new window (must be unused)
    pub window_id: WindowId,
    /// Parent window ID (new window becomes a child of this window)
    pub parent_id: WindowId,
    /// Window depth in bits (must match visual or be 0 for CopyFromParent)
    pub depth: u8,
    /// X coordinate relative to parent window's origin
    pub x: i16,
    /// Y coordinate relative to parent window's origin
    pub y: i16,
    /// Window width in pixels (must be > 0)
    pub width: u16,
    /// Window height in pixels (must be > 0)
    pub height: u16,
    /// Border width in pixels (0 for no border)
    pub border_width: u16,
    /// Window class determining capabilities
    pub class: CreateWindowClass,
    /// Visual specification for color interpretation
    pub visual: CreateWindowVisual,
    /// Bitmask indicating which attributes are specified in value_list
    pub value_mask: u32,
    /// Attribute values in X11 protocol order (matches bits set in value_mask)
    pub value_list: Vec<u32>,
}

// -----------------------------------------------------------------------------
// GetGeometry response type
// -----------------------------------------------------------------------------

/// Response to a GetGeometry request containing drawable properties.
///
/// This response provides complete geometric information about
/// a drawable object (window or pixmap) including its position,
/// size, depth, and relationship to the screen hierarchy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetGeometry {
    /// Root window of the screen containing this drawable
    pub root: WindowId,
    /// Color depth of the drawable in bits per pixel
    pub depth: u8,
    /// X coordinate relative to parent (0 for root window or pixmaps)
    pub x: i16,
    /// Y coordinate relative to parent (0 for root window or pixmaps)
    pub y: i16,
    /// Current width in pixels
    pub width: u16,
    /// Current height in pixels
    pub height: u16,
    /// Current border width in pixels (0 for pixmaps)
    pub border_width: u16,
}

// -----------------------------------------------------------------------------
// InternAtom response type
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternAtomResponse {
    /// Atom ID for the interned atom
    pub atom: Atom,
}
