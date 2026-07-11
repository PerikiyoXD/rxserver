// connection.rs - Simplified and cleaned up
//! Connection handler implementing the X11 connection state machine

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, error, info, trace};

use crate::protocol::{
    ByteOrder, ByteOrderWriter, RequestHandlerRegistry, X11RequestParser,
    create_standard_handler_registry,
};
use crate::server::Server;
use crate::server::client_system::Client;
use crate::transport::{ConnectionEvent, TransportKind, TransportMessage};

pub struct ConnectionHandler<S> {
    client: Arc<Mutex<Client>>,
    server: Arc<Mutex<Server>>,
    stream: S,
    handlers: RequestHandlerRegistry,
    client_addr: std::net::SocketAddr,
    message_sender: Option<mpsc::UnboundedSender<TransportMessage>>,
    transport_kind: TransportKind,
}

impl<S> ConnectionHandler<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    pub async fn new(
        server: Arc<Mutex<Server>>,
        stream: S,
        client_addr: std::net::SocketAddr,
    ) -> Result<Self> {
        let (client, handlers) = {
            let mut server_guard = server.lock().await;
            let (_, client) = server_guard.register_client(client_addr);
            let handlers = create_standard_handler_registry(server_guard.extensions());
            (client, handlers)
        };

        Ok(Self {
            server,
            client,
            stream,
            handlers,
            client_addr,
            message_sender: None,
            transport_kind: TransportKind::Tcp, // Default to TCP for now
        })
    }

    pub fn with_transport_info(
        mut self,
        message_sender: mpsc::UnboundedSender<TransportMessage>,
        transport_kind: TransportKind,
    ) -> Self {
        self.message_sender = Some(message_sender);
        self.transport_kind = transport_kind;
        self
    }

    pub async fn handle(mut self) -> Result<()> {
        let client_id = self.client.lock().await.id();
        debug!("Handling connection for client {}", client_id);
        info!("Client {} connected", client_id);
        let mut buffer = vec![0u8; 4096];

        let result = async {
            loop {
                trace!("Waiting for data from client {}", client_id);
                // Try to read data from the client with a longer timeout
                let read_result = tokio::time::timeout(
                    std::time::Duration::from_secs(30), // Much longer timeout for idle connections
                    self.stream.read(&mut buffer),
                )
                .await;

                let bytes_read = match read_result {
                    Ok(Ok(0)) => {
                        info!("Client {} disconnected (EOF)", client_id);
                        break;
                    }
                    Ok(Ok(n)) => n,
                    Ok(Err(e)) => {
                        error!("IO error for client {}: {}", client_id, e);
                        break;
                    }
                    Err(_) => {
                        // Timeout occurred - connection is idle but may still be alive
                        debug!(
                            "Read timeout for client {}, assuming idle connection",
                            client_id
                        );
                        continue; // Just continue reading, don't interfere with the stream
                    }
                };

                debug!("Client {} sent {} bytes", client_id, bytes_read);
                let data = &buffer[..bytes_read];
                let is_authenticated = {
                    let client = self.client.lock().await;
                    client.is_authenticated()
                };

                if !is_authenticated {
                    if let Err(e) = self.handle_connection_setup(data).await {
                        error!("Setup failed for client {}: {}", client_id, e);
                        break;
                    }
                } else {
                    if let Err(e) = self.handle_requests(data).await {
                        error!("Request processing failed for client {}: \n{}", client_id, e);
                        break;
                    }
                }
                trace!("Completed processing data from client {}", client_id);
            }
            trace!("Exiting main loop for client {}", client_id);
            Ok(())
        }
        .await;

        // Cleanup - this should always run
        info!("Starting cleanup for client {}", client_id);
        {
            let mut server = self.server.lock().await;
            server.unregister_client(client_id);
        }

        // Send connection closed message
        if let Some(sender) = &self.message_sender {
            let event = ConnectionEvent {
                client_addr: self.client_addr.to_string(),
                transport_kind: self.transport_kind,
            };
            if let Err(e) = sender.send(TransportMessage::ConnectionClosed(event)) {
                error!("Failed to send connection closed message: {}", e);
            }
        }

        info!("Client {} disconnected and cleaned up", client_id);
        result
    }

    async fn handle_connection_setup(&mut self, data: &[u8]) -> Result<()> {
        if data.len() < 12 {
            return Err(anyhow::anyhow!("Connection setup request too short"));
        }

        let byte_order = match data[0] {
            0x42 => ByteOrder::BigEndian,
            0x6C => ByteOrder::LittleEndian,
            _ => return Err(anyhow::anyhow!("Invalid byte order")),
        };

        let protocol_major = u16::from_le_bytes([data[2], data[3]]);

        if protocol_major != 11 {
            self.send_connection_setup_failed("Unsupported protocol version")
                .await?;
            return Err(anyhow::anyhow!("Unsupported protocol version"));
        }

        let resource_id_base = 0x00400000;
        let resource_id_mask = 0x003FFFFF;

        {
            let mut client = self.client.lock().await;
            client.authenticate(resource_id_base, resource_id_mask, byte_order);
        }

        self.send_connection_setup_success().await?;
        info!("Client {} authenticated", self.client.lock().await.id());
        Ok(())
    }

    async fn handle_requests(&mut self, data: &[u8]) -> Result<()> {
        let mut offset = 0;

        trace!("Handling requests for client {}", self.client.lock().await.id());
        trace!("Received {} bytes of data", data.len());

        let byte_order = self.client.lock().await.byte_order();

        while offset < data.len() {
            trace!("Processing request at offset {}", offset);
            if offset + 4 > data.len() {
                trace!("Insufficient data for request header, breaking");
                break;
            }

            let request_length = {
                // Normal request length is always 2 bytes, regardless of big requests enabled
                let length_field = if byte_order == ByteOrder::BigEndian {
                    u16::from_be_bytes([data[offset + 2], data[offset + 3]])
                } else {
                    u16::from_le_bytes([data[offset + 2], data[offset + 3]])
                };
                (length_field as usize) * 4
            };

            if offset + request_length > data.len() {
                trace!("Incomplete request data, breaking");
                break;
            }

            let request_data = &data[offset..offset + request_length];
            trace!("RAW request_data bytes: {:?}", request_data);
            let client_id = self.client.lock().await.id();
            let mut request = {
                let server_guard = self.server.lock().await;
                X11RequestParser::parse_dynamic(request_data, server_guard.extensions())
            }
            .map_err(|e| {
                anyhow::anyhow!("Failed to parse request from client {}: \n{}", client_id, e)
            })?;

            let sequence_number = {
                let mut client = self.client.lock().await;
                client.next_sequence_number()
            };
            request.sequence_number = sequence_number;

            let handler_result = {
                let client_id = self.client.lock().await.id();
                let server = Arc::clone(&self.server);
                self.handlers
                    .handle_request(client_id, &request, server)
                    .await
            };

            match handler_result {
                Ok(Some(response)) => {
                    debug!("Handler returned response with {} bytes", response.len());
                    self.stream
                        .write_all(&response)
                        .await
                        .context("Failed to send response")?;
                }
                Ok(None) => {
                    debug!("Handler returned no response");
                }
                Err(e) => {
                    error!("Handler returned error: {}", e);
                    // Maybe send an error response here
                }
            }

            // Send any pending events for this client
            let pending_events = {
                let mut client = self.client.lock().await;
                client.pending_events()
            };

            for event_data in pending_events {
                self.stream
                    .write_all(&event_data)
                    .await
                    .context("Failed to send event")?;
            }

            offset += request_length;
        }

        Ok(())
    }

    async fn send_connection_setup_success(&mut self) -> Result<()> {
        let (byte_order, base, mask, root_id, screen_width, screen_height) = {
            let client = self.client.lock().await;
            let server = self.server.lock().await;
            let (width, height) = server.get_screen_size(0);
            (
                client.byte_order(),
                client.resource_id_base(),
                client.resource_id_mask(),
                server.get_root_window().id,
                width,
                height,
            )
        };

        let mut writer = ByteOrderWriter::new(byte_order);

        // Connection setup response header
        writer.write_u8(1); // Success
        writer.write_u8(0); // Unused
        writer.write_u16(11); // Protocol major
        writer.write_u16(0); // Protocol minor
        writer.write_u16(30); // Additional data length

        // Fixed data
        writer.write_u32(0x00010000); // Release number
        writer.write_u32(base);
        writer.write_u32(mask);
        writer.write_u32(0); // Motion buffer size

        let vendor = "RxServer";
        writer.write_u16(vendor.len() as u16);
        writer.write_u16(65535); // Maximum request length

        // Format and capability info
        writer.write_u8(1); // Number of screens
        writer.write_u8(1); // Number of pixmap formats
        writer.write_u8(0); // Image byte order (LSBFirst)
        writer.write_u8(0); // Bitmap bit order
        writer.write_u8(32); // Bitmap scanline unit
        writer.write_u8(32); // Bitmap scanline pad
        writer.write_u8(8); // Min keycode
        writer.write_u8(255); // Max keycode
        writer.write_bytes(&[0u8; 4]); // Unused

        // Vendor string with padding
        writer.write_bytes(vendor.as_bytes());
        let vendor_pad = (4 - (vendor.len() % 4)) % 4;
        if vendor_pad > 0 {
            writer.write_bytes(&vec![0u8; vendor_pad]);
        }

        // Pixmap format
        writer.write_u8(24); // Depth
        writer.write_u8(32); // Bits per pixel
        writer.write_u8(32); // Scanline pad
        writer.write_bytes(&[0u8; 5]); // Unused

        // Screen information
        writer.write_u32(root_id); // Root window
        writer.write_u32(1); // Default colormap
        writer.write_u32(0xFFFFFF); // White pixel
        writer.write_u32(0x000000); // Black pixel
        writer.write_u32(0); // Current input masks
        writer.write_u16(screen_width); // Width
        writer.write_u16(screen_height); // Height
        writer.write_u16(270); // Width in mm
        writer.write_u16(203); // Height in mm
        writer.write_u16(1); // Min installed maps
        writer.write_u16(1); // Max installed maps
        writer.write_u32(0x21); // Root visual
        writer.write_u8(0); // Backing stores
        writer.write_u8(0); // Save unders
        writer.write_u8(24); // Root depth
        writer.write_u8(1); // Number of depths

        // Depth structure
        writer.write_u8(24); // Depth
        writer.write_u8(0); // Unused
        writer.write_u16(1); // Number of visuals
        writer.write_bytes(&[0u8; 4]); // Padding

        // Visual type structure
        writer.write_u32(0x21); // Visual id
        writer.write_u8(4); // Class (TrueColor)
        writer.write_u8(8); // Bits per RGB
        writer.write_u16(256); // Colormap entries
        writer.write_u32(0xFF0000); // Red mask
        writer.write_u32(0x00FF00); // Green mask
        writer.write_u32(0x0000FF); // Blue mask
        writer.write_bytes(&[0u8; 4]); // Padding

        let response = writer.into_vec();
        self.stream
            .write_all(&response)
            .await
            .context("Failed to send setup success")?;

        Ok(())
    }

    async fn send_connection_setup_failed(&mut self, reason: &str) -> Result<()> {
        let byte_order = {
            let client = self.client.lock().await;
            client.byte_order()
        };

        let mut writer = ByteOrderWriter::new(byte_order);
        writer.write_u8(0); // Failed
        writer.write_u8(reason.len() as u8);
        writer.write_u16(11); // Protocol major
        writer.write_u16(0); // Protocol minor
        writer.write_u16(((reason.len() + 3) / 4) as u16);
        writer.write_bytes(reason.as_bytes());

        let padding = (4 - (reason.len() % 4)) % 4;
        writer.write_bytes(&vec![0u8; padding]);

        let response = writer.into_vec();
        self.stream
            .write_all(&response)
            .await
            .context("Failed to send setup failure")?;

        Ok(())
    }
}

