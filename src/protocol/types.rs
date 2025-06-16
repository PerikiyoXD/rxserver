//! X11 Protocol Types and Opcodes
//!
//! This module defines the core types and opcodes used in the X11 protocol.

use std::fmt;

/// Client identifier
pub type ClientId = u32;

/// X11 Atom identifier
pub type Atom = u32;

/// Special atom value representing NONE
pub const NONE: u32 = 0;

/// BIG-REQUESTS extension opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BigRequestsOpcode {
    Enable = 0,
}

/// XC-MISC extension opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum XcMiscOpcode {
    GetVersion = 0,
    GetXIDRange = 1,
    GetXIDList = 2,
}

/// Extension-specific opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtensionOpcode {
    BigRequests(BigRequestsOpcode),
    XcMisc(XcMiscOpcode),
    Unknown(u8, u8), // (major_opcode, minor_opcode)
}

/// X11 Protocol opcodes - handles both core and extension opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opcode {
    // Core X11 Protocol Requests (1-127)
    CreateWindow,
    ChangeWindowAttributes,
    GetWindowAttributes,
    DestroyWindow,
    DestroySubwindows,
    ChangeSaveSet,
    ReparentWindow,
    MapWindow,
    MapSubwindows,
    UnmapWindow,
    UnmapSubwindows,
    ConfigureWindow,
    CirculateWindow,
    GetGeometry,
    QueryTree,
    InternAtom,
    GetAtomName,
    ChangeProperty,
    DeleteProperty,
    GetProperty,
    ListProperties,
    SetSelectionOwner,
    GetSelectionOwner,
    ConvertSelection,
    SendEvent,
    GrabPointer,
    UngrabPointer,
    GrabButton,
    UngrabButton,
    ChangeActivePointerGrab,
    GrabKeyboard,
    UngrabKeyboard,
    GrabKey,
    UngrabKey,
    AllowEvents,
    GrabServer,
    UngrabServer,
    QueryPointer,
    GetMotionEvents,
    TranslateCoordinates,
    WarpPointer,
    SetInputFocus,
    GetInputFocus,
    QueryKeymap,
    OpenFont,
    CloseFont,
    QueryFont,
    QueryTextExtents,
    ListFonts,
    ListFontsWithInfo,
    SetFontPath,
    GetFontPath,
    CreatePixmap,
    FreePixmap,
    CreateGC,
    ChangeGC,
    CopyGC,
    SetDashes,
    SetClipRectangles,
    FreeGC,
    ClearArea,
    CopyArea,
    CopyPlane,
    PolyPoint,
    PolyLine,
    PolySegment,
    PolyRectangle,
    PolyArc,
    FillPoly,
    PolyFillRectangle,
    PolyFillArc,
    PutImage,
    GetImage,
    PolyText8,
    PolyText16,
    ImageText8,
    ImageText16,
    CreateColormap,
    FreeColormap,
    CopyColormapAndFree,
    InstallColormap,
    UninstallColormap,
    ListInstalledColormaps,
    AllocColor,
    AllocNamedColor,
    AllocColorCells,
    AllocColorPlanes,
    FreeColors,
    StoreColors,
    StoreNamedColor,
    QueryColors,
    LookupColor,
    CreateCursor,
    CreateGlyphCursor,
    FreeCursor,
    RecolorCursor,
    QueryBestSize,
    QueryExtension,
    ListExtensions,
    ChangeKeyboardMapping,
    GetKeyboardMapping,
    ChangeKeyboardControl,
    GetKeyboardControl,
    Bell,
    ChangePointerControl,
    GetPointerControl,
    SetScreenSaver,
    GetScreenSaver,
    ChangeHosts,
    ListHosts,
    SetAccessControl,
    SetCloseDownMode,
    KillClient,
    RotateProperties,
    ForceScreenSaver,
    SetPointerMapping,
    GetPointerMapping,
    SetModifierMapping,
    GetModifierMapping,
    NoOperation,

    // Extension opcodes
    Extension(ExtensionOpcode),
}

impl BigRequestsOpcode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Enable),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Enable => "BigReqEnable",
        }
    }
}

impl XcMiscOpcode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::GetVersion),
            1 => Some(Self::GetXIDRange),
            2 => Some(Self::GetXIDList),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::GetVersion => "XCMiscGetVersion",
            Self::GetXIDRange => "XCMiscGetXIDRange",
            Self::GetXIDList => "XCMiscGetXIDList",
        }
    }
}

