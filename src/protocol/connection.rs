//! X11 Connection Setup Protocol
//!
//! This module implements the X11 connection setup and authentication protocol.
//! The connection setup happens before any regular X11 requests.

use crate::{todo_medium, Result};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// X11 Connection Setup Request (sent by client first)
#[derive(Debug, Clone)]
pub struct ConnectionSetupRequest {
    /// Protocol major version (usually 11)
    pub protocol_major_version: u16,
    /// Protocol minor version (usually 0)
    pub protocol_minor_version: u16,
    /// Length of authorization protocol name
    pub authorization_protocol_name_length: u16,
    /// Length of authorization protocol data
    pub authorization_protocol_data_length: u16,
    /// Authorization protocol name (e.g., "MIT-MAGIC-COOKIE-1")
    pub authorization_protocol_name: String,
    /// Authorization protocol data (the actual auth cookie)
    pub authorization_protocol_data: Vec<u8>,
}

/// X11 Connection Setup Response Status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionSetupStatus {
    /// Connection successful
    Success = 1,
    /// Connection failed
    Failed = 0,
    /// Authentication required (for multi-pass auth)
    Authenticate = 2,
}

/// X11 Connection Setup Response (sent by server)
#[derive(Debug, Clone)]
pub struct ConnectionSetupResponse {
    /// Response status
    pub status: ConnectionSetupStatus,
    /// Length of reason string (if failed/authenticate)
    pub reason_length: u8,
    /// Protocol major version
    pub protocol_major_version: u16,
    /// Protocol minor version
    pub protocol_minor_version: u16,
    /// Additional length in 4-byte units
    pub additional_data_length: u16,
    /// Reason string (for failure/authenticate)
    pub reason: String,
    /// Server information (if successful)
    pub server_info: Option<ServerInfo>,
}

/// Server information returned on successful connection
#[derive(Debug, Clone)]
pub struct ServerInfo {
    /// Server release number
    pub release: u32,
    /// Resource ID base
    pub resource_id_base: u32,
    /// Resource ID mask
    pub resource_id_mask: u32,
    /// Motion buffer size
    pub motion_buffer_size: u32,
    /// Vendor name length
    pub vendor_length: u16,
    /// Maximum request length in 4-byte units
    pub maximum_request_length: u16,
    /// Number of screens
    pub number_of_screens: u8,
    /// Number of pixmap formats
    pub number_of_pixmap_formats: u8,
    /// Image byte order (0=LSB first, 1=MSB first)
    pub image_byte_order: u8,
    /// Bitmap format bit order (0=LSB first, 1=MSB first)
    pub bitmap_format_bit_order: u8,
    /// Bitmap format scanline unit (8, 16, or 32)
    pub bitmap_format_scanline_unit: u8,
    /// Bitmap format scanline pad (8, 16, or 32)
    pub bitmap_format_scanline_pad: u8,
    /// Minimum keycode
    pub min_keycode: u8,
    /// Maximum keycode
    pub max_keycode: u8,
    /// Vendor name
    pub vendor: String,
    /// Pixmap formats
    pub pixmap_formats: Vec<PixmapFormat>,
    /// Screen information
    pub screens: Vec<ScreenInfo>,
}

/// Pixmap format information
#[derive(Debug, Clone)]
pub struct PixmapFormat {
    /// Depth in bits
    pub depth: u8,
    /// Bits per pixel
    pub bits_per_pixel: u8,
    /// Scanline pad
    pub scanline_pad: u8,
}

/// Screen information
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
    /// Root visual ID
    pub root_visual: u32,
    /// Backing stores support
    pub backing_stores: u8,
    /// Save unders support
    pub save_unders: bool,
    /// Root depth
    pub root_depth: u8,
    /// Number of allowed depths
    pub allowed_depths_len: u8,
    /// Allowed depths
    pub allowed_depths: Vec<DepthInfo>,
}

/// Depth information
#[derive(Debug, Clone)]
pub struct DepthInfo {
    /// Depth value
    pub depth: u8,
    /// Number of visuals for this depth
    pub visuals_len: u16,
    /// Visual types for this depth
    pub visuals: Vec<VisualType>,
}

