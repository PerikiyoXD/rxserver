//! Endian-aware writer utilities for X11 protocol responses
//! 
//! This module provides utilities for writing X11 protocol data with proper
//! byte order handling as required by the X11 specification.

use super::types::ByteOrder;

/// Utility for handling connection-specific byte order operations
/// This is mandatory by X11 specification for proper protocol compliance
pub struct EndianWriter<'a> {
    buffer: &'a mut Vec<u8>,
    byte_order: ByteOrder,
}

impl<'a> EndianWriter<'a> {
    /// Create a new EndianWriter with the specified byte order
    pub fn new(buffer: &'a mut Vec<u8>, byte_order: ByteOrder) -> Self {
        Self { buffer, byte_order }
    }

    /// Write a single byte (no endianness concerns)
    pub fn write_u8(&mut self, value: u8) {
        self.buffer.push(value);
    }

    /// Write a 16-bit value with proper byte order
    pub fn write_u16(&mut self, value: u16) {
        match self.byte_order {
            ByteOrder::LittleEndian => self.buffer.extend_from_slice(&value.to_le_bytes()),
            ByteOrder::BigEndian => self.buffer.extend_from_slice(&value.to_be_bytes()),
        }
    }

    /// Write a 32-bit value with proper byte order
    pub fn write_u32(&mut self, value: u32) {
        match self.byte_order {
            ByteOrder::LittleEndian => self.buffer.extend_from_slice(&value.to_le_bytes()),
            ByteOrder::BigEndian => self.buffer.extend_from_slice(&value.to_be_bytes()),
        }
    }

    /// Write raw bytes (no endianness conversion)
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// Write a string with 4-byte padding as required by X11 protocol
    pub fn write_string_with_padding(&mut self, s: &str) {
        self.buffer.extend_from_slice(s.as_bytes());
        let padding = (4 - (s.len() % 4)) % 4;
        self.buffer.extend_from_slice(&vec![0u8; padding]);
    }

    /// Write a string with custom alignment padding
    pub fn write_padded_string(&mut self, s: &str, align: usize) {
        self.buffer.extend_from_slice(s.as_bytes());
        let padding = (align - (s.len() % align)) % align;
        self.buffer.extend_from_slice(&vec![0u8; padding]);
    }

    /// Get the current length of the buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get the byte order being used
    pub fn byte_order(&self) -> ByteOrder {
        self.byte_order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endian_writer_little_endian() {
        let mut buffer = Vec::new();
        let mut writer = EndianWriter::new(&mut buffer, ByteOrder::LittleEndian);
        
        writer.write_u16(0x1234);
        assert_eq!(buffer, vec![0x34, 0x12]);
        
        buffer.clear();
        writer.write_u32(0x12345678);
        assert_eq!(buffer, vec![0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_endian_writer_big_endian() {
        let mut buffer = Vec::new();
        let mut writer = EndianWriter::new(&mut buffer, ByteOrder::BigEndian);
        
        writer.write_u16(0x1234);
        assert_eq!(buffer, vec![0x12, 0x34]);
        
        buffer.clear();
        writer.write_u32(0x12345678);
        assert_eq!(buffer, vec![0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn test_string_padding() {
        let mut buffer = Vec::new();
        let mut writer = EndianWriter::new(&mut buffer, ByteOrder::LittleEndian);
        
        writer.write_string_with_padding("ab");
        assert_eq!(buffer, vec![b'a', b'b', 0, 0]); // 2 bytes padding to align to 4
        
        buffer.clear();
        writer.write_string_with_padding("abcd");
        assert_eq!(buffer, vec![b'a', b'b', b'c', b'd']); // No padding needed
    }
}
