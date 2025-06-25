//! Connection handler implementing the X11 connection state machine

use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error, info, trace};

use crate::protocol::{ByteOrder, Request, RequestValidator, X11Error, X11RequestValidator, RequestHandlerRegistry, create_standard_handler_registry, EndianWriter};
use crate::server::state::{ClientId, ClientState, ServerState};

/// Connection handler that manages the complete lifecycle of an X11 client connection
pub struct ConnectionHandler {
    /// Global server state
    server_state: Arc<Mutex<ServerState>>,
    /// This client's state
    client_state: Arc<Mutex<ClientState>>,
    /// Client ID
    client_id: ClientId,
    /// TCP stream
    stream: TcpStream,
    /// Request handler registry
    handler_registry: RequestHandlerRegistry,
}

impl ConnectionHandler {
    /// Create a new connection handler
    pub fn new(server_state: Arc<Mutex<ServerState>>, stream: TcpStream) -> Result<Self, X11Error> {
        let peer_addr = stream.peer_addr().map_err(|e| X11Error::Io(e))?;
        trace!("info: creating handler for {}", peer_addr);

        let (client_id, client_state) = {
            let mut server = server_state
                .lock()
                .map_err(|_| X11Error::Protocol("Failed to lock server state".to_string()))?;
            server.register_client(peer_addr)
        };        debug!(
            "info: handler created client {} from {}",
            client_id, peer_addr
        );
        Ok(Self {
            server_state,
            client_state,
            client_id,
            stream,
            handler_registry: create_standard_handler_registry(),
        })
    }

    /// Main connection handling loop implementing the state machine
    pub async fn handle(mut self) -> Result<(), X11Error> {
        info!(
            "info: client {} connected from {}",
            self.client_id,
            self.stream
                .peer_addr()
                .unwrap_or_else(|_| "unknown".parse().unwrap())
        );

        let mut buffer = vec![0u8; 4096];
        trace!(
            "info: starting loop client {} buffer {}B",
            self.client_id,
            buffer.len()
        );

        loop {
            trace!("trace: waiting data client {}", self.client_id);
            let bytes_read = match self.stream.read(&mut buffer).await {
                Ok(0) => {
                    info!("info: client {} disconnected", self.client_id);
                    break;
                }
                Ok(n) => {
                    trace!(
                        "trace: read {}B client {} ({:.1}%)",
                        n,
                        self.client_id,
                        (n as f32 / buffer.len() as f32) * 100.0
                    );
                    n
                }
                Err(e) => {
                    error!("error: io client {}: {:?}", self.client_id, e);
                    break;
                }
            };

            let data = &buffer[..bytes_read];
            trace!(
                "trace: processing {}B client {}",
                data.len(),
                self.client_id
            );

            let is_authenticated = {
                let client = self
                    .client_state
                    .lock()
                    .map_err(|_| X11Error::Protocol("Failed to lock client state".to_string()))?;
                trace!(
                    "trace: auth status client {} = {}",
                    self.client_id, client.is_authenticated
                );
                client.is_authenticated
            };

            if !is_authenticated {
                trace!("trace: setup phase client {}", self.client_id);
                if let Err(e) = self.handle_connection_setup(data).await {
                    error!("error: setup failed client {}: {:?}", self.client_id, e);
                    trace!("trace: breaking setup failure client {}", self.client_id);
                    break;
                }
                trace!("trace: setup complete client {}", self.client_id);
            } else {
                trace!("trace: request phase client {}", self.client_id);
                if let Err(e) = self.handle_authenticated_request(data).await {
                    error!("error: request failed client {}: {:?}", self.client_id, e);
                    trace!("trace: continuing after error client {}", self.client_id);
                    continue;
                }
                trace!("trace: request success client {}", self.client_id);
            }
        }

        trace!("trace: exiting loop client {}", self.client_id);
        // Clean up client state
        {
            let mut server = self
                .server_state
                .lock()
                .map_err(|_| X11Error::Protocol("Failed to lock server state".to_string()))?;
            server.unregister_client(self.client_id);
        }
        debug!("info: cleanup complete client {}", self.client_id);

        Ok(())
    }

