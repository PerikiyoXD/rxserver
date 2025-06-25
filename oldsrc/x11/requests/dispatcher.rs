use crate::{
    Request, Response,
    types::Result,
    x11::{
        protocol::{RequestKind, errors::ProtocolError},
        requests::handlers::{
            X11RequestHandler,
            atom::InternAtomRequestHandler,
            cursor::CreateGlyphCursorRequestHandler,
            font::OpenFontRequestHandler,
            setup::connection_setup::ConnectionSetupRequestHandler,
            utility::{GetGeometryRequestHandler, NoOperationRequestHandler},
            window::{
                CreateWindowRequestHandler, DestroyWindowRequestHandler, MapWindowRequestHandler,
                UnmapWindowRequestHandler,
            },
        },
    },
};

pub async fn process_request(request: &Request) -> Result<Option<Response>> {
    match request.kind {
        // Connection setup requests
        RequestKind::ConnectionSetup(_) => ConnectionSetupRequestHandler::handle(request).await,

        // Window management requests
        RequestKind::CreateWindow(_) => CreateWindowRequestHandler::handle(request).await,
        RequestKind::DestroyWindow(_) => DestroyWindowRequestHandler::handle(request).await,
        RequestKind::MapWindow(_) => MapWindowRequestHandler::handle(request).await,
        RequestKind::UnmapWindow(_) => UnmapWindowRequestHandler::handle(request).await,

        // Atom management requests
        RequestKind::InternAtom(_) => InternAtomRequestHandler::handle(request).await,

        // Cursor management requests
        RequestKind::CreateGlyphCursor(_) => CreateGlyphCursorRequestHandler::handle(request).await,

        // Font management requests
        RequestKind::OpenFont(_) => OpenFontRequestHandler::handle(request).await,

        // Utility requests
        RequestKind::NoOperation(_) => NoOperationRequestHandler::handle(request).await,
        RequestKind::GetGeometry(_) => GetGeometryRequestHandler::handle(request).await,

        // Unimplemented or unknown requests
        _ => {
            tracing::warn!(
                "Received unimplemented or unknown request: {:?}",
                request.kind
            );
            Err(crate::types::Error::Protocol(
                ProtocolError::InvalidRequestType {
                    expected: "Known request type".to_string(),
                    actual: format!("{:?}", request.kind),
                },
            ))
        }
    }
}
