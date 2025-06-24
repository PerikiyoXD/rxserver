//! Network protocol handling module
//!
//! Handles network-level protocol negotiation, framing, and message processing.

pub mod compression;
pub mod encryption;
pub mod framing;
pub mod negotiation;

// Re-export commonly used items
pub use compression::{CompressionError, CompressionManager, CompressionType};
pub use encryption::{EncryptionError, EncryptionManager, EncryptionType};
pub use framing::{Frame, FrameProcessor, FrameType, FramingError};
pub use negotiation::{NegotiationError, NegotiationResult, ProtocolNegotiator};
