//! X11 Protocol Handshake Implementation
//!
//! This module implements the X11 connection setup handshake between client and server.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, error, info, warn};

use crate::{ServerError, ServerResult};

/// Display configuration for screen info
#[derive(Debug, Clone)]
pub struct DisplayConfig {
    pub width: u16,
    pub height: u16,
    pub width_mm: u16,
    pub height_mm: u16,
    pub depth: u8,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            // Assume a ~24" monitor with standard DPI (~96 DPI)
            width_mm: (1920.0 / 96.0 * 25.4) as u16, // ~508mm
            height_mm: (1080.0 / 96.0 * 25.4) as u16, // ~286mm
            depth: 24,
        }
    }
}

/// X11 Connection Setup Request
#[derive(Debug, Clone)]
pub struct ConnectionSetupRequest {
    /// Byte order: 0x42 (MSB first) or 0x6c (LSB first)
    pub byte_order: u8,
    /// Protocol major version (usually 11)
    pub protocol_major_version: u16,
    /// Protocol minor version (usually 0)
    pub protocol_minor_version: u16,
    /// Authorization protocol name (e.g., "MIT-MAGIC-COOKIE-1")
    pub authorization_protocol_name: String,
    /// Authorization protocol data
    pub authorization_protocol_data: Vec<u8>,
}

/// X11 Connection Setup Response Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupStatus {
    /// Connection successful
    Success = 1,
    /// Connection failed
    Failed = 0,
    /// Additional authentication required
    Authenticate = 2,
}

/// X11 Connection Setup Response
#[derive(Debug, Clone)]
pub struct ConnectionSetupResponse {
    /// Response status
    pub status: SetupStatus,
    /// Response data (server info for success, reason for failure, auth data for authenticate)
    pub data: Vec<u8>,
}

/// X11 Server Information (for successful setup)
#[derive(Debug, Clone)]
pub struct ServerInfo {
    /// Protocol major version
    pub protocol_major_version: u16,
    /// Protocol minor version
    pub protocol_minor_version: u16,
    /// Release number
    pub release_number: u32,
    /// Resource ID base
    pub resource_id_base: u32,
    /// Resource ID mask
    pub resource_id_mask: u32,
    /// Motion buffer size
    pub motion_buffer_size: u32,
    /// Vendor string
    pub vendor: String,
    /// Maximum request length
    pub maximum_request_length: u16,
    /// Number of screens
    pub number_of_screens: u8,
    /// Number of formats
    pub number_of_formats: u8,
    /// Image byte order
    pub image_byte_order: u8,
    /// Bitmap format bit order
    pub bitmap_format_bit_order: u8,
    /// Bitmap format scanline unit
    pub bitmap_format_scanline_unit: u8,
    /// Bitmap format scanline pad
    pub bitmap_format_scanline_pad: u8,
    /// Minimum keycode
    pub min_keycode: u8,
    /// Maximum keycode
    pub max_keycode: u8,
    /// Screen information (simplified for now)
    pub screens: Vec<ScreenInfo>,
}

/// Screen Information
#[derive(Debug, Clone)]
pub struct ScreenInfo {
    /// Root window ID
    pub root: u32,
    /// Default colormap
    pub default_colormap: u32,
    /// White pixel value
    pub white_pixel: u32,
    /// Black pixel value
    pub black_pixel: u32,
    /// Current input masks
    pub current_input_masks: u32,
    /// Width in pixels
    pub width_in_pixels: u16,
    /// Height in pixels
    pub height_in_pixels: u16,
    /// Width in millimeters
    pub width_in_millimeters: u16,
    /// Height in millimeters
    pub height_in_millimeters: u16,
    /// Minimum installed maps
    pub min_installed_maps: u16,
    /// Maximum installed maps
    pub max_installed_maps: u16,
    /// Root visual
    pub root_visual: u32,
    /// Backing stores
    pub backing_stores: u8,
    /// Save unders
    pub save_unders: u8,
    /// Root depth
    pub root_depth: u8,
    /// Number of allowed depths
    pub allowed_depths_len: u8,
}