impl ExtensionOpcode {
    /// Create extension opcode from major and minor opcode bytes
    pub fn from_opcodes(major: u8, minor: u8) -> Self {
        match major {
            132 => {
                if let Some(bigreq_op) = BigRequestsOpcode::from_u8(minor) {
                    Self::BigRequests(bigreq_op)
                } else {
                    Self::Unknown(major, minor)
                }
            }
            133 => {
                if let Some(xcmisc_op) = XcMiscOpcode::from_u8(minor) {
                    Self::XcMisc(xcmisc_op)
                } else {
                    Self::Unknown(major, minor)
                }
            }
            _ => Self::Unknown(major, minor),
        }
    }

    pub fn major_opcode(self) -> u8 {
        match self {
            Self::BigRequests(_) => 132,
            Self::XcMisc(_) => 133,
            Self::Unknown(major, _) => major,
        }
    }

    pub fn minor_opcode(self) -> u8 {
        match self {
            Self::BigRequests(op) => op as u8,
            Self::XcMisc(op) => op as u8,
            Self::Unknown(_, minor) => minor,
        }
    }

    pub fn name(self) -> String {
        match self {
            Self::BigRequests(op) => op.name().to_string(),
            Self::XcMisc(op) => op.name().to_string(),
            Self::Unknown(major, minor) => format!("UnknownExtension({}, {})", major, minor),
        }
    }
}