/// Visual type information
#[derive(Debug, Clone)]
pub struct VisualType {
    /// Visual ID
    pub visual_id: u32,
    /// Visual class (StaticGray, GrayScale, StaticColor, PseudoColor, TrueColor, DirectColor)
    pub class: u8,
    /// Bits per RGB value
    pub bits_per_rgb_value: u8,
    /// Number of colormap entries
    pub colormap_entries: u16,
    /// Red mask
    pub red_mask: u32,
    /// Green mask
    pub green_mask: u32,
    /// Blue mask
    pub blue_mask: u32,
}

/// X11 Connection Handler
pub struct ConnectionHandler {
    /// Known authentication protocols
    auth_protocols: HashMap<String, Box<dyn AuthenticationProtocol>>,
    /// Server info to send to clients
    server_info: ServerInfo,
}

impl ConnectionHandler {
    /// Create a new connection handler
    pub fn new() -> Self {
        let server_info = Self::create_default_server_info();
        let mut auth_protocols = HashMap::new();

        // Register default authentication protocols
        auth_protocols.insert(
            "".to_string(), // Empty = no authentication
            Box::new(NoAuthProtocol) as Box<dyn AuthenticationProtocol>,
        );
        auth_protocols.insert(
            "MIT-MAGIC-COOKIE-1".to_string(),
            Box::new(MitMagicCookieProtocol::new()),
        );

        Self {
            auth_protocols,
            server_info,
        }
    }

    /// Handle connection setup request
    pub async fn handle_connection_setup(
        &self,
        setup_request: ConnectionSetupRequest,
    ) -> Result<ConnectionSetupResponse> {
        info!("Processing X11 connection setup");
        debug!(
            "Client protocol version: {}.{}",
            setup_request.protocol_major_version, setup_request.protocol_minor_version
        );

        // Check protocol version
        if setup_request.protocol_major_version != 11 {
            return Ok(ConnectionSetupResponse {
                status: ConnectionSetupStatus::Failed,
                reason_length: 0,
                protocol_major_version: 11,
                protocol_minor_version: 0,
                additional_data_length: 0,
                reason: "Unsupported protocol version".to_string(),
                server_info: None,
            });
        }

        // Handle authentication
        let auth_result = self
            .authenticate(
                &setup_request.authorization_protocol_name,
                &setup_request.authorization_protocol_data,
            )
            .await?;

        match auth_result {
            AuthenticationResult::Success => {
                info!("Client authentication successful");
                Ok(ConnectionSetupResponse {
                    status: ConnectionSetupStatus::Success,
                    reason_length: 0,
                    protocol_major_version: 11,
                    protocol_minor_version: 0,
                    additional_data_length: self.calculate_server_info_length(),
                    reason: String::new(),
                    server_info: Some(self.server_info.clone()),
                })
            }
            AuthenticationResult::Failed(reason) => {
                warn!("Client authentication failed: {}", reason);
                Ok(ConnectionSetupResponse {
                    status: ConnectionSetupStatus::Failed,
                    reason_length: reason.len() as u8,
                    protocol_major_version: 11,
                    protocol_minor_version: 0,
                    additional_data_length: 0,
                    reason,
                    server_info: None,
                })
            }
            AuthenticationResult::AuthenticateRequired(data) => {
                todo_medium!(
                    "connection_setup",
                    "Multi-pass authentication not implemented"
                );
                Ok(ConnectionSetupResponse {
                    status: ConnectionSetupStatus::Authenticate,
                    reason_length: data.len() as u8,
                    protocol_major_version: 11,
                    protocol_minor_version: 0,
                    additional_data_length: 0,
                    reason: String::from_utf8_lossy(&data).to_string(),
                    server_info: None,
                })
            }
        }
    }