impl<S> Drop for ConnectionHandler<S> {
    fn drop(&mut self) {
        // This will run if the connection handler is dropped without proper cleanup
        info!(
            "ConnectionHandler dropped for client at {}",
            self.client_addr
        );
    }
}

#[cfg(unix)]
impl ConnectionHandler<tokio::net::UnixStream> {
    pub async fn new_unix(
        server_state: Arc<Mutex<Server>>,
        stream: tokio::net::UnixStream,
        socket_path: String,
    ) -> Result<Self> {
        let dummy_addr = "127.0.0.1:0".parse().unwrap();

        let (client_id, client_state, handler_registry) = {
            let mut server = server_state.lock().await;
            let (client_id, client_state) = server.register_client(dummy_addr);
            let handler_registry = create_standard_handler_registry(server.extensions());
            (client_id, client_state, handler_registry)
        };

        debug!(
            "Created Unix connection handler for client {} at {}",
            client_id, socket_path
        );

        // TODO: windows/unix divergence: these field names (server_state,
        // client_state, client_id, handler_registry) don't match
        // ConnectionHandler<S>'s actual fields (server, client, handlers,
        // client_addr, message_sender, transport_kind). This is gated by
        // #[cfg(unix)] so it never builds on Windows and the mismatch has
        // gone unnoticed; needs a real fix before this is ever built on
        // Unix. Pre-existing, not introduced by this change.
        Ok(Self {
            server_state,
            client_state,
            client_id,
            stream,
            handler_registry,
        })
    }
}
