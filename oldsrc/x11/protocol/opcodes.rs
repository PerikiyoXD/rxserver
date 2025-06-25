//! X11 Protocol Opcode Definitions
//!
//! This module defines all X11 protocol opcodes and provides routing functionality.

/// X11 Request opcodes (core protocol)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    // Window management
    CreateWindow = 1,
    ChangeWindowAttributes = 2,
    GetWindowAttributes = 3,
    DestroyWindow = 4,
    DestroySubwindows = 5,
    ChangeSaveSet = 6,
    ReparentWindow = 7,
    MapWindow = 8,
    MapSubwindows = 9,
    UnmapWindow = 10,
    UnmapSubwindows = 11,
    ConfigureWindow = 12,
    CirculateWindow = 13,
    GetGeometry = 14,
    QueryTree = 15,

    // Atom operations
    InternAtom = 16,
    GetAtomName = 17,

    // Property operations
    ChangeProperty = 18,
    DeleteProperty = 19,
    GetProperty = 20,
    ListProperties = 21,

    // Selection operations
    SetSelectionOwner = 22,
    GetSelectionOwner = 23,
    ConvertSelection = 24,

    // Event operations
    SendEvent = 25,
    GrabPointer = 26,
    UngrabPointer = 27,
    GrabButton = 28,
    UngrabButton = 29,
    ChangeActivePointerGrab = 30,
    GrabKeyboard = 31,
    UngrabKeyboard = 32,
    GrabKey = 33,
    UngrabKey = 34,
    AllowEvents = 35,
    GrabServer = 36,
    UngrabServer = 37,

    // Pointer and keyboard
    QueryPointer = 38,
    GetMotionEvents = 39,
    TranslateCoordinates = 40,
    WarpPointer = 41,
    SetInputFocus = 42,
    GetInputFocus = 43,
    QueryKeymap = 44,

    // Font operations
    OpenFont = 45,
    CloseFont = 46,
    QueryFont = 47,
    QueryTextExtents = 48,
    ListFonts = 49,
    ListFontsWithInfo = 50,
    SetFontPath = 51,
    GetFontPath = 52,

    // Pixmap operations
    CreatePixmap = 53,
    FreePixmap = 54,

    // Graphics context operations
    CreateGC = 55,
    ChangeGC = 56,
    CopyGC = 57,
    SetDashes = 58,
    SetClipRectangles = 59,
    FreeGC = 60,

    // Drawing operations
    ClearArea = 61,
    CopyArea = 62,
    CopyPlane = 63,
    PolyPoint = 64,
    PolyLine = 65,
    PolySegment = 66,
    PolyRectangle = 67,
    PolyArc = 68,
    FillPoly = 69,
    PolyFillRectangle = 70,
    PolyFillArc = 71,
    PutImage = 72,
    GetImage = 73,
    PolyText8 = 74,
    PolyText16 = 75,
    ImageText8 = 76,
    ImageText16 = 77,

    // Colormap operations
    CreateColormap = 78,
    FreeColormap = 79,
    CopyColormapAndFree = 80,
    InstallColormap = 81,
    UninstallColormap = 82,
    ListInstalledColormaps = 83,
    AllocColor = 84,
    AllocNamedColor = 85,
    AllocColorCells = 86,
    AllocColorPlanes = 87,
    FreeColors = 88,
    StoreColors = 89,
    StoreNamedColor = 90,
    QueryColors = 91,
    LookupColor = 92,

    // Cursor operations
    CreateCursor = 93,
    CreateGlyphCursor = 94,
    FreeCursor = 95,
    RecolorCursor = 96,
    QueryBestSize = 97,

    // Extension operations
    QueryExtension = 98,
    ListExtensions = 99,

    // Keyboard and pointer control
    ChangeKeyboardMapping = 100,
    GetKeyboardMapping = 101,
    ChangeKeyboardControl = 102,
    GetKeyboardControl = 103,
    Bell = 104,
    ChangePointerControl = 105,
    GetPointerControl = 106,
    SetScreenSaver = 107,
    GetScreenSaver = 108,
    ChangeHosts = 109,
    ListHosts = 110,
    SetAccessControl = 111,
    SetCloseDownMode = 112,
    KillClient = 113,
    RotateProperties = 114,
    ForceScreenSaver = 115,
    SetPointerMapping = 116,
    GetPointerMapping = 117,
    SetModifierMapping = 118,
    GetModifierMapping = 119,

    // Special
    NoOperation = 127,
}

