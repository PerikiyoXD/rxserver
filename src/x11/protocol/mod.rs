//! X11 Wire Protocol Implementation
//!
//! This module handles the X11 wire protocol, including message parsing,
//! serialization, and protocol validation.

pub mod endianness;
pub mod errors;
pub mod opcodes;
pub mod parser;
pub mod serializer;
pub mod types;
pub mod validation;
pub mod wire;

// Re-export commonly used types
pub use errors::{ProtocolError, X11Error};
pub use opcodes::Opcode;
pub use types::*;
pub use wire::{MessageHeader, WireFormat};

use crate::network::ConnectionId;
use crate::server::configuration::ServerConfig;
use crate::x11::protocol::parser::ProtocolParser;
use crate::x11::resources::types::atom::AtomRegistry;
use crate::x11::security::SecurityManager;
use crate::{fonts::FontRegistry, x11::resources::types::window::BackingStore};
use std::sync::{Arc, Mutex};

/// X11 Protocol handler for processing requests and generating responses
#[derive(Debug)]
pub struct ProtocolHandler {
    /// Connection setup state
    setup_complete: std::collections::HashMap<ConnectionId, bool>,
    /// Protocol version for each connection
    versions: std::collections::HashMap<ConnectionId, (u16, u16)>,
    /// Protocol parsers for each connection (different byte orders)
    parsers: std::collections::HashMap<ConnectionId, ProtocolParser>,
    /// Server configuration for generating setup responses
    server_config: Option<Arc<ServerConfig>>,
    /// Atom registry for managing atoms
    atom_registry: Arc<Mutex<AtomRegistry>>,
    /// Font registry for managing fonts
    font_registry: Arc<Mutex<FontRegistry>>,
    /// Security manager for authentication and authorization
    security_manager: Arc<Mutex<SecurityManager>>,
}

impl ProtocolHandler {
    /// Create a new protocol handler
    pub fn new() -> Self {
        Self {
            setup_complete: std::collections::HashMap::new(),
            versions: std::collections::HashMap::new(),
            parsers: std::collections::HashMap::new(),
            server_config: None,
            atom_registry: Arc::new(Mutex::new(AtomRegistry::new())),
            font_registry: Arc::new(Mutex::new(FontRegistry::new())),
            security_manager: Arc::new(Mutex::new(SecurityManager::new())),
        }
    }
    /// Create a new protocol handler with server configuration
    pub fn with_config(config: Arc<ServerConfig>) -> Self {
        Self {
            setup_complete: std::collections::HashMap::new(),
            versions: std::collections::HashMap::new(),
            parsers: std::collections::HashMap::new(),
            server_config: Some(config),
            atom_registry: Arc::new(Mutex::new(AtomRegistry::new())),
            font_registry: Arc::new(Mutex::new(FontRegistry::new())),
            security_manager: Arc::new(Mutex::new(SecurityManager::new())),
        }
    }

    /// Set the server configuration
    pub fn set_config(&mut self, config: Arc<ServerConfig>) {
        self.server_config = Some(config);
    }

