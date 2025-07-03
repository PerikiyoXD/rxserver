// SPDX-License-Identifier: Apache-2.0

//! Utilities for reading and writing X11 protocol data with correct endianness.
//!
//! Provides `EndianWriter` and `EndianReader` for encoding and decoding integers and strings
//! according to the byte order required by the X11 protocol.

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

/// Utility for reading X11 protocol data with proper byte order handling
/// This complements EndianWriter for parsing incoming requests
pub struct EndianReader<'a> {
    data: &'a [u8],
    offset: usize,
    byte_order: ByteOrder,
}

impl<'a> EndianReader<'a> {
    /// Create a new EndianReader with the specified byte order
    pub fn new(data: &'a [u8], byte_order: ByteOrder) -> Self {
        Self {
            data,
            offset: 0,
            byte_order,
        }
    }

    /// Read a single byte
    pub fn read_u8(&mut self) -> Result<u8, &'static str> {
        if self.offset >= self.data.len() {
            return Err("Not enough data to read u8");
        }
        let value = self.data[self.offset];
        self.offset += 1;
        Ok(value)
    }

    /// Read a 16-bit value with proper byte order
    pub fn read_u16(&mut self) -> Result<u16, &'static str> {
        if self.offset + 2 > self.data.len() {
            return Err("Not enough data to read u16");
        }
        let bytes = [self.data[self.offset], self.data[self.offset + 1]];
        self.offset += 2;

        match self.byte_order {
            ByteOrder::LittleEndian => Ok(u16::from_le_bytes(bytes)),
            ByteOrder::BigEndian => Ok(u16::from_be_bytes(bytes)),
        }
    }

    /// Read a 32-bit value with proper byte order
    pub fn read_u32(&mut self) -> Result<u32, &'static str> {
        if self.offset + 4 > self.data.len() {
            return Err("Not enough data to read u32");
        }
        let bytes = [
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
        ];
        self.offset += 4;

        match self.byte_order {
            ByteOrder::LittleEndian => Ok(u32::from_le_bytes(bytes)),
            ByteOrder::BigEndian => Ok(u32::from_be_bytes(bytes)),
        }
    }

    /// Read raw bytes
    pub fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], &'static str> {
        if self.offset + count > self.data.len() {
            return Err("Not enough data to read bytes");
        }
        let bytes = &self.data[self.offset..self.offset + count];
        self.offset += count;
        Ok(bytes)
    }

    /// Read a string with specified length, accounting for padding
    pub fn read_string(&mut self, length: usize) -> Result<String, &'static str> {
        let bytes = self.read_bytes(length)?;
        // Find the null terminator or use full length
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(length);
        String::from_utf8(bytes[..end].to_vec()).map_err(|_| "Invalid UTF-8 in string")
    }

    /// Skip padding bytes to align to specified boundary
    pub fn skip_padding(&mut self, align: usize) -> Result<(), &'static str> {
        let padding = (align - (self.offset % align)) % align;
        if self.offset + padding > self.data.len() {
            return Err("Not enough data for padding");
        }
        self.offset += padding;
        Ok(())
    }

    /// Get the current offset
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get the remaining bytes count
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }

    /// Check if there are any bytes remaining
    pub fn has_remaining(&self) -> bool {
        self.offset < self.data.len()
    }

    /// Get the byte order being used
    pub fn byte_order(&self) -> ByteOrder {
        self.byte_order
    }

    /// Peek at the next byte without advancing the offset
    pub fn peek_u8(&self) -> Result<u8, &'static str> {
        if self.offset >= self.data.len() {
            return Err("Not enough data to peek u8");
        }
        Ok(self.data[self.offset])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endian_writer_little_endian() {
        let mut buffer = Vec::new();
        {
            let mut writer = EndianWriter::new(&mut buffer, ByteOrder::LittleEndian);
            writer.write_u16(0x1234);
        }
        assert_eq!(buffer, vec![0x34, 0x12]);

        buffer.clear();
        {
            let mut writer = EndianWriter::new(&mut buffer, ByteOrder::LittleEndian);
            writer.write_u32(0x12345678);
        }
        assert_eq!(buffer, vec![0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_endian_writer_big_endian() {
        let mut buffer = Vec::new();
        {
            let mut writer = EndianWriter::new(&mut buffer, ByteOrder::BigEndian);
            writer.write_u16(0x1234);
        }
        assert_eq!(buffer, vec![0x12, 0x34]);

        buffer.clear();
        {
            let mut writer = EndianWriter::new(&mut buffer, ByteOrder::BigEndian);
            writer.write_u32(0x12345678);
        }
        assert_eq!(buffer, vec![0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn test_string_padding() {
        let mut buffer = Vec::new();
        {
            let mut writer = EndianWriter::new(&mut buffer, ByteOrder::LittleEndian);
            writer.write_string_with_padding("ab");
        }
        assert_eq!(buffer, vec![b'a', b'b', 0, 0]); // 2 bytes padding to align to 4

        buffer.clear();
        {
            let mut writer = EndianWriter::new(&mut buffer, ByteOrder::LittleEndian);
            writer.write_string_with_padding("abcd");
        }
        assert_eq!(buffer, vec![b'a', b'b', b'c', b'd']); // No padding needed
    }

    #[test]
    fn test_endian_reader_little_endian() {
        let data = vec![0x34, 0x12, 0x78, 0x56, 0x34, 0x12];
        let mut reader = EndianReader::new(&data, ByteOrder::LittleEndian);

        assert_eq!(reader.read_u16().unwrap(), 0x1234);
        assert_eq!(reader.read_u32().unwrap(), 0x12345678);
    }

    #[test]
    fn test_endian_reader_big_endian() {
        let data = vec![0x12, 0x34, 0x12, 0x34, 0x56, 0x78];
        let mut reader = EndianReader::new(&data, ByteOrder::BigEndian);

        assert_eq!(reader.read_u16().unwrap(), 0x1234);
        assert_eq!(reader.read_u32().unwrap(), 0x12345678);
    }

    #[test]
    fn test_reader_bounds_checking() {
        let data = vec![0x12];
        let mut reader = EndianReader::new(&data, ByteOrder::LittleEndian);

        assert!(reader.read_u8().is_ok());
        assert!(reader.read_u8().is_err()); // Should fail - no more data

        assert!(reader.read_u16().is_err()); // Should fail - not enough data
    }

    #[test]
    fn test_reader_string_handling() {
        let data = b"hello\0\0\0"; // "hello" with null padding
        let mut reader = EndianReader::new(data, ByteOrder::LittleEndian);

        let result = reader.read_string(8).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_string_reading() {
        let data = vec![b'a', b'b', 0, 0, b'c', b'd', b'e', b'f'];
        let mut reader = EndianReader::new(&data, ByteOrder::LittleEndian);

        assert_eq!(reader.read_string(2).unwrap(), "ab");
        reader.skip_padding(4).unwrap();
        assert_eq!(reader.read_string(4).unwrap(), "cdef");
    }
}
