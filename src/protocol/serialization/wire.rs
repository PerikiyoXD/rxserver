//! Wire protocol serialization and deserialization
//!
//! This module handles the low-level binary format of the X11 protocol,
//! providing safe serialization and deserialization of messages.

use crate::Result;
use bytes::{BufMut, BytesMut, Buf};

/// Trait for types that can be serialized to the X11 wire format
pub trait Serialize {
    /// Serialize this type to bytes
    fn serialize(&self, buf: &mut BytesMut) -> Result<()>;
    
    /// Get the serialized length in bytes
    fn serialized_length(&self) -> usize;
}

/// Trait for types that can be deserialized from the X11 wire format
pub trait Deserialize: Sized {
    /// Deserialize from bytes
    fn deserialize(buf: &mut bytes::Bytes) -> Result<Self>;
}

/// Extension to BytesMut for X11-specific operations
pub trait X11BufMutExt {
    /// Put an X11 string (length-prefixed)
    fn put_x11_string(&mut self, s: &str);
    
    /// Put padding bytes to align to 4-byte boundary
    fn put_padding(&mut self, len: usize);
}

impl X11BufMutExt for BytesMut {
    fn put_x11_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.put_u16(bytes.len() as u16);
        self.put_slice(bytes);
        
        // Add padding to 4-byte boundary
        let padding = (4 - (bytes.len() % 4)) % 4;
        for _ in 0..padding {
            self.put_u8(0);
        }
    }
    
    fn put_padding(&mut self, len: usize) {
        let padding = (4 - (len % 4)) % 4;
        for _ in 0..padding {
            self.put_u8(0);
        }
    }
}

/// Extension to Bytes for X11-specific operations
pub trait X11BufExt {
    /// Get an X11 string (length-prefixed)
    fn get_x11_string(&mut self) -> Result<String>;
    
    /// Skip padding bytes to align to 4-byte boundary
    fn skip_padding(&mut self, data_len: usize) -> Result<()>;
}

impl X11BufExt for bytes::Bytes {
    fn get_x11_string(&mut self) -> Result<String> {
        if self.remaining() < 2 {
            return Err(crate::Error::Protocol("Not enough data for string length".to_string()));
        }
        
        let len = self.get_u16() as usize;
        if self.remaining() < len {
            return Err(crate::Error::Protocol("Not enough data for string content".to_string()));
        }
        
        let string_bytes = self.split_to(len);
        let s = String::from_utf8(string_bytes.to_vec())
            .map_err(|e| crate::Error::Protocol(format!("Invalid UTF-8: {}", e)))?;
        
        // Skip padding
        let padding = (4 - (len % 4)) % 4;
        if self.remaining() < padding {
            return Err(crate::Error::Protocol("Not enough data for string padding".to_string()));
        }
        self.advance(padding);
        
        Ok(s)
    }
    
    fn skip_padding(&mut self, data_len: usize) -> Result<()> {
        let padding = (4 - (data_len % 4)) % 4;
        if self.remaining() < padding {
            return Err(crate::Error::Protocol("Not enough data for padding".to_string()));
        }
        self.advance(padding);
        Ok(())
    }
}

/// Utility functions for X11 wire format operations
pub mod utils {
    /// Calculate padding needed to align to 4-byte boundary
    pub fn padding_for_length(len: usize) -> usize {
        (4 - (len % 4)) % 4
    }
    
    /// Calculate total length including padding
    pub fn padded_length(len: usize) -> usize {
        len + padding_for_length(len)
    }
    
    /// Validate that a buffer has enough data for a message
    pub fn validate_buffer_length(buf: &[u8], expected: usize) -> crate::Result<()> {
        if buf.len() < expected {
            return Err(crate::Error::Protocol(format!(
                "Buffer too short: expected {}, got {}",
                expected, buf.len()
            )));
        }
        Ok(())
    }
}