    /// Process incoming data from a connection
    pub async fn process_data(
        &mut self,
        connection_id: ConnectionId,
        data: &[u8],
    ) -> Result<Vec<u8>, ProtocolError> {
        if !self.setup_complete.get(&connection_id).unwrap_or(&false) {
            // Handle connection setup
            self.handle_connection_setup(connection_id, data).await
        } else {
            // Handle X11 requests
            self.handle_request(connection_id, data).await
        }
    }
    /// Handle X11 connection setup
    async fn handle_connection_setup(
        &mut self,
        connection_id: ConnectionId,
        data: &[u8],
    ) -> Result<Vec<u8>, ProtocolError> {
        // Any data less than 12 bytes is invalid for setup
        if data.len() < 12 {
            return Err(ProtocolError::InsufficientData);
        }

        // Determine byte order from first byte
        let byte_order = match data[0] {
            b'l' => ByteOrder::LittleEndian,
            b'B' => ByteOrder::BigEndian,
            invalid => return Err(ProtocolError::InvalidByteOrder(invalid)),
        };

        // Create parser for this connection
        let parser: ProtocolParser = ProtocolParser::new(byte_order);

        // Parse connection setup request using the parser
        let setup_request: SetupRequest = parser.parse_setup_request(data)?;

        tracing::debug!("[conn {}] {:?}", connection_id, setup_request);

        // Authenticate the client using SecurityManager
        {
            let mut security_manager = self.security_manager.lock().unwrap();

            // Convert connection_id to ClientId for authentication
            let client_id = connection_id as u32;

            // Attempt authentication based on the authorization protocol
            let auth_method = if setup_request.authorization_protocol_name.is_empty() {
                "" // No authentication
            } else {
                &setup_request.authorization_protocol_name
            };

            // Note: In a real implementation, you would pass the authorization_protocol_data
            // to the authentication method for validation
            tracing::debug!(
                "Authenticating client {} with method '{}' and {} bytes of auth data",
                client_id,
                auth_method,
                setup_request.authorization_protocol_data.len()
            );

            // Attempt authentication
            if let Err(auth_error) = security_manager.authenticate_client(client_id) {
                tracing::warn!(
                    "Authentication failed for client {}: {:?}",
                    client_id,
                    auth_error
                );

                // For now, we'll continue processing even if authentication fails
                // In a production environment, you might want to return an authentication failure
                // return Err(ProtocolError::AuthenticationFailed);
            } else {
                tracing::debug!("Client {} authenticated successfully", client_id);
            }

            // Check authorization for setup operation
            if let Err(authz_error) = security_manager.authorize_operation(client_id, "setup") {
                tracing::warn!(
                    "Authorization failed for client {} operation 'setup': {:?}",
                    client_id,
                    authz_error
                );

                // For now, we'll continue processing even if authorization fails
                // In a production environment, you might want to return an authorization failure
                // return Err(ProtocolError::AuthorizationDenied);
            } else {
                tracing::debug!("Client {} authorized for setup operation", client_id);
            }
        }

        // Store parser and version information for this connection
        self.parsers.insert(connection_id, parser);
        self.versions.insert(
            connection_id,
            (
                setup_request.protocol_major_version,
                setup_request.protocol_minor_version,
            ),
        );

        // Generate setup response
        let setup_response = self.generate_setup_response(&setup_request)?;
        self.setup_complete.insert(connection_id, true);

        let serialized = setup_response.serialize();
        tracing::debug!(
            "Generated setup response for connection {}: {} bytes, vendor: '{}', screens: {}, formats: {}",
            connection_id,
            serialized.len(),
            setup_response.vendor,
            setup_response.number_of_screens,
            setup_response.number_of_formats
        );

        //tracing::trace!("Generated setup response data: {:02X?}", serialized);

        // DEBUG! Print the length of the response
        tracing::debug!(
            "Setup response length for connection {}: {} bytes",
            connection_id,
            serialized.len()
        );

        Ok(serialized)
    }
    /// Handle X11 protocol requests (supports sequential requests in same buffer)
    async fn handle_request(
        &mut self,
        connection_id: ConnectionId,
        data: &[u8],
    ) -> Result<Vec<u8>, ProtocolError> {
        let mut offset = 0;
        let mut responses = Vec::new();

        // Process sequential requests in the data buffer
        while offset < data.len() {
            // Need at least 4 bytes for a request header
            if offset + 4 > data.len() {
                if offset == 0 {
                    return Err(ProtocolError::InsufficientData);
                }
                // Partial request at the end - this shouldn't happen in well-formed X11 streams
                tracing::warn!(
                    "Partial request data at end of buffer: {} bytes remaining",
                    data.len() - offset
                );
                break;
            }

            let request_data = &data[offset..];
            let header = RequestHeader::parse(request_data)?;

            // Calculate the total request size in bytes (length field is in 4-byte units)
            let request_size_bytes = (header.length as usize) * 4;

            // Ensure we have enough data for the complete request
            if offset + request_size_bytes > data.len() {
                tracing::warn!(
                    "Incomplete request: expected {} bytes, got {} bytes",
                    request_size_bytes,
                    data.len() - offset
                );
                return Err(ProtocolError::InsufficientData);
            }

            // Extract the complete request data
            let complete_request_data = &data[offset..offset + request_size_bytes];

            let opcode = Opcode::from_u8(header.opcode);

            tracing::trace!(
                "Request from {} at offset {}: opcode={:?} ({}), length={} ({} bytes)",
                connection_id,
                offset,
                opcode,
                header.opcode,
                header.length,
                request_size_bytes
            );

            let parser = self
                .parsers
                .get(&connection_id)
                .ok_or(ProtocolError::InvalidFormat)?;

            // Process the individual request
            let response = match opcode {
                Some(Opcode::NoOperation) => {
                    self.handle_no_operation(connection_id, &header, complete_request_data)
                        .await?
                }
                Some(Opcode::QueryExtension) => {
                    self.handle_query_extension(connection_id, &header, complete_request_data)
                        .await?
                }
                Some(op) => match parser.parse_request(complete_request_data) {
                    Ok(request) => {
                        tracing::debug!("Successfully parsed request {:?}: {:?}", op, request);
                        self.handle_parsed_request(connection_id, &header, request)
                            .await?
                    }
                    Err(e) => {
                        tracing::warn!("Parse failed for opcode {}: {}", header.opcode, e);
                        return Err(ProtocolError::BadRequest);
                    }
                },
                None => {
                    tracing::warn!("Unknown opcode: {}", header.opcode);
                    return Err(ProtocolError::BadRequest);
                }
            };

            // Add non-empty responses to the response buffer
            if !response.is_empty() {
                responses.extend_from_slice(&response);
            }

            // Move to the next request
            offset += request_size_bytes;
        }
        tracing::debug!(
            "Processed {} bytes of request data, generated {} bytes of responses",
            data.len(),
            responses.len()
        );

        Ok(responses)
    }

