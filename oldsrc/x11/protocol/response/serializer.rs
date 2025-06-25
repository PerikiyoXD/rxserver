// SPDX-License-Identifier: Apache-2.0

//! X11 Response Serializers

use crate::{
    ProtocolError::Serialization,
    Response,
    types::{Error::Protocol, Result},
    x11::protocol::{
        endianness::X11WriteExt,
        responses::{
            ConnectionSetupAcceptedPixmapFormat, ConnectionSetupAcceptedResponse,
            ConnectionSetupAcceptedScreen, ConnectionSetupAcceptedScreenDepth,
            ConnectionSetupAcceptedScreenDepthVisual, ConnectionSetupAuthRequiredResponse,
            ConnectionSetupRefusedResponse, GetGeometry, InternAtom,
        },
        serializer::align_to_4,
    },
};

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

const X11_REPLY_SIZE: usize = 32;
const AUTH_REQUIRED: u8 = 2;
const REPLY: u8 = 1;

// -----------------------------------------------------------------------------
// Response Types
// -----------------------------------------------------------------------------

/// All possible connection setup responses
#[derive(Debug)]
pub enum ConnectionSetupResponse {
    Accepted(ConnectionSetupAcceptedResponse),
    Refused(ConnectionSetupRefusedResponse),
    AuthRequired(ConnectionSetupAuthRequiredResponse),
}

// -----------------------------------------------------------------------------
// Response Serializer Trait
// -----------------------------------------------------------------------------

pub trait ResponseSerializer {
    fn serialize(&self, response: &Response) -> Result<Option<Vec<u8>>>;
    fn serialized_size(&self) -> usize;
}

// -----------------------------------------------------------------------------
// Helper Utilities
// -----------------------------------------------------------------------------

/// Optimized reply packet wrapper with proper memory management
pub struct ReplySerializationWrapper;

impl ReplySerializationWrapper {
    /// Wraps response data in a standard X11 reply format with proper padding.
    ///
    /// # Arguments
    /// * `body` - The response body data
    /// * `sequence_number` - Request sequence number for matching
    /// * `maybe_byte` - Optional byte field (used by specific response types)
    ///
    /// # Returns
    /// A properly formatted 32-byte X11 reply packet
    #[inline]
    pub fn wrap(body: Vec<u8>, sequence_number: u16, maybe_byte: Option<u8>) -> Vec<u8> {
        let mut wrapped = Vec::with_capacity(X11_REPLY_SIZE);

        // Write reply header
        wrapped.push(status_codes::REPLY);
        wrapped.push(maybe_byte.unwrap_or(0));
        wrapped.extend_from_slice(&sequence_number.to_le_bytes());

        // Append body and pad to standard size
        wrapped.extend_from_slice(&body);
        wrapped.resize(X11_REPLY_SIZE, 0);

        wrapped
    }

    /// Creates a minimal reply with just header information
    #[inline]
    pub fn create_minimal_reply(sequence_number: u16, maybe_byte: Option<u8>) -> Vec<u8> {
        Self::wrap(Vec::new(), sequence_number, maybe_byte)
    }
}

/// Efficient buffer writer with error handling
struct SerializationBuffer {
    data: Vec<u8>,
}

impl SerializationBuffer {
    /// Creates a new buffer with specified initial capacity
    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Writes bytes with error handling
    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.data.extend_from_slice(bytes);
        Ok(())
    }

    /// Writes a value in little-endian format
    #[inline]
    fn write_le<T>(&mut self, value: T) -> Result<()>
    where
        T: Into<u64>,
    {
        let val: u64 = value.into();
        match std::mem::size_of::<T>() {
            1 => self.data.push(val as u8),
            2 => self.data.extend_from_slice(&(val as u16).to_le_bytes()),
            4 => self.data.extend_from_slice(&(val as u32).to_le_bytes()),
            8 => self.data.extend_from_slice(&val.to_le_bytes()),
            _ => {
                return Err(Protocol(Serialization(
                    "Unsupported data type size".to_string(),
                )));
            }
        }
        Ok(())
    }

    /// Writes a value in little-endian format for signed integers
    #[inline]
    fn write_le_signed<T>(&mut self, value: T) -> Result<()>
    where
        T: Copy,
    {
        match std::mem::size_of::<T>() {
            2 => {
                let val = unsafe { std::mem::transmute_copy::<T, i16>(&value) };
                self.data.extend_from_slice(&val.to_le_bytes());
            }
            4 => {
                let val = unsafe { std::mem::transmute_copy::<T, i32>(&value) };
                self.data.extend_from_slice(&val.to_le_bytes());
            }
            8 => {
                let val = unsafe { std::mem::transmute_copy::<T, i64>(&value) };
                self.data.extend_from_slice(&val.to_le_bytes());
            }
            _ => {
                return Err(Protocol(Serialization(
                    "Unsupported signed type size".to_string(),
                )));
            }
        }
        Ok(())
    }

    /// Writes padding bytes
    #[inline]
    fn write_padding(&mut self, count: usize) {
        self.data.resize(self.data.len() + count, 0);
    }

    /// Consumes the buffer and returns the data
    #[inline]
    fn into_vec(self) -> Vec<u8> {
        self.data
    }
}

