//! Wire protocol serialization and deserialization
//!
//! This module handles the low-level binary format of the X11 protocol,
//! providing safe serialization and deserialization of messages.

use crate::ServerResult;
use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Trait for types that can be serialized to the X11 wire format
pub trait Serialize {
    /// Serialize this type to bytes
    fn serialize(&self, buf: &mut BytesMut) -> ServerResult<()>;

    /// Get the serialized length in bytes
    fn serialized_length(&self) -> usize;
}

/// Trait for types that can be deserialized from the X11 wire format
pub trait Deserialize: Sized {
    /// Deserialize from bytes
    fn deserialize(buf: &mut bytes::Bytes) -> ServerResult<Self>;
}

/// Extension to BytesMut for X11-specific operations
pub trait X11BufMutExt {
    /// Put an X11 string (length-prefixed)
    fn put_x11_string(&mut self, s: &str);

    /// Put padding bytes to align to 4-byte boundary
    fn put_padding(&mut self, len: usize);
}

impl X11BufMutExt for BytesMut {
    fn put_x11_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.put_u16_le(bytes.len() as u16);
        self.put_slice(bytes);

        // Add padding to 4-byte boundary
        let padding = (4 - (bytes.len() % 4)) % 4;
        for _ in 0..padding {
            self.put_u8(0);
        }
    }

    fn put_padding(&mut self, len: usize) {
        let padding = (4 - (len % 4)) % 4;
        for _ in 0..padding {
            self.put_u8(0);
        }
    }
}

/// Extension to Bytes for X11-specific operations
pub trait X11BufExt {
    /// Get an X11 string (length-prefixed)
    fn get_x11_string(&mut self) -> ServerResult<String>;

    /// Skip padding bytes to align to 4-byte boundary
    fn skip_padding(&mut self, data_len: usize) -> ServerResult<()>;
}

impl X11BufExt for bytes::Bytes {
    fn get_x11_string(&mut self) -> ServerResult<String> {
        if self.remaining() < 2 {
            return Err(crate::ServerError::ProtocolError(
                crate::protocol::ProtocolError::InvalidMessage(
                    "Not enough data for string length".to_string(),
                ),
            ));
        }

        let len = self.get_u16_le() as usize;
        if self.remaining() < len {
            return Err(crate::ServerError::ProtocolError(
                crate::protocol::ProtocolError::InvalidMessage(
                    "Not enough data for string content".to_string(),
                ),
            ));
        }

        let string_bytes = self.split_to(len);
        let s = String::from_utf8(string_bytes.to_vec()).map_err(|e| {
            crate::ServerError::ProtocolError(crate::protocol::ProtocolError::InvalidMessage(
                format!("Invalid UTF-8: {}", e),
            ))
        })?;

        // Skip padding
        let padding = (4 - (len % 4)) % 4;
        if self.remaining() < padding {
            return Err(crate::ServerError::ProtocolError(
                crate::protocol::ProtocolError::InvalidMessage(
                    "Not enough data for string padding".to_string(),
                ),
            ));
        }
        self.advance(padding);

        Ok(s)
    }

    fn skip_padding(&mut self, data_len: usize) -> ServerResult<()> {
        let padding = (4 - (data_len % 4)) % 4;
        if self.remaining() < padding {
            return Err(crate::ServerError::ProtocolError(
                crate::protocol::ProtocolError::InvalidMessage(
                    "Not enough data for padding".to_string(),
                ),
            ));
        }
        self.advance(padding);
        Ok(())
    }
}

/// Utility functions for X11 wire format operations
pub mod utils {
    /// Calculate padding needed to align to 4-byte boundary
    pub fn padding_for_length(len: usize) -> usize {
        (4 - (len % 4)) % 4
    }

    /// Calculate total length including padding
    pub fn padded_length(len: usize) -> usize {
        len + padding_for_length(len)
    }

    /// Validate that a buffer has enough data for a message
    pub fn validate_buffer_length(buf: &[u8], expected: usize) -> crate::ServerResult<()> {
        if buf.len() < expected {
            return Err(crate::ServerError::ProtocolError(
                crate::protocol::ProtocolError::InvalidMessage(format!(
                    "Buffer too short: expected {}, got {}",
                    expected,
                    buf.len()
                )),
            ));
        }
        Ok(())
    }
}

