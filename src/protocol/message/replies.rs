//! X11 Protocol Replies
//!
//! This module contains all X11 reply structures that are sent from server to clients
//! in response to specific requests. Each reply corresponds to a specific request type.

use crate::protocol::types::*;

/// X11 reply to a client request
#[derive(Debug, Clone)]
pub enum Reply {
    GetWindowAttributes(GetWindowAttributesReply),
    GetGeometry(GetGeometryReply),
    QueryTree(QueryTreeReply),
    InternAtom(InternAtomReply),
    GetAtomName(GetAtomNameReply),
    GetProperty(GetPropertyReply),
    ListProperties(ListPropertiesReply),
    GetSelectionOwner(GetSelectionOwnerReply),
    GrabPointer(GrabPointerReply),
    GrabKeyboard(GrabKeyboardReply),
    QueryPointer(QueryPointerReply),
    GetMotionEvents(GetMotionEventsReply),
    TranslateCoordinates(TranslateCoordinatesReply),
    GetInputFocus(GetInputFocusReply),
    QueryKeymap(QueryKeymapReply),
    QueryFont(QueryFontReply),
    QueryTextExtents(QueryTextExtentsReply),
    ListFonts(ListFontsReply),
    ListFontsWithInfo(ListFontsWithInfoReply),
    GetFontPath(GetFontPathReply),
    CreatePixmap(CreatePixmapReply),
    GetImage(GetImageReply),
    ListInstalledColormaps(ListInstalledColormapsReply),
    AllocColor(AllocColorReply),
    AllocNamedColor(AllocNamedColorReply),
    AllocColorCells(AllocColorCellsReply),
    AllocColorPlanes(AllocColorPlanesReply),
    QueryColors(QueryColorsReply),
    LookupColor(LookupColorReply),
    QueryBestSize(QueryBestSizeReply),
    QueryExtension(QueryExtensionReply),
    ListExtensions(ListExtensionsReply),
    SetModifierMapping(SetModifierMappingReply),
    GetModifierMapping(GetModifierMappingReply),
    SetPointerMapping(SetPointerMappingReply),
    GetPointerMapping(GetPointerMappingReply),
    SetKeyboardMapping(SetKeyboardMappingReply),
    GetKeyboardMapping(GetKeyboardMappingReply),
    GetKeyboardControl(GetKeyboardControlReply),
    GetPointerControl(GetPointerControlReply),
    GetScreenSaver(GetScreenSaverReply),
    ListHosts(ListHostsReply),
}

/// GetWindowAttributes reply
#[derive(Debug, Clone)]
pub struct GetWindowAttributesReply {
    pub backing_store: u8,
    pub visual: VisualId,
    pub class: WindowClass,
    pub bit_gravity: Gravity,
    pub win_gravity: Gravity,
    pub backing_planes: u32,
    pub backing_pixel: u32,
    pub save_under: bool,
    pub map_is_installed: bool,
    pub map_state: u8,
    pub override_redirect: bool,
    pub colormap: Colormap,
    pub all_event_masks: EventMask,
    pub your_event_mask: EventMask,
    pub do_not_propagate_mask: EventMask,
}

/// GetGeometry reply
#[derive(Debug, Clone)]
pub struct GetGeometryReply {
    pub depth: u8,
    pub root: Window,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
}

/// QueryTree reply
#[derive(Debug, Clone)]
pub struct QueryTreeReply {
    pub root: Window,
    pub parent: Window,
    pub children: Vec<Window>,
}

/// InternAtom reply
#[derive(Debug, Clone)]
pub struct InternAtomReply {
    pub atom: Atom,
}

/// GetAtomName reply
#[derive(Debug, Clone)]
pub struct GetAtomNameReply {
    pub name: String,
}

/// GetProperty reply
#[derive(Debug, Clone)]
pub struct GetPropertyReply {
    pub format: u8,
    pub property_type: Atom,
    pub bytes_after: u32,
    pub data: Vec<u8>,
}

/// ListProperties reply
#[derive(Debug, Clone)]
pub struct ListPropertiesReply {
    pub atoms: Vec<Atom>,
}

