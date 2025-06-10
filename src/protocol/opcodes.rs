/*!
 * X11 Protocol Opcodes
 * 
 * Defines all X11 protocol opcodes organized by functional category.
 * These constants define the wire protocol command numbers.
 */

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

/// Graphics and drawing opcodes
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

/// Input and event opcodes
pub mod input {
    pub const GRAB_POINTER: u8 = 26;
    pub const UNGRAB_POINTER: u8 = 27;
    pub const GRAB_BUTTON: u8 = 28;
    pub const UNGRAB_BUTTON: u8 = 29;
    pub const CHANGE_ACTIVE_POINTER_GRAB: u8 = 30;
    pub const GRAB_KEYBOARD: u8 = 31;
    pub const UNGRAB_KEYBOARD: u8 = 32;
    pub const GRAB_KEY: u8 = 33;
    pub const UNGRAB_KEY: u8 = 34;
    pub const ALLOW_EVENTS: u8 = 35;
    pub const GRAB_SERVER: u8 = 36;
    pub const UNGRAB_SERVER: u8 = 37;
    pub const QUERY_POINTER: u8 = 38;
    pub const GET_MOTION_EVENTS: u8 = 39;
    pub const TRANSLATE_COORDINATES: u8 = 40;
    pub const WARP_POINTER: u8 = 41;
    pub const SET_INPUT_FOCUS: u8 = 42;
    pub const GET_INPUT_FOCUS: u8 = 43;
    pub const QUERY_KEYMAP: u8 = 44;
}

/// Text and font opcodes
pub mod text {
    pub const OPEN_FONT: u8 = 45;
    pub const CLOSE_FONT: u8 = 46;
    pub const QUERY_FONT: u8 = 47;
    pub const QUERY_TEXT_EXTENTS: u8 = 48;
    pub const LIST_FONTS: u8 = 49;
    pub const LIST_FONTS_WITH_INFO: u8 = 50;
    pub const SET_FONT_PATH: u8 = 51;
    pub const GET_FONT_PATH: u8 = 52;
    pub const POLY_TEXT8: u8 = 74;
    pub const POLY_TEXT16: u8 = 75;
    pub const IMAGE_TEXT8: u8 = 76;
    pub const IMAGE_TEXT16: u8 = 77;
}

/// Pixmap and drawable opcodes
pub mod pixmap {
    pub const CREATE_PIXMAP: u8 = 53;
    pub const FREE_PIXMAP: u8 = 54;
}

/// Cursor opcodes
pub mod cursor {
    pub const CREATE_CURSOR: u8 = 93;
    pub const CREATE_GLYPH_CURSOR: u8 = 94;
    pub const FREE_CURSOR: u8 = 95;
    pub const RECOLOR_CURSOR: u8 = 96;
}

/// Colormap opcodes
pub mod colormap {
    pub const CREATE_COLORMAP: u8 = 78;
    pub const FREE_COLORMAP: u8 = 79;
    pub const COPY_COLORMAP_AND_FREE: u8 = 80;
    pub const INSTALL_COLORMAP: u8 = 81;
    pub const UNINSTALL_COLORMAP: u8 = 82;
    pub const LIST_INSTALLED_COLORMAPS: u8 = 83;
    pub const ALLOC_COLOR: u8 = 84;
    pub const ALLOC_NAMED_COLOR: u8 = 85;
    pub const ALLOC_COLOR_CELLS: u8 = 86;
    pub const ALLOC_COLOR_PLANES: u8 = 87;
    pub const FREE_COLORS: u8 = 88;
    pub const STORE_COLORS: u8 = 89;
    pub const STORE_NAMED_COLOR: u8 = 90;
    pub const QUERY_COLORS: u8 = 91;
    pub const LOOKUP_COLOR: u8 = 92;
}

/// Atom and property opcodes
pub mod atom {
    pub const INTERN_ATOM: u8 = 16;
    pub const GET_ATOM_NAME: u8 = 17;
    pub const CHANGE_PROPERTY: u8 = 18;
    pub const DELETE_PROPERTY: u8 = 19;
    pub const GET_PROPERTY: u8 = 20;
    pub const LIST_PROPERTIES: u8 = 21;
    pub const SET_SELECTION_OWNER: u8 = 22;
    pub const GET_SELECTION_OWNER: u8 = 23;
    pub const CONVERT_SELECTION: u8 = 24;
}

