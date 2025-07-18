use super::response::*;
use super::types::*;

/// Trait for serializing X11 protocol responses
pub trait ResponseSerializer {
    /// Serialize a response to bytes
    fn serialize(&self, response: &Response) -> Result<Option<Vec<u8>>>;

    /// Get the serialized size without allocating
    fn serialized_size(&self) -> usize {
        panic!("serialized_size not implemented");
    }
}

/// Serializer for GetGeometry responses
impl ResponseSerializer for GetGeometryResponse {
    fn serialize(&self, _response: &Response) -> Result<Option<Vec<u8>>> {
        // GetGeometry response is fixed size (32 bytes)
        let mut bytes = Vec::with_capacity(32);

        bytes.push(self.response_type);
        bytes.push(self.depth);
        bytes.extend_from_slice(&self.sequence_number.to_le_bytes());
        bytes.extend_from_slice(&self.length.to_le_bytes());
        bytes.extend_from_slice(&self.root.to_le_bytes());
        bytes.extend_from_slice(&self.x.to_le_bytes());
        bytes.extend_from_slice(&self.y.to_le_bytes());
        bytes.extend_from_slice(&self.width.to_le_bytes());
        bytes.extend_from_slice(&self.height.to_le_bytes());
        bytes.extend_from_slice(&self.border_width.to_le_bytes());
        bytes.extend_from_slice(&self.unused);

        Ok(Some(bytes))
    }

    fn serialized_size(&self) -> usize {
        32 // GetGeometry response is always 32 bytes
    }
}

/// Generic response serializer that dispatches to specific serializers
pub struct X11ResponseSerializer;

impl ResponseSerializer for X11ResponseSerializer {
    fn serialize(&self, response: &Response) -> Result<Option<Vec<u8>>> {
        match &response.kind {
            ResponseKind::GetGeometry(get_geo) => get_geo.serialize(response),
            ResponseKind::Reply => Ok(Some(vec![])), // Empty reply
            ResponseKind::ConnectionSetup(_) => {
                // TODO: Implement connection setup serialization
                Ok(Some(vec![]))
            }
            ResponseKind::Raw(raw) => Ok(Some(raw.data.to_vec())),
        }
    }
}

// Legacy compatibility - remove the incorrect type reference
// Note: ConnectionSetupAcceptedResponse doesn't exist, using placeholder

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_geometry_response_serialization() {
        let response = GetGeometryResponse::new(
            1234, // sequence_number
            24,   // depth
            100,  // root window
            10,   // x
            20,   // y
            800,  // width
            600,  // height
            2,    // border_width
        );

        let dummy_response = Response::default();
        let serialized = response.serialize(&dummy_response).unwrap().unwrap();

        // Verify the serialized length
        assert_eq!(serialized.len(), 32);

        // Verify first few bytes
        assert_eq!(serialized[0], 1); // response_type
        assert_eq!(serialized[1], 24); // depth

        // Verify sequence number (little endian)
        let seq_bytes = u16::from_le_bytes([serialized[2], serialized[3]]);
        assert_eq!(seq_bytes, 1234);

        // Verify width and height
        let x_bytes = i16::from_le_bytes([serialized[12], serialized[13]]);
        let y_bytes = i16::from_le_bytes([serialized[14], serialized[15]]);
        let width_bytes = u16::from_le_bytes([serialized[16], serialized[17]]);
        let height_bytes = u16::from_le_bytes([serialized[18], serialized[19]]);

        assert_eq!(x_bytes, 10);
        assert_eq!(y_bytes, 20);
        assert_eq!(width_bytes, 800);
        assert_eq!(height_bytes, 600);
    }

    #[test]
    fn test_get_geometry_response_size() {
        let response = GetGeometryResponse::new(0, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(response.serialized_size(), 32);
    }

    #[test]
    fn test_x11_response_serializer_get_geometry() {
        let geo_response = GetGeometryResponse::new(100, 24, 1, 0, 0, 640, 480, 1);
        let response = Response {
            kind: ResponseKind::GetGeometry(geo_response),
            sequence_number: 100,
            byte_order: ByteOrder::LittleEndian,
        };

        let serializer = X11ResponseSerializer;
        let result = serializer.serialize(&response).unwrap().unwrap();
        assert_eq!(result.len(), 32);
    }
}
