//! Basic Extension Protocol Handler
//!
//! Handles basic X11 extension queries like QueryExtension and ListExtensions.

use async_trait::async_trait;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use tracing::{debug, warn};

use crate::{
    protocol::{
        wire::X11BufExt, BigRequestsOpcode, ClientId, ExtensionOpcode, Opcode, ProtocolError,
        ProtocolHandler, Request, Response,
    },
    ServerResult,
};

/// Information about a supported extension
#[derive(Clone)]
struct ExtensionInfo {
    name: String,
    major_opcode: u8,
    first_event: u8,
    first_error: u8,
}

/// Basic extension handler for common extension requests
#[derive(Clone)]
pub struct ExtensionHandler {
    supported_extensions: Vec<ExtensionInfo>,
}

impl ExtensionHandler {
    pub fn new() -> Self {
        Self {
            supported_extensions: vec![
                ExtensionInfo {
                    name: "BIG-REQUESTS".to_string(),
                    major_opcode: 132,
                    first_event: 0, // BIG-REQUESTS has no events
                    first_error: 0, // BIG-REQUESTS has no errors
                },
                ExtensionInfo {
                    name: "XC-MISC".to_string(),
                    major_opcode: 133,
                    first_event: 0,
                    first_error: 0,
                },
            ],
        }
    }

    /// Build a QueryExtension response using proper X11 wire format
    fn build_query_extension_response(
        &self,
        present: bool,
        major_opcode: u8,
        first_event: u8,
        first_error: u8,
    ) -> Response {
        let mut buf = BytesMut::new();

        // QueryExtension reply format (32 bytes total):
        // 1 byte: reply type (1)
        // 1 byte: present flag (1 if present, 0 if not)
        // 2 bytes: sequence number (will be filled by protocol layer)
        // 4 bytes: reply length (0 for fixed-size reply)
        // 1 byte: major opcode
        // 1 byte: first event
        // 1 byte: first error
        // 21 bytes: unused padding

        buf.put_u8(1); // Reply type
        buf.put_u8(if present { 1 } else { 0 }); // Present flag
        buf.put_u16_le(0); // Sequence number (placeholder)
        buf.put_u32_le(0); // Reply length (0 for fixed reply)
        buf.put_u8(major_opcode);
        buf.put_u8(first_event);
        buf.put_u8(first_error);

        // Pad to 32 bytes total (we have 11 bytes so far, need 21 more)
        buf.extend_from_slice(&[0u8; 21]);

        Response::new(1, buf.to_vec())
    }

    /// Handle BigReqEnable request specifically
    fn handle_big_req_enable(&self) -> Response {
        let mut buf = BytesMut::new();

        // BigReqEnable reply format (32 bytes total):
        // 1 byte: reply type (1)
        // 1 byte: unused
        // 2 bytes: sequence number (placeholder)
        // 4 bytes: reply length (0)
        // 4 bytes: maximum-request-length (CARD32)
        // 20 bytes: unused padding

        buf.put_u8(1); // Reply type
        buf.put_u8(0); // Unused
        buf.put_u16_le(0); // Sequence number (placeholder)
        buf.put_u32_le(0); // Reply length
        buf.put_u32_le(0x3FFFFFFF); // Maximum request length (~1GB in 4-byte units)
        buf.extend_from_slice(&[0u8; 20]); // Padding

        Response::new(1, buf.to_vec())
    }
    /// Parse QueryExtension request data
    fn parse_query_extension_request(&self, data: &[u8]) -> Option<String> {
        // QueryExtension request format:
        // 1 byte: unused (already consumed from header)
        // 1 byte: unused
        // 2 bytes: name length (n)
        // 2 bytes: unused
        // n bytes: name
        // padding to multiple of 4 bytes

        if data.len() < 4 {
            warn!("QueryExtension request too short");
            return None;
        }

        // Create a Bytes buffer for proper parsing
        let mut buf = Bytes::copy_from_slice(data);

        // Skip the first 4 bytes (2 unused + 2 length + 2 unused)
        // The X11 string format includes the length, so we need to position at the length field
        if buf.remaining() < 4 {
            warn!("QueryExtension request insufficient data for header");
            return None;
        }

        // Skip 2 unused bytes to get to the length field
        buf.advance(2);

        // Use X11BufExt to properly parse the X11 string
        match buf.get_x11_string() {
            Ok(name) => Some(name),
            Err(e) => {
                warn!("Failed to parse extension name: {}", e);
                None
            }
        }
    }

