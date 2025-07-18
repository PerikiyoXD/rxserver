// connection.rs - Simplified and cleaned up
//! Connection handler implementing the X11 connection state machine

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::protocol::{
    ByteOrder, ByteOrderWriter, Request, RequestHandlerRegistry, RequestParser, X11RequestParser,
    create_standard_handler_registry,
};
use crate::server::Server;
use crate::server::client_system::Client;

pub struct ConnectionHandler<S> {
    client: Arc<Mutex<Client>>,
    server: Arc<Mutex<Server>>,
    stream: S,
    handlers: RequestHandlerRegistry,
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
        let (_, client) = server.lock().await.register_client(client_addr);

        Ok(Self {
            server,
            client,
            stream,
            handlers: create_standard_handler_registry(),
        })
    }

    pub async fn handle(mut self) -> Result<()> {
        let client_id = self.client.lock().await.id();
        debug!("Handling connection for client {}", client_id);
        info!("Client {} connected", client_id);
        let mut buffer = vec![0u8; 4096];

        loop {
            let bytes_read = match self.stream.read(&mut buffer).await {
                Ok(0) => {
                    info!("Client {} disconnected", client_id);
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    error!("IO error for client {}: {}", client_id, e);
                    break;
                }
            };

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
                    error!("Request processing failed for client {}: {}", client_id, e);
                }
            }
        }

        // Cleanup
        {
            let mut server = self.server.lock().await;
            server.unregister_client(client_id);
        }

        Ok(())
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

        while offset < data.len() {
            if offset + 4 > data.len() {
                break;
            }

            let length_field = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
            let request_length = (length_field as usize) * 4;

            if offset + request_length > data.len() {
                break;
            }

            let request_data = &data[offset..offset + request_length];
            let mut request =
                X11RequestParser::parse(request_data).context("Failed to parse request")?;

            let sequence_number = {
                let mut client = self.client.lock().await;
                client.next_sequence_number()
            };
            request.sequence_number = sequence_number;

            if let Ok(Some(response)) = {
                let this = &mut *self;
                let request: &Request = &request;
                async move {
                    this.handlers
                        .handle_request(
                            this.client.lock().await.id(),
                            request,
                            Arc::clone(&this.server),
                        )
                        .await
                        .map_err(|e| anyhow::anyhow!("Handler error: {}", e))
                }
            }
            .await
            {
                self.stream
                    .write_all(&response)
                    .await
                    .context("Failed to send response")?;
            }

            offset += request_length;
        }

        Ok(())
    }

    async fn send_connection_setup_success(&mut self) -> Result<()> {
        let (byte_order, base, mask, root_id) = {
            let client = self.client.lock().await;
            let server = self.server.lock().await;
            (
                client.byte_order(),
                client.resource_id_base(),
                client.resource_id_mask(),
                server.get_root_window().id,
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
        writer.write_u16(1024); // Width
        writer.write_u16(768); // Height
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

#[cfg(unix)]
impl ConnectionHandler<tokio::net::UnixStream> {
    pub async fn new_unix(
        server_state: Arc<Mutex<Server>>,
        stream: tokio::net::UnixStream,
        socket_path: String,
    ) -> Result<Self> {
        let dummy_addr = "127.0.0.1:0".parse().unwrap();

        let (client_id, client_state) = {
            let mut server = server_state.lock().await;
            server.register_client(dummy_addr)
        };

        debug!(
            "Created Unix connection handler for client {} at {}",
            client_id, socket_path
        );

        Ok(Self {
            server_state,
            client_state,
            client_id,
            stream,
            handler_registry: create_standard_handler_registry(),
        })
    }
}