// -----------------------------------------------------------------------------
// Connection Setup Serializers
// -----------------------------------------------------------------------------

impl X11ResponseSerializer for ConnectionSetupAuthRequiredResponse {
    /// Serializes an authentication required response.
    ///
    /// This response indicates the server requires authentication before
    /// accepting the connection. The reason field provides details about
    /// the required authentication method.
    fn serialize(&self, _response: &Response) -> Result<Option<Vec<u8>>> {
        // Calculate total size: header (8 bytes) + padded reason string
        let reason_padded_len = align_to_4(self.reason.len());
        let total_size = 8 + reason_padded_len;

        let mut buffer = SerializationBuffer::with_capacity(total_size);

        // Write fixed header fields
        buffer.write_bytes(&[status_codes::AUTH_REQUIRED, 0])?; // status + unused
        buffer.write_le(self.protocol_major_version)?;
        buffer.write_le(self.protocol_minor_version)?;

        // Calculate and write additional data length in 4-byte units
        let additional_data_length = (reason_padded_len / 4) as u16;
        buffer.write_le(additional_data_length)?;

        // Write reason string with proper padding
        buffer.write_bytes(self.reason.as_bytes())?;
        let padding_needed = reason_padded_len - self.reason.len();
        buffer.write_padding(padding_needed);

        Ok(Some(buffer.into_vec()))
    }

    /// Calculates the serialized size for packet length calculations.
    /// Size = header (8 bytes) + padded reason string
    fn serialized_size(&self) -> usize {
        8 + align_to_4(self.reason.len())
    }
}

impl ConnectionSetupAcceptedResponse {
    /// Serializes the vendor identification string with proper padding
    #[inline]
    fn serialize_vendor_string(&self, buffer: &mut SerializationBuffer) -> Result<()> {
        buffer.write_bytes(self.vendor.as_bytes())?;
        let padding_needed = align_to_4(self.vendor.len()) - self.vendor.len();
        buffer.write_padding(padding_needed);
        Ok(())
    }

    /// Serializes all pixmap formats efficiently
    #[inline]
    fn serialize_pixmap_formats(&self, buffer: &mut SerializationBuffer) -> Result<()> {
        for format in &self.pixmap_formats {
            let format_data = format.serialize(&Response::default())?.ok_or_else(|| {
                Protocol(Serialization(
                    "Failed to serialize pixmap format".to_string(),
                ))
            })?;
            buffer.write_bytes(&format_data)?;
        }
        Ok(())
    }

    /// Serializes all screen information
    #[inline]
    fn serialize_screens(
        &self,
        buffer: &mut SerializationBuffer,
        response: &Response,
    ) -> Result<()> {
        for screen in &self.screens {
            let screen_data = screen.serialize(response)?.ok_or_else(|| {
                Protocol(Serialization("Failed to serialize screen data".to_string()))
            })?;
            buffer.write_bytes(&screen_data)?;
        }
        Ok(())
    }
}