    /// Build ListExtensions response
    fn build_list_extensions_response(&self) -> Response {
        let mut buf = BytesMut::new();

        // ListExtensions reply format:
        // 1 byte: reply type (1)
        // 1 byte: number of extensions
        // 2 bytes: sequence number (placeholder)
        // 4 bytes: reply length (additional data length in 4-byte units)
        // For each extension:
        //   1 byte: name length
        //   N bytes: name
        //   padding to align next extension to byte boundary

        let num_extensions = self.supported_extensions.len() as u8;

        // Calculate total length of extension data
        let mut data_length = 0;
        for ext in &self.supported_extensions {
            data_length += 1 + ext.name.len(); // 1 byte for length + name bytes
        }
        // Round up to 4-byte units for reply length field
        let reply_length = (data_length + 3) / 4;

        buf.put_u8(1); // Reply type
        buf.put_u8(num_extensions);
        buf.put_u16_le(0); // Sequence number (placeholder)
        buf.put_u32_le(reply_length as u32); // Reply length in 4-byte units

        // Add each extension name
        for ext in &self.supported_extensions {
            buf.put_u8(ext.name.len() as u8);
            buf.extend_from_slice(ext.name.as_bytes());
        }

        // Pad to 4-byte boundary
        while buf.len() % 4 != 0 {
            buf.put_u8(0);
        }

        Response::new(1, buf.to_vec())
    }

    /// Handle extension-specific requests
    fn handle_extension_request(
        &self,
        client_id: ClientId,
        opcode: ExtensionOpcode,
    ) -> ServerResult<Response> {
        match opcode {
            ExtensionOpcode::BigRequests(BigRequestsOpcode::Enable) => {
                debug!("Handling BigReqEnable request from client {}", client_id);
                Ok(self.handle_big_req_enable())
            }
            ExtensionOpcode::XcMisc(_) => {
                debug!("XC-MISC requests not yet implemented");
                Err(crate::ServerError::ProtocolError(
                    ProtocolError::UnimplementedOpcode(Opcode::Extension(opcode)),
                ))
            }
            ExtensionOpcode::Unknown(major, minor) => {
                warn!(
                    "Unknown extension request: major={}, minor={}",
                    major, minor
                );
                Err(crate::ServerError::ProtocolError(
                    ProtocolError::UnimplementedOpcode(Opcode::Extension(opcode)),
                ))
            }
        }
    }
}

#[async_trait]
impl ProtocolHandler for ExtensionHandler {
    async fn handle_client(&mut self, _stream: &mut tokio::net::TcpStream) -> ServerResult<()> {
        // Extension handler doesn't handle full client connections
        Ok(())
    }
    async fn handle_request(
        &mut self,
        client_id: ClientId,
        request: Request,
    ) -> ServerResult<Option<Response>> {
        match request.opcode() {
            Opcode::QueryExtension => {
                debug!("Handling QueryExtension request from client {}", client_id);

                if let Some(extension_name) = self.parse_query_extension_request(&request.data) {
                    debug!("Client {} queried extension: {}", client_id, extension_name);

                    // Check if we support this extension
                    if let Some(ext_info) = self
                        .supported_extensions
                        .iter()
                        .find(|ext| ext.name == extension_name)
                    {
                        // Extension present
                        Ok(Some(self.build_query_extension_response(
                            true,
                            ext_info.major_opcode,
                            ext_info.first_event,
                            ext_info.first_error,
                        )))
                    } else {
                        // Extension not present
                        Ok(Some(self.build_query_extension_response(false, 0, 0, 0)))
                    }
                } else {
                    // Failed to parse request
                    Ok(Some(self.build_query_extension_response(false, 0, 0, 0)))
                }
            }
            Opcode::ListExtensions => {
                debug!("Handling ListExtensions request from client {}", client_id);
                Ok(Some(self.build_list_extensions_response()))
            }
            Opcode::Extension(ext_opcode) => self
                .handle_extension_request(client_id, ext_opcode)
                .map(Some),
            _ => Err(crate::ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
        }
    }

    fn supported_opcodes(&self) -> &[Opcode] {
        &[
            Opcode::QueryExtension,
            Opcode::ListExtensions,
            Opcode::Extension(ExtensionOpcode::BigRequests(BigRequestsOpcode::Enable)),
        ]
    }
}