    /// Handle connection setup phase
    async fn handle_connection_setup(&mut self, data: &[u8]) -> Result<(), X11Error> {
        debug!("info: setup client {}", self.client_id);
        trace!(
            "trace: setup data {}B client {} first: {:02x?}",
            data.len(),
            self.client_id,
            &data[..data.len().min(4)]
        );

        if data.len() < 12 {
            error!(
                "error: setup too short {}B client {} (min 12)",
                data.len(),
                self.client_id
            );
            return Err(X11Error::Protocol(
                "Connection setup request too short".to_string(),
            ));
        }

        trace!(
            "trace: byte order 0x{:02x} client {}",
            data[0], self.client_id
        );
        let byte_order = match data[0] {
            0x42 => {
                trace!("trace: big endian client {}", self.client_id);
                ByteOrder::BigEndian
            }
            0x6C => {
                trace!("trace: little endian client {}", self.client_id);
                ByteOrder::LittleEndian
            }
            _ => {
                error!(
                    "error: invalid byte order 0x{:02x} client {}",
                    data[0], self.client_id
                );
                return Err(X11Error::Protocol("Invalid byte order".to_string()));
            }
        };

        let protocol_major = u16::from_le_bytes([data[2], data[3]]);
        let protocol_minor = u16::from_le_bytes([data[4], data[5]]);
        debug!(
            "info: protocol {}.{} client {}",
            protocol_major, protocol_minor, self.client_id
        );

        if protocol_major != 11 {
            error!(
                "error: unsupported protocol {}.{} client {}",
                protocol_major, protocol_minor, self.client_id
            );
            trace!("trace: sending failure client {}", self.client_id);
            return self
                .send_connection_setup_failed(
                    protocol_major,
                    protocol_minor,
                    "Unsupported protocol version",
                )
                .await;
        }

        let resource_id_base = 0x00400000;
        let resource_id_mask = 0x003FFFFF;
        trace!(
            "trace: allocating resources base=0x{:08x} mask=0x{:08x} client {} ({})",
            resource_id_base,
            resource_id_mask,
            self.client_id,
            (resource_id_mask as u32).count_ones()
        );

        {
            let mut client = self
                .client_state
                .lock()
                .map_err(|_| X11Error::Protocol("Failed to lock client state".to_string()))?;
            trace!(
                "trace: authenticating client {} order={:?}",
                self.client_id, byte_order
            );
            client.authenticate(resource_id_base, resource_id_mask, byte_order);
        }

        trace!("trace: sending success client {}", self.client_id);
        self.send_connection_setup_success(protocol_major, protocol_minor)
            .await?;

        info!("info: authenticated client {}", self.client_id);
        Ok(())
    }
    /// Handle authenticated X11 requests - may contain multiple requests in one buffer
    async fn handle_authenticated_request(&mut self, data: &[u8]) -> Result<(), X11Error> {
        let mut offset = 0;
        let mut request_count = 0;

        trace!(
            "trace: processing buffer {}B client {} (may contain multiple requests)",
            data.len(),
            self.client_id
        );

        // Process all requests in the buffer
        while offset < data.len() {
            if offset + 4 > data.len() {
                // Not enough data for a complete request header
                trace!(
                    "trace: incomplete request header at offset {} client {}",
                    offset, self.client_id
                );
                break;
            }

            let opcode = data[offset];
            let length_field = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
            let request_length = (length_field as usize) * 4; // Convert from 32-bit words to bytes

            trace!(
                "trace: request {} opcode=0x{:02x} length={}B at offset={} client {}",
                request_count + 1,
                opcode,
                request_length,
                offset,
                self.client_id
            );

            if offset + request_length > data.len() {
                trace!(
                    "trace: incomplete request body: need {}B have {}B at offset {} client {}",
                    request_length,
                    data.len() - offset,
                    offset,
                    self.client_id
                );
                break;
            }

            // Extract this request's data
            let request_data = &data[offset..offset + request_length];

            // Parse and process this individual request
            let mut request = Request::parse(request_data)?;
            trace!("trace: parsed {:?} client {}", request.kind, self.client_id);

            let sequence_number = {
                let mut client = self
                    .client_state
                    .lock()
                    .map_err(|_| X11Error::Protocol("Failed to lock client state".to_string()))?;
                let seq = client.next_sequence_number();
                trace!(
                    "trace: seq={} client {} (next={})",
                    seq,
                    self.client_id,
                    seq.wrapping_add(1)
                );
                seq
            };
            request.sequence_number = sequence_number;
            debug!(
                "info: processing request {} seq={} client {}",
                request_count + 1,
                sequence_number,
                self.client_id
            );

            trace!(
                "trace: validating request {} client {}",
                request_count + 1,
                self.client_id
            );
            X11RequestValidator::validate(&request)?;
            trace!(
                "trace: validation passed request {} client {}",
                request_count + 1,
                self.client_id
            );
            trace!(
                "trace: routing request {} client {}",
                request_count + 1,
                self.client_id
            );

            // Process the actual request and generate appropriate response
            match self.process_request(&request).await {
                Ok(Some(response)) => {
                    // Send response if one was generated
                    self.stream
                        .write_all(&response)
                        .await
                        .map_err(|e| X11Error::Io(e))?;
                    trace!(
                        "trace: sent {}B response for request {} client {}",
                        response.len(),
                        request_count + 1,
                        self.client_id
                    );
                }
                Ok(None) => {
                    // No response needed for this request type
                    trace!(
                        "trace: no response needed for request {} client {}",
                        request_count + 1,
                        self.client_id
                    );
                }
                Err(e) => {
                    error!(
                        "error: failed to process request {} client {}: {:?}",
                        request_count + 1,
                        self.client_id,
                        e
                    );
                    // Continue processing other requests instead of failing the whole buffer
                }
            }

            debug!(
                "info: request {} processed client {}",
                request_count + 1,
                self.client_id
            );

            // Move to next request
            offset += request_length;
            request_count += 1;
        }

        if request_count > 1 {
            debug!(
                "info: processed {} requests from {}B buffer client {}",
                request_count,
                data.len(),
                self.client_id
            );
        }

        Ok(())
    }    /// Process an individual X11 request and generate response if needed
    async fn process_request(&mut self, request: &Request) -> Result<Option<Vec<u8>>, X11Error> {
        let byte_order = {
            let client = self.client_state.lock()
                .map_err(|_| X11Error::Protocol("Failed to lock client state".to_string()))?;
            client.byte_order
        };

        // Use the handler registry to process the request
        self.handler_registry.handle_request(
            self.client_id,
            request,
            Arc::clone(&self.server_state),
            Arc::clone(&self.client_state),
            byte_order,
        ).await
    }