    /// Authenticate a connection request
    async fn authenticate(
        &self,
        protocol_name: &str,
        protocol_data: &[u8],
    ) -> Result<AuthenticationResult> {
        debug!("Authenticating with protocol: '{}'", protocol_name);

        if let Some(auth_protocol) = self.auth_protocols.get(protocol_name) {
            auth_protocol.authenticate(protocol_data).await
        } else {
            warn!("Unknown authentication protocol: {}", protocol_name);
            Ok(AuthenticationResult::Failed(
                "Unknown authentication protocol".to_string(),
            ))
        }
    }

    /// Calculate the length of server info in 4-byte units
    fn calculate_server_info_length(&self) -> u16 {
        let mut length = 0u16;

        // Fixed part of server info (32 bytes)
        length += 8; // 8 * 4-byte units = 32 bytes

        // Vendor string (padded to 4-byte boundary)
        let vendor_len = self.server_info.vendor.len();
        length += ((vendor_len + 3) / 4) as u16;

        // Pixmap formats (3 bytes each, padded)
        let pixmap_formats_len = self.server_info.pixmap_formats.len() * 3;
        length += ((pixmap_formats_len + 3) / 4) as u16;

        // Screen info for each screen
        for screen in &self.server_info.screens {
            length += 10; // 40 bytes for fixed part of screen info

            // Allowed depths
            for depth in &screen.allowed_depths {
                length += 2; // 8 bytes for depth info

                // Visual types (24 bytes each)
                length += (depth.visuals.len() * 6) as u16; // 6 * 4-byte units = 24 bytes
            }
        }

        length
    }

    /// Create default server info
    fn create_default_server_info() -> ServerInfo {
        let vendor = "RX-Server (Rust X11)".to_string();

        ServerInfo {
            release: 11000000, // X11 Release
            resource_id_base: 0x00400000,
            resource_id_mask: 0x003fffff,
            motion_buffer_size: 256,
            vendor_length: vendor.len() as u16,
            maximum_request_length: 65535, // Maximum for u16
            number_of_screens: 1,
            number_of_pixmap_formats: 1,
            image_byte_order: 0,        // LSB first
            bitmap_format_bit_order: 0, // LSB first
            bitmap_format_scanline_unit: 32,
            bitmap_format_scanline_pad: 32,
            min_keycode: 8,
            max_keycode: 255,
            vendor,
            pixmap_formats: vec![PixmapFormat {
                depth: 24,
                bits_per_pixel: 32,
                scanline_pad: 32,
            }],
            screens: vec![ScreenInfo {
                root: 1, // Root window ID
                default_colormap: 2,
                white_pixel: 0xffffff,
                black_pixel: 0x000000,
                current_input_masks: 0,
                width_in_pixels: 1920,
                height_in_pixels: 1080,
                width_in_millimeters: 508, // ~96 DPI
                height_in_millimeters: 286,
                min_installed_maps: 1,
                max_installed_maps: 1,
                root_visual: 0x21,
                backing_stores: 0, // Never
                save_unders: false,
                root_depth: 24,
                allowed_depths_len: 1,
                allowed_depths: vec![DepthInfo {
                    depth: 24,
                    visuals_len: 1,
                    visuals: vec![VisualType {
                        visual_id: 0x21,
                        class: 4, // TrueColor
                        bits_per_rgb_value: 8,
                        colormap_entries: 256,
                        red_mask: 0x00ff0000,
                        green_mask: 0x0000ff00,
                        blue_mask: 0x000000ff,
                    }],
                }],
            }],
        }
    }