impl Opcode {
    /// Convert from u8 to core Opcode (extensions need separate handling)
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::CreateWindow,
            2 => Self::ChangeWindowAttributes,
            3 => Self::GetWindowAttributes,
            4 => Self::DestroyWindow,
            5 => Self::DestroySubwindows,
            6 => Self::ChangeSaveSet,
            7 => Self::ReparentWindow,
            8 => Self::MapWindow,
            9 => Self::MapSubwindows,
            10 => Self::UnmapWindow,
            11 => Self::UnmapSubwindows,
            12 => Self::ConfigureWindow,
            13 => Self::CirculateWindow,
            14 => Self::GetGeometry,
            15 => Self::QueryTree,
            16 => Self::InternAtom,
            17 => Self::GetAtomName,
            18 => Self::ChangeProperty,
            19 => Self::DeleteProperty,
            20 => Self::GetProperty,
            21 => Self::ListProperties,
            22 => Self::SetSelectionOwner,
            23 => Self::GetSelectionOwner,
            24 => Self::ConvertSelection,
            25 => Self::SendEvent,
            26 => Self::GrabPointer,
            27 => Self::UngrabPointer,
            28 => Self::GrabButton,
            29 => Self::UngrabButton,
            30 => Self::ChangeActivePointerGrab,
            31 => Self::GrabKeyboard,
            32 => Self::UngrabKeyboard,
            33 => Self::GrabKey,
            34 => Self::UngrabKey,
            35 => Self::AllowEvents,
            36 => Self::GrabServer,
            37 => Self::UngrabServer,
            38 => Self::QueryPointer,
            39 => Self::GetMotionEvents,
            40 => Self::TranslateCoordinates,
            41 => Self::WarpPointer,
            42 => Self::SetInputFocus,
            43 => Self::GetInputFocus,
            44 => Self::QueryKeymap,
            45 => Self::OpenFont,
            46 => Self::CloseFont,
            47 => Self::QueryFont,
            48 => Self::QueryTextExtents,
            49 => Self::ListFonts,
            50 => Self::ListFontsWithInfo,
            51 => Self::SetFontPath,
            52 => Self::GetFontPath,
            53 => Self::CreatePixmap,
            54 => Self::FreePixmap,
            55 => Self::CreateGC,
            56 => Self::ChangeGC,
            57 => Self::CopyGC,
            58 => Self::SetDashes,
            59 => Self::SetClipRectangles,
            60 => Self::FreeGC,
            61 => Self::ClearArea,
            62 => Self::CopyArea,
            63 => Self::CopyPlane,
            64 => Self::PolyPoint,
            65 => Self::PolyLine,
            66 => Self::PolySegment,
            67 => Self::PolyRectangle,
            68 => Self::PolyArc,
            69 => Self::FillPoly,
            70 => Self::PolyFillRectangle,
            71 => Self::PolyFillArc,
            72 => Self::PutImage,
            73 => Self::GetImage,
            74 => Self::PolyText8,
            75 => Self::PolyText16,
            76 => Self::ImageText8,
            77 => Self::ImageText16,
            78 => Self::CreateColormap,
            79 => Self::FreeColormap,
            80 => Self::CopyColormapAndFree,
            81 => Self::InstallColormap,
            82 => Self::UninstallColormap,
            83 => Self::ListInstalledColormaps,
            84 => Self::AllocColor,
            85 => Self::AllocNamedColor,
            86 => Self::AllocColorCells,
            87 => Self::AllocColorPlanes,
            88 => Self::FreeColors,
            89 => Self::StoreColors,
            90 => Self::StoreNamedColor,
            91 => Self::QueryColors,
            92 => Self::LookupColor,
            93 => Self::CreateCursor,
            94 => Self::CreateGlyphCursor,
            95 => Self::FreeCursor,
            96 => Self::RecolorCursor,
            97 => Self::QueryBestSize,
            98 => Self::QueryExtension,
            99 => Self::ListExtensions,
            100 => Self::ChangeKeyboardMapping,
            101 => Self::GetKeyboardMapping,
            102 => Self::ChangeKeyboardControl,
            103 => Self::GetKeyboardControl,
            104 => Self::Bell,
            105 => Self::ChangePointerControl,
            106 => Self::GetPointerControl,
            107 => Self::SetScreenSaver,
            108 => Self::GetScreenSaver,
            109 => Self::ChangeHosts,
            110 => Self::ListHosts,
            111 => Self::SetAccessControl,
            112 => Self::SetCloseDownMode,
            113 => Self::KillClient,
            114 => Self::RotateProperties,
            115 => Self::ForceScreenSaver,
            116 => Self::SetPointerMapping,
            117 => Self::GetPointerMapping,
            118 => Self::SetModifierMapping,
            119 => Self::GetModifierMapping,
            127 => Self::NoOperation,
            // Extension opcodes need major+minor, so we can't create them from just major
            ext => Self::Extension(ExtensionOpcode::Unknown(ext, 0)),
        }
    }

    /// Create extension opcode from major and minor opcode
    pub fn from_extension(major: u8, minor: u8) -> Self {
        Self::Extension(ExtensionOpcode::from_opcodes(major, minor))
    }

    /// Convert to u8 (returns major opcode for extensions)
    pub fn to_u8(self) -> u8 {
        match self {
            Self::CreateWindow => 1,
            Self::ChangeWindowAttributes => 2,
            Self::GetWindowAttributes => 3,
            Self::DestroyWindow => 4,
            Self::DestroySubwindows => 5,
            Self::ChangeSaveSet => 6,
            Self::ReparentWindow => 7,
            Self::MapWindow => 8,
            Self::MapSubwindows => 9,
            Self::UnmapWindow => 10,
            Self::UnmapSubwindows => 11,
            Self::ConfigureWindow => 12,
            Self::CirculateWindow => 13,
            Self::GetGeometry => 14,
            Self::QueryTree => 15,
            Self::InternAtom => 16,
            Self::GetAtomName => 17,
            Self::ChangeProperty => 18,
            Self::DeleteProperty => 19,
            Self::GetProperty => 20,
            Self::ListProperties => 21,
            Self::SetSelectionOwner => 22,
            Self::GetSelectionOwner => 23,
            Self::ConvertSelection => 24,
            Self::SendEvent => 25,
            Self::GrabPointer => 26,
            Self::UngrabPointer => 27,
            Self::GrabButton => 28,
            Self::UngrabButton => 29,
            Self::ChangeActivePointerGrab => 30,
            Self::GrabKeyboard => 31,
            Self::UngrabKeyboard => 32,
            Self::GrabKey => 33,
            Self::UngrabKey => 34,
            Self::AllowEvents => 35,
            Self::GrabServer => 36,
            Self::UngrabServer => 37,
            Self::QueryPointer => 38,
            Self::GetMotionEvents => 39,
            Self::TranslateCoordinates => 40,
            Self::WarpPointer => 41,
            Self::SetInputFocus => 42,
            Self::GetInputFocus => 43,
            Self::QueryKeymap => 44,
            Self::OpenFont => 45,
            Self::CloseFont => 46,
            Self::QueryFont => 47,
            Self::QueryTextExtents => 48,
            Self::ListFonts => 49,
            Self::ListFontsWithInfo => 50,
            Self::SetFontPath => 51,
            Self::GetFontPath => 52,
            Self::CreatePixmap => 53,
            Self::FreePixmap => 54,
            Self::CreateGC => 55,
            Self::ChangeGC => 56,
            Self::CopyGC => 57,
            Self::SetDashes => 58,
            Self::SetClipRectangles => 59,
            Self::FreeGC => 60,
            Self::ClearArea => 61,
            Self::CopyArea => 62,
            Self::CopyPlane => 63,
            Self::PolyPoint => 64,
            Self::PolyLine => 65,
            Self::PolySegment => 66,
            Self::PolyRectangle => 67,
            Self::PolyArc => 68,
            Self::FillPoly => 69,
            Self::PolyFillRectangle => 70,
            Self::PolyFillArc => 71,
            Self::PutImage => 72,
            Self::GetImage => 73,
            Self::PolyText8 => 74,
            Self::PolyText16 => 75,
            Self::ImageText8 => 76,
            Self::ImageText16 => 77,
            Self::CreateColormap => 78,
            Self::FreeColormap => 79,
            Self::CopyColormapAndFree => 80,
            Self::InstallColormap => 81,
            Self::UninstallColormap => 82,
            Self::ListInstalledColormaps => 83,
            Self::AllocColor => 84,
            Self::AllocNamedColor => 85,
            Self::AllocColorCells => 86,
            Self::AllocColorPlanes => 87,
            Self::FreeColors => 88,
            Self::StoreColors => 89,
            Self::StoreNamedColor => 90,
            Self::QueryColors => 91,
            Self::LookupColor => 92,
            Self::CreateCursor => 93,
            Self::CreateGlyphCursor => 94,
            Self::FreeCursor => 95,
            Self::RecolorCursor => 96,
            Self::QueryBestSize => 97,
            Self::QueryExtension => 98,
            Self::ListExtensions => 99,
            Self::ChangeKeyboardMapping => 100,
            Self::GetKeyboardMapping => 101,
            Self::ChangeKeyboardControl => 102,
            Self::GetKeyboardControl => 103,
            Self::Bell => 104,
            Self::ChangePointerControl => 105,
            Self::GetPointerControl => 106,
            Self::SetScreenSaver => 107,
            Self::GetScreenSaver => 108,
            Self::ChangeHosts => 109,
            Self::ListHosts => 110,
            Self::SetAccessControl => 111,
            Self::SetCloseDownMode => 112,
            Self::KillClient => 113,
            Self::RotateProperties => 114,
            Self::ForceScreenSaver => 115,
            Self::SetPointerMapping => 116,
            Self::GetPointerMapping => 117,
            Self::SetModifierMapping => 118,
            Self::GetModifierMapping => 119,
            Self::NoOperation => 127,
            Self::Extension(ext_op) => ext_op.major_opcode(),
        }
    }

    /// Check if this is a core protocol opcode
    pub fn is_core_protocol(self) -> bool {
        !matches!(self, Self::Extension(_))
    }

    /// Check if this is an extension opcode
    pub fn is_extension(self) -> bool {
        matches!(self, Self::Extension(_))
    }

    /// Get the name of the opcode
    pub fn name(self) -> String {
        match self {
            Self::CreateWindow => "CreateWindow".to_string(),
            Self::ChangeWindowAttributes => "ChangeWindowAttributes".to_string(),
            Self::GetWindowAttributes => "GetWindowAttributes".to_string(),
            Self::DestroyWindow => "DestroyWindow".to_string(),
            Self::DestroySubwindows => "DestroySubwindows".to_string(),
            Self::ChangeSaveSet => "ChangeSaveSet".to_string(),
            Self::ReparentWindow => "ReparentWindow".to_string(),
            Self::MapWindow => "MapWindow".to_string(),
            Self::MapSubwindows => "MapSubwindows".to_string(),
            Self::UnmapWindow => "UnmapWindow".to_string(),
            Self::UnmapSubwindows => "UnmapSubwindows".to_string(),
            Self::ConfigureWindow => "ConfigureWindow".to_string(),
            Self::CirculateWindow => "CirculateWindow".to_string(),
            Self::GetGeometry => "GetGeometry".to_string(),
            Self::QueryTree => "QueryTree".to_string(),
            Self::InternAtom => "InternAtom".to_string(),
            Self::GetAtomName => "GetAtomName".to_string(),
            Self::ChangeProperty => "ChangeProperty".to_string(),
            Self::DeleteProperty => "DeleteProperty".to_string(),
            Self::GetProperty => "GetProperty".to_string(),
            Self::ListProperties => "ListProperties".to_string(),
            Self::SetSelectionOwner => "SetSelectionOwner".to_string(),
            Self::GetSelectionOwner => "GetSelectionOwner".to_string(),
            Self::ConvertSelection => "ConvertSelection".to_string(),
            Self::SendEvent => "SendEvent".to_string(),
            Self::GrabPointer => "GrabPointer".to_string(),
            Self::UngrabPointer => "UngrabPointer".to_string(),
            Self::GrabButton => "GrabButton".to_string(),
            Self::UngrabButton => "UngrabButton".to_string(),
            Self::ChangeActivePointerGrab => "ChangeActivePointerGrab".to_string(),
            Self::GrabKeyboard => "GrabKeyboard".to_string(),
            Self::UngrabKeyboard => "UngrabKeyboard".to_string(),
            Self::GrabKey => "GrabKey".to_string(),
            Self::UngrabKey => "UngrabKey".to_string(),
            Self::AllowEvents => "AllowEvents".to_string(),
            Self::GrabServer => "GrabServer".to_string(),
            Self::UngrabServer => "UngrabServer".to_string(),
            Self::QueryPointer => "QueryPointer".to_string(),
            Self::GetMotionEvents => "GetMotionEvents".to_string(),
            Self::TranslateCoordinates => "TranslateCoordinates".to_string(),
            Self::WarpPointer => "WarpPointer".to_string(),
            Self::SetInputFocus => "SetInputFocus".to_string(),
            Self::GetInputFocus => "GetInputFocus".to_string(),
            Self::QueryKeymap => "QueryKeymap".to_string(),
            Self::OpenFont => "OpenFont".to_string(),
            Self::CloseFont => "CloseFont".to_string(),
            Self::QueryFont => "QueryFont".to_string(),
            Self::QueryTextExtents => "QueryTextExtents".to_string(),
            Self::ListFonts => "ListFonts".to_string(),
            Self::ListFontsWithInfo => "ListFontsWithInfo".to_string(),
            Self::SetFontPath => "SetFontPath".to_string(),
            Self::GetFontPath => "GetFontPath".to_string(),
            Self::CreatePixmap => "CreatePixmap".to_string(),
            Self::FreePixmap => "FreePixmap".to_string(),
            Self::CreateGC => "CreateGC".to_string(),
            Self::ChangeGC => "ChangeGC".to_string(),
            Self::CopyGC => "CopyGC".to_string(),
            Self::SetDashes => "SetDashes".to_string(),
            Self::SetClipRectangles => "SetClipRectangles".to_string(),
            Self::FreeGC => "FreeGC".to_string(),
            Self::ClearArea => "ClearArea".to_string(),
            Self::CopyArea => "CopyArea".to_string(),
            Self::CopyPlane => "CopyPlane".to_string(),
            Self::PolyPoint => "PolyPoint".to_string(),
            Self::PolyLine => "PolyLine".to_string(),
            Self::PolySegment => "PolySegment".to_string(),
            Self::PolyRectangle => "PolyRectangle".to_string(),
            Self::PolyArc => "PolyArc".to_string(),
            Self::FillPoly => "FillPoly".to_string(),
            Self::PolyFillRectangle => "PolyFillRectangle".to_string(),
            Self::PolyFillArc => "PolyFillArc".to_string(),
            Self::PutImage => "PutImage".to_string(),
            Self::GetImage => "GetImage".to_string(),
            Self::PolyText8 => "PolyText8".to_string(),
            Self::PolyText16 => "PolyText16".to_string(),
            Self::ImageText8 => "ImageText8".to_string(),
            Self::ImageText16 => "ImageText16".to_string(),
            Self::CreateColormap => "CreateColormap".to_string(),
            Self::FreeColormap => "FreeColormap".to_string(),
            Self::CopyColormapAndFree => "CopyColormapAndFree".to_string(),
            Self::InstallColormap => "InstallColormap".to_string(),
            Self::UninstallColormap => "UninstallColormap".to_string(),
            Self::ListInstalledColormaps => "ListInstalledColormaps".to_string(),
            Self::AllocColor => "AllocColor".to_string(),
            Self::AllocNamedColor => "AllocNamedColor".to_string(),
            Self::AllocColorCells => "AllocColorCells".to_string(),
            Self::AllocColorPlanes => "AllocColorPlanes".to_string(),
            Self::FreeColors => "FreeColors".to_string(),
            Self::StoreColors => "StoreColors".to_string(),
            Self::StoreNamedColor => "StoreNamedColor".to_string(),
            Self::QueryColors => "QueryColors".to_string(),
            Self::LookupColor => "LookupColor".to_string(),
            Self::CreateCursor => "CreateCursor".to_string(),
            Self::CreateGlyphCursor => "CreateGlyphCursor".to_string(),
            Self::FreeCursor => "FreeCursor".to_string(),
            Self::RecolorCursor => "RecolorCursor".to_string(),
            Self::QueryBestSize => "QueryBestSize".to_string(),
            Self::QueryExtension => "QueryExtension".to_string(),
            Self::ListExtensions => "ListExtensions".to_string(),
            Self::ChangeKeyboardMapping => "ChangeKeyboardMapping".to_string(),
            Self::GetKeyboardMapping => "GetKeyboardMapping".to_string(),
            Self::ChangeKeyboardControl => "ChangeKeyboardControl".to_string(),
            Self::GetKeyboardControl => "GetKeyboardControl".to_string(),
            Self::Bell => "Bell".to_string(),
            Self::ChangePointerControl => "ChangePointerControl".to_string(),
            Self::GetPointerControl => "GetPointerControl".to_string(),
            Self::SetScreenSaver => "SetScreenSaver".to_string(),
            Self::GetScreenSaver => "GetScreenSaver".to_string(),
            Self::ChangeHosts => "ChangeHosts".to_string(),
            Self::ListHosts => "ListHosts".to_string(),
            Self::SetAccessControl => "SetAccessControl".to_string(),
            Self::SetCloseDownMode => "SetCloseDownMode".to_string(),
            Self::KillClient => "KillClient".to_string(),
            Self::RotateProperties => "RotateProperties".to_string(),
            Self::ForceScreenSaver => "ForceScreenSaver".to_string(),
            Self::SetPointerMapping => "SetPointerMapping".to_string(),
            Self::GetPointerMapping => "GetPointerMapping".to_string(),
            Self::SetModifierMapping => "SetModifierMapping".to_string(),
            Self::GetModifierMapping => "GetModifierMapping".to_string(),
            Self::NoOperation => "NoOperation".to_string(),
            Self::Extension(ext_op) => ext_op.name(),
        }
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name(), self.to_u8())
    }
}

