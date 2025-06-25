use super::types::*;

/// X11 protocol opcodes
pub mod opcodes {
    pub const CONNECTION_SETUP: u8 = 0;
    pub const GET_GEOMETRY: u8 = 14;
}

#[derive(Debug, Clone)]
pub enum RequestKind {
    ConnectionSetup,
    GetGeometry(GetGeometryRequest),
    // CreateWindow,
    // DestroyWindow,
    // Add other request variants as needed
}

#[derive(Debug, Clone)]
pub struct Request {
    pub kind: RequestKind,
    pub sequence_number: SequenceNumber,
}

/// GetGeometry request structure matching X11 protocol
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetGeometryRequest {
    pub opcode: u8,  // Should be 14
    pub unused: u8,  // Padding
    pub length: u16, // Request length in 4-byte units (always 2)
    pub drawable: DrawableId,
}

impl Request {
    /// Parse a request from raw bytes
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(X11Error::Protocol("Empty request".to_string()));
        }

        let opcode = bytes[0];
        match opcode {
            opcodes::CONNECTION_SETUP => {
                Ok(Request {
                    kind: RequestKind::ConnectionSetup,
                    sequence_number: 0, // Connection setup doesn't have sequence number
                })
            }
            opcodes::GET_GEOMETRY => todo!(), // Implement GetGeometry parsing
            _ => Err(X11Error::Protocol(format!("Unknown opcode: {}", opcode))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get_geometry_request() {
        // Create a valid GetGeometry request
        let mut bytes = vec![0u8; 8];
        bytes[0] = opcodes::GET_GEOMETRY; // opcode
        bytes[1] = 0; // unused
        bytes[2..4].copy_from_slice(&2u16.to_le_bytes()); // length
        bytes[4..8].copy_from_slice(&0x12345678u32.to_le_bytes()); // drawable

        let result = Request::parse(&bytes).unwrap();

        match result.kind {
            _ => todo!(), // Implement GetGeometry request handling
        }
    }

    #[test]
    fn test_parse_get_geometry_request_too_short() {
        let bytes = vec![opcodes::GET_GEOMETRY, 0, 2]; // Only 3 bytes, need 8
        let result = Request::parse(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_request() {
        let bytes = vec![];
        let result = Request::parse(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unknown_opcode() {
        let bytes = vec![255u8; 8]; // Unknown opcode
        let result = Request::parse(&bytes);
        assert!(result.is_err());
    }
}