impl X11ResponseSerializer for ConnectionSetupAcceptedResponse {
    /// Serializes a successful connection setup response.
    ///
    /// This is the most complex response type, containing comprehensive
    /// information about server capabilities, supported formats, and screens.
    fn serialize(&self, response: &Response) -> Result<Option<Vec<u8>>> {
        // Pre-calculate sizes for efficient memory allocation
        let vendor_padded_len = align_to_4(self.vendor.len());
        let screens_len: usize = self.screens.iter().map(|s| s.serialized_size()).sum();
        let pixmap_formats_len = self.pixmap_formats.len() * 8; // 8 bytes per format

        // X11 spec formula: 8 + 2n + (v + p + m) / 4
        let additional_data_len =
            8 + 2 * self.pixmap_formats.len() + (vendor_padded_len + screens_len) / 4;

        let total_estimated_size = 40 + vendor_padded_len + pixmap_formats_len + screens_len;
        let mut buffer = SerializationBuffer::with_capacity(total_estimated_size);

        // Serialize fixed header (32 bytes)
        buffer.write_bytes(&[self.success, 0])?; // success + unused
        buffer.write_le(self.protocol_major_version)?;
        buffer.write_le(self.protocol_minor_version)?;
        buffer.write_le(additional_data_len as u16)?;
        buffer.write_le(self.release_number)?;
        buffer.write_le(self.resource_id_base)?;
        buffer.write_le(self.resource_id_mask)?;
        buffer.write_le(self.motion_buffer_size)?;
        buffer.write_le(self.vendor_length)?;
        buffer.write_le(self.maximum_request_length)?;

        // Single-byte fields
        buffer.write_bytes(&[
            self.number_of_screens,
            self.number_of_formats,
            self.image_byte_order,
            self.bitmap_format_bit_order,
            self.bitmap_format_scanline_unit,
            self.bitmap_format_scanline_pad,
            self.min_keycode,
            self.max_keycode,
        ])?;

        // 4-byte padding
        buffer.write_padding(4);

        // Serialize variable-length sections
        self.serialize_vendor_string(&mut buffer)?;
        self.serialize_pixmap_formats(&mut buffer)?;
        self.serialize_screens(&mut buffer, response)?;

        Ok(Some(buffer.into_vec()))
    }

    /// Calculates the serialized size for packet length calculations.
    /// Size = fixed header (40 bytes) + vendor string + pixmap formats + screens
    fn serialized_size(&self) -> usize {
        let vendor_padded_len = align_to_4(self.vendor.len());
        let pixmap_formats_len = self.pixmap_formats.len() * 8; // 8 bytes per format
        let screens_len: usize = self.screens.iter().map(|s| s.serialized_size()).sum();

        40 + vendor_padded_len + pixmap_formats_len + screens_len
    }
}

impl X11ResponseSerializer for ConnectionSetupRefusedResponse {
    /// Serializes a connection refusal response.
    ///
    /// This response indicates the server has refused the connection,
    /// typically due to authentication failure or policy restrictions.
    fn serialize(&self, _response: &Response) -> Result<Option<Vec<u8>>> {
        let reason_padded_len = align_to_4(self.reason.len());
        let total_size = 8 + reason_padded_len;

        let mut buffer = SerializationBuffer::with_capacity(total_size);

        // Use native byte order for refusal responses (per X11 spec)
        buffer.write_bytes(&self.protocol_major_version.to_ne_bytes())?;
        buffer.write_bytes(&self.protocol_minor_version.to_ne_bytes())?;
        buffer.write_bytes(&self.additional_data_length.to_ne_bytes())?;

        // Write reason string with padding
        buffer.write_bytes(self.reason.as_bytes())?;
        let padding_needed = reason_padded_len - self.reason.len();
        buffer.write_padding(padding_needed);

        Ok(Some(buffer.into_vec()))
    }

    /// Calculates the serialized size for packet length calculations.
    /// Size = header (8 bytes) + padded reason string
    fn serialized_size(&self) -> usize {
        8 + align_to_4(self.reason.len())
    }
}

// -----------------------------------------------------------------------------
// Format and Structure Serializers
// -----------------------------------------------------------------------------

impl X11ResponseSerializer for ConnectionSetupAcceptedPixmapFormat {
    /// Serializes a pixmap format descriptor.
    ///
    /// Contains depth, bits per pixel, and scanline padding information
    /// for a supported pixmap format.
    fn serialize(&self, _response: &Response) -> Result<Option<Vec<u8>>> {
        let mut buffer = SerializationBuffer::with_capacity(8);

        buffer.write_bytes(&[
            self.depth,
            self.bits_per_pixel,
            self.scanline_pad,
            0, // padding byte
        ])?;
        buffer.write_padding(4); // Additional padding to 8 bytes total

        Ok(Some(buffer.into_vec()))
    }

    /// Calculates the serialized size for packet length calculations.
    /// Size = 8 bytes (fixed format size)
    fn serialized_size(&self) -> usize {
        8
    }
}

impl X11ResponseSerializer for ConnectionSetupAcceptedScreenDepth {
    /// Serializes screen depth information including associated visuals.
    ///
    /// This contains the depth value and a list of all visual types
    /// available at that depth.
    fn serialize(&self, response: &Response) -> Result<Option<Vec<u8>>> {
        // Estimate size: header (4 bytes) + visuals (24 bytes each)
        let estimated_size = 4 + self.visuals.len() * 24;
        let mut buffer = SerializationBuffer::with_capacity(estimated_size);

        // Write depth header
        buffer.write_bytes(&[self.depth, 0])?; // depth + unused
        buffer.write_le(self.visuals.len() as u16)?; // number of visuals

        // Serialize all visuals
        for visual in &self.visuals {
            let visual_data = visual.serialize(response)?.ok_or_else(|| {
                Protocol(Serialization(
                    error_messages::VISUAL_SERIALIZATION_FAILED.to_string(),
                ))
            })?;
            buffer.write_bytes(&visual_data)?;
        }

        Ok(Some(buffer.into_vec()))
    }