/// GetSelectionOwner reply
#[derive(Debug, Clone)]
pub struct GetSelectionOwnerReply {
    pub owner: Window,
}

/// GrabPointer reply
#[derive(Debug, Clone)]
pub struct GrabPointerReply {
    pub status: u8,
}

/// GrabKeyboard reply
#[derive(Debug, Clone)]
pub struct GrabKeyboardReply {
    pub status: u8,
}

/// QueryPointer reply
#[derive(Debug, Clone)]
pub struct QueryPointerReply {
    pub same_screen: bool,
    pub root: Window,
    pub child: Window,
    pub root_x: i16,
    pub root_y: i16,
    pub win_x: i16,
    pub win_y: i16,
    pub mask: u16,
}

/// GetMotionEvents reply
#[derive(Debug, Clone)]
pub struct GetMotionEventsReply {
    pub events: Vec<TimeCoord>,
}

/// Time coordinate for motion events
#[derive(Debug, Clone)]
pub struct TimeCoord {
    pub time: Timestamp,
    pub x: i16,
    pub y: i16,
}

/// TranslateCoordinates reply
#[derive(Debug, Clone)]
pub struct TranslateCoordinatesReply {
    pub same_screen: bool,
    pub child: Window,
    pub dst_x: i16,
    pub dst_y: i16,
}

/// GetInputFocus reply
#[derive(Debug, Clone)]
pub struct GetInputFocusReply {
    pub revert_to: u8,
    pub focus: Window,
}

/// QueryKeymap reply
#[derive(Debug, Clone)]
pub struct QueryKeymapReply {
    pub keys: [u8; 32],
}

/// QueryFont reply
#[derive(Debug, Clone)]
pub struct QueryFontReply {
    pub font_info: FontInfo,
    pub char_infos: Vec<CharInfo>,
}

/// Font information
#[derive(Debug, Clone)]
pub struct FontInfo {
    pub min_bounds: CharInfo,
    pub max_bounds: CharInfo,
    pub min_char_or_byte2: u16,
    pub max_char_or_byte2: u16,
    pub min_byte1: u16,
    pub max_byte1: u16,
    pub all_chars_exist: bool,
    pub font_ascent: i16,
    pub font_descent: i16,
    pub properties: Vec<FontProp>,
}

/// Character information
#[derive(Debug, Clone)]
pub struct CharInfo {
    pub left_side_bearing: i16,
    pub right_side_bearing: i16,
    pub character_width: i16,
    pub ascent: i16,
    pub descent: i16,
    pub attributes: u16,
}

/// Font property
#[derive(Debug, Clone)]
pub struct FontProp {
    pub name: Atom,
    pub value: u32,
}

/// QueryTextExtents reply
#[derive(Debug, Clone)]
pub struct QueryTextExtentsReply {
    pub draw_direction: u8,
    pub font_ascent: i16,
    pub font_descent: i16,
    pub overall_ascent: i16,
    pub overall_descent: i16,
    pub overall_width: i32,
    pub overall_left: i32,
    pub overall_right: i32,
}

/// ListFonts reply
#[derive(Debug, Clone)]
pub struct ListFontsReply {
    pub names: Vec<String>,
}

/// ListFontsWithInfo reply
#[derive(Debug, Clone)]
pub struct ListFontsWithInfoReply {
    pub name: String,
    pub info: FontInfo,
}

/// GetFontPath reply
#[derive(Debug, Clone)]
pub struct GetFontPathReply {
    pub path: Vec<String>,
}

/// CreatePixmap reply
#[derive(Debug, Clone)]
pub struct CreatePixmapReply {
    // Empty - CreatePixmap has no reply body
}

/// GetImage reply
#[derive(Debug, Clone)]
pub struct GetImageReply {
    pub depth: u8,
    pub visual: VisualId,
    pub data: Vec<u8>,
}

/// ListInstalledColormaps reply
#[derive(Debug, Clone)]
pub struct ListInstalledColormapsReply {
    pub colormaps: Vec<Colormap>,
}