/// Parse an X11 request from a TCP stream
pub async fn parse_request(
    stream: &mut tokio::net::TcpStream,
    sequence_counter: &mut u16,
) -> ServerResult<Option<crate::protocol::Request>> {
    use crate::protocol::{Opcode, Request};
    use tracing::debug;

    // Read the 4-byte request header
    let mut header = [0u8; 4];
    match stream.read_exact(&mut header).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            // Client disconnected cleanly
            debug!("Client disconnected during header read");
            return Ok(None);
        }
        Err(e) => {
            return Err(crate::ServerError::NetworkError(format!(
                "Failed to read request header: {}",
                e
            )));
        }
    }

    // Parse header
    let major_opcode = header[0];
    let minor_opcode_or_data = header[1]; // This is minor opcode for extensions, or data for core
    let length = u16::from_le_bytes([header[2], header[3]]);

    debug!(
        "Parsing request: opcode={}, minor/data={}, length={}",
        major_opcode, minor_opcode_or_data, length
    );

    // Validate length (must be at least 1 for the header itself)
    if length == 0 {
        return Err(crate::ServerError::ProtocolError(
            crate::protocol::ProtocolError::InvalidMessage(
                "Request length cannot be zero".to_string(),
            ),
        ));
    }

    // Determine if this is a core or extension opcode
    let opcode = if major_opcode <= 127 {
        // Core protocol opcode
        Opcode::from_u8(major_opcode)
    } else {
        // Extension opcode - use major and minor
        Opcode::from_extension(major_opcode, minor_opcode_or_data)
    };

    // Calculate total request size and remaining data to read
    let total_request_bytes = length as usize * 4;
    let remaining_bytes = total_request_bytes.saturating_sub(4);

    debug!(
        "Request parsing: total_bytes={}, remaining_bytes={}",
        total_request_bytes, remaining_bytes
    );

    // Prepare data including the header for proper request parsing
    // For X11 requests, parsers expect the full request including header
    let mut request_data = Vec::with_capacity(total_request_bytes);

    // Include the complete header
    request_data.extend_from_slice(&header);

    // Read any remaining bytes after the header
    if remaining_bytes > 0 {
        let mut remaining_data = vec![0u8; remaining_bytes];
        match stream.read_exact(&mut remaining_data).await {
            Ok(_) => {
                request_data.extend_from_slice(&remaining_data);
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                debug!("Client disconnected during data read");
                return Ok(None);
            }
            Err(e) => {
                return Err(crate::ServerError::NetworkError(format!(
                    "Failed to read request data: {}",
                    e
                )));
            }
        }
    }

    debug!("Successfully read {} bytes for request", request_data.len());

    // Increment sequence number for this request
    *sequence_counter = sequence_counter.wrapping_add(1);

    // Create Request with data that includes the header for proper parsing
    Ok(Some(Request::new_with_header(
        opcode,
        *sequence_counter,
        request_data,
    )))
}

/// Parse an X11 request from a buffer (for testing or buffered reading)
pub fn parse_request_from_buffer(
    buf: &[u8],
    sequence_counter: &mut u16,
) -> ServerResult<Option<crate::protocol::Request>> {
    use crate::protocol::{Opcode, Request};
    use tracing::debug;

    if buf.len() < 4 {
        return Ok(None); // Not enough data for header
    }

    // Parse header
    let major_opcode = buf[0];
    let minor_opcode_or_data = buf[1];
    let length = u16::from_le_bytes([buf[2], buf[3]]);

    debug!(
        "Parsing request from buffer: opcode={}, minor/data={}, length={}",
        major_opcode, minor_opcode_or_data, length
    );

    // Validate length
    if length == 0 {
        return Err(crate::ServerError::ProtocolError(
            crate::protocol::ProtocolError::InvalidMessage(
                "Request length cannot be zero".to_string(),
            ),
        ));
    }

    let total_length = length as usize * 4;
    if buf.len() < total_length {
        debug!(
            "Buffer too short: need {} bytes, have {}",
            total_length,
            buf.len()
        );
        return Ok(None); // Not enough data for complete request
    }

    // Determine opcode type
    let opcode = if major_opcode <= 127 {
        Opcode::from_u8(major_opcode)
    } else {
        Opcode::from_extension(major_opcode, minor_opcode_or_data)
    };

    // Extract the complete request data (including header)
    let request_data = buf[0..total_length].to_vec();

    debug!(
        "Successfully parsed {} bytes from buffer for request",
        request_data.len()
    );

    // Increment sequence number for this request
    *sequence_counter = sequence_counter.wrapping_add(1);

    Ok(Some(Request::new_with_header(
        opcode,
        *sequence_counter,
        request_data,
    )))
}

/// Write an X11 response to a TCP stream
pub async fn write_response(
    stream: &mut tokio::net::TcpStream,
    response: &crate::protocol::Response,
    sequence_number: u16,
) -> ServerResult<()> {
    let mut buf = BytesMut::new();

    // Build complete response with sequence number inserted
    buf.put_u8(response.response_type);

    // For most responses, insert sequence number at bytes 2-3
    if response.data.len() >= 3 {
        buf.put_slice(&response.data[0..1]); // First data byte
        buf.put_u16_le(sequence_number); // Sequence number
        buf.put_slice(&response.data[3..]); // Rest of data
    } else {
        // Handle short responses
        buf.extend_from_slice(&response.data);
        // Pad if necessary
        while buf.len() < 32 {
            buf.put_u8(0);
        }
    }

    // Write complete response
    stream.write_all(&buf).await.map_err(|e| {
        crate::ServerError::NetworkError(format!("Failed to write response: {}", e))
    })?;

    stream.flush().await.map_err(|e| {
        crate::ServerError::NetworkError(format!("Failed to flush response: {}", e))
    })?;

    Ok(())
}

/// Write an X11 response to a TCP stream (legacy version without sequence number)
pub async fn write_response_legacy(
    stream: &mut tokio::net::TcpStream,
    response: &crate::protocol::Response,
) -> ServerResult<()> {
    // Write response type
    stream.write_u8(response.response_type).await.map_err(|e| {
        crate::ServerError::NetworkError(format!("Failed to write response type: {}", e))
    })?;

    // Write response data
    stream.write_all(&response.data).await.map_err(|e| {
        crate::ServerError::NetworkError(format!("Failed to write response data: {}", e))
    })?;

    stream.flush().await.map_err(|e| {
        crate::ServerError::NetworkError(format!("Failed to flush response: {}", e))
    })?;
    Ok(())
}
