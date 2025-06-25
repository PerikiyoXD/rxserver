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
    InternAtom(InternAtomRequest),
}

#[derive(Debug, Clone)]
pub struct Request {
    pub kind: RequestKind,
    pub sequence_number: SequenceNumber,
    pub data: Vec<u8>,    // Raw request data for parsing
    pub opcode_value: u8, // Store the actual opcode
}

impl Request {
    /// Get the opcode for this request
    pub fn opcode(&self) -> u8 {
        self.opcode_value
    }

    /// Create a new request with header data
    pub fn new_with_header(opcode: u8, sequence_number: SequenceNumber, data: Vec<u8>) -> Self {
        // Determine the kind based on opcode
        let kind = match opcode {
            opcodes::CONNECTION_SETUP => RequestKind::ConnectionSetup,
            opcodes::GET_GEOMETRY => RequestKind::GetGeometry(GetGeometryRequest {
                opcode,
                unused: 0,
                length: 2,
                drawable: 0, // Will be parsed from data
            }),
            16 => RequestKind::InternAtom(InternAtomRequest {
                opcode,
                unused: 0,
                length: 2,
                name: [0; 128], // Will be parsed from data
            }),
            _ => RequestKind::ConnectionSetup, // Default fallback
        };

        Self {
            kind,
            sequence_number,
            data,
            opcode_value: opcode,
        }
    }
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

/// InternAtom request structure matching X11 protocol
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InternAtomRequest {
    pub opcode: u8,      // Should be 0
    pub unused: u8,      // Padding
    pub length: u16,     // Request length in 4-byte units (always 2)
    pub name: [u8; 128], // Atom name (null-terminated)
}

impl Request {
    /// Parse a request from raw bytes
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(X11Error::Protocol("Empty request".to_string()));
        }

        let opcode = bytes[0];
        let sequence_number = 0; // Will be set by connection handler

        match opcode {
            opcodes::CONNECTION_SETUP => Ok(Request {
                kind: RequestKind::ConnectionSetup,
                sequence_number,
                data: bytes.to_vec(),
                opcode_value: opcode,
            }),
            opcodes::GET_GEOMETRY => {
                if bytes.len() < 8 {
                    return Err(X11Error::Protocol(
                        "GetGeometry request too short".to_string(),
                    ));
                }

                let drawable = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
                let request = GetGeometryRequest {
                    opcode,
                    unused: bytes[1],
                    length: u16::from_le_bytes([bytes[2], bytes[3]]),
                    drawable,
                };

                Ok(Request {
                    kind: RequestKind::GetGeometry(request),
                    sequence_number,
                    data: bytes.to_vec(),
                    opcode_value: opcode,
                })
            }
            _ => {
                // For unknown opcodes, create a generic request
                Ok(Request {
                    kind: RequestKind::ConnectionSetup, // Default
                    sequence_number,
                    data: bytes.to_vec(),
                    opcode_value: opcode,
                })
            }
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