    /// Parse connection setup request from bytes
    pub fn parse_setup_request(data: &[u8]) -> Result<ConnectionSetupRequest> {
        if data.len() < 12 {
            return Err(crate::Error::Protocol(
                "Connection setup request too short".to_string(),
            ));
        }

        let byte_order = data[0];
        if byte_order != b'l' && byte_order != b'B' {
            return Err(crate::Error::Protocol("Invalid byte order".to_string()));
        }

        let is_little_endian = byte_order == b'l';

        let protocol_major_version = if is_little_endian {
            u16::from_le_bytes([data[2], data[3]])
        } else {
            u16::from_be_bytes([data[2], data[3]])
        };

        let protocol_minor_version = if is_little_endian {
            u16::from_le_bytes([data[4], data[5]])
        } else {
            u16::from_be_bytes([data[4], data[5]])
        };

        let auth_name_length = if is_little_endian {
            u16::from_le_bytes([data[6], data[7]])
        } else {
            u16::from_be_bytes([data[6], data[7]])
        } as usize;

        let auth_data_length = if is_little_endian {
            u16::from_le_bytes([data[8], data[9]])
        } else {
            u16::from_be_bytes([data[8], data[9]])
        } as usize;

        let expected_len = 12 + ((auth_name_length + 3) & !3) + ((auth_data_length + 3) & !3);
        if data.len() < expected_len {
            return Err(crate::Error::Protocol(
                "Connection setup request incomplete".to_string(),
            ));
        }

        let mut offset = 12;

        // Read auth protocol name
        let auth_name = if auth_name_length > 0 {
            String::from_utf8_lossy(&data[offset..offset + auth_name_length]).to_string()
        } else {
            String::new()
        };
        offset += (auth_name_length + 3) & !3; // Pad to 4-byte boundary

        // Read auth protocol data
        let auth_data = if auth_data_length > 0 {
            data[offset..offset + auth_data_length].to_vec()
        } else {
            Vec::new()
        };

        Ok(ConnectionSetupRequest {
            protocol_major_version,
            protocol_minor_version,
            authorization_protocol_name_length: auth_name_length as u16,
            authorization_protocol_data_length: auth_data_length as u16,
            authorization_protocol_name: auth_name,
            authorization_protocol_data: auth_data,
        })
    }

