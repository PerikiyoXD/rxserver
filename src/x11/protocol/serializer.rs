//! X11 Protocol Message Serializer
//!
//! This module provides functionality for serializing structured X11 types
//! into wire format bytes.

use crate::x11::protocol::{
    endianness::ByteOrder,
    types::*,
    wire::{EventMessage, ResponseMessage},
};

/// Protocol message serializer
#[derive(Debug, Clone)]
pub struct ProtocolSerializer {
    byte_order: ByteOrder,
}

impl ProtocolSerializer {
    /// Create a new serializer with the given byte order
    pub fn new(byte_order: ByteOrder) -> Self {
        Self { byte_order }
    }

    /// Serialize a response to bytes
    pub fn serialize_response(
        &self,
        response: &Response,
        sequence_number: SequenceNumber,
    ) -> Vec<u8> {
        match response {
            Response::Empty => self.serialize_empty_response(sequence_number),
            Response::GetGeometry {
                root,
                x,
                y,
                width,
                height,
                border_width,
                depth,
            } => self.serialize_get_geometry_response(
                *root,
                *x,
                *y,
                *width,
                *height,
                *border_width,
                *depth,
                sequence_number,
            ),
            Response::QueryTree {
                root,
                parent,
                children,
            } => self.serialize_query_tree_response(*root, *parent, children, sequence_number),
            Response::InternAtom { atom } => {
                self.serialize_intern_atom_response(*atom, sequence_number)
            }
        }
    }

    /// Serialize an error response
    pub fn serialize_error(&self, error: &ErrorResponse) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(32);

        bytes.push(0); // Error indicator
        bytes.push(error.error_code);
        bytes.extend_from_slice(&self.write_u16(error.sequence_number));
        bytes.extend_from_slice(&self.write_u32(error.resource_id));
        bytes.extend_from_slice(&self.write_u16(error.minor_opcode));
        bytes.push(error.major_opcode);

