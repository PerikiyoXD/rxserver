use std::convert::TryInto;

// SPDX-License-Identifier: Apache-2.0

//! X11 Request Deserializers
//!
//! This module contains the deserializers for X11 protocol requests.

// -----------------------------------------------------------------------------
// Request Deserializer Trait
// -----------------------------------------------------------------------------
pub trait X11RequestDeserializer {
    /// Deserialize a request from a byte vector
    fn deserialize(&self, data: &[u8]) -> Result<Request, Error>;
}

// -----------------------------------------------------------------------------
// Connection Setup Deserializer
// -----------------------------------------------------------------------------
pub struct ConnectionSetupRequest;

impl X11RequestDeserializer for ConnectionSetupRequest {
    fn deserialize(&self, data: &[u8]) -> Result<Request, Error> {

        if data.len() < 12 {
            return Err(Error::InvalidLength);
        }

        let byte_order = data[0];
        let protocol_major_version = u16::from_ne_bytes([data[2], data[3]]);
        let protocol_minor_version = u16::from_ne_bytes([data[4], data[5]]);
        let n = u16::from_ne_bytes([data[6], data[7]]) as usize;
        let d = u16::from_ne_bytes([data[8], data[9]]) as usize;

        // Calculate offsets
        let mut offset = 12;
        if data.len() < offset + n {
            return Err(Error::InvalidLength);
        }
        let auth_protocol_name = &data[offset..offset + n];
        offset += n;
        let p = (4 - (n % 4)) % 4;
        offset += p;

        if data.len() < offset + d {
            return Err(Error::InvalidLength);
        }
        let auth_protocol_data = &data[offset..offset + d];
        offset += d;
        let q = (4 - (d % 4)) % 4;
        offset += q;

        // Construct the Request (assuming a Request::ConnectionSetup variant exists)
        Ok(Request::ConnectionSetup {
            byte_order,
            protocol_major_version,
            protocol_minor_version,
            auth_protocol_name: auth_protocol_name.to_vec(),
            auth_protocol_data: auth_protocol_data.to_vec(),
        })
    }
}