    /// Serialize connection setup response to bytes
    pub fn serialize_setup_response(&self, response: &ConnectionSetupResponse) -> Result<Vec<u8>> {
        let mut data = Vec::new();

        // Status (1 byte)
        data.push(response.status as u8);

        // Reason length (1 byte)
        data.push(response.reason_length);

        // Protocol version (4 bytes)
        data.extend_from_slice(&response.protocol_major_version.to_le_bytes());
        data.extend_from_slice(&response.protocol_minor_version.to_le_bytes());

        // Additional data length (2 bytes)
        data.extend_from_slice(&response.additional_data_length.to_le_bytes());

        // Reason string (padded to 4-byte boundary)
        if !response.reason.is_empty() {
            let reason_bytes = response.reason.as_bytes();
            data.extend_from_slice(reason_bytes);
            // Pad to 4-byte boundary
            let padding = (4 - (reason_bytes.len() % 4)) % 4;
            data.extend_from_slice(&vec![0; padding]);
        }

        // Server info (if successful)
        if let Some(ref server_info) = response.server_info {
            let server_info_start_len = data.len();
            debug!(
                "Starting server info serialization at byte {}",
                server_info_start_len
            );

            // Serialize the complete server info structure according to X11 protocol

            // Release number (4 bytes)
            data.extend_from_slice(&server_info.release.to_le_bytes());

            // Resource ID base (4 bytes)
            data.extend_from_slice(&server_info.resource_id_base.to_le_bytes());

            // Resource ID mask (4 bytes)
            data.extend_from_slice(&server_info.resource_id_mask.to_le_bytes());

            // Motion buffer size (4 bytes)
            data.extend_from_slice(&server_info.motion_buffer_size.to_le_bytes());

            // Vendor length (2 bytes)
            data.extend_from_slice(&(server_info.vendor.len() as u16).to_le_bytes());

            // Maximum request length (2 bytes)
            data.extend_from_slice(&server_info.maximum_request_length.to_le_bytes());

            // Number of screens (1 byte)
            data.push(server_info.number_of_screens);

            // Number of pixmap formats (1 byte)
            data.push(server_info.number_of_pixmap_formats);

            // Image byte order (1 byte)
            data.push(server_info.image_byte_order);

            // Bitmap format bit order (1 byte)
            data.push(server_info.bitmap_format_bit_order);

            // Bitmap format scanline unit (1 byte)
            data.push(server_info.bitmap_format_scanline_unit);

            // Bitmap format scanline pad (1 byte)
            data.push(server_info.bitmap_format_scanline_pad);

            // Min keycode (1 byte)
            data.push(server_info.min_keycode);

            // Max keycode (1 byte)
            data.push(server_info.max_keycode);

            // Unused padding (4 bytes)
            data.extend_from_slice(&[0u8; 4]);

            // Vendor string (padded to 4-byte boundary)
            data.extend_from_slice(server_info.vendor.as_bytes());
            let vendor_padding = (4 - (server_info.vendor.len() % 4)) % 4;
            data.extend_from_slice(&vec![0u8; vendor_padding]);

            // Pixmap formats (3 bytes each, padded to 4-byte boundary)
            for format in &server_info.pixmap_formats {
                data.push(format.depth);
                data.push(format.bits_per_pixel);
                data.push(format.scanline_pad);
                data.push(0); // Padding
            }

            // Screen information
            for screen in &server_info.screens {
                // Root window (4 bytes)
                data.extend_from_slice(&screen.root.to_le_bytes());

                // Default colormap (4 bytes)
                data.extend_from_slice(&screen.default_colormap.to_le_bytes());

                // White pixel (4 bytes)
                data.extend_from_slice(&screen.white_pixel.to_le_bytes());

                // Black pixel (4 bytes)
                data.extend_from_slice(&screen.black_pixel.to_le_bytes());

                // Current input masks (4 bytes)
                data.extend_from_slice(&screen.current_input_masks.to_le_bytes());

                // Width in pixels (2 bytes)
                data.extend_from_slice(&screen.width_in_pixels.to_le_bytes());

                // Height in pixels (2 bytes)
                data.extend_from_slice(&screen.height_in_pixels.to_le_bytes());

                // Width in millimeters (2 bytes)
                data.extend_from_slice(&screen.width_in_millimeters.to_le_bytes());

                // Height in millimeters (2 bytes)
                data.extend_from_slice(&screen.height_in_millimeters.to_le_bytes());

                // Min installed maps (2 bytes)
                data.extend_from_slice(&screen.min_installed_maps.to_le_bytes());

                // Max installed maps (2 bytes)
                data.extend_from_slice(&screen.max_installed_maps.to_le_bytes());

                // Root visual (4 bytes)
                data.extend_from_slice(&screen.root_visual.to_le_bytes());

                // Backing stores (1 byte)
                data.push(screen.backing_stores);

                // Save unders (1 byte)
                data.push(if screen.save_unders { 1 } else { 0 });

                // Root depth (1 byte)
                data.push(screen.root_depth);

                // Number of allowed depths (1 byte)
                data.push(screen.allowed_depths.len() as u8);

                // Allowed depths
                for depth in &screen.allowed_depths {
                    // Depth (1 byte)
                    data.push(depth.depth);

                    // Unused (1 byte)
                    data.push(0);

                    // Number of visuals (2 bytes)
                    data.extend_from_slice(&(depth.visuals.len() as u16).to_le_bytes());

                    // Unused (4 bytes)
                    data.extend_from_slice(&[0u8; 4]);

                    // Visual types
                    for visual in &depth.visuals {
                        // Visual ID (4 bytes)
                        data.extend_from_slice(&visual.visual_id.to_le_bytes());

                        // Class (1 byte)
                        data.push(visual.class);

                        // Bits per RGB value (1 byte)
                        data.push(visual.bits_per_rgb_value);

                        // Colormap entries (2 bytes)
                        data.extend_from_slice(&visual.colormap_entries.to_le_bytes());

                        // Red mask (4 bytes)
                        data.extend_from_slice(&visual.red_mask.to_le_bytes());

                        // Green mask (4 bytes)
                        data.extend_from_slice(&visual.green_mask.to_le_bytes());

                        // Blue mask (4 bytes)
                        data.extend_from_slice(&visual.blue_mask.to_le_bytes());

                        // Unused (4 bytes)
                        data.extend_from_slice(&[0u8; 4]);
                    }
                }
            }

            let server_info_end_len = data.len();
            let actual_server_info_len = server_info_end_len - server_info_start_len;
            let calculated_len = self.calculate_server_info_length() as usize * 4;
            debug!("Server info serialization complete:");
            debug!(
                "  Actual serialized length: {} bytes",
                actual_server_info_len
            );
            debug!("  Calculated length: {} bytes", calculated_len);
            debug!(
                "  Length match: {}",
                actual_server_info_len == calculated_len
            );
        }

        debug!("Total handshake response length: {} bytes", data.len());
        Ok(data)
    }
}

