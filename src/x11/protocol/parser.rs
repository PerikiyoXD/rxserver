//! X11 Protocol Message Parser
//!
//! This module provides functionality for parsing X11 protocol messages
//! from wire format into structured types.

use crate::x11::protocol::{
    endianness::ByteOrder,
    errors::ProtocolError,
    opcodes::Opcode,
    types::*,
    wire::{MessageHeader, RequestMessage},
};

/// Main protocol parser
#[derive(Debug)]
pub struct ProtocolParser {
    byte_order: ByteOrder,
}

impl ProtocolParser {
    /// Create a new parser with the given byte order
    pub fn new(byte_order: ByteOrder) -> Self {
        Self { byte_order }
    }

    /// Parse a complete request message from bytes
    pub fn parse_request(&self, bytes: &[u8]) -> Result<Request, ProtocolError> {
        self.validate_length(bytes, 4)?;

        let header = MessageHeader::from_bytes(bytes, self.byte_order)?;
        let opcode =
            Opcode::from_u8(header.opcode).ok_or(ProtocolError::InvalidOpcode(header.opcode))?;

        let request_msg = RequestMessage::from_bytes(bytes, self.byte_order)?;

        self.parse_request_by_opcode(opcode, &request_msg)
    }

    /// Parse a request based on its opcode
    fn parse_request_by_opcode(
        &self,
        opcode: Opcode,
        msg: &RequestMessage,
    ) -> Result<Request, ProtocolError> {
        match opcode {
            Opcode::NoOperation => Ok(Request::NoOperation),
            Opcode::CreateWindow => self.parse_create_window(msg),
            Opcode::MapWindow => self.parse_map_window(msg),
            Opcode::UnmapWindow => self.parse_unmap_window(msg),
            Opcode::DestroyWindow => self.parse_destroy_window(msg),
            Opcode::InternAtom => self.parse_intern_atom(msg), // Add more opcodes as we implement them
            Opcode::OpenFont => self.parse_open_font(msg),
            Opcode::CreateGlyphCursor => self.parse_create_glyph_cursor(msg),
            _ => {
                // For now, return NoOperation for unimplemented opcodes
                // TODO: Implement all request types
                tracing::debug!("Unimplemented opcode: {:?}", opcode);
                Ok(Request::NoOperation)
            }
        }
    }
    /// Parse CreateWindow request
    fn parse_create_window(&self, msg: &RequestMessage) -> Result<Request, ProtocolError> {
        self.validate_length(&msg.data, 28)?;

        let depth = msg.header.detail;
        let wid = self.read_u32(&msg.data[0..4])?;
        let parent = self.read_u32(&msg.data[4..8])?;
        let x = self.read_i16(&msg.data[8..10])?;
        let y = self.read_i16(&msg.data[10..12])?;
        let width = self.read_u16(&msg.data[12..14])?;
        let height = self.read_u16(&msg.data[14..16])?;
        let border_width = self.read_u16(&msg.data[16..18])?;
        let class = self.read_u16(&msg.data[18..20])?;
        let visual = self.read_u32(&msg.data[20..24])?;
        let value_mask = self.read_u32(&msg.data[24..28])?;

        // Value list follows after the fixed part
        let value_list = msg.data[28..].to_vec();

        Ok(Request::CreateWindow {
            depth,
            wid,
            parent,
            x,
            y,
            width,
            height,
            border_width,
            class,
            visual,
            value_mask,
            value_list,
        })
    }

    /// Parse MapWindow request
    fn parse_map_window(&self, msg: &RequestMessage) -> Result<Request, ProtocolError> {
        self.validate_length(&msg.data, 4)?;
        let window = self.read_u32(&msg.data[0..4])?;
        Ok(Request::MapWindow { window })
    }

    /// Parse UnmapWindow request
    fn parse_unmap_window(&self, msg: &RequestMessage) -> Result<Request, ProtocolError> {
        self.validate_length(&msg.data, 4)?;
        let window = self.read_u32(&msg.data[0..4])?;
        Ok(Request::UnmapWindow { window })
    }