impl ConnectionSetupRequest {
    /// Parse a connection setup request from bytes
    pub fn parse(data: &[u8]) -> ServerResult<Self> {
        if data.len() < 12 {
            return Err(ServerError::ProtocolError(
                crate::protocol::ProtocolError::InvalidMessage(
                    "Connection setup request too short".to_string(),
                ),
            ));
        }
        let byte_order = data[0];

        // Determine byte order for reading multi-byte values
        let (protocol_major_version, protocol_minor_version, auth_name_len, auth_data_len) =
            if byte_order == 0x42 {
                // MSB first (big endian)
                (
                    u16::from_be_bytes([data[2], data[3]]),
                    u16::from_be_bytes([data[4], data[5]]),
                    u16::from_be_bytes([data[6], data[7]]) as usize,
                    u16::from_be_bytes([data[8], data[9]]) as usize,
                )
            } else {
                // LSB first (little endian)
                (
                    u16::from_le_bytes([data[2], data[3]]),
                    u16::from_le_bytes([data[4], data[5]]),
                    u16::from_le_bytes([data[6], data[7]]) as usize,
                    u16::from_le_bytes([data[8], data[9]]) as usize,
                )
            };

        let mut offset = 12;

        // Read authorization protocol name
        let authorization_protocol_name = if auth_name_len > 0 {
            if offset + auth_name_len > data.len() {
                return Err(ServerError::ProtocolError(
                    crate::protocol::ProtocolError::InvalidMessage(
                        "Invalid auth name length".to_string(),
                    ),
                ));
            }
            let name_bytes = &data[offset..offset + auth_name_len];
            offset += auth_name_len; // Pad to 4-byte boundary (p = pad(n))
            offset += (4 - (auth_name_len % 4)) % 4;
            String::from_utf8_lossy(name_bytes).to_string()
        } else {
            String::new()
        }; // Read authorization protocol data
        let authorization_protocol_data = if auth_data_len > 0 {
            if offset + auth_data_len > data.len() {
                return Err(ServerError::ProtocolError(
                    crate::protocol::ProtocolError::InvalidMessage(
                        "Invalid auth data length".to_string(),
                    ),
                ));
            }
            let auth_data = data[offset..offset + auth_data_len].to_vec();
            auth_data
        } else {
            Vec::new()
        };

        Ok(Self {
            byte_order,
            protocol_major_version,
            protocol_minor_version,
            authorization_protocol_name,
            authorization_protocol_data,
        })
    }
}

impl ConnectionSetupResponse {
    /// Create a successful setup response
    pub fn success(server_info: ServerInfo) -> Self {
        let data = server_info.serialize();
        Self {
            status: SetupStatus::Success,
            data,
        }
    }

    /// Create a failed setup response
    pub fn failed(reason: &str) -> Self {
        let mut data = Vec::new();
        data.push(reason.len() as u8);
        data.extend_from_slice(reason.as_bytes());
        // Pad to 4-byte boundary
        while data.len() % 4 != 0 {
            data.push(0);
        }
        Self {
            status: SetupStatus::Failed,
            data,
        }
    }
    /// Serialize the response to bytes
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        match self.status {
            SetupStatus::Success => {
                // Success response format (from X11 spec):
                // byte 0: 1 (Success)
                // byte 1: unused
                // bytes 2-3: protocol-major-version
                // bytes 4-5: protocol-minor-version
                // bytes 6-7: length in 4-byte units of "additional data"
                // ... then additional data follows
                bytes.push(1); // Success
                bytes.push(0); // unused

                // Protocol version - extract from server info or use defaults
                if self.data.len() >= 4 {
                    bytes.extend_from_slice(&self.data[0..4]); // protocol major + minor
                } else {
                    bytes.extend_from_slice(&11u16.to_le_bytes()); // protocol major = 11
                    bytes.extend_from_slice(&0u16.to_le_bytes()); // protocol minor = 0
                }

                // Length in 4-byte units of additional data (everything after protocol version)
                // The server info data already includes the protocol version in first 4 bytes
                let additional_data = if self.data.len() >= 4 {
                    &self.data[4..]
                } else {
                    &[]
                };
                let length_in_units = ((additional_data.len() + 3) / 4) as u16;
                bytes.extend_from_slice(&length_in_units.to_le_bytes());

                // Additional data (server info without the protocol version)
                bytes.extend_from_slice(additional_data);
            }
            SetupStatus::Failed => {
                // Failed response format (from X11 spec):
                // byte 0: 0 (Failed)
                // byte 1: n (length of reason in bytes)
                // bytes 2-3: protocol-major-version
                // bytes 4-5: protocol-minor-version
                // bytes 6-7: (n+p)/4 (length in 4-byte units of "additional data")
                // ... then reason string + padding
                bytes.push(0); // Failed

                let reason_len = if !self.data.is_empty() {
                    self.data[0]
                } else {
                    0
                };
                bytes.push(reason_len);

                // Protocol version
                bytes.extend_from_slice(&11u16.to_le_bytes()); // protocol major = 11
                bytes.extend_from_slice(&0u16.to_le_bytes()); // protocol minor = 0

                // Length in 4-byte units of additional data (reason + padding)
                let padded_len = ((self.data.len() + 3) / 4) as u16;
                bytes.extend_from_slice(&padded_len.to_le_bytes());

                // Reason string + padding
                bytes.extend_from_slice(&self.data);
                // Add padding to make total length a multiple of 4
                while bytes.len() % 4 != 0 {
                    bytes.push(0);
                }
            }
            SetupStatus::Authenticate => {
                // Authenticate response format (from X11 spec):
                // byte 0: 2 (Authenticate)
                // bytes 1-5: unused
                // bytes 6-7: (n+p)/4 (length in 4-byte units of "additional data")
                // ... then reason string + padding
                bytes.push(2); // Authenticate
                bytes.extend_from_slice(&[0u8; 5]); // 5 bytes unused

                // Length in 4-byte units of additional data
                let padded_len = ((self.data.len() + 3) / 4) as u16;
                bytes.extend_from_slice(&padded_len.to_le_bytes());

                bytes.extend_from_slice(&self.data);
                // Add padding to make total length a multiple of 4
                while bytes.len() % 4 != 0 {
                    bytes.push(0);
                }
            }
        }

