use crate::{
    Request, Response,
    types::Result,
    x11::{
        protocol::{RequestKind, errors::ProtocolError, requests::UnmapWindow},
        requests::handlers::X11RequestHandler,
    },
};

pub struct UnmapWindowRequestHandler;

impl X11RequestHandler for UnmapWindowRequestHandler {
    async fn handle(request: &Request) -> Result<Option<Response>> {
        if let RequestKind::UnmapWindow(ref req) = request.kind {
            // Validate the unmap window request
            Self::validate_request(req)?;

            // Process the unmap window request
            Self::process_request(req).await?;

            // UnmapWindow is typically a request-only operation with no response
            Ok(None)
        } else {
            Err(crate::types::Error::Protocol(
                ProtocolError::InvalidRequestType {
                    expected: "UnmapWindow".to_string(),
                    actual: format!("{:?}", request.kind),
                }
            ))
        }
    }
}

impl UnmapWindowRequestHandler {
    /// Validate the unmap window request
    fn validate_request(req: &UnmapWindow) -> Result<()> {
        // Validate window ID
        if req.window == 0 {
            return Err(crate::types::Error::Protocol(
                ProtocolError::InvalidWindowId { window_id: req.window }
            ));
        }

        Ok(())
    }

    /// Process the unmap window request
    async fn process_request(req: &UnmapWindow) -> Result<()> {
        tracing::info!("Unmapping window: id={:?}", req.window);

        // TODO: Implement actual window unmapping logic
        // This would typically involve:
        // 1. Validating the window exists
        // 2. Making the window invisible
        // 3. Sending UnmapNotify events

        Ok(())
    }
}
