pub mod serializers;
pub mod types;

pub use serializers::*;
pub use types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub kind: ResponseKind,
    pub sequence_number: SequenceNumber,
    pub byte_order: ByteOrder,
}

impl Default for Response {
    fn default() -> Self {
        Self {
            kind: ResponseKind::Reply,
            sequence_number: 0,
            byte_order: ByteOrder::LittleEndian,
        }
    }
}