    /// Calculates the serialized size for packet length calculations.
    /// Size = header (4 bytes) + visuals (24 bytes each)
    fn serialized_size(&self) -> usize {
        4 + self.visuals.len() * 24
    }
}

impl X11ResponseSerializer for ConnectionSetupAcceptedScreen {
    /// Serializes complete screen information.
    ///
    /// This includes root window, colormap, dimensions, and all
    /// supported depth/visual combinations for the screen.
    fn serialize(&self, response: &Response) -> Result<Option<Vec<u8>>> {
        // Estimate size based on fixed fields + depths
        let estimated_depths_size: usize = self
            .allowed_depths
            .iter()
            .map(|d| 4 + d.visuals.len() * 24) // depth header + visuals
            .sum();
        let estimated_size = 40 + estimated_depths_size; // 40 bytes fixed fields

        let mut buffer = SerializationBuffer::with_capacity(estimated_size);

        // Serialize fixed screen properties (36 bytes)
        buffer.write_le(self.root)?;
        buffer.write_le(self.default_colormap)?;
        buffer.write_le(self.white_pixel)?;
        buffer.write_le(self.black_pixel)?;
        buffer.write_le(self.current_input_masks)?;
        buffer.write_le(self.width_in_pixels)?;
        buffer.write_le(self.height_in_pixels)?;
        buffer.write_le(self.width_in_millimeters)?;
        buffer.write_le(self.height_in_millimeters)?;
        buffer.write_le(self.min_installed_maps)?;
        buffer.write_le(self.max_installed_maps)?;
        buffer.write_le(self.root_visual)?;

        // Single-byte fields (4 bytes)
        buffer.write_bytes(&[
            self.backing_stores as u8,
            self.save_unders,
            self.root_depth,
            self.allowed_depths.len() as u8,
        ])?;

        // Serialize all depth information
        for depth in &self.allowed_depths {
            let depth_data = depth.serialize(response)?.ok_or_else(|| {
                Protocol(Serialization(
                    error_messages::DEPTH_SERIALIZATION_FAILED.to_string(),
                ))
            })?;
            buffer.write_bytes(&depth_data)?;
        }

        Ok(Some(buffer.into_vec()))
    }

    /// Calculates the serialized size for packet length calculations.
    /// Size = fixed fields (40 bytes) + all depth structures
    fn serialized_size(&self) -> usize {
        40 + self
            .allowed_depths
            .iter()
            .map(|d| d.serialized_size())
            .sum::<usize>()
    }
}

// Add helper method for size estimation
impl ConnectionSetupAcceptedScreen {
    /// Estimates the serialized size for memory allocation optimization
    #[inline]
    fn estimated_serialized_size(&self) -> usize {
        self.serialized_size()
    }
}

// -----------------------------------------------------------------------------
// Request Response Serializers
// -----------------------------------------------------------------------------

impl X11ResponseSerializer for InternAtomResponse {
    /// Serializes an InternAtom response containing the atom ID.
    ///
    /// Returns the atom identifier for the requested atom name,
    /// or None if the atom doesn't exist and only_if_exists was true.
    fn serialize(&self, _response: &Response) -> Result<Option<Vec<u8>>> {
        let mut buffer = SerializationBuffer::with_capacity(24);

        // Write reply body
        buffer.write_padding(4); // reply length (always 0 for InternAtom)
        buffer.write_le(self.atom)?; // atom ID
        buffer.write_padding(20); // unused bytes (total 24 bytes body)

        Ok(Some(buffer.into_vec()))
    }

    /// Calculates the serialized size for packet length calculations.
    /// Size = 24 bytes (fixed InternAtom response body size)
    fn serialized_size(&self) -> usize {
        24
    }
}

