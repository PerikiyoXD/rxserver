//! X11 Wire Format Implementation
//!
//! This module handles the low-level wire format for X11 messages,
//! including byte order handling and message framing.

use crate::x11::protocol::{endianness::ByteOrder, errors::ProtocolError, types::SequenceNumber};

/// Standard X11 message header (first 4 bytes of every message)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageHeader {
    /// Message opcode (request type or response/event type)
    pub opcode: u8,
    /// Request detail or event detail
    pub detail: u8,
    /// Total message length in 4-byte units
    pub length: u16,
}

impl MessageHeader {
    /// Create a new message header
    pub fn new(opcode: u8, detail: u8, length: u16) -> Self {
        Self {
            opcode,
            detail,
            length,
        }
    }

    /// Parse a message header from bytes
    pub fn from_bytes(bytes: &[u8], byte_order: ByteOrder) -> Result<Self, ProtocolError> {
        if bytes.len() < 4 {
            return Err(ProtocolError::MessageTooShort {
                expected: 4,
                actual: bytes.len(),
            });
        }

        let opcode = bytes[0];
        let detail = bytes[1];
        let length = match byte_order {
            ByteOrder::LittleEndian => u16::from_le_bytes([bytes[2], bytes[3]]),
            ByteOrder::BigEndian => u16::from_be_bytes([bytes[2], bytes[3]]),
        };

        Ok(Self {
            opcode,
            detail,
            length,
        })
    }

    /// Serialize header to bytes
    pub fn to_bytes(&self, byte_order: ByteOrder) -> [u8; 4] {
        let length_bytes = match byte_order {
            ByteOrder::LittleEndian => self.length.to_le_bytes(),
            ByteOrder::BigEndian => self.length.to_be_bytes(),
        };

        [self.opcode, self.detail, length_bytes[0], length_bytes[1]]
    }

    /// Get the total message size in bytes
    pub fn message_size(&self) -> usize {
        (self.length as usize) * 4
    }
}

/// Wire format utilities for X11 protocol messages
pub trait WireFormat {
    /// Parse from wire format bytes
    fn from_wire(bytes: &[u8], byte_order: ByteOrder) -> Result<Self, ProtocolError>
    where
        Self: Sized;

    /// Serialize to wire format bytes
    fn to_wire(&self, byte_order: ByteOrder) -> Vec<u8>;

    /// Get the wire format size in bytes
    fn wire_size(&self) -> usize;
}

/// X11 Request message structure
#[derive(Debug, Clone)]
pub struct RequestMessage {
    pub header: MessageHeader,
    pub data: Vec<u8>,
}

impl RequestMessage {
    /// Create a new request message
    pub fn new(opcode: u8, detail: u8, data: Vec<u8>) -> Self {
        let length = ((data.len() + 4 + 3) / 4) as u16; // Round up to 4-byte boundary
        let header = MessageHeader::new(opcode, detail, length);
        Self { header, data }
    }

    /// Parse a complete request message from bytes
    pub fn from_bytes(bytes: &[u8], byte_order: ByteOrder) -> Result<Self, ProtocolError> {
        let header = MessageHeader::from_bytes(bytes, byte_order)?;
        let total_size = header.message_size();

        if bytes.len() < total_size {
            return Err(ProtocolError::MessageTooShort {
                expected: total_size,
                actual: bytes.len(),
            });
        }

        let data = bytes[4..total_size].to_vec();
        Ok(Self { header, data })
    }

    /// Serialize to complete message bytes
    pub fn to_bytes(&self, byte_order: ByteOrder) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.header.to_bytes(byte_order));
        bytes.extend_from_slice(&self.data);

        // Pad to 4-byte boundary
        while bytes.len() % 4 != 0 {
            bytes.push(0);
        }

        bytes
    }
}

/// X11 Response message structure
#[derive(Debug, Clone)]
pub struct ResponseMessage {
    pub reply_type: u8,
    pub detail: u8,
    pub sequence_number: SequenceNumber,
    pub length: u32,
    pub data: Vec<u8>,
}

impl ResponseMessage {
    /// Create a new response message
    pub fn new(reply_type: u8, detail: u8, sequence_number: SequenceNumber, data: Vec<u8>) -> Self {
        let length = (data.len() / 4) as u32; // Length in 4-byte units beyond the initial 32 bytes
        Self {
            reply_type,
            detail,
            sequence_number,
            length,
            data,
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self, byte_order: ByteOrder) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(1); // Reply indicator
        bytes.push(self.detail);

        let seq_bytes = match byte_order {
            ByteOrder::LittleEndian => self.sequence_number.to_le_bytes(),
            ByteOrder::BigEndian => self.sequence_number.to_be_bytes(),
        };
        bytes.extend_from_slice(&seq_bytes);

        let len_bytes = match byte_order {
            ByteOrder::LittleEndian => self.length.to_le_bytes(),
            ByteOrder::BigEndian => self.length.to_be_bytes(),
        };
        bytes.extend_from_slice(&len_bytes);

        // Add data
        bytes.extend_from_slice(&self.data);

        // Ensure minimum 32-byte size and 4-byte alignment
        while bytes.len() < 32 {
            bytes.push(0);
        }
        while bytes.len() % 4 != 0 {
            bytes.push(0);
        }

        bytes
    }
}

/// X11 Event message structure
#[derive(Debug, Clone)]
pub struct EventMessage {
    pub event_type: u8,
    pub detail: u8,
    pub sequence_number: SequenceNumber,
    pub data: [u8; 28], // Events are always 32 bytes total
}

impl EventMessage {
    /// Create a new event message
    pub fn new(
        event_type: u8,
        detail: u8,
        sequence_number: SequenceNumber,
        data: [u8; 28],
    ) -> Self {
        Self {
            event_type,
            detail,
            sequence_number,
            data,
        }
    }

    /// Serialize to bytes (always 32 bytes)
    pub fn to_bytes(&self, byte_order: ByteOrder) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        bytes[0] = self.event_type;
        bytes[1] = self.detail;

        let seq_bytes = match byte_order {
            ByteOrder::LittleEndian => self.sequence_number.to_le_bytes(),
            ByteOrder::BigEndian => self.sequence_number.to_be_bytes(),
        };
        bytes[2..4].copy_from_slice(&seq_bytes);
        bytes[4..32].copy_from_slice(&self.data);

        bytes
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8; 32], byte_order: ByteOrder) -> Self {
        let event_type = bytes[0];
        let detail = bytes[1];
        let sequence_number = match byte_order {
            ByteOrder::LittleEndian => u16::from_le_bytes([bytes[2], bytes[3]]),
            ByteOrder::BigEndian => u16::from_be_bytes([bytes[2], bytes[3]]),
        };

        let mut data = [0u8; 28];
        data.copy_from_slice(&bytes[4..32]);

        Self {
            event_type,
            detail,
            sequence_number,
            data,
        }
    }
}

/// Utility functions for wire format padding and alignment
pub mod padding {
    /// Calculate padding needed to align to 4-byte boundary
    pub fn calculate_padding(size: usize) -> usize {
        (4 - (size % 4)) % 4
    }

    /// Pad a vector to 4-byte boundary
    pub fn pad_to_boundary(mut vec: Vec<u8>) -> Vec<u8> {
        let padding = calculate_padding(vec.len());
        vec.resize(vec.len() + padding, 0);
        vec
    }

    /// Calculate total size including padding
    pub fn padded_size(size: usize) -> usize {
        size + calculate_padding(size)
    }
}
