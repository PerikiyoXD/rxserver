pub mod deserializer;
pub mod types;

pub use deserializer::*;
pub use types::*;

use crate::SequenceNumber;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub kind: RequestKind,
    pub sequence_number: SequenceNumber,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// X11 Request types
pub enum RequestKind {
    ConnectionSetup(request::ConnectionSetupRequestHandler),
    CreateWindow(request::CreateWindowRequestHandler),
    CreateGlyphCursor(request::CreateGlyphCursorRequestHandler),
    MapWindow(request::MapWindowRequestHandler),
    UnmapWindow(request::UnmapWindowRequestHandler),
    DestroyWindow(request::DestroyWindowRequestHandler),
    InternAtom(request::InternAtomRequestHandler),
    OpenFont(request::OpenFontRequestHandler),
    NoOperation(request::NoOperationRequestHandler),
    GetGeometry(request::GetGeometryRequestHandler),
}

///
impl RequestKind {
    pub fn request_type(&self) -> RequestType {
        match self {
            RequestKind::ConnectionSetup(_) => RequestType::ConnectionSetup,
            RequestKind::CreateWindow(_) => RequestType::CreateWindow,
            RequestKind::CreateGlyphCursor(_) => RequestType::CreateGlyphCursor,
            RequestKind::MapWindow(_) => RequestType::MapWindow,
            RequestKind::UnmapWindow(_) => RequestType::UnmapWindow,
            RequestKind::DestroyWindow(_) => RequestType::DestroyWindow,
            RequestKind::InternAtom(_) => RequestType::InternAtom,
            RequestKind::OpenFont(_) => RequestType::OpenFont,
            RequestKind::NoOperation(_) => RequestType::NoOperation,
            RequestKind::GetGeometry(_) => RequestType::GetGeometry,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RequestType {
    ConnectionSetup,
    CreateWindow,
    CreateGlyphCursor,
    MapWindow,
    UnmapWindow,
    DestroyWindow,
    InternAtom,
    OpenFont,
    NoOperation,
    GetGeometry,
}