        bytes
    }
}

impl ServerInfo {
    /// Create server info from display configuration
    pub fn from_display_config(display_config: &DisplayConfig) -> Self {
        Self {
            protocol_major_version: 11,
            protocol_minor_version: 0,
            release_number: 12000000, // X11R12
            resource_id_base: 0x00400000,
            resource_id_mask: 0x003fffff,
            motion_buffer_size: 256,
            vendor: "RXServer".to_string(),
            maximum_request_length: 65535,
            number_of_screens: 1,
            number_of_formats: 1,
            image_byte_order: 0,        // LSB first
            bitmap_format_bit_order: 0, // LSB first
            bitmap_format_scanline_unit: 32,
            bitmap_format_scanline_pad: 32,
            min_keycode: 8,
            max_keycode: 255,
            screens: vec![ScreenInfo::from_display_config(display_config)],
        }
    }

    /// Create default server info
    pub fn default() -> Self {
        Self::from_display_config(&DisplayConfig::default())
    }
    /// Serialize server info to bytes according to X11 specification
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Protocol version (will be used in response header)
        bytes.extend_from_slice(&self.protocol_major_version.to_le_bytes());
        bytes.extend_from_slice(&self.protocol_minor_version.to_le_bytes());

        // Start building the additional data section
        let mut additional_data = Vec::new();

        // Release number (4 bytes)
        additional_data.extend_from_slice(&self.release_number.to_le_bytes());

        // Resource ID base and mask (8 bytes)
        additional_data.extend_from_slice(&self.resource_id_base.to_le_bytes());
        additional_data.extend_from_slice(&self.resource_id_mask.to_le_bytes());

        // Motion buffer size (4 bytes)
        additional_data.extend_from_slice(&self.motion_buffer_size.to_le_bytes());

        // Vendor length (2 bytes) and maximum request length (2 bytes)
        let vendor_bytes = self.vendor.as_bytes();
        additional_data.extend_from_slice(&(vendor_bytes.len() as u16).to_le_bytes());
        additional_data.extend_from_slice(&self.maximum_request_length.to_le_bytes());

        // Number of screens and formats (2 bytes)
        additional_data.push(self.number_of_screens);
        additional_data.push(self.number_of_formats);

        // Image byte order and bitmap format info (6 bytes)
        additional_data.push(self.image_byte_order);
        additional_data.push(self.bitmap_format_bit_order);
        additional_data.push(self.bitmap_format_scanline_unit);
        additional_data.push(self.bitmap_format_scanline_pad);
        additional_data.push(self.min_keycode);
        additional_data.push(self.max_keycode);

        // 4 bytes unused
        additional_data.extend_from_slice(&[0u8; 4]);

        // Vendor string (v bytes)
        additional_data.extend_from_slice(vendor_bytes);

