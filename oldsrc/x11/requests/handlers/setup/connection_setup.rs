use crate::{
    ConnectionSetupRequest, Request, RequestKind, Response,
    types::{Error::Protocol, Result},
    x11::requests::handlers::X11RequestHandler,
};

pub struct ConnectionSetupRequestHandler;

impl X11RequestHandler for ConnectionSetupRequestHandler {
    async fn handle(request: &Request) -> Result<Option<Response>> {
        if let RequestKind::ConnectionSetup(req) = request.kind() {
            // Validate the connection setup request
            req.validate()?;

            // Process the connection setup request
            let response = req.process().await?;

            // Return the response wrapped in an Option
            Ok(Some(Response::ConnectionSetup(response)))
        } else {
            Err(Protocol(Protocol::InvalidRequestType {
                expected: "ConnectionSetup",
                actual: request.kind().to_string(),
            }))
        }
    }
}

impl ConnectionSetupRequest {
    /// Validate the connection setup request
    pub fn validate(&self) -> Result<()> {
        // Perform any necessary validation on the request fields
        if self.protocol_major_version < 1 || self.protocol_minor_version < 0 {
            return Err(Protocol(Protocol::InvalidVersion {
                major: self.protocol_major_version,
                minor: self.protocol_minor_version,
            }));
        }
        // Additional validation logic can be added here
        Ok(())
    }

    /// Process the connection setup request and generate a response
    pub async fn process(&self) -> Result<ConnectionSetupResponse> {}
}