/// Server and extension opcodes
pub mod server {
    pub const SEND_EVENT: u8 = 25;
    pub const QUERY_EXTENSION: u8 = 98;
    pub const LIST_EXTENSIONS: u8 = 99;
    pub const CHANGE_KEYBOARD_MAPPING: u8 = 100;
    pub const GET_KEYBOARD_MAPPING: u8 = 101;
    pub const CHANGE_KEYBOARD_CONTROL: u8 = 102;
    pub const GET_KEYBOARD_CONTROL: u8 = 103;
    pub const BELL: u8 = 104;
    pub const CHANGE_POINTER_CONTROL: u8 = 105;
    pub const GET_POINTER_CONTROL: u8 = 106;
    pub const SET_SCREEN_SAVER: u8 = 107;
    pub const GET_SCREEN_SAVER: u8 = 108;
    pub const CHANGE_HOSTS: u8 = 109;
    pub const LIST_HOSTS: u8 = 110;
    pub const SET_ACCESS_CONTROL: u8 = 111;
    pub const SET_CLOSE_DOWN_MODE: u8 = 112;
    pub const KILL_CLIENT: u8 = 113;
    pub const ROTATE_PROPERTIES: u8 = 114;
    pub const FORCE_SCREEN_SAVER: u8 = 115;
    pub const SET_POINTER_MAPPING: u8 = 116;
    pub const GET_POINTER_MAPPING: u8 = 117;
    pub const SET_MODIFIER_MAPPING: u8 = 118;
    pub const GET_MODIFIER_MAPPING: u8 = 119;
    pub const NO_OPERATION: u8 = 127;
}

/// Event opcodes (for events sent to clients)
pub mod event {
    pub const KEY_PRESS: u8 = 2;
    pub const KEY_RELEASE: u8 = 3;
    pub const BUTTON_PRESS: u8 = 4;
    pub const BUTTON_RELEASE: u8 = 5;
    pub const MOTION_NOTIFY: u8 = 6;
    pub const ENTER_NOTIFY: u8 = 7;
    pub const LEAVE_NOTIFY: u8 = 8;
    pub const FOCUS_IN: u8 = 9;
    pub const FOCUS_OUT: u8 = 10;
    pub const KEYMAP_NOTIFY: u8 = 11;
    pub const EXPOSE: u8 = 12;
    pub const GRAPHICS_EXPOSURE: u8 = 13;
    pub const NO_EXPOSURE: u8 = 14;
    pub const VISIBILITY_NOTIFY: u8 = 15;
    pub const CREATE_NOTIFY: u8 = 16;
    pub const DESTROY_NOTIFY: u8 = 17;
    pub const UNMAP_NOTIFY: u8 = 18;
    pub const MAP_NOTIFY: u8 = 19;
    pub const MAP_REQUEST: u8 = 20;
    pub const REPARENT_NOTIFY: u8 = 21;
    pub const CONFIGURE_NOTIFY: u8 = 22;
    pub const CONFIGURE_REQUEST: u8 = 23;
    pub const GRAVITY_NOTIFY: u8 = 24;
    pub const RESIZE_REQUEST: u8 = 25;
    pub const CIRCULATE_NOTIFY: u8 = 26;
    pub const CIRCULATE_REQUEST: u8 = 27;
    pub const PROPERTY_NOTIFY: u8 = 28;
    pub const SELECTION_CLEAR: u8 = 29;
    pub const SELECTION_REQUEST: u8 = 30;
    pub const SELECTION_NOTIFY: u8 = 31;
    pub const COLORMAP_NOTIFY: u8 = 32;
    pub const CLIENT_MESSAGE: u8 = 33;
    pub const MAPPING_NOTIFY: u8 = 34;
}

/// Error codes
pub mod error {
    pub const REQUEST: u8 = 1;
    pub const VALUE: u8 = 2;
    pub const WINDOW: u8 = 3;
    pub const PIXMAP: u8 = 4;
    pub const ATOM: u8 = 5;
    pub const CURSOR: u8 = 6;
    pub const FONT: u8 = 7;
    pub const MATCH: u8 = 8;
    pub const DRAWABLE: u8 = 9;
    pub const ACCESS: u8 = 10;
    pub const ALLOC: u8 = 11;
    pub const COLORMAP: u8 = 12;
    pub const GCONTEXT: u8 = 13;
    pub const IDCHOICE: u8 = 14;
    pub const NAME: u8 = 15;
    pub const LENGTH: u8 = 16;
    pub const IMPLEMENTATION: u8 = 17;
}