        // Padding for vendor string (p = pad(v))
        let vendor_padding = (4 - (vendor_bytes.len() % 4)) % 4;
        additional_data.extend_from_slice(&vec![0u8; vendor_padding]);

        // Pixmap formats (8 bytes per format)
        for _ in 0..self.number_of_formats {
            // FORMAT: depth(1) + bits-per-pixel(1) + scanline-pad(1) + unused(5)
            additional_data.push(24); // depth
            additional_data.push(32); // bits per pixel
            additional_data.push(32); // scanline pad
            additional_data.extend_from_slice(&[0u8; 5]); // 5 bytes unused
        }

        // Screen information (variable length)
        for screen in &self.screens {
            additional_data.extend_from_slice(&screen.serialize());
        }

        // Now assemble the complete response:
        // First the protocol version
        bytes.extend_from_slice(&additional_data);

        bytes
    }
}

impl ScreenInfo {
    /// Create screen info from display configuration
    pub fn from_display_config(display_config: &DisplayConfig) -> Self {
        Self {
            root: 1,
            default_colormap: 1,
            white_pixel: 0xffffff,
            black_pixel: 0x000000,
            current_input_masks: 0,
            width_in_pixels: display_config.width,
            height_in_pixels: display_config.height,
            width_in_millimeters: display_config.width_mm,
            height_in_millimeters: display_config.height_mm,
            min_installed_maps: 1,
            max_installed_maps: 1,
            root_visual: 1,
            backing_stores: 0, // Never
            save_unders: 0,    // False
            root_depth: display_config.depth,
            allowed_depths_len: 1,
        }
    }

    /// Create default screen info
    pub fn default() -> Self {
        Self {
            root: 1,
            default_colormap: 1,
            white_pixel: 0xffffff,
            black_pixel: 0x000000,
            current_input_masks: 0,
            width_in_pixels: 1920,
            height_in_pixels: 1080,
            width_in_millimeters: 508,
            height_in_millimeters: 286,
            min_installed_maps: 1,
            max_installed_maps: 1,
            root_visual: 1,
            backing_stores: 0,
            save_unders: 0,
            root_depth: 24,
            allowed_depths_len: 1,
        }
    }
    /// Calculate the byte size of this screen info when serialized
    pub fn byte_size(&self) -> usize {
        // Base screen structure: 40 bytes according to X11 spec
        let base_size = 40;

        // Add size for each depth: 8 bytes + (24 * number_of_visuals) bytes
        let depth_size = if self.allowed_depths_len > 0 {
            // Each DEPTH: 8 bytes + (24 * number_of_visuals) bytes
            // We have 1 depth with 1 visual, so: 8 + 24 = 32 bytes
            (self.allowed_depths_len as usize) * (8 + 24) // Assuming 1 visual per depth
        } else {
            0
        };

        base_size + depth_size
    }

    /// Serialize screen info to bytes according to X11 specification
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // SCREEN structure (40 bytes base):
        // 4 bytes: root window
        bytes.extend_from_slice(&self.root.to_le_bytes());
        // 4 bytes: default colormap
        bytes.extend_from_slice(&self.default_colormap.to_le_bytes());
        // 4 bytes: white pixel
        bytes.extend_from_slice(&self.white_pixel.to_le_bytes());
        // 4 bytes: black pixel
        bytes.extend_from_slice(&self.black_pixel.to_le_bytes());
        // 4 bytes: current input masks
        bytes.extend_from_slice(&self.current_input_masks.to_le_bytes());
        // 2 bytes: width in pixels
        bytes.extend_from_slice(&self.width_in_pixels.to_le_bytes());
        // 2 bytes: height in pixels
        bytes.extend_from_slice(&self.height_in_pixels.to_le_bytes());
        // 2 bytes: width in millimeters
        bytes.extend_from_slice(&self.width_in_millimeters.to_le_bytes());
        // 2 bytes: height in millimeters
        bytes.extend_from_slice(&self.height_in_millimeters.to_le_bytes());
        // 2 bytes: min installed maps
        bytes.extend_from_slice(&self.min_installed_maps.to_le_bytes());
        // 2 bytes: max installed maps
        bytes.extend_from_slice(&self.max_installed_maps.to_le_bytes());
        // 4 bytes: root visual
        bytes.extend_from_slice(&self.root_visual.to_le_bytes());
        // 1 byte: backing stores
        bytes.push(self.backing_stores);
        // 1 byte: save unders
        bytes.push(self.save_unders);
        // 1 byte: root depth
        bytes.push(self.root_depth);
        // 1 byte: number of allowed depths
        bytes.push(self.allowed_depths_len);