    /// Generate connection setup response
    fn generate_setup_response(
        &self,
        _request: &SetupRequest,
    ) -> Result<ConnectionSetupSuccess, ProtocolError> {
        // Get server configuration or use defaults
        let config = self.server_config.as_ref();
        // Extract configuration values or use sensible defaults
        let (vendor, release_number, screen_width, screen_height, color_depth) =
            if let Some(cfg) = config {
                // Get display config or use defaults
                let display_config = cfg.display.as_ref();
                let (width, height, depth) = if let Some(display) = display_config {
                    let depth_value = match display.default_color_depth {
                        crate::display::types::ColorDepth::Depth1 => 1,
                        crate::display::types::ColorDepth::Depth4 => 4,
                        crate::display::types::ColorDepth::Depth8 => 8,
                        crate::display::types::ColorDepth::Depth15 => 15,
                        crate::display::types::ColorDepth::Depth16 => 16,
                        crate::display::types::ColorDepth::Depth24 => 24,
                        crate::display::types::ColorDepth::Depth32 => 32,
                    };
                    (
                        display.default_resolution.width,
                        display.default_resolution.height,
                        depth_value,
                    )
                } else {
                    (1920, 1080, 24)
                };

                (
                    "RXServer".to_string(), // Use default vendor as the config doesn't have it
                    1,                      // Use default release
                    width,
                    height,
                    depth,
                )
            } else {
                ("RXServer".to_string(), 1, 1920, 1080, 24)
            };

        // Calculate physical dimensions from resolution (assume 96 DPI)
        let dpi = 96.0;
        let width_mm = ((screen_width as f32 / dpi) * 25.4) as u16;
        let height_mm = ((screen_height as f32 / dpi) * 25.4) as u16;

        // Create basic visual formats (common for most displays)
        let pixmap_formats = vec![
            PixmapFormat {
                depth: 1,
                bits_per_pixel: 1,
                scanline_pad: 32,
            },
            PixmapFormat {
                depth: 8,
                bits_per_pixel: 8,
                scanline_pad: 32,
            },
            PixmapFormat {
                depth: 15,
                bits_per_pixel: 16,
                scanline_pad: 32,
            },
            PixmapFormat {
                depth: 16,
                bits_per_pixel: 16,
                scanline_pad: 32,
            },
            PixmapFormat {
                depth: 24,
                bits_per_pixel: 32,
                scanline_pad: 32,
            },
            PixmapFormat {
                depth: 32,
                bits_per_pixel: 32,
                scanline_pad: 32,
            },
        ];

        // Create appropriate visual based on color depth
        let (visual_class, bits_per_rgb, red_mask, green_mask, blue_mask, colormap_entries) =
            match color_depth {
                1 => (0, 1, 0, 0, 0, 2),                            // StaticGray
                8 => (3, 8, 0, 0, 0, 256),                          // PseudoColor
                15 => (4, 5, 0x7c00, 0x03e0, 0x001f, 0),            // TrueColor 15-bit
                16 => (4, 6, 0xf800, 0x07e0, 0x001f, 0),            // TrueColor 16-bit
                24 | 32 => (4, 8, 0xff0000, 0x00ff00, 0x0000ff, 0), // TrueColor 24/32-bit
                _ => (4, 8, 0xff0000, 0x00ff00, 0x0000ff, 0),       // Default to 24-bit TrueColor
            };

        let default_visual = Visual {
            visual_id: 0x21, // Common visual ID
            class: visual_class,
            bits_per_rgb_value: bits_per_rgb,
            colormap_entries,
            red_mask,
            green_mask,
            blue_mask,
        };

        // Create depth information based on color depth
        let depths = vec![Depth {
            depth: color_depth as u8,
            visuals: vec![default_visual],
        }];

        // Create the main screen with configuration-based values
        let screen = Screen {
            root: 1,             // Root window XID
            default_colormap: 2, // Default colormap XID
            white_pixel: if color_depth == 1 { 1 } else { 0xffffff },
            black_pixel: 0x000000,
            current_input_masks: 0,
            width_in_pixels: screen_width as u16,
            height_in_pixels: screen_height as u16,
            width_in_millimeters: width_mm,
            height_in_millimeters: height_mm,
            min_installed_maps: 1,
            max_installed_maps: 1,
            root_visual: 0x21,                   // Match the visual ID above
            backing_stores: BackingStore::Never, // Never
            save_unders: 0,                      // False
            root_depth: color_depth as u8,
            allowed_depths: depths,
        };

        let setup_response = ConnectionSetupSuccess {
            success: 1, // Success
            protocol_major_version: 11,
            protocol_minor_version: 0,
            additional_data_length: 0, // Will be calculated in serialize()
            release_number,
            resource_id_base: 0x400000,
            resource_id_mask: 0x3fffff,
            motion_buffer_size: 0,
            vendor_length: vendor.len() as u16, // Correctly calculate vendor string length
            maximum_request_length: 65535,      // Maximum value for u16
            number_of_screens: 1,
            number_of_formats: pixmap_formats.len() as u8,
            image_byte_order: 0,        // LSBFirst
            bitmap_format_bit_order: 0, // LeastSignificant
            bitmap_format_scanline_unit: 32,
            bitmap_format_scanline_pad: 32,
            min_keycode: 8,
            max_keycode: 255,
            vendor,
            pixmap_formats,
            screens: vec![screen],
        };

        Ok(setup_response)
    }

