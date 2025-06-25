use super::types::*;

#[derive(Debug, Clone)]
pub enum RequestKind {
    ConnectionSetup,
    // Populate with real opcodes as needed
    CreateWindow,
    DestroyWindow,
    // etc
    // Add other request variants
}

#[derive(Debug, Clone)]
pub struct Request {
    pub kind: RequestKind,
    // Fill with fields as needed for your protocol
}

impl Request {
    pub fn parse(_bytes: &[u8]) -> Result<Self> {
        // TODO: Implement real parsing
        Ok(Request {
            kind: RequestKind::ConnectionSetup,
        })
    }
}