        // DEPTH structures (8 bytes each + 24 bytes per visual)
        for depth_index in 0..self.allowed_depths_len {
            // DEPTH structure (8 bytes):
            // 1 byte: depth
            bytes.push(if depth_index == 0 {
                self.root_depth
            } else {
                24
            });
            // 1 byte: unused
            bytes.push(0);
            // 2 bytes: number of visuals
            bytes.extend_from_slice(&1u16.to_le_bytes()); // 1 visual per depth
                                                          // 4 bytes: unused
            bytes.extend_from_slice(&[0u8; 4]);

            // VISUALTYPE structure (24 bytes):
            // 4 bytes: visual ID
            bytes.extend_from_slice(&self.root_visual.to_le_bytes());
            // 1 byte: class (4 = TrueColor)
            bytes.push(4);
            // 1 byte: bits per RGB value
            bytes.push(8);
            // 2 bytes: colormap entries
            bytes.extend_from_slice(&256u16.to_le_bytes());
            // 4 bytes: red mask
            bytes.extend_from_slice(&0x00ff0000u32.to_le_bytes());
            // 4 bytes: green mask
            bytes.extend_from_slice(&0x0000ff00u32.to_le_bytes());
            // 4 bytes: blue mask
            bytes.extend_from_slice(&0x000000ffu32.to_le_bytes());
            // 4 bytes: unused
            bytes.extend_from_slice(&[0u8; 4]);
        }

        bytes
    }
}

/// Perform X11 handshake with a client using the provided display configuration
pub async fn perform_handshake(
    stream: &mut tokio::net::TcpStream,
    display_config: Option<&DisplayConfig>,
) -> ServerResult<ConnectionSetupRequest> {
    debug!("Starting X11 handshake");

    // Read the connection setup request
    let mut buffer = [0u8; 4096];
    let bytes_read = stream.read(&mut buffer).await.map_err(|e| {
        error!("Failed to read connection setup request: {}", e);
        ServerError::NetworkError(format!("Failed to read setup request: {}", e))
    })?;

    if bytes_read < 12 {
        let msg = format!("Connection setup request too short: {} bytes", bytes_read);
        error!("{}", msg);
        return Err(ServerError::ProtocolError(
            crate::protocol::ProtocolError::InvalidMessage(msg),
        ));
    }

    // Parse the connection setup request
    let setup_request = ConnectionSetupRequest::parse(&buffer[..bytes_read])?;
    debug!("Parsed connection setup request: {:?}", setup_request);

    // Validate protocol version
    if setup_request.protocol_major_version != 11 {
        warn!(
            "Unsupported protocol version: {}.{}",
            setup_request.protocol_major_version, setup_request.protocol_minor_version
        );
        let response = ConnectionSetupResponse::failed(&format!(
            "Unsupported protocol version {}.{}",
            setup_request.protocol_major_version, setup_request.protocol_minor_version
        ));
        let response_bytes = response.serialize();
        stream.write_all(&response_bytes).await.map_err(|e| {
            error!("Failed to send setup response: {}", e);
            ServerError::NetworkError(format!("Failed to send setup response: {}", e))
        })?;
        return Err(ServerError::ProtocolError(
            crate::protocol::ProtocolError::UnsupportedProtocolVersion(
                setup_request.protocol_major_version,
                setup_request.protocol_minor_version,
            ),
        ));
    }

    // For now, we'll accept any authentication (or lack thereof)
    // In a real implementation, you'd validate the auth data here
    if !setup_request.authorization_protocol_name.is_empty() {
        debug!(
            "Client provided auth: {}",
            setup_request.authorization_protocol_name
        );
    }
    // Create and send successful setup response
    let server_info = if let Some(config) = display_config {
        ServerInfo::from_display_config(config)
    } else {
        ServerInfo::default()
    };
    let response = ConnectionSetupResponse::success(server_info);
    let response_bytes = response.serialize();

    debug!("Sending setup response: {} bytes", response_bytes.len());
    stream.write_all(&response_bytes).await.map_err(|e| {
        error!("Failed to send setup response: {}", e);
        ServerError::NetworkError(format!("Failed to send setup response: {}", e))
    })?;

    info!("X11 handshake completed successfully");
    Ok(setup_request)
}