    /// Handle NoOperation request
    async fn handle_no_operation(
        &mut self,
        connection_id: ConnectionId,
        header: &RequestHeader,
        data: &[u8],
    ) -> Result<Vec<u8>, ProtocolError> {
        // NoOperation doesn't generate a response
        tracing::debug!(
            "NoOperation request received from connection: {:?}, header: {:?}, data: {:02X?}",
            connection_id,
            header,
            data
        );
        Ok(Vec::new())
    }

    /// Handle QueryExtension request
    async fn handle_query_extension(
        &mut self,
        connection_id: ConnectionId,
        header: &RequestHeader,
        data: &[u8],
    ) -> Result<Vec<u8>, ProtocolError> {
        tracing::debug!(
            "QueryExtension request from connection: {:?}, header: {:?}, data: {:02X?}",
            connection_id,
            header,
            data
        );
        if data.len() < 8 {
            return Err(ProtocolError::InsufficientData);
        }

        // Parse extension name length
        let name_length = u16::from_le_bytes([data[4], data[5]]) as usize;

        if data.len() < 8 + name_length {
            return Err(ProtocolError::InsufficientData);
        }

        let extension_name = String::from_utf8_lossy(&data[8..8 + name_length]);

        tracing::debug!("QueryExtension request for: {}", extension_name);

        // For Phase 1, no extensions are supported
        let response = QueryExtensionResponse {
            present: 0, // False
            major_opcode: 0,
            first_event: 0,
            first_error: 0,
        };
        Ok(response.serialize(header.sequence_number))
    }