impl X11ResponseSerializer for GetGeometry {
    /// Serializes a GetGeometry response with drawable properties.
    ///
    /// Returns comprehensive geometric information about the specified
    /// drawable including position, size, depth, and border width.
    fn serialize(&self, response: &Response) -> Result<Option<Vec<u8>>> {
        let mut buffer = SerializationBuffer::with_capacity(24);

        // Serialize geometry data
        buffer.write_le(self.root)?;
        buffer.write_le_signed(self.x)?;
        buffer.write_le_signed(self.y)?;
        buffer.write_le(self.width)?;
        buffer.write_le(self.height)?;
        buffer.write_le(self.border_width)?;
        buffer.write_le(self.depth as u16)?; // Pad to 2 bytes

        // Wrap in standard reply format with depth in optional byte field
        let wrapped = ReplySerializationWrapper::wrap(
            buffer.into_vec(),
            response.sequence_number,
            Some(self.depth),
        );

        Ok(Some(wrapped))
    }

    /// Calculates the serialized size for packet length calculations.
    /// Size = 32 bytes (standard X11 reply packet size)
    fn serialized_size(&self) -> usize {
        X11_REPLY_SIZE
    }
}

impl X11ResponseSerializer for ConnectionSetupAcceptedScreenDepthVisual {
    /// Serializes visual information for a specific screen depth.
    ///
    /// Contains complete visual specification including color masks,
    /// class type, and colormap information.
    fn serialize(&self, response: &Response) -> Result<Option<Vec<u8>>> {
        let mut buffer = SerializationBuffer::with_capacity(24);

        // Use byte order-aware writing for all multi-byte fields
        buffer
            .data
            .write_with_order(self.id, response.byte_order)
            .map_err(|e| Protocol(Serialization(format!("Failed to write visual ID: {}", e))))?;

        buffer
            .data
            .write_with_order(self.class as u8, response.byte_order)
            .map_err(|e| {
                Protocol(Serialization(format!(
                    "Failed to write visual class: {}",
                    e
                )))
            })?;

        buffer
            .data
            .write_with_order(self.bits_per_rgb_value, response.byte_order)
            .map_err(|e| Protocol(Serialization(format!("Failed to write RGB bits: {}", e))))?;

        buffer
            .data
            .write_with_order(self.colormap_entries, response.byte_order)
            .map_err(|e| {
                Protocol(Serialization(format!(
                    "Failed to write colormap entries: {}",
                    e
                )))
            })?;

        buffer
            .data
            .write_with_order(self.red_mask, response.byte_order)
            .map_err(|e| Protocol(Serialization(format!("Failed to write red mask: {}", e))))?;

        buffer
            .data
            .write_with_order(self.green_mask, response.byte_order)
            .map_err(|e| Protocol(Serialization(format!("Failed to write green mask: {}", e))))?;

        buffer
            .data
            .write_with_order(self.blue_mask, response.byte_order)
            .map_err(|e| Protocol(Serialization(format!("Failed to write blue mask: {}", e))))?;

        // Add required padding
        buffer.write_padding(4);

        Ok(Some(buffer.into_vec()))
    }

    /// Calculates the serialized size for packet length calculations.
    /// Size = 32 bytes (standard X11 reply packet size)
    fn serialized_size(&self) -> usize {
        24 // Fixed size for visual structure
    }
}

// -----------------------------------------------------------------------------
// Module Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::ByteOrder;

    use super::*;

    #[test]
    fn test_reply_wrapper_minimal() {
        let reply = ReplySerializationWrapper::create_minimal_reply(42, Some(8));
        assert_eq!(reply.len(), X11_REPLY_SIZE);
        assert_eq!(reply[0], status_codes::REPLY);
        assert_eq!(reply[1], 8);
        assert_eq!(u16::from_le_bytes([reply[2], reply[3]]), 42);
    }

    #[test]
    fn test_serialization_buffer() {
        let mut buffer = SerializationBuffer::with_capacity(16);
        assert!(buffer.write_le(0x1234u16).is_ok());
        assert!(buffer.write_le(0x56789ABCu32).is_ok());

        let data = buffer.into_vec();
        assert_eq!(data.len(), 6);
        assert_eq!(&data[0..2], &0x1234u16.to_le_bytes());
        assert_eq!(&data[2..6], &0x56789ABCu32.to_le_bytes());
    }

    #[test]
    fn test_auth_required_serialization() {
        let auth_req = ConnectionSetupAuthRequiredResponse {
            protocol_major_version: 11,
            protocol_minor_version: 0,
            additional_data_length: 1, // Will be recalculated
            reason: "AUTH".to_string(),
        };

        let response = Response {
            sequence_number: 1,
            byte_order: ByteOrder::LittleEndian,
            ..Response::default()
        };

        let result = auth_req.serialize(&response).unwrap().unwrap();
        assert_eq!(result[0], status_codes::AUTH_REQUIRED);
        assert_eq!(result[1], 0); // unused
        assert!(result.len() >= 8); // Minimum header size
    }
}
