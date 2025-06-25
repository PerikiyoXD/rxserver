//! X11 Request Handling System
//!
//! This module implements the request processing pipeline for X11 protocol requests.
//! It follows CLEAN architecture principles with clear separation between request
//! dispatching, validation, and response generation.

use crate::{Request, RequestKind, Response, ResponseKind, types::Result};

pub mod dispatcher;
pub mod handlers;

pub use dispatcher::*;

/// Request validation utilities
pub mod validation {
    /// Validate basic request structure
    pub fn validate_request_basic(data: &[u8]) -> crate::types::Result<()> {
        if data.len() < 4 {
            return Err(crate::types::Error::Protocol(
                crate::x11::protocol::errors::ProtocolError::MessageTooShort {
                    expected: 4,
                    actual: data.len(),
                },
            ));
        }
        Ok(())
    }
}

/// Response generation utilities  
pub mod response {

    /// Generate basic success response
    pub fn success_response(sequence: u16) -> Vec<u8> {
        vec![1, 0, (sequence & 0xFF) as u8, (sequence >> 8) as u8]
    }

    /// Generate error response
    pub fn error_response(error_code: u8, sequence: u16) -> Vec<u8> {
        vec![
            0,
            error_code,
            (sequence & 0xFF) as u8,
            (sequence >> 8) as u8,
        ]
    }
}