/// AllocColor reply
#[derive(Debug, Clone)]
pub struct AllocColorReply {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
    pub pixel: u32,
}

/// AllocNamedColor reply
#[derive(Debug, Clone)]
pub struct AllocNamedColorReply {
    pub pixel: u32,
    pub exact_red: u16,
    pub exact_green: u16,
    pub exact_blue: u16,
    pub visual_red: u16,
    pub visual_green: u16,
    pub visual_blue: u16,
}

/// AllocColorCells reply
#[derive(Debug, Clone)]
pub struct AllocColorCellsReply {
    pub pixels: Vec<u32>,
    pub masks: Vec<u32>,
}

/// AllocColorPlanes reply
#[derive(Debug, Clone)]
pub struct AllocColorPlanesReply {
    pub pixels: Vec<u32>,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
}

/// QueryColors reply
#[derive(Debug, Clone)]
pub struct QueryColorsReply {
    pub colors: Vec<RgbValue>,
}

/// RGB color value
#[derive(Debug, Clone)]
pub struct RgbValue {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
    pub flags: u8,
}

/// LookupColor reply
#[derive(Debug, Clone)]
pub struct LookupColorReply {
    pub exact_red: u16,
    pub exact_green: u16,
    pub exact_blue: u16,
    pub visual_red: u16,
    pub visual_green: u16,
    pub visual_blue: u16,
}

/// QueryBestSize reply
#[derive(Debug, Clone)]
pub struct QueryBestSizeReply {
    pub width: u16,
    pub height: u16,
}

/// QueryExtension reply
#[derive(Debug, Clone)]
pub struct QueryExtensionReply {
    pub present: bool,
    pub major_opcode: u8,
    pub first_event: u8,
    pub first_error: u8,
}

/// ListExtensions reply
#[derive(Debug, Clone)]
pub struct ListExtensionsReply {
    pub names: Vec<String>,
}

/// SetModifierMapping reply
#[derive(Debug, Clone)]
pub struct SetModifierMappingReply {
    pub status: u8,
}

/// GetModifierMapping reply
#[derive(Debug, Clone)]
pub struct GetModifierMappingReply {
    pub keycodes_per_modifier: u8,
    pub keycodes: Vec<KeyCode>,
}

/// SetPointerMapping reply
#[derive(Debug, Clone)]
pub struct SetPointerMappingReply {
    pub status: u8,
}

/// GetPointerMapping reply
#[derive(Debug, Clone)]
pub struct GetPointerMappingReply {
    pub map: Vec<u8>,
}

/// SetKeyboardMapping reply
#[derive(Debug, Clone)]
pub struct SetKeyboardMappingReply {
    // Empty - SetKeyboardMapping has no reply body
}

/// GetKeyboardMapping reply
#[derive(Debug, Clone)]
pub struct GetKeyboardMappingReply {
    pub keysyms_per_keycode: u8,
    pub keysyms: Vec<KeySym>,
}

/// GetKeyboardControl reply
#[derive(Debug, Clone)]
pub struct GetKeyboardControlReply {
    pub global_auto_repeat: bool,
    pub led_mask: u32,
    pub key_click_percent: u8,
    pub bell_percent: u8,
    pub bell_pitch: u16,
    pub bell_duration: u16,
    pub auto_repeats: [u8; 32],
}

/// GetPointerControl reply
#[derive(Debug, Clone)]
pub struct GetPointerControlReply {
    pub acceleration_numerator: u16,
    pub acceleration_denominator: u16,
    pub threshold: u16,
}

/// GetScreenSaver reply
#[derive(Debug, Clone)]
pub struct GetScreenSaverReply {
    pub timeout: u16,
    pub interval: u16,
    pub prefer_blanking: u8,
    pub allow_exposures: u8,
}

/// ListHosts reply
#[derive(Debug, Clone)]
pub struct ListHostsReply {
    pub enabled: bool,
    pub hosts: Vec<Host>,
}

/// Host entry
#[derive(Debug, Clone)]
pub struct Host {
    pub family: u8,
    pub address: Vec<u8>,
}
