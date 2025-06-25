use crate::{
    Request, Response,
    types::Result,
    x11::{
        protocol::{RequestKind, errors::ProtocolError, requests::NoOperation},
        requests::handlers::X11RequestHandler,
    },
};

pub struct NoOperationRequestHandler;

impl X11RequestHandler for NoOperationRequestHandler {
    async fn handle(request: &Request) -> Result<Option<Response>> {
        if let RequestKind::NoOperation(ref _req) = request.kind {
            // NoOperation request requires no validation or processing
            Self::process_request().await?;

            // NoOperation has no response
            Ok(None)
        } else {
            Err(crate::types::Error::Protocol(
                ProtocolError::InvalidRequestType {
                    expected: "NoOperation".to_string(),
                    actual: format!("{:?}", request.kind),
                }
            ))
        }
    }
}

impl NoOperationRequestHandler {
    /// Process the no operation request
    async fn process_request() -> Result<()> {
        tracing::debug!("Processing NoOperation request");
        
        // NoOperation literally does nothing - it's used for:
        // 1. Testing connection liveness
        // 2. Synchronization points
        // 3. Padding/alignment in request streams
        
        Ok(())
    }
}
