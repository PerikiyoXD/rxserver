//! X11 Protocol Serialization
//!
//! This module handles the conversion of X11 protocol messages to and from wire format.
//! It provides a centralized location for all serialization logic, ensuring consistency
//! and maintainability.

pub mod events;
pub mod replies;
pub mod errors;
pub mod wire;

use crate::protocol::message::Response;
use bytes::BytesMut;

/// Serializer for X11 protocol responses
pub struct ResponseSerializer;

impl ResponseSerializer {
    /// Serialize a response to wire format bytes
    pub fn serialize(response: &Response, sequence: u16) -> Vec<u8> {
        let mut buf = BytesMut::new();
        
        match response {
            Response::Reply(reply) => {
                replies::serialize_reply(reply, sequence, &mut buf);
            }
            Response::Event(event) => {
                events::serialize_event(event, &mut buf);
            }
            Response::Error(error) => {
                errors::serialize_error(error, &mut buf);
            }
        }

        buf.to_vec()
    }
}
