use crate::{
    Request, Response,
    types::Result,
    x11::{
        protocol::{RequestKind, errors::ProtocolError, requests::InternAtom},
        requests::handlers::X11RequestHandler,
    },
};

pub struct InternAtomRequestHandler;

impl X11RequestHandler for InternAtomRequestHandler {
    async fn handle(request: &Request) -> Result<Option<Response>> {
        if let RequestKind::InternAtom(ref req) = request.kind {
            // Validate the intern atom request
            Self::validate_request(req)?;

            // Process the intern atom request
            let response = Self::process_request(req).await?;

            // InternAtom returns a response with the atom ID
            Ok(Some(response))
        } else {
            Err(crate::types::Error::Protocol(
                ProtocolError::InvalidRequestType {
                    expected: "InternAtom".to_string(),
                    actual: format!("{:?}", request.kind),
                }
            ))
        }
    }
}

impl InternAtomRequestHandler {
    /// Validate the intern atom request
    fn validate_request(req: &InternAtom) -> Result<()> {
        // Validate atom name
        if req.name.is_empty() {
            return Err(crate::types::Error::Protocol(
                ProtocolError::InvalidAtomName { name: req.name.clone() }
            ));
        }

        // Check maximum atom name length (X11 typically limits this)
        if req.name.len() > 255 {
            return Err(crate::types::Error::Protocol(
                ProtocolError::AtomNameTooLong { 
                    name: req.name.clone(),
                    length: req.name.len()
                }
            ));
        }

        Ok(())
    }

    /// Process the intern atom request
    async fn process_request(req: &InternAtom) -> Result<Response> {
        tracing::info!("Interning atom: name='{}', only_if_exists={}", req.name, req.only_if_exists);

        // TODO: Implement actual atom interning logic
        // This would typically involve:
        // 1. Looking up existing atom by name
        // 2. If only_if_exists is true and atom doesn't exist, return None (0)
        // 3. If atom doesn't exist and only_if_exists is false, create new atom
        // 4. Return atom ID

        // For now, return a dummy atom ID
        let atom_id = 42; // This would be the actual atom ID from the atom registry

        Ok(Response {
            kind: crate::x11::protocol::ResponseKind::Reply,
            sequence_number: 0, // This would be set by the dispatcher
            byte_order: crate::x11::protocol::ByteOrder::LittleEndian,
        })
    }
}