        // Pad to 32 bytes
        bytes.resize(32, 0);
        bytes
    }

    /// Serialize setup response
    pub fn serialize_setup_response(&self, response: &SetupResponse) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(1); // Success status
        bytes.push(0); // Pad
        bytes.extend_from_slice(&self.write_u16(response.protocol_major_version));
        bytes.extend_from_slice(&self.write_u16(response.protocol_minor_version));
        bytes.extend_from_slice(&self.write_u16(response.length));
        bytes.extend_from_slice(&self.write_u32(response.release_number));
        bytes.extend_from_slice(&self.write_u32(response.resource_id_base));
        bytes.extend_from_slice(&self.write_u32(response.resource_id_mask));
        bytes.extend_from_slice(&self.write_u32(response.motion_buffer_size));

        // Vendor string length and data
        let vendor_bytes = response.vendor.as_bytes();
        bytes.extend_from_slice(&self.write_u16(vendor_bytes.len() as u16));
        bytes.extend_from_slice(&self.write_u16(response.maximum_request_length));
        bytes.push(response.number_of_screens);
        bytes.push(response.number_of_formats);
        bytes.push(response.image_byte_order);
        bytes.push(response.bitmap_format_bit_order);
        bytes.push(response.bitmap_format_scanline_unit);
        bytes.push(response.bitmap_format_scanline_pad);
        bytes.push(response.min_keycode);
        bytes.push(response.max_keycode);

        // Pad to 4-byte boundary
        while bytes.len() % 4 != 0 {
            bytes.push(0);
        }

        // Add vendor string
        bytes.extend_from_slice(vendor_bytes);

        // Pad vendor string to 4-byte boundary
        while bytes.len() % 4 != 0 {
            bytes.push(0);
        }

        // TODO: Add pixmap formats and screen information

        bytes
    }

    /// Serialize empty response (for requests that don't return data)
    fn serialize_empty_response(&self, sequence_number: SequenceNumber) -> Vec<u8> {
        let response = ResponseMessage::new(1, 0, sequence_number, Vec::new());
        response.to_bytes(self.byte_order)
    }

    /// Serialize GetGeometry response
    fn serialize_get_geometry_response(
        &self,
        root: WindowId,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        depth: u8,
        sequence_number: SequenceNumber,
    ) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(depth);
        data.extend_from_slice(&[0, 0, 0]); // 3 bytes padding
        data.extend_from_slice(&self.write_u32(root));
        data.extend_from_slice(&self.write_i16(x));
        data.extend_from_slice(&self.write_i16(y));
        data.extend_from_slice(&self.write_u16(width));
        data.extend_from_slice(&self.write_u16(height));
        data.extend_from_slice(&self.write_u16(border_width));
        data.extend_from_slice(&[0, 0]); // 2 bytes padding

        let response = ResponseMessage::new(1, 0, sequence_number, data);
        response.to_bytes(self.byte_order)
    }

    /// Serialize QueryTree response
    fn serialize_query_tree_response(
        &self,
        root: WindowId,
        parent: WindowId,
        children: &[WindowId],
        sequence_number: SequenceNumber,
    ) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&[0, 0, 0, 0]); // 4 bytes padding
        data.extend_from_slice(&self.write_u32(root));
        data.extend_from_slice(&self.write_u32(parent));
        data.extend_from_slice(&self.write_u16(children.len() as u16));
        data.extend_from_slice(&[0, 0]); // 2 bytes padding

        // Add children list
        for &child in children {
            data.extend_from_slice(&self.write_u32(child));
        }
        let response = ResponseMessage::new(1, 0, sequence_number, data);
        response.to_bytes(self.byte_order)
    }

    /// Serialize InternAtom response
    fn serialize_intern_atom_response(
        &self,
        atom: XID,
        sequence_number: SequenceNumber,
    ) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.write_u32(atom)); // Atom ID
        data.extend_from_slice(&[0, 0, 0, 0]); // 4 bytes padding to align to 32 bytes
        data.extend_from_slice(&[0, 0, 0, 0]); // 4 more bytes padding
        data.extend_from_slice(&[0, 0, 0, 0]); // 4 more bytes padding
        data.extend_from_slice(&[0, 0, 0, 0]); // 4 more bytes padding
        data.extend_from_slice(&[0, 0, 0, 0]); // 4 more bytes padding
        data.extend_from_slice(&[0, 0, 0, 0]); // 4 more bytes padding

        let response = ResponseMessage::new(1, 0, sequence_number, data);
        response.to_bytes(self.byte_order)
    }

    /// Helper to write u16 with current byte order
    fn write_u16(&self, value: u16) -> [u8; 2] {
        match self.byte_order {
            ByteOrder::LittleEndian => value.to_le_bytes(),
            ByteOrder::BigEndian => value.to_be_bytes(),
        }
    }

    /// Helper to write u32 with current byte order
    fn write_u32(&self, value: u32) -> [u8; 4] {
        match self.byte_order {
            ByteOrder::LittleEndian => value.to_le_bytes(),
            ByteOrder::BigEndian => value.to_be_bytes(),
        }
    }

    /// Helper to write i16 with current byte order
    fn write_i16(&self, value: i16) -> [u8; 2] {
        match self.byte_order {
            ByteOrder::LittleEndian => value.to_le_bytes(),
            ByteOrder::BigEndian => value.to_be_bytes(),
        }
    }

    /// Helper to write i32 with current byte order
    fn write_i32(&self, value: i32) -> [u8; 4] {
        match self.byte_order {
            ByteOrder::LittleEndian => value.to_le_bytes(),
            ByteOrder::BigEndian => value.to_be_bytes(),
        }
    }
}

/// Event serialization utilities
#[derive(Debug)]
pub struct EventSerializer {
    byte_order: ByteOrder,
    sequence_counter: SequenceNumber,
}

impl EventSerializer {
    /// Create a new event serializer
    pub fn new(byte_order: ByteOrder) -> Self {
        Self {
            byte_order,
            sequence_counter: 0,
        }
    }

    /// Get the next sequence number
    pub fn next_sequence(&mut self) -> SequenceNumber {
        self.sequence_counter = self.sequence_counter.wrapping_add(1);
        self.sequence_counter
    }

    /// Serialize a basic event (placeholder for now)
    pub fn serialize_event(&mut self, event_type: u8, detail: u8, data: [u8; 28]) -> [u8; 32] {
        let sequence = self.next_sequence();
        let event = EventMessage::new(event_type, detail, sequence, data);
        event.to_bytes(self.byte_order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_empty_response() {
        let serializer = ProtocolSerializer::new(ByteOrder::LittleEndian);
        let response = Response::Empty;
        let bytes = serializer.serialize_response(&response, 123);

        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 1); // Reply indicator
        assert_eq!(bytes[2], 123); // Sequence number (little-endian)
        assert_eq!(bytes[3], 0);
    }

    #[test]
    fn test_serialize_error() {
        let serializer = ProtocolSerializer::new(ByteOrder::LittleEndian);
        let error = ErrorResponse {
            error_code: 3,
            sequence_number: 456,
            resource_id: 0x12345678,
            minor_opcode: 0,
            major_opcode: 8,
        };

        let bytes = serializer.serialize_error(&error);
        assert_eq!(bytes.len(), 32);
        assert_eq!(bytes[0], 0); // Error indicator
        assert_eq!(bytes[1], 3); // Error code
    }
}