/// Authentication result
#[derive(Debug)]
pub enum AuthenticationResult {
    /// Authentication successful
    Success,
    /// Authentication failed with reason
    Failed(String),
    /// Multi-pass authentication required with data
    AuthenticateRequired(Vec<u8>),
}

/// Authentication protocol trait
#[async_trait::async_trait]
pub trait AuthenticationProtocol: Send + Sync {
    /// Authenticate using this protocol
    async fn authenticate(&self, data: &[u8]) -> Result<AuthenticationResult>;
}

/// No authentication protocol (empty auth name)
struct NoAuthProtocol;

#[async_trait::async_trait]
impl AuthenticationProtocol for NoAuthProtocol {
    async fn authenticate(&self, _data: &[u8]) -> Result<AuthenticationResult> {
        debug!("No authentication - allowing connection");
        Ok(AuthenticationResult::Success)
    }
}

/// MIT-MAGIC-COOKIE-1 authentication protocol
struct MitMagicCookieProtocol {
    /// Valid cookies
    valid_cookies: Vec<Vec<u8>>,
}

impl MitMagicCookieProtocol {
    fn new() -> Self {
        Self::new_with_cookies(Self::load_cookies())
    }

    /// Create a new MIT-MAGIC-COOKIE-1 protocol with specific cookies
    fn new_with_cookies(cookies: Vec<Vec<u8>>) -> Self {
        Self {
            valid_cookies: cookies,
        }
    }

    /// Load cookies from various sources
    fn load_cookies() -> Vec<Vec<u8>> {
        let mut cookies = Vec::new();

        // Try to load from XAUTHORITY environment variable
        if let Some(xauth_cookies) = Self::load_from_xauthority() {
            cookies.extend(xauth_cookies);
        }

        // Try to load from .Xauthority file in home directory
        if let Some(home_cookies) = Self::load_from_home_xauthority() {
            cookies.extend(home_cookies);
        }

        // If no cookies found, generate a temporary one for development
        if cookies.is_empty() {
            warn!("No X11 auth cookies found, generating temporary development cookie");
            cookies.push(Self::generate_temporary_cookie());
        }

        info!(
            "Loaded {} MIT-MAGIC-COOKIE-1 authentication cookies",
            cookies.len()
        );
        cookies
    }

    /// Try to load cookies from XAUTHORITY environment variable
    fn load_from_xauthority() -> Option<Vec<Vec<u8>>> {
        if let Ok(xauth_path) = std::env::var("XAUTHORITY") {
            debug!(
                "Attempting to load X11 auth from XAUTHORITY: {}",
                xauth_path
            );
            Self::parse_xauthority_file(&xauth_path)
        } else {
            debug!("XAUTHORITY environment variable not set");
            None
        }
    }

    /// Try to load cookies from ~/.Xauthority
    fn load_from_home_xauthority() -> Option<Vec<Vec<u8>>> {
        if let Some(home_dir) = dirs::home_dir() {
            let xauth_path = home_dir.join(".Xauthority");
            if xauth_path.exists() {
                debug!("Attempting to load X11 auth from ~/.Xauthority");
                return Self::parse_xauthority_file(xauth_path.to_str()?);
            }
        }
        debug!("~/.Xauthority file not found");
        None
    }

    /// Parse .Xauthority file format
    fn parse_xauthority_file(path: &str) -> Option<Vec<Vec<u8>>> {
        use std::fs::File;
        use std::io::{BufReader, Read};

        let file = File::open(path).ok()?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();

        if reader.read_to_end(&mut buffer).is_err() {
            warn!("Failed to read .Xauthority file: {}", path);
            return None;
        }

        Self::parse_xauthority_data(&buffer)
    }