    /// Handle parsed X11 requests
    async fn handle_parsed_request(
        &mut self,
        connection_id: ConnectionId,
        header: &RequestHeader,
        request: Request,
    ) -> Result<Vec<u8>, ProtocolError> {
        match request {
            Request::InternAtom {
                only_if_exists,
                name,
            } => {
                self.handle_intern_atom_request(connection_id, header, only_if_exists, name)
                    .await
            }
            Request::NoOperation => {
                // NoOperation doesn't generate a response
                Ok(Vec::new())
            }
            Request::OpenFont { font_id, name } => {
                self.handle_open_font_request(connection_id, header, font_id, name)
                    .await
            }
            Request::CreateGlyphCursor {
                cursor_id,
                source_font,
                mask_font,
                source_char,
                mask_char,
                fore_red,
                fore_green,
                fore_blue,
                back_red,
                back_green,
                back_blue,
            } => {
                self.handle_create_glyph_cursor_request(
                    connection_id,
                    header,
                    cursor_id,
                    source_font,
                    mask_font,
                    source_char,
                    mask_char,
                    fore_red,
                    fore_green,
                    fore_blue,
                    back_red,
                    back_green,
                    back_blue,
                )
                .await
            }

            _ => {
                tracing::debug!("Unimplemented request: {:?}", request);
                // For Phase 1, return empty response for unimplemented requests
                Ok(Vec::new())
            }
        }
    }
    /// Handle InternAtom request
    async fn handle_intern_atom_request(
        &mut self,
        connection_id: ConnectionId,
        header: &RequestHeader,
        only_if_exists: bool,
        name: String,
    ) -> Result<Vec<u8>, ProtocolError> {
        tracing::debug!(
            "InternAtom request: name='{}', only_if_exists={}",
            name,
            only_if_exists
        );

        let atom_id = {
            let mut registry = self.atom_registry.lock().unwrap();
            if let Some(atom) = registry.intern_atom(&name, only_if_exists, connection_id) {
                atom
            } else {
                0 // Return None (0) if only_if_exists=true and atom doesn't exist
            }
        };

        tracing::debug!("InternAtom response: atom_id={}", atom_id);

        // Create InternAtom response
        let response_data = self.serialize_intern_atom_response(header.sequence_number, atom_id)?;
        Ok(response_data)
    }

    /// Serialize InternAtom response
    fn serialize_intern_atom_response(
        &self,
        sequence_number: u16,
        atom_id: XID,
    ) -> Result<Vec<u8>, ProtocolError> {
        let mut response = Vec::with_capacity(32);

        // Response header
        response.push(1); // Reply type
        response.push(0); // Unused
        response.extend_from_slice(&sequence_number.to_le_bytes()); // Sequence number
        response.extend_from_slice(&0u32.to_le_bytes()); // Length (always 0 for this response)
        response.extend_from_slice(&atom_id.to_le_bytes()); // Atom ID

        // Pad to 32 bytes total
        response.resize(32, 0);

        Ok(response)
    }
    /// Handle OpenFont request
    async fn handle_open_font_request(
        &mut self,
        connection_id: ConnectionId,
        _header: &RequestHeader,
        font_id: XID,
        name: String,
    ) -> Result<Vec<u8>, ProtocolError> {
        tracing::debug!("OpenFont request: font_id={}, name='{}'", font_id, name);

        // Use the font registry to open the font
        let mut registry = self
            .font_registry
            .lock()
            .map_err(|_| ProtocolError::BadRequest)?;

        match registry.open_font(font_id, &name, connection_id) {
            Ok(()) => {
                tracing::info!(
                    "Successfully opened font '{}' with ID {} for client {:?}",
                    name,
                    font_id,
                    connection_id
                );
                // OpenFont has no response body, just success
                Ok(Vec::new())
            }
            Err(e) => {
                tracing::warn!("Failed to open font '{}': {}", name, e);
                Err(ProtocolError::BadRequest)
            }
        }
    }

