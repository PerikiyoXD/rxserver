//! Byte Order Handling for X11 Protocol
//!
//! This module provides utilities for handling different byte orders
//! in X11 protocol messages.

/// Byte order for protocol messages
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    /// Little-endian byte order (LSB first)
    LittleEndian = b'l',
    /// Big-endian byte order (MSB first)
    BigEndian = b'B',
}

impl ByteOrder {
    /// Byte order marker for little-endian
    pub const LITTLE_ENDIAN_MARKER: u8 = b'l';
    /// Byte order marker for big-endian  
    pub const BIG_ENDIAN_MARKER: u8 = b'B';

    /// Create from byte order marker
    pub fn from_marker(marker: u8) -> Option<Self> {
        match marker {
            Self::LITTLE_ENDIAN_MARKER => Some(ByteOrder::LittleEndian),
            Self::BIG_ENDIAN_MARKER => Some(ByteOrder::BigEndian),
            _ => None,
        }
    }

    /// Get the byte order marker
    pub fn marker(self) -> u8 {
        match self {
            ByteOrder::LittleEndian => Self::LITTLE_ENDIAN_MARKER,
            ByteOrder::BigEndian => Self::BIG_ENDIAN_MARKER,
        }
    }

    /// Get the native byte order of this system
    pub fn native() -> Self {
        if cfg!(target_endian = "little") {
            ByteOrder::LittleEndian
        } else {
            ByteOrder::BigEndian
        }
    }

    /// Check if this byte order matches the native byte order
    pub fn is_native(self) -> bool {
        self == Self::native()
    }
}

/// Trait for converting between byte orders
pub trait ByteOrderConversion {
    /// Convert from the given byte order to native byte order
    fn from_byte_order(self, order: ByteOrder) -> Self;
    /// Convert from native byte order to the given byte order
    fn to_byte_order(self, order: ByteOrder) -> Self;
}

impl ByteOrderConversion for u16 {
    fn from_byte_order(self, order: ByteOrder) -> Self {
        match order {
            ByteOrder::LittleEndian => u16::from_le(self),
            ByteOrder::BigEndian => u16::from_be(self),
        }
    }

    fn to_byte_order(self, order: ByteOrder) -> Self {
        match order {
            ByteOrder::LittleEndian => self.to_le(),
            ByteOrder::BigEndian => self.to_be(),
        }
    }
}

impl ByteOrderConversion for u32 {
    fn from_byte_order(self, order: ByteOrder) -> Self {
        match order {
            ByteOrder::LittleEndian => u32::from_le(self),
            ByteOrder::BigEndian => u32::from_be(self),
        }
    }

    fn to_byte_order(self, order: ByteOrder) -> Self {
        match order {
            ByteOrder::LittleEndian => self.to_le(),
            ByteOrder::BigEndian => self.to_be(),
        }
    }
}

impl ByteOrderConversion for i16 {
    fn from_byte_order(self, order: ByteOrder) -> Self {
        match order {
            ByteOrder::LittleEndian => i16::from_le(self),
            ByteOrder::BigEndian => i16::from_be(self),
        }
    }

    fn to_byte_order(self, order: ByteOrder) -> Self {
        match order {
            ByteOrder::LittleEndian => self.to_le(),
            ByteOrder::BigEndian => self.to_be(),
        }
    }
}

impl ByteOrderConversion for i32 {
    fn from_byte_order(self, order: ByteOrder) -> Self {
        match order {
            ByteOrder::LittleEndian => i32::from_le(self),
            ByteOrder::BigEndian => i32::from_be(self),
        }
    }

    fn to_byte_order(self, order: ByteOrder) -> Self {
        match order {
            ByteOrder::LittleEndian => self.to_le(),
            ByteOrder::BigEndian => self.to_be(),
        }
    }
}

/// Utility functions for reading multibyte values with specific byte order
pub mod read {
    use super::ByteOrder;

    /// Read an u16 from bytes with the given byte order
    pub fn u16_from_bytes(bytes: &[u8], order: ByteOrder) -> Option<u16> {
        if bytes.len() < 2 {
            return None;
        }

        let value = match order {
            ByteOrder::LittleEndian => u16::from_le_bytes([bytes[0], bytes[1]]),
            ByteOrder::BigEndian => u16::from_be_bytes([bytes[0], bytes[1]]),
        };

        Some(value)
    }

    /// Read an u32 from bytes with the given byte order
    pub fn u32_from_bytes(bytes: &[u8], order: ByteOrder) -> Option<u32> {
        if bytes.len() < 4 {
            return None;
        }

        let value = match order {
            ByteOrder::LittleEndian => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            ByteOrder::BigEndian => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        };

        Some(value)
    }

    /// Read an i16 from bytes with the given byte order
    pub fn i16_from_bytes(bytes: &[u8], order: ByteOrder) -> Option<i16> {
        if bytes.len() < 2 {
            return None;
        }

        let value = match order {
            ByteOrder::LittleEndian => i16::from_le_bytes([bytes[0], bytes[1]]),
            ByteOrder::BigEndian => i16::from_be_bytes([bytes[0], bytes[1]]),
        };

        Some(value)
    }

    /// Read an i32 from bytes with the given byte order
    pub fn i32_from_bytes(bytes: &[u8], order: ByteOrder) -> Option<i32> {
        if bytes.len() < 4 {
            return None;
        }

        let value = match order {
            ByteOrder::LittleEndian => i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            ByteOrder::BigEndian => i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        };

        Some(value)
    }
}

/// Utility functions for writing multi-byte values with specific byte order
pub mod write {
    use super::ByteOrder;

    /// Write a u16 to bytes with the given byte order
    pub fn u16_to_bytes(value: u16, order: ByteOrder) -> [u8; 2] {
        match order {
            ByteOrder::LittleEndian => value.to_le_bytes(),
            ByteOrder::BigEndian => value.to_be_bytes(),
        }
    }

    /// Write a u32 to bytes with the given byte order
    pub fn u32_to_bytes(value: u32, order: ByteOrder) -> [u8; 4] {
        match order {
            ByteOrder::LittleEndian => value.to_le_bytes(),
            ByteOrder::BigEndian => value.to_be_bytes(),
        }
    }

    /// Write an i16 to bytes with the given byte order
    pub fn i16_to_bytes(value: i16, order: ByteOrder) -> [u8; 2] {
        match order {
            ByteOrder::LittleEndian => value.to_le_bytes(),
            ByteOrder::BigEndian => value.to_be_bytes(),
        }
    }

    /// Write an i32 to bytes with the given byte order
    pub fn i32_to_bytes(value: i32, order: ByteOrder) -> [u8; 4] {
        match order {
            ByteOrder::LittleEndian => value.to_le_bytes(),
            ByteOrder::BigEndian => value.to_be_bytes(),
        }
    }
}
