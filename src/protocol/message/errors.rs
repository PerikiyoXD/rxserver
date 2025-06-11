//! X11 Protocol Errors
//!
//! This module contains all X11 error definitions that can be sent from server to clients
//! when requests fail or invalid operations are attempted.

use crate::protocol::types::X11Error;

/// X11 error response sent when a request fails
#[derive(Debug, Clone)]
pub struct ErrorResponse {
    /// The type of error that occurred
    pub error_code: X11Error,
    /// Sequence number of the failed request
    pub sequence_number: u16,
    /// The invalid resource ID or value that caused the error
    pub bad_value: u32,
    /// Minor opcode if this was an extension request
    pub minor_opcode: u16,
    /// Major opcode of the failed request
    pub major_opcode: u8,
}