    /// Parse DestroyWindow request
    fn parse_destroy_window(&self, msg: &RequestMessage) -> Result<Request, ProtocolError> {
        self.validate_length(&msg.data, 4)?;
        let window = self.read_u32(&msg.data[0..4])?;
        Ok(Request::DestroyWindow { window })
    }
    /// Parse InternAtom request
    fn parse_intern_atom(&self, msg: &RequestMessage) -> Result<Request, ProtocolError> {
        self.validate_length(&msg.data, 4)?;

        let only_if_exists = msg.header.detail != 0;

        // InternAtom format:
        // 0-1: name length (u16)
        // 2-3: unused padding
        // 4+: name string (padded to 4-byte boundary)
        let name_length = self.read_u16(&msg.data[0..2])? as usize;

        if name_length == 0 {
            return Ok(Request::InternAtom {
                only_if_exists,
                name: String::new(),
            });
        }

        // Read the name string starting at offset 4, with padding
        let (name, _) = self.read_padded_string(&msg.data, 4, name_length)?;

        Ok(Request::InternAtom {
            only_if_exists,
            name,
        })
    }

    /// Parse OpenFont request
    fn parse_open_font(&self, msg: &RequestMessage) -> Result<Request, ProtocolError> {
        self.validate_length(&msg.data, 8)?;

        // OpenFont format:
        // 0-3: font id (u32)
        // 4-5: name length (u16)
        // 6-7: unused padding
        let font_id = self.read_u32(&msg.data[0..4])?;
        let name_length = self.read_u16(&msg.data[4..6])? as usize;

        if name_length == 0 {
            return Ok(Request::OpenFont {
                font_id,
                name: String::new(),
            });
        }

        // Read the font name string starting at offset 8, with padding
        let (name, _) = self.read_padded_string(&msg.data, 8, name_length)?;

        Ok(Request::OpenFont { font_id, name })
    }

    /// Parse CreateGlyphCursor request
    fn parse_create_glyph_cursor(&self, msg: &RequestMessage) -> Result<Request, ProtocolError> {
        self.validate_length(&msg.data, 16)?;

        // CreateGlyphCursor
        //  1     94                              opcode
        //  1                                     unused
        //  2     8                               request length
        //  4     CURSOR                          cid
        //  4     FONT                            source-font
        //  4     FONT                            mask-font
        //       0     None
        //  2     CARD16                          source-char
        //  2     CARD16                          mask-char
        //  2     CARD16                          fore-red
        //  2     CARD16                          fore-green
        //  2     CARD16                          fore-blue
        //  2     CARD16                          back-red
        //  2     CARD16                          back-green
        //  2     CARD16                          back-blue

        let cid = self.read_u32(&msg.data[0..4])?;
        let source_font = self.read_u32(&msg.data[4..8])?;
        let mask_font = self.read_u32(&msg.data[8..12])?;
        let source_char = self.read_u16(&msg.data[12..14])?;
        let mask_char = self.read_u16(&msg.data[14..16])?;
        let fore_red = self.read_u16(&msg.data[16..18])?;
        let fore_green = self.read_u16(&msg.data[18..20])?;
        let fore_blue = self.read_u16(&msg.data[20..22])?;
        let back_red = self.read_u16(&msg.data[22..24])?;
        let back_green = self.read_u16(&msg.data[24..26])?;
        let back_blue = self.read_u16(&msg.data[26..28])?;

        Ok(Request::CreateGlyphCursor {
            cursor_id: cid,
            source_font,
            mask_font: Some(mask_font),
            source_char,
            mask_char,
            fore_red,
            fore_green,
            fore_blue,
            back_red,
            back_green,
            back_blue,
        })
    }

    /// Parse connection setup request
    pub fn parse_setup_request(&self, bytes: &[u8]) -> Result<SetupRequest, ProtocolError> {
        self.validate_length(bytes, 12)?;

        let byte_order = bytes[0];
        if ByteOrder::from_marker(byte_order).is_none() {
            return Err(ProtocolError::InvalidByteOrder(byte_order));
        }

        // Skip pad byte at offset 1
        let protocol_major_version = self.read_u16(&bytes[2..4])?;
        let protocol_minor_version = self.read_u16(&bytes[4..6])?;
        let auth_proto_name_len = self.read_u16(&bytes[6..8])? as usize;
        let auth_proto_data_len = self.read_u16(&bytes[8..10])? as usize;

        // Skip 2 pad bytes at offset 10-11, start reading auth data at offset 12
        let (authorization_protocol_name, authorization_protocol_data, _) =
            self.read_auth_data(bytes, 12, auth_proto_name_len, auth_proto_data_len)?;

        Ok(SetupRequest {
            byte_order,
            protocol_major_version,
            protocol_minor_version,
            authorization_protocol_name,
            authorization_protocol_data,
        })
    }

