//! Property Protocol Handler
//!
//! Handles X11 property-related requests including GetProperty, ChangeProperty, and DeleteProperty.

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, warn};

use crate::{
    plugins::WindowPlugin,
    protocol::{ClientId, Opcode, ProtocolHandler, Request, Response},
    ServerError, ServerResult,
};

/// Protocol handler specialized for property operations
pub struct PropertyProtocolHandler {
    window_plugin: Arc<WindowPlugin>,
}

impl PropertyProtocolHandler {
    pub fn new(window_plugin: Arc<WindowPlugin>) -> Self {
        Self { window_plugin }
    }

    /// Parse GetProperty request and extract parameters
    fn parse_get_property_request(data: &[u8]) -> ServerResult<(bool, u32, u32, u32, u32, u32)> {
        if data.len() < 24 {
            return Err(ServerError::ProtocolError(
                crate::protocol::ProtocolError::InvalidMessage(
                    "GetProperty request too short".to_string(),
                ),
            ));
        }

        // Skip first 4 bytes (header: opcode, delete, length)
        let delete = data[1] != 0;
        let window = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let property = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let property_type = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let long_offset = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let long_length = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);

        Ok((
            delete,
            window,
            property,
            property_type,
            long_offset,
            long_length,
        ))
    }

    /// Create GetProperty response
    fn create_get_property_response(
        sequence: u16,
        type_atom: u32,
        format: u8,
        bytes_after: u32,
        value_data: Vec<u8>,
    ) -> ServerResult<Response> {
        let mut response_data = Vec::new();

        // Response header
        response_data.push(1); // reply type
        response_data.push(format); // format
        response_data.extend_from_slice(&sequence.to_le_bytes()); // sequence number

        // Calculate reply length (total response length in 4-byte units, minus the first 32 bytes)
        let value_len = value_data.len();
        let padded_value_len = (value_len + 3) & !3; // Round up to 4-byte boundary
        let total_length = 32 + padded_value_len; // 32 bytes header + padded value
        let reply_length = ((total_length - 32) / 4) as u32; // Length of data after first 32 bytes
        response_data.extend_from_slice(&reply_length.to_le_bytes());

        // Property information
        response_data.extend_from_slice(&type_atom.to_le_bytes()); // type
        response_data.extend_from_slice(&bytes_after.to_le_bytes()); // bytes-after

        // Length of value in format units
        let value_length_units = match format {
            0 => 0,
            8 => value_len as u32,
            16 => (value_len / 2) as u32,
            32 => (value_len / 4) as u32,
            _ => {
                return Err(ServerError::ProtocolError(
                    crate::protocol::ProtocolError::InvalidMessage(format!(
                        "Invalid property format: {}",
                        format
                    )),
                ))
            }
        };
        response_data.extend_from_slice(&value_length_units.to_le_bytes());

        // Unused padding (12 bytes)
        response_data.extend_from_slice(&[0u8; 12]);

        // Value data
        response_data.extend_from_slice(&value_data);

        // Pad to 4-byte boundary
        while response_data.len() % 4 != 0 {
            response_data.push(0);
        }

        Ok(Response::new(1, response_data))
    }
}

#[async_trait]
impl ProtocolHandler for PropertyProtocolHandler {
    async fn handle_request(
        &mut self,
        client_id: ClientId,
        request: Request,
    ) -> ServerResult<Option<Response>> {
        debug!(
            "PropertyProtocolHandler handling request from client {}: {:?}",
            client_id,
            request.opcode()
        );

        match request.opcode() {
            Opcode::GetProperty => {
                let (delete, window, property_atom, type_filter, long_offset, long_length) =
                    Self::parse_get_property_request(&request.data)?;
                debug!(
                    "GetProperty: window={:x}, property={}, type={}, offset={}, length={}, delete={}",
                    window, property_atom, type_filter, long_offset, long_length, delete
                );

                let properties = self.window_plugin.get_properties();
                let properties = properties.lock().map_err(|_| {
                    ServerError::ProtocolError(crate::protocol::ProtocolError::InvalidMessage(
                        "Failed to lock window properties".to_string(),
                    ))
                })?;

                if let Some((property, bytes_after)) = properties.get_property_full(
                    window,
                    property_atom,
                    type_filter,
                    long_offset,
                    long_length,
                ) {
                    // Property found and matches type filter
                    let response = Self::create_get_property_response(
                        request.sequence_number,
                        property.type_atom,
                        property.format,
                        bytes_after,
                        property.data,
                    )?;

                    // Delete property if requested and no bytes remaining
                    if delete && bytes_after == 0 {
                        drop(properties); // Release lock
                        let properties_mut = self.window_plugin.get_properties();
                        let mut properties_mut = properties_mut.lock().map_err(|_| {
                            ServerError::ProtocolError(
                                crate::protocol::ProtocolError::InvalidMessage(
                                    "Failed to lock window properties for deletion".to_string(),
                                ),
                            )
                        })?;
                        properties_mut.delete_property_full(window, property_atom)?;
                        debug!("Property deleted as requested");
                    }

                    Ok(Some(response))
                } else {
                    // Check if property exists but type doesn't match
                    if let Some((property, _)) = properties.get_property_full(
                        window,
                        property_atom,
                        0, // AnyPropertyType
                        0,
                        0,
                    ) {
                        // Property exists but type doesn't match
                        let total_bytes = property.data.len() as u32;
                        let response = Self::create_get_property_response(
                            request.sequence_number,
                            property.type_atom,
                            property.format,
                            total_bytes,
                            Vec::new(), // Empty value
                        )?;
                        Ok(Some(response))
                    } else {
                        // Property doesn't exist
                        let response = Self::create_get_property_response(
                            request.sequence_number,
                            0,          // None
                            0,          // format 0
                            0,          // no bytes after
                            Vec::new(), // empty value
                        )?;
                        Ok(Some(response))
                    }
                }
            }
            Opcode::ChangeProperty => {
                // TODO: Implement ChangeProperty
                warn!("ChangeProperty not yet implemented");
                Ok(Some(Response::new(1, vec![0; 32])))
            }
            Opcode::DeleteProperty => {
                // TODO: Implement DeleteProperty
                warn!("DeleteProperty not yet implemented");
                Ok(Some(Response::new(1, vec![0; 32])))
            }
            _ => Err(ServerError::ProtocolError(
                crate::protocol::ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
        }
    }

    fn supported_opcodes(&self) -> &[Opcode] {
        &[
            Opcode::GetProperty,
            Opcode::ChangeProperty,
            Opcode::DeleteProperty,
        ]
    }

    async fn initialize(&mut self) -> ServerResult<()> {
        debug!("PropertyProtocolHandler initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> ServerResult<()> {
        debug!("PropertyProtocolHandler shutdown");
        Ok(())
    }
}