    /// Parse .Xauthority binary data format
    fn parse_xauthority_data(data: &[u8]) -> Option<Vec<Vec<u8>>> {
        let mut cookies = Vec::new();
        let mut offset = 0;

        while offset + 10 < data.len() {
            // Read family (2 bytes, big-endian)
            let family = u16::from_be_bytes([data[offset], data[offset + 1]]);
            offset += 2;

            // Read address length (2 bytes, big-endian)
            let addr_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;

            // Skip address
            if offset + addr_len >= data.len() {
                break;
            }
            offset += addr_len;

            // Read display length (2 bytes, big-endian)
            if offset + 2 >= data.len() {
                break;
            }
            let display_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;

            // Skip display
            if offset + display_len >= data.len() {
                break;
            }
            offset += display_len;

            // Read protocol name length (2 bytes, big-endian)
            if offset + 2 >= data.len() {
                break;
            }
            let proto_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;

            // Read protocol name
            if offset + proto_len >= data.len() {
                break;
            }
            let protocol_name = &data[offset..offset + proto_len];
            offset += proto_len;

            // Read cookie length (2 bytes, big-endian)
            if offset + 2 >= data.len() {
                break;
            }
            let cookie_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;

            // Read cookie data
            if offset + cookie_len > data.len() {
                break;
            }
            let cookie_data = &data[offset..offset + cookie_len];
            offset += cookie_len;

            // Check if this is a MIT-MAGIC-COOKIE-1 entry
            if protocol_name == b"MIT-MAGIC-COOKIE-1" && cookie_len == 16 {
                cookies.push(cookie_data.to_vec());
                debug!("Found MIT-MAGIC-COOKIE-1 cookie in .Xauthority");
            }
        }

        if cookies.is_empty() {
            None
        } else {
            Some(cookies)
        }
    }

    /// Generate a temporary cookie for development purposes
    fn generate_temporary_cookie() -> Vec<u8> {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Generate a pseudo-random cookie based on current time
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let mut cookie = Vec::with_capacity(16);

        // Mix timestamp with some constants to create a 16-byte cookie
        cookie.extend_from_slice(&timestamp.to_le_bytes());
        cookie.extend_from_slice(&0x42525853u32.to_le_bytes()); // "RXSB" in little endian
        cookie.extend_from_slice(&0xDEADBEEFu32.to_le_bytes());

        debug!("Generated temporary development cookie");
        cookie
    }

    /// Add a new valid cookie
    pub fn add_cookie(&mut self, cookie: Vec<u8>) -> Result<()> {
        if cookie.len() != 16 {
            return Err(crate::Error::Protocol(
                "MIT-MAGIC-COOKIE-1 requires 16-byte cookie".to_string(),
            ));
        }
        self.valid_cookies.push(cookie);
        Ok(())
    }

    /// Remove all cookies
    pub fn clear_cookies(&mut self) {
        self.valid_cookies.clear();
    }

    /// Get the number of valid cookies
    pub fn cookie_count(&self) -> usize {
        self.valid_cookies.len()
    }
}

#[async_trait::async_trait]
impl AuthenticationProtocol for MitMagicCookieProtocol {
    async fn authenticate(&self, data: &[u8]) -> Result<AuthenticationResult> {
        debug!(
            "MIT-MAGIC-COOKIE-1 authentication with {} bytes",
            data.len()
        );

        if data.len() != 16 {
            return Ok(AuthenticationResult::Failed(
                "MIT-MAGIC-COOKIE-1 requires 16-byte cookie".to_string(),
            ));
        }

        // Check if cookie is valid
        for valid_cookie in &self.valid_cookies {
            if data == valid_cookie.as_slice() {
                debug!("MIT-MAGIC-COOKIE-1 authentication successful");
                return Ok(AuthenticationResult::Success);
            }
        }

        warn!("MIT-MAGIC-COOKIE-1 authentication failed - invalid cookie");
        Ok(AuthenticationResult::Failed("Invalid cookie".to_string()))
    }
}