    /// Send connection setup success response
    async fn send_connection_setup_success(
        &mut self,
        protocol_major: u16,
        protocol_minor: u16,
    ) -> Result<(), X11Error> {
        let byte_order = {
            let client = self
                .client_state
                .lock()
                .map_err(|_| X11Error::Protocol("Failed to lock client state".to_string()))?;
            client.byte_order
        }; // Get real display configuration from virtual display
        let display_config = {
            let server = self
                .server_state
                .lock()
                .map_err(|_| X11Error::Protocol("Failed to lock server state".to_string()))?;

            if let Some(ref virtual_display) = server.virtual_display {
                virtual_display.get_config()
            } else {
                // Fallback to default if no virtual display
                crate::server::display::DisplayConfig::default()
            }
        };

        let vendor = "RxServer";
        let vendor_len = vendor.len();
        let vendor_pad = (4 - (vendor_len % 4)) % 4;

        // Calculate correct length according to X11 protocol:
        // 8 (fixed overhead) + 2*n (pixmap formats) + (v+p+m)/4 (vendor + screen data)
        // n=1 format (8 bytes = 2 words), v=8 vendor bytes, p=0 pad, m=72 screen bytes
        // 8 + 2*1 + (8+0+72)/4 = 8 + 2 + 20 = 30 words
        let additional_data_length = 30u16;

        let mut response = Vec::new();
        let mut writer = EndianWriter::new(&mut response, byte_order);

        // Connection setup response header (8 bytes)
        writer.write_u8(1); // Success
        writer.write_u8(0); // Unused
        writer.write_u16(protocol_major);
        writer.write_u16(protocol_minor);
        writer.write_u16(additional_data_length);

        // Fixed data (20 bytes)
        writer.write_u32(0x00010000); // Release number

        // Resource ID base and mask (from client state)
        let (base, mask) = {
            let client = self
                .client_state
                .lock()
                .map_err(|_| X11Error::Protocol("Failed to lock client state".to_string()))?;
            (client.resource_id_base, client.resource_id_mask)
        };
        writer.write_u32(base);
        writer.write_u32(mask);
        writer.write_u32(0); // Motion buffer size
        writer.write_u16(vendor_len as u16); // Vendor length
        writer.write_u16(65535); // Maximum request length

        // Format and capability info (8 bytes)
        writer.write_u8(1); // Number of screens
        writer.write_u8(1); // Number of pixmap formats

        // Image and bitmap format info
        match byte_order {
            ByteOrder::LittleEndian => writer.write_u8(0), // LSBFirst
            ByteOrder::BigEndian => writer.write_u8(1),    // MSBFirst
        }
        writer.write_u8(0); // Bitmap bit order (LeastSignificant)
        writer.write_u8(32); // Bitmap format scanline unit
        writer.write_u8(32); // Bitmap format scanline pad
        writer.write_u8(8); // Min keycode
        writer.write_u8(255); // Max keycode
        writer.write_bytes(&[0u8; 4]); // 4 bytes unused        // Vendor string with proper padding
        writer.write_bytes(vendor.as_bytes());
        if vendor_pad > 0 {
            writer.write_bytes(&vec![0u8; vendor_pad]);
        }

        // Pixmap format (8 bytes total as per X11 spec) - using real display config
        writer.write_u8(display_config.depth); // Use real depth from virtual display
        writer.write_u8(32); // bits-per-pixel
        writer.write_u8(32); // scanline-pad
        writer.write_bytes(&[0u8; 5]); // 5 bytes unused    
        
            // Screen information using real display data (40 bytes base + depths)
        let screen_data = {
            let server = self
                .server_state
                .lock()
                .map_err(|_| X11Error::Protocol("Failed to lock server state".to_string()))?;
            let root = server.get_root_window();

            let mut screen = Vec::new();
            let mut screen_writer = EndianWriter::new(&mut screen, byte_order);

            // SCREEN structure (40 bytes) - using real display dimensions
            screen_writer.write_u32(root.id); // root window
            screen_writer.write_u32(1); // default colormap
            screen_writer.write_u32(0xFFFFFF); // white pixel
            screen_writer.write_u32(0x000000); // black pixel
            screen_writer.write_u32(0); // current input masks
            screen_writer.write_u16(display_config.width); // Real display width
            screen_writer.write_u16(display_config.height); // Real display height
            screen_writer.write_u16(display_config.width_mm); // Real physical width in mm
            screen_writer.write_u16(display_config.height_mm); // Real physical height in mm
            screen_writer.write_u16(1); // min installed maps
            screen_writer.write_u16(1); // max installed maps
            screen_writer.write_u32(0x21); // root visual
            screen_writer.write_u8(0); // backing stores (Never)
            screen_writer.write_u8(0); // save unders (False)
            screen_writer.write_u8(display_config.depth); // Real root depth
            screen_writer.write_u8(1); // number of allowed depths

            // DEPTH structure (8 bytes)
            screen_writer.write_u8(display_config.depth); // Real depth
            screen_writer.write_u8(0); // unused
            screen_writer.write_u16(1); // number of visuals
            screen_writer.write_bytes(&[0u8; 4]); // padding

            // VISUALTYPE structure (24 bytes)
            screen_writer.write_u32(0x21); // visual id
            screen_writer.write_u8(4); // class (TrueColor)
            screen_writer.write_u8(8); // bits per RGB value
            screen_writer.write_u16(256); // colormap entries
            screen_writer.write_u32(0xFF0000); // red mask
            screen_writer.write_u32(0x00FF00); // green mask
            screen_writer.write_u32(0x0000FF); // blue mask
            screen_writer.write_bytes(&[0u8; 4]); // padding

            screen
        };
        writer.write_bytes(&screen_data);

        self.stream
            .write_all(&response)
            .await
            .map_err(|e| X11Error::Io(e))?;
        trace!(
            "trace: sent {}B response client {} with real display {}x{}@{}bpp ({}x{}mm)",
            response.len(),
            self.client_id,
            display_config.width,
            display_config.height,
            display_config.depth,
            display_config.width_mm,
            display_config.height_mm
        );
        Ok(())
    }