impl From<Opcode> for u8 {
    fn from(opcode: Opcode) -> Self {
        opcode.to_u8()
    }
}

impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        Self::from_u8(value)
    }
}

/// Basic X11 request structure
#[derive(Debug, Clone)]
pub struct Request {
    pub opcode: Opcode,
    pub length: u16,
    pub sequence_number: u16,
    pub data: Vec<u8>,
}

impl Request {
    pub fn new(opcode: Opcode, sequence_number: u16, data: Vec<u8>) -> Self {
        let length = ((data.len() + 4) / 4) as u16; // Length in 4-byte units
        Self {
            opcode,
            length,
            sequence_number,
            data,
        }
    }

    /// Create a new Request where data already includes the 4-byte header
    pub fn new_with_header(
        opcode: Opcode,
        sequence_number: u16,
        data_with_header: Vec<u8>,
    ) -> Self {
        let length = (data_with_header.len() / 4) as u16; // Length in 4-byte units
        Self {
            opcode,
            length,
            sequence_number,
            data: data_with_header,
        }
    }

    pub fn opcode(&self) -> Opcode {
        self.opcode
    }
}

/// Basic X11 response structure
#[derive(Debug, Clone)]
pub struct Response {
    pub response_type: u8,
    pub data: Vec<u8>,
}

impl Response {
    pub fn new(response_type: u8, data: Vec<u8>) -> Self {
        Self {
            response_type,
            data,
        }
    }

    /// Create a "no response" marker (for requests that don't expect responses)
    pub fn no_response() -> Option<Self> {
        None
    }
}
