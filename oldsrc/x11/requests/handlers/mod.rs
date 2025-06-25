use crate::{Request, RequestKind, Response, types::Result};

pub mod atom;
pub mod cursor;
pub mod font;
pub mod setup;
pub mod utility;
pub mod window;

/// Trait for processing X11 protocol requests.
///
/// Each implementation handles a specific request type, returning either
/// a response or an error. Connects protocol parsing to business logic.
///
/// # Example
/// ```
/// use crate::x11::requests::handlers::RequestHandler;
/// use crate::{Request, Response, types::Result};
///
/// struct MyHandler;
///
/// impl RequestHandler for MyHandler {
///     async fn handle(request: &Request) -> Result<Option<Response>> {
///         // Process request and return response
///         Ok(None)
///     }
/// }
/// ```
pub trait X11RequestHandler {
    /// Process a request and return the appropriate response.
    async fn handle(request: &Request) -> Result<Option<Response>>;
}

pub struct RequestHandlers;

impl RequestHandlers {
    /// Handle a request by delegating to the appropriate handler.
    pub async fn handle_request(request: &Request) -> Result<Option<Response>> {
        match request.kind {
            // Atom requests
            RequestKind::InternAtom(ref req) => {
                atom::intern::InternAtomRequestHandler::handle(request).await
            }
            RequestKind::GetAtomName(ref req) => {
                atom::get_name::GetAtomNameRequestHandler::handle(request).await
            }

            // Window requests
            RequestKind::CreateWindow(ref req) => {
                window::create::CreateWindowRequestHandler::handle(request).await
            }
            RequestKind::DestroyWindow(ref req) => {
                window::destroy::DestroyWindowRequestHandler::handle(request).await
            }

            // Cursor requests
            RequestKind::CreateCursor(ref req) => {
                cursor::create::CreateCursorRequestHandler::handle(request).await
            }

            // Font requests
            RequestKind::OpenFont(ref req) => {
                font::open::OpenFontRequestHandler::handle(request).await
            }

            // Utility requests
            RequestKind::GetInputFocus(ref req) => {
                utility::get_input_focus::GetInputFocusRequestHandler::handle(request).await
            }

            _ => Err(crate::types::Error::Protocol(
                ProtocolError::UnsupportedRequest {
                    kind: format!("{:?}", request.kind),
                },
            )),
        }
    }
}