    /// Send connection setup failed response
    async fn send_connection_setup_failed(
        &mut self,
        protocol_major: u16,
        protocol_minor: u16,
        reason: &str,
    ) -> Result<(), X11Error> {
        error!("error: setup failure client {}: {}", self.client_id, reason);

        let byte_order = {
            let client = self
                .client_state
                .lock()
                .map_err(|_| X11Error::Protocol("Failed to lock client state".to_string()))?;
            client.byte_order
        };

        let mut response = Vec::new();
        let mut writer = EndianWriter::new(&mut response, byte_order);

        writer.write_u8(0); // Failed
        writer.write_u8(reason.len() as u8); // reason length
        writer.write_u16(protocol_major);
        writer.write_u16(protocol_minor);
        writer.write_u16(((reason.len() + 3) / 4) as u16); // length in 4-byte units
        writer.write_bytes(reason.as_bytes());

        // Pad to 4-byte boundary
        let padding = (4 - (reason.len() % 4)) % 4;
        writer.write_bytes(&vec![0u8; padding]);

        self.stream
            .write_all(&response)
            .await
            .map_err(|e| X11Error::Io(e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::opcodes;
    use tokio::io::AsyncWriteExt;
    use tokio::net::{TcpListener, TcpStream};

    #[tokio::test]
    async fn test_connection_setup_and_get_geometry() {
        // Start test server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_state = ServerState::new();

        // Spawn server handler
        let server_state_clone = server_state.clone();
        tokio::spawn(async move {
            if let Ok((socket, _)) = listener.accept().await {
                if let Ok(handler) = ConnectionHandler::new(server_state_clone, socket) {
                    let _ = handler.handle().await;
                }
            }
        });

        // Connect as client
        let mut client = TcpStream::connect(addr).await.unwrap();

        // Send connection setup
        let mut setup_request = Vec::new();
        setup_request.push(0x6C); // LSB first
        setup_request.push(0); // unused
        setup_request.extend_from_slice(&11u16.to_le_bytes()); // protocol major
        setup_request.extend_from_slice(&0u16.to_le_bytes()); // protocol minor
        setup_request.extend_from_slice(&0u16.to_le_bytes()); // auth protocol name length
        setup_request.extend_from_slice(&0u16.to_le_bytes()); // auth protocol data length
        setup_request.extend_from_slice(&0u16.to_le_bytes()); // unused

        client.write_all(&setup_request).await.unwrap();

        // Read connection setup response
        let mut setup_response = vec![0u8; 8];
        client.read_exact(&mut setup_response).await.unwrap();
        assert_eq!(setup_response[0], 1); // Success

        // Read additional data length and skip the rest of setup response
        let additional_length =
            u16::from_le_bytes([setup_response[6], setup_response[7]]) as usize * 4;
        let mut additional_data = vec![0u8; additional_length];
        client.read_exact(&mut additional_data).await.unwrap();

        // Now send GetGeometry request
        let mut geo_request = vec![0u8; 8];
        geo_request[0] = opcodes::GET_GEOMETRY;
        geo_request[1] = 0;
        geo_request[2..4].copy_from_slice(&2u16.to_le_bytes()); // length
        geo_request[4..8].copy_from_slice(&1u32.to_le_bytes()); // root window

        client.write_all(&geo_request).await.unwrap();

        // Read GetGeometry response
        let mut geo_response = vec![0u8; 32];
        client.read_exact(&mut geo_response).await.unwrap();

        assert_eq!(geo_response[0], 1); // response type (reply)
        assert_eq!(geo_response[1], 24); // depth

        info!("info: connection setup and GetGeometry test passed");
    }
}