    /// Validate minimum buffer length
    fn validate_length(&self, bytes: &[u8], required: usize) -> Result<(), ProtocolError> {
        if bytes.len() < required {
            return Err(ProtocolError::MessageTooShort {
                expected: required,
                actual: bytes.len(),
            });
        }
        Ok(())
    }

    /// Read a fixed-length string from buffer at offset
    fn read_fixed_string(
        &self,
        bytes: &[u8],
        offset: usize,
        length: usize,
    ) -> Result<String, ProtocolError> {
        self.validate_length(bytes, offset + length)?;
        let string_bytes = &bytes[offset..offset + length];
        Ok(String::from_utf8_lossy(string_bytes).to_string())
    }

    /// Read a length-prefixed string (u16 length followed by string data)
    fn read_length_prefixed_string(
        &self,
        bytes: &[u8],
        offset: usize,
    ) -> Result<(String, usize), ProtocolError> {
        self.validate_length(bytes, offset + 2)?;

        let length = self.read_u16(&bytes[offset..offset + 2])? as usize;
        let string_offset = offset + 2;

        if length == 0 {
            return Ok((String::new(), string_offset));
        }

        let string = self.read_fixed_string(bytes, string_offset, length)?;
        Ok((string, string_offset + length))
    }

    /// Read a padded string (string data padded to 4-byte boundary)
    fn read_padded_string(
        &self,
        bytes: &[u8],
        offset: usize,
        length: usize,
    ) -> Result<(String, usize), ProtocolError> {
        if length == 0 {
            return Ok((String::new(), offset));
        }

        let string = self.read_fixed_string(bytes, offset, length)?;
        let padded_offset = (offset + length + 3) & !3; // Align to 4-byte boundary
        Ok((string, padded_offset))
    }

    /// Read authorization data (name and data with padding)
    fn read_auth_data(
        &self,
        bytes: &[u8],
        offset: usize,
        name_len: usize,
        data_len: usize,
    ) -> Result<(String, Vec<u8>, usize), ProtocolError> {
        let mut current_offset = offset;

        // Read authorization protocol name with padding
        let (auth_name, next_offset) = self.read_padded_string(bytes, current_offset, name_len)?;
        current_offset = next_offset;

        // Read authorization protocol data
        let auth_data = if data_len > 0 {
            self.validate_length(bytes, current_offset + data_len)?;
            let data = bytes[current_offset..current_offset + data_len].to_vec();
            current_offset += data_len;
            // Pad to 4-byte boundary
            current_offset = (current_offset + 3) & !3;
            data
        } else {
            Vec::new()
        };

        Ok((auth_name, auth_data, current_offset))
    }

    /// Helper to read u16 with current byte order
    fn read_u16(&self, bytes: &[u8]) -> Result<u16, ProtocolError> {
        self.validate_length(bytes, 2)?;
        Ok(match self.byte_order {
            ByteOrder::LittleEndian => u16::from_le_bytes([bytes[0], bytes[1]]),
            ByteOrder::BigEndian => u16::from_be_bytes([bytes[0], bytes[1]]),
        })
    }

    /// Helper to read u32 with current byte order
    fn read_u32(&self, bytes: &[u8]) -> Result<u32, ProtocolError> {
        self.validate_length(bytes, 4)?;
        Ok(match self.byte_order {
            ByteOrder::LittleEndian => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            ByteOrder::BigEndian => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        })
    }