    /// Handle CreateGlyphCursor request
    async fn handle_create_glyph_cursor_request(
        &mut self,
        connection_id: ConnectionId,
        _header: &RequestHeader,
        cursor_id: XID,
        source_font: XID,
        mask_font: XID,
        source_char: u16,
        mask_char: u16,
        fore_red: u16,
        fore_green: u16,
        fore_blue: u16,
        back_red: u16,
        back_green: u16,
        back_blue: u16,
    ) -> Result<Vec<u8>, ProtocolError> {
        tracing::debug!(
            "CreateGlyphCursor request: cursor_id={}, source_font={}, mask_font={}, source_char={}, mask_char={}, fore_red={}, fore_green={}, fore_blue={}, back_red={}, back_green={}, back_blue={}",
            cursor_id,
            source_font,
            mask_font,
            source_char,
            mask_char,
            fore_red,
            fore_green,
            fore_blue,
            back_red,
            back_green,
            back_blue
        );

        // Use the cursor registry to create the glyph cursor
        let mut registry = self
            .cursor_registry
            .lock()
            .map_err(|_| ProtocolError::BadRequest)?;

        match registry.create_glyph_cursor(
            cursor_id,
            source_font,
            mask_font,
            source_char,
            mask_char,
            fore_red,
            fore_green,
            fore_blue,
            back_red,
            back_green,
            back_blue,
            connection_id,
        ) {
            Ok(()) => {
                tracing::info!(
                    "Successfully created glyph cursor {:?} for client {:?}",
                    cursor_id,
                    connection_id
                );
                // CreateGlyphCursor has no response body, just success
                Ok(Vec::new())
            }
            Err(e) => {
                tracing::warn!("Failed to create glyph cursor {:?}: {}", cursor_id, e);
                Err(ProtocolError::BadRequest)
            }
        }
    }

    /// Remove connection state when client disconnects
    pub fn remove_connection(&mut self, connection_id: ConnectionId) {
        tracing::debug!("Removing connection state for: {}", connection_id);

        // Clean up connection state
        self.setup_complete.remove(&connection_id);
        self.versions.remove(&connection_id);
        self.parsers.remove(&connection_id);

        // Clean up opened fonts for this client
        if let Ok(mut registry) = self.font_registry.lock() {
            let closed_fonts = registry.close_client_fonts(connection_id);
            if !closed_fonts.is_empty() {
                tracing::info!(
                    "Closed {} fonts for disconnected client {:?}",
                    closed_fonts.len(),
                    connection_id
                );
            }
        }
    }

    /// Get a reference to the security manager
    pub fn security_manager(&self) -> Arc<Mutex<SecurityManager>> {
        self.security_manager.clone()
    }
    /// Configure the security manager with authentication methods
    pub fn configure_security(&mut self, auth_methods: Vec<String>) {
        // This is a placeholder for more sophisticated configuration
        // In a real implementation, you would configure specific auth methods
        tracing::info!(
            "Configured security manager with auth methods: {:?}",
            auth_methods
        );

        // Example: Enable MIT-MAGIC-COOKIE-1 if requested
        if auth_methods.contains(&"MIT-MAGIC-COOKIE-1".to_string()) {
            // Configure MIT-MAGIC-COOKIE-1 authentication
            // This would involve setting up the cookie authentication
            tracing::debug!("MIT-MAGIC-COOKIE-1 authentication enabled");
        }
    }

    /// Set up MIT-MAGIC-COOKIE-1 authentication with a specific cookie
    pub fn setup_mit_cookie_auth(&mut self, cookie: Vec<u8>) {
        // This would be implemented to configure the SecurityManager's AuthenticationManager
        // with the provided cookie for MIT-MAGIC-COOKIE-1 authentication
        tracing::info!(
            "Setting up MIT-MAGIC-COOKIE-1 authentication with {} byte cookie",
            cookie.len()
        );

        // In a real implementation, you would:
        // 1. Access the security manager's authentication manager
        // 2. Configure it with the provided cookie
        // 3. Set MIT-MAGIC-COOKIE-1 as an available authentication method
    }

    /// Enable host-based authentication
    pub fn enable_host_based_auth(&mut self) {
        tracing::info!("Enabling host-based authentication");

        // In a real implementation, you would:
        // 1. Access the security manager's authentication manager
        // 2. Enable host-based authentication
        // 3. Configure allowed hosts if needed
    }
}

impl Default for ProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}
