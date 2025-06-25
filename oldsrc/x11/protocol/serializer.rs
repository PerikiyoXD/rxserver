//! X11 Protocol Message Serializer
//!
//! This module provides functionality for serializing and deserializing
//! structured X11 types and requests into wire format bytes.

use crate::types::Error;
use crate::x11::protocol::{endianness::ByteOrder, types::*, wire::EventMessage};


/// ResponseFormat
///
/// Sometimes packets need to be wrapped in a response format,
/// and there's these variants:
/// - Reply: A standard response to a request
/// - Error: An error response indicating a problem with the request
/// - Raw: A raw response that doesn't fit into the standard reply/error structure, such as a raw event or connection setup response.
#[derive(Debug, Clone)]
pub enum ResponseFormat {
    /// Standard successful response to a request
    Reply,
    /// Error response indicating a problem with the request
    Error,
    /// Raw response that doesn't fit into standard reply/error structure
    Raw,
}

/// Extensions for Vec<u8> to support X11 protocol serialization.
pub trait X11VecExt {
    /// Appends a string with 4-byte padding alignment.
    fn append_padded_string(&mut self, s: &str);

    fn write_padding(&mut self, len: usize);
}

impl X11VecExt for Vec<u8> {
    fn append_padded_string(&mut self, s: &str) {
        self.extend_from_slice(s.as_bytes());
        while self.len() % 4 != 0 {
            self.push(0);
        }
    }

    fn write_padding(&mut self, len: usize) {
        let padding = align_to_4(len) - len;
        if padding > 0 {
            self.extend_from_slice(&vec![0; padding]);
        }
    }
}

/// Aligns a length to the next 4-byte boundary.
pub fn align_to_4(len: usize) -> usize {
    (len + 3) & !3
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