    /// Helper to read i16 with current byte order
    fn read_i16(&self, bytes: &[u8]) -> Result<i16, ProtocolError> {
        self.validate_length(bytes, 2)?;
        Ok(match self.byte_order {
            ByteOrder::LittleEndian => i16::from_le_bytes([bytes[0], bytes[1]]),
            ByteOrder::BigEndian => i16::from_be_bytes([bytes[0], bytes[1]]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_no_operation() {
        let parser = ProtocolParser::new(ByteOrder::LittleEndian);
        let bytes = [127, 0, 1, 0]; // NoOperation opcode, detail=0, length=1

        let result = parser.parse_request(&bytes);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Request::NoOperation));
    }
    #[test]
    fn test_parse_intern_atom() {
        let parser = ProtocolParser::new(ByteOrder::LittleEndian);

        // Create an InternAtom request for "_NET_WM_NAME"
        let atom_name = "_NET_WM_NAME";
        let name_len = atom_name.len() as u16;

        let mut data = Vec::new();
        data.extend_from_slice(&name_len.to_le_bytes()); // name length (2 bytes)
        data.extend_from_slice(&[0, 0]); // padding (2 bytes)
        data.extend_from_slice(atom_name.as_bytes()); // name string

        // Pad the name to 4-byte boundary
        while (data.len() - 4) % 4 != 0 {
            data.push(0);
        }

        // Calculate total length in 4-byte units
        let total_len = ((data.len() + 4 + 3) / 4) as u16; // +4 for header, round up

        let mut bytes = Vec::new();
        bytes.push(16); // InternAtom opcode
        bytes.push(0); // only_if_exists = false
        bytes.extend_from_slice(&total_len.to_le_bytes()); // request length
        bytes.extend_from_slice(&data); // request data

        let result = parser.parse_request(&bytes);
        assert!(result.is_ok());

        if let Request::InternAtom {
            only_if_exists,
            name,
        } = result.unwrap()
        {
            assert_eq!(only_if_exists, false);
            assert_eq!(name, "_NET_WM_NAME");
        } else {
            panic!("Expected InternAtom request");
        }
    }
    #[test]
    fn test_parse_intern_atom_only_if_exists() {
        let parser = ProtocolParser::new(ByteOrder::LittleEndian);

        // Create an InternAtom request for "UTF8_STRING" with only_if_exists=true
        let atom_name = "UTF8_STRING";
        let name_len = atom_name.len() as u16;

        let mut data = Vec::new();
        data.extend_from_slice(&name_len.to_le_bytes()); // name length (2 bytes)
        data.extend_from_slice(&[0, 0]); // padding (2 bytes)
        data.extend_from_slice(atom_name.as_bytes()); // name string

        // Pad the name to 4-byte boundary
        while (data.len() - 4) % 4 != 0 {
            data.push(0);
        }

        // Calculate total length in 4-byte units
        let total_len = ((data.len() + 4 + 3) / 4) as u16; // +4 for header, round up

        let mut bytes = Vec::new();
        bytes.push(16); // InternAtom opcode
        bytes.push(1); // only_if_exists = true
        bytes.extend_from_slice(&total_len.to_le_bytes()); // request length
        bytes.extend_from_slice(&data); // request data

        let result = parser.parse_request(&bytes);
        if let Err(ref e) = result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());

        if let Request::InternAtom {
            only_if_exists,
            name,
        } = result.unwrap()
        {
            assert_eq!(only_if_exists, true);
            assert_eq!(name, "UTF8_STRING");
        } else {
            panic!("Expected InternAtom request");
        }
    }

    #[test]
    fn test_parse_intern_atom_empty_name() {
        let parser = ProtocolParser::new(ByteOrder::LittleEndian);

        let mut data = Vec::new();
        data.extend_from_slice(&[0, 0]); // name length = 0 (2 bytes)
        data.extend_from_slice(&[0, 0]); // padding (2 bytes)

        // Calculate total length in 4-byte units
        let total_len = ((data.len() + 4 + 3) / 4) as u16; // +4 for header, round up

        let mut bytes = Vec::new();
        bytes.push(16); // InternAtom opcode
        bytes.push(0); // only_if_exists = false
        bytes.extend_from_slice(&total_len.to_le_bytes()); // request length
        bytes.extend_from_slice(&data); // request data

        let result = parser.parse_request(&bytes);
        assert!(result.is_ok());

        if let Request::InternAtom {
            only_if_exists,
            name,
        } = result.unwrap()
        {
            assert_eq!(only_if_exists, false);
            assert_eq!(name, "");
        } else {
            panic!("Expected InternAtom request");
        }
    }

    #[test]
    fn test_parse_setup_request() {
        let parser = ProtocolParser::new(ByteOrder::LittleEndian);
        let bytes = vec![
            b'l', 0, // Byte order, pad
            11, 0, // Protocol major version
            0, 0, // Protocol minor version
            0, 0, // Auth protocol name length
            0, 0, // Auth protocol data length
            0, 0, // Pad
        ];

        let result = parser.parse_setup_request(&bytes);
        assert!(result.is_ok());

        let setup = result.unwrap();
        assert_eq!(setup.byte_order, b'l');
        assert_eq!(setup.protocol_major_version, 11);
        assert_eq!(setup.protocol_minor_version, 0);
    }
}