impl Opcode {
    /// Convert from u8 to Opcode
    pub fn from_u8(opcode: u8) -> Option<Self> {
        match opcode {
            1 => Some(Opcode::CreateWindow),
            2 => Some(Opcode::ChangeWindowAttributes),
            3 => Some(Opcode::GetWindowAttributes),
            4 => Some(Opcode::DestroyWindow),
            5 => Some(Opcode::DestroySubwindows),
            6 => Some(Opcode::ChangeSaveSet),
            7 => Some(Opcode::ReparentWindow),
            8 => Some(Opcode::MapWindow),
            9 => Some(Opcode::MapSubwindows),
            10 => Some(Opcode::UnmapWindow),
            11 => Some(Opcode::UnmapSubwindows),
            12 => Some(Opcode::ConfigureWindow),
            13 => Some(Opcode::CirculateWindow),
            14 => Some(Opcode::GetGeometry),
            15 => Some(Opcode::QueryTree),
            16 => Some(Opcode::InternAtom),
            17 => Some(Opcode::GetAtomName),
            18 => Some(Opcode::ChangeProperty),
            19 => Some(Opcode::DeleteProperty),
            20 => Some(Opcode::GetProperty),
            21 => Some(Opcode::ListProperties),
            22 => Some(Opcode::SetSelectionOwner),
            23 => Some(Opcode::GetSelectionOwner),
            24 => Some(Opcode::ConvertSelection),
            25 => Some(Opcode::SendEvent),
            26 => Some(Opcode::GrabPointer),
            27 => Some(Opcode::UngrabPointer),
            28 => Some(Opcode::GrabButton),
            29 => Some(Opcode::UngrabButton),
            30 => Some(Opcode::ChangeActivePointerGrab),
            31 => Some(Opcode::GrabKeyboard),
            32 => Some(Opcode::UngrabKeyboard),
            33 => Some(Opcode::GrabKey),
            34 => Some(Opcode::UngrabKey),
            35 => Some(Opcode::AllowEvents),
            36 => Some(Opcode::GrabServer),
            37 => Some(Opcode::UngrabServer),
            38 => Some(Opcode::QueryPointer),
            39 => Some(Opcode::GetMotionEvents),
            40 => Some(Opcode::TranslateCoordinates),
            41 => Some(Opcode::WarpPointer),
            42 => Some(Opcode::SetInputFocus),
            43 => Some(Opcode::GetInputFocus),
            44 => Some(Opcode::QueryKeymap),
            45 => Some(Opcode::OpenFont),
            46 => Some(Opcode::CloseFont),
            47 => Some(Opcode::QueryFont),
            48 => Some(Opcode::QueryTextExtents),
            49 => Some(Opcode::ListFonts),
            50 => Some(Opcode::ListFontsWithInfo),
            51 => Some(Opcode::SetFontPath),
            52 => Some(Opcode::GetFontPath),
            53 => Some(Opcode::CreatePixmap),
            54 => Some(Opcode::FreePixmap),
            55 => Some(Opcode::CreateGC),
            56 => Some(Opcode::ChangeGC),
            57 => Some(Opcode::CopyGC),
            58 => Some(Opcode::SetDashes),
            59 => Some(Opcode::SetClipRectangles),
            60 => Some(Opcode::FreeGC),
            61 => Some(Opcode::ClearArea),
            62 => Some(Opcode::CopyArea),
            63 => Some(Opcode::CopyPlane),
            64 => Some(Opcode::PolyPoint),
            65 => Some(Opcode::PolyLine),
            66 => Some(Opcode::PolySegment),
            67 => Some(Opcode::PolyRectangle),
            68 => Some(Opcode::PolyArc),
            69 => Some(Opcode::FillPoly),
            70 => Some(Opcode::PolyFillRectangle),
            71 => Some(Opcode::PolyFillArc),
            72 => Some(Opcode::PutImage),
            73 => Some(Opcode::GetImage),
            74 => Some(Opcode::PolyText8),
            75 => Some(Opcode::PolyText16),
            76 => Some(Opcode::ImageText8),
            77 => Some(Opcode::ImageText16),
            78 => Some(Opcode::CreateColormap),
            79 => Some(Opcode::FreeColormap),
            80 => Some(Opcode::CopyColormapAndFree),
            81 => Some(Opcode::InstallColormap),
            82 => Some(Opcode::UninstallColormap),
            83 => Some(Opcode::ListInstalledColormaps),
            84 => Some(Opcode::AllocColor),
            85 => Some(Opcode::AllocNamedColor),
            86 => Some(Opcode::AllocColorCells),
            87 => Some(Opcode::AllocColorPlanes),
            88 => Some(Opcode::FreeColors),
            89 => Some(Opcode::StoreColors),
            90 => Some(Opcode::StoreNamedColor),
            91 => Some(Opcode::QueryColors),
            92 => Some(Opcode::LookupColor),
            93 => Some(Opcode::CreateCursor),
            94 => Some(Opcode::CreateGlyphCursor),
            95 => Some(Opcode::FreeCursor),
            96 => Some(Opcode::RecolorCursor),
            97 => Some(Opcode::QueryBestSize),
            98 => Some(Opcode::QueryExtension),
            99 => Some(Opcode::ListExtensions),
            100 => Some(Opcode::ChangeKeyboardMapping),
            101 => Some(Opcode::GetKeyboardMapping),
            102 => Some(Opcode::ChangeKeyboardControl),
            103 => Some(Opcode::GetKeyboardControl),
            104 => Some(Opcode::Bell),
            105 => Some(Opcode::ChangePointerControl),
            106 => Some(Opcode::GetPointerControl),
            107 => Some(Opcode::SetScreenSaver),
            108 => Some(Opcode::GetScreenSaver),
            109 => Some(Opcode::ChangeHosts),
            110 => Some(Opcode::ListHosts),
            111 => Some(Opcode::SetAccessControl),
            112 => Some(Opcode::SetCloseDownMode),
            113 => Some(Opcode::KillClient),
            114 => Some(Opcode::RotateProperties),
            115 => Some(Opcode::ForceScreenSaver),
            116 => Some(Opcode::SetPointerMapping),
            117 => Some(Opcode::GetPointerMapping),
            118 => Some(Opcode::SetModifierMapping),
            119 => Some(Opcode::GetModifierMapping),
            127 => Some(Opcode::NoOperation),
            _ => None,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Check if this opcode requires a response
    pub fn has_response(self) -> bool {
        matches!(
            self,
            Opcode::GetWindowAttributes
                | Opcode::GetGeometry
                | Opcode::QueryTree
                | Opcode::GetAtomName
                | Opcode::GetProperty
                | Opcode::ListProperties
                | Opcode::GetSelectionOwner
                | Opcode::QueryPointer
                | Opcode::GetMotionEvents
                | Opcode::TranslateCoordinates
                | Opcode::GetInputFocus
                | Opcode::QueryKeymap
                | Opcode::QueryFont
                | Opcode::QueryTextExtents
                | Opcode::ListFonts
                | Opcode::ListFontsWithInfo
                | Opcode::GetFontPath
                | Opcode::GetImage
                | Opcode::ListInstalledColormaps
                | Opcode::AllocColor
                | Opcode::AllocNamedColor
                | Opcode::AllocColorCells
                | Opcode::AllocColorPlanes
                | Opcode::QueryColors
                | Opcode::LookupColor
                | Opcode::QueryBestSize
                | Opcode::QueryExtension
                | Opcode::ListExtensions
                | Opcode::GetKeyboardMapping
                | Opcode::GetKeyboardControl
                | Opcode::GetPointerControl
                | Opcode::GetScreenSaver
                | Opcode::ListHosts
                | Opcode::GetPointerMapping
                | Opcode::GetModifierMapping
        )
    }

    /// Get a human-readable name for the opcode
    pub fn name(self) -> &'static str {
        match self {
            Opcode::CreateWindow => "CreateWindow",
            Opcode::ChangeWindowAttributes => "ChangeWindowAttributes",
            Opcode::GetWindowAttributes => "GetWindowAttributes",
            Opcode::DestroyWindow => "DestroyWindow",
            Opcode::DestroySubwindows => "DestroySubwindows",
            Opcode::ChangeSaveSet => "ChangeSaveSet",
            Opcode::ReparentWindow => "ReparentWindow",
            Opcode::MapWindow => "MapWindow",
            Opcode::MapSubwindows => "MapSubwindows",
            Opcode::UnmapWindow => "UnmapWindow",
            Opcode::UnmapSubwindows => "UnmapSubwindows",
            Opcode::ConfigureWindow => "ConfigureWindow",
            Opcode::CirculateWindow => "CirculateWindow",
            Opcode::GetGeometry => "GetGeometry",
            Opcode::QueryTree => "QueryTree",
            Opcode::InternAtom => "InternAtom",
            Opcode::GetAtomName => "GetAtomName",
            Opcode::ChangeProperty => "ChangeProperty",
            Opcode::DeleteProperty => "DeleteProperty",
            Opcode::GetProperty => "GetProperty",
            Opcode::ListProperties => "ListProperties",
            Opcode::SetSelectionOwner => "SetSelectionOwner",
            Opcode::GetSelectionOwner => "GetSelectionOwner",
            Opcode::ConvertSelection => "ConvertSelection",
            Opcode::SendEvent => "SendEvent",
            Opcode::GrabPointer => "GrabPointer",
            Opcode::UngrabPointer => "UngrabPointer",
            Opcode::GrabButton => "GrabButton",
            Opcode::UngrabButton => "UngrabButton",
            Opcode::ChangeActivePointerGrab => "ChangeActivePointerGrab",
            Opcode::GrabKeyboard => "GrabKeyboard",
            Opcode::UngrabKeyboard => "UngrabKeyboard",
            Opcode::GrabKey => "GrabKey",
            Opcode::UngrabKey => "UngrabKey",
            Opcode::AllowEvents => "AllowEvents",
            Opcode::GrabServer => "GrabServer",
            Opcode::UngrabServer => "UngrabServer",
            Opcode::QueryPointer => "QueryPointer",
            Opcode::GetMotionEvents => "GetMotionEvents",
            Opcode::TranslateCoordinates => "TranslateCoordinates",
            Opcode::WarpPointer => "WarpPointer",
            Opcode::SetInputFocus => "SetInputFocus",
            Opcode::GetInputFocus => "GetInputFocus",
            Opcode::QueryKeymap => "QueryKeymap",
            Opcode::OpenFont => "OpenFont",
            Opcode::CloseFont => "CloseFont",
            Opcode::QueryFont => "QueryFont",
            Opcode::QueryTextExtents => "QueryTextExtents",
            Opcode::ListFonts => "ListFonts",
            Opcode::ListFontsWithInfo => "ListFontsWithInfo",
            Opcode::SetFontPath => "SetFontPath",
            Opcode::GetFontPath => "GetFontPath",
            Opcode::CreatePixmap => "CreatePixmap",
            Opcode::FreePixmap => "FreePixmap",
            Opcode::CreateGC => "CreateGC",
            Opcode::ChangeGC => "ChangeGC",
            Opcode::CopyGC => "CopyGC",
            Opcode::SetDashes => "SetDashes",
            Opcode::SetClipRectangles => "SetClipRectangles",
            Opcode::FreeGC => "FreeGC",
            Opcode::ClearArea => "ClearArea",
            Opcode::CopyArea => "CopyArea",
            Opcode::CopyPlane => "CopyPlane",
            Opcode::PolyPoint => "PolyPoint",
            Opcode::PolyLine => "PolyLine",
            Opcode::PolySegment => "PolySegment",
            Opcode::PolyRectangle => "PolyRectangle",
            Opcode::PolyArc => "PolyArc",
            Opcode::FillPoly => "FillPoly",
            Opcode::PolyFillRectangle => "PolyFillRectangle",
            Opcode::PolyFillArc => "PolyFillArc",
            Opcode::PutImage => "PutImage",
            Opcode::GetImage => "GetImage",
            Opcode::PolyText8 => "PolyText8",
            Opcode::PolyText16 => "PolyText16",
            Opcode::ImageText8 => "ImageText8",
            Opcode::ImageText16 => "ImageText16",
            Opcode::CreateColormap => "CreateColormap",
            Opcode::FreeColormap => "FreeColormap",
            Opcode::CopyColormapAndFree => "CopyColormapAndFree",
            Opcode::InstallColormap => "InstallColormap",
            Opcode::UninstallColormap => "UninstallColormap",
            Opcode::ListInstalledColormaps => "ListInstalledColormaps",
            Opcode::AllocColor => "AllocColor",
            Opcode::AllocNamedColor => "AllocNamedColor",
            Opcode::AllocColorCells => "AllocColorCells",
            Opcode::AllocColorPlanes => "AllocColorPlanes",
            Opcode::FreeColors => "FreeColors",
            Opcode::StoreColors => "StoreColors",
            Opcode::StoreNamedColor => "StoreNamedColor",
            Opcode::QueryColors => "QueryColors",
            Opcode::LookupColor => "LookupColor",
            Opcode::CreateCursor => "CreateCursor",
            Opcode::CreateGlyphCursor => "CreateGlyphCursor",
            Opcode::FreeCursor => "FreeCursor",
            Opcode::RecolorCursor => "RecolorCursor",
            Opcode::QueryBestSize => "QueryBestSize",
            Opcode::QueryExtension => "QueryExtension",
            Opcode::ListExtensions => "ListExtensions",
            Opcode::ChangeKeyboardMapping => "ChangeKeyboardMapping",
            Opcode::GetKeyboardMapping => "GetKeyboardMapping",
            Opcode::ChangeKeyboardControl => "ChangeKeyboardControl",
            Opcode::GetKeyboardControl => "GetKeyboardControl",
            Opcode::Bell => "Bell",
            Opcode::ChangePointerControl => "ChangePointerControl",
            Opcode::GetPointerControl => "GetPointerControl",
            Opcode::SetScreenSaver => "SetScreenSaver",
            Opcode::GetScreenSaver => "GetScreenSaver",
            Opcode::ChangeHosts => "ChangeHosts",
            Opcode::ListHosts => "ListHosts",
            Opcode::SetAccessControl => "SetAccessControl",
            Opcode::SetCloseDownMode => "SetCloseDownMode",
            Opcode::KillClient => "KillClient",
            Opcode::RotateProperties => "RotateProperties",
            Opcode::ForceScreenSaver => "ForceScreenSaver",
            Opcode::SetPointerMapping => "SetPointerMapping",
            Opcode::GetPointerMapping => "GetPointerMapping",
            Opcode::SetModifierMapping => "SetModifierMapping",
            Opcode::GetModifierMapping => "GetModifierMapping",
            Opcode::NoOperation => "NoOperation",
        }
    }
}

impl From<u8> for Opcode {
    fn from(opcode: u8) -> Self {
        Opcode::from_u8(opcode).unwrap_or(Opcode::NoOperation)
    }
}

/// Event type codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

impl EventType {
    /// Convert from u8 to EventType
    pub fn from_u8(event_type: u8) -> Option<Self> {
        match event_type {
            2 => Some(EventType::KeyPress),
            3 => Some(EventType::KeyRelease),
            4 => Some(EventType::ButtonPress),
            5 => Some(EventType::ButtonRelease),
            6 => Some(EventType::MotionNotify),
            7 => Some(EventType::EnterNotify),
            8 => Some(EventType::LeaveNotify),
            9 => Some(EventType::FocusIn),
            10 => Some(EventType::FocusOut),
            11 => Some(EventType::KeymapNotify),
            12 => Some(EventType::Expose),
            13 => Some(EventType::GraphicsExpose),
            14 => Some(EventType::NoExpose),
            15 => Some(EventType::VisibilityNotify),
            16 => Some(EventType::CreateNotify),
            17 => Some(EventType::DestroyNotify),
            18 => Some(EventType::UnmapNotify),
            19 => Some(EventType::MapNotify),
            20 => Some(EventType::MapRequest),
            21 => Some(EventType::ReparentNotify),
            22 => Some(EventType::ConfigureNotify),
            23 => Some(EventType::ConfigureRequest),
            24 => Some(EventType::GravityNotify),
            25 => Some(EventType::ResizeRequest),
            26 => Some(EventType::CirculateNotify),
            27 => Some(EventType::CirculateRequest),
            28 => Some(EventType::PropertyNotify),
            29 => Some(EventType::SelectionClear),
            30 => Some(EventType::SelectionRequest),
            31 => Some(EventType::SelectionNotify),
            32 => Some(EventType::ColormapNotify),
            33 => Some(EventType::ClientMessage),
            34 => Some(EventType::MappingNotify),
            _ => None,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}
