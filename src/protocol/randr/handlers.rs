//! RANDR (Resize and Rotate) Extension Handlers
//!
//! This module contains the request handlers for RANDR extension operations.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    protocol::{
        ByteOrder, ByteOrderWriter, HandlerResult, RandrOpcode, Request, RequestHandler,
        RequestKind, X11Error,
    },
    server::{Server, client_system::ClientId},
};

/// Handler for RandrQueryVersion requests (minor opcode 0)
pub struct RandrQueryVersionHandler {
    major_opcode: u8,
}

impl RandrQueryVersionHandler {
    pub fn new(major_opcode: u8) -> Self {
        Self { major_opcode }
    }
}

#[async_trait]
impl RequestHandler for RandrQueryVersionHandler {
    async fn handle_request(
        &self,
        _client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let _query_request = match &request.kind {
            RequestKind::RandrQueryVersion(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for RandrQueryVersion: {:?}",
                    request.kind
                )));
            }
        };

        let server_guard = server.lock().await;
        let randr_state = server_guard.randr_state();

        // Create response
        let mut writer = ByteOrderWriter::new(ByteOrder::LittleEndian);
        writer.write_u8(1); // Reply
        writer.write_u8(0); // Unused
        writer.write_u16(request.sequence_number); // Sequence number
        writer.write_u32(0); // Reply length
        writer.write_u32(randr_state.version_major); // Server major version
        writer.write_u32(randr_state.version_minor); // Server minor version
        writer.write_padding(16); // Padding to 32 bytes

        Ok(Some(writer.into_vec()))
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (self.major_opcode, Some(RandrOpcode::QueryVersion.to_u8()))
    }

    fn name(&self) -> &'static str {
        "RandrQueryVersion"
    }
}

/// Handler for RandrGetScreenResources requests (minor opcode 1)
pub struct RandrGetScreenResourcesHandler {
    major_opcode: u8,
}

impl RandrGetScreenResourcesHandler {
    pub fn new(major_opcode: u8) -> Self {
        Self { major_opcode }
    }
}

#[async_trait]
impl RequestHandler for RandrGetScreenResourcesHandler {
    async fn handle_request(
        &self,
        _client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let screen_request = match &request.kind {
            RequestKind::RandrGetScreenResources(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for RandrGetScreenResources: {:?}",
                    request.kind
                )));
            }
        };

        let server_guard = server.lock().await;
        let randr_state = server_guard.randr_state();

        // For now, assume screen 0 (root window's screen)
        let screen = match randr_state.get_screen(screen_request.window as u32) {
            Some(s) => s,
            None => {
                // Try screen 0 as fallback
                randr_state
                    .get_screen(0)
                    .ok_or_else(|| X11Error::Protocol("Screen not found".to_string()))?
            }
        };

        // Calculate reply length
        let num_crtcs = screen.crtcs.len() as u16;
        let num_outputs = screen.outputs.len() as u16;
        let num_modes = screen.modes.len() as u16;
        let names_len: u16 = screen
            .outputs
            .iter()
            .map(|o| o.name.len() as u16 + 1) // +1 for null terminator
            .sum();

        let reply_length = (32 + num_crtcs * 4 + num_outputs * 4 + num_modes * 8 + names_len) / 4;

        // Create response
        let mut writer = ByteOrderWriter::new(ByteOrder::LittleEndian);
        writer.write_u8(1); // Reply
        writer.write_u8(0); // Unused
        writer.write_u16(request.sequence_number); // Sequence number
        writer.write_u32(reply_length as u32); // Reply length
        writer.write_u32(0); // Timestamp (config timestamp)
        writer.write_u32(0); // Config timestamp
        writer.write_u16(num_crtcs); // Number of CRTCs
        writer.write_u16(num_outputs); // Number of outputs
        writer.write_u16(num_modes); // Number of modes
        writer.write_u16(names_len); // Names length

        // Write CRTC IDs
        for crtc in &screen.crtcs {
            writer.write_u32(crtc.id);
        }

        // Write output IDs
        for output in &screen.outputs {
            writer.write_u32(output.id);
        }

        // Write modes
        for mode in &screen.modes {
            writer.write_u32(mode.id);
            writer.write_u16(mode.width);
            writer.write_u16(mode.height);
            writer.write_u32(mode.refresh_rate);
            writer.write_u16(0); // Mode flags (unused for now)
            writer.write_u16(mode.name.len() as u16 + 1); // Name length + null
        }

        // Write output names
        for output in &screen.outputs {
            writer.write_bytes(output.name.as_bytes());
            writer.write_u8(0); // Null terminator
        }

        Ok(Some(writer.into_vec()))
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (
            self.major_opcode,
            Some(RandrOpcode::GetScreenResources.to_u8()),
        )
    }

    fn name(&self) -> &'static str {
        "RandrGetScreenResources"
    }
}

/// Handler for RandrGetOutputInfo requests (minor opcode 2)
pub struct RandrGetOutputInfoHandler {
    major_opcode: u8,
}

impl RandrGetOutputInfoHandler {
    pub fn new(major_opcode: u8) -> Self {
        Self { major_opcode }
    }
}

#[async_trait]
impl RequestHandler for RandrGetOutputInfoHandler {
    async fn handle_request(
        &self,
        _client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let output_request = match &request.kind {
            RequestKind::RandrGetOutputInfo(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for RandrGetOutputInfo: {:?}",
                    request.kind
                )));
            }
        };

        let server_guard = server.lock().await;
        let randr_state = server_guard.randr_state();

        // Find the output (for now, assume screen 0)
        let screen = randr_state
            .get_screen(0)
            .ok_or_else(|| X11Error::Protocol("Screen not found".to_string()))?;

        let output = screen
            .outputs
            .iter()
            .find(|o| o.id == output_request.output)
            .ok_or_else(|| {
                X11Error::Protocol(format!("Output {} not found", output_request.output))
            })?;

        // Calculate reply length
        let num_crtcs = if output.crtc_id.is_some() { 1 } else { 0 };
        let num_modes = output.modes.len() as u16;
        let num_preferred = if output.preferred_mode.is_some() {
            1
        } else {
            0
        };
        let names_len = output.name.len() as u16 + 1; // +1 for null terminator

        let reply_length = (32 + num_crtcs * 4 + num_modes * 4 + num_preferred * 4 + names_len) / 4;

        // Create response
        let mut writer = ByteOrderWriter::new(ByteOrder::LittleEndian);
        writer.write_u8(1); // Reply
        writer.write_u8(0); // Unused
        writer.write_u16(request.sequence_number); // Sequence number
        writer.write_u32(reply_length as u32); // Reply length
        writer.write_u8(if output.connected { 1 } else { 0 }); // Status (connected)
        writer.write_u8(output.crtc_id.map(|_| 1).unwrap_or(0)); // CRTC count
        writer.write_u16(num_modes); // Number of modes
        writer.write_u16(num_preferred); // Number of preferred modes
        writer.write_u16(names_len); // Names length

        // Write timestamp
        writer.write_u32(0); // Timestamp

        // Write CRTC ID if connected
        if let Some(crtc_id) = output.crtc_id {
            writer.write_u32(crtc_id);
        }

        // Write mode IDs
        for mode_id in &output.modes {
            writer.write_u32(*mode_id);
        }

        // Write preferred mode IDs
        if let Some(preferred) = output.preferred_mode {
            writer.write_u32(preferred);
        }

        // Write output name
        writer.write_bytes(output.name.as_bytes());
        writer.write_u8(0); // Null terminator

        Ok(Some(writer.into_vec()))
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (self.major_opcode, Some(RandrOpcode::GetOutputInfo.to_u8()))
    }

    fn name(&self) -> &'static str {
        "RandrGetOutputInfo"
    }
}

/// Handler for RandrGetCrtcInfo requests (minor opcode 13)
pub struct RandrGetCrtcInfoHandler {
    major_opcode: u8,
}

impl RandrGetCrtcInfoHandler {
    pub fn new(major_opcode: u8) -> Self {
        Self { major_opcode }
    }
}

#[async_trait]
impl RequestHandler for RandrGetCrtcInfoHandler {
    async fn handle_request(
        &self,
        _client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let crtc_request = match &request.kind {
            RequestKind::RandrGetCrtcInfo(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for RandrGetCrtcInfo: {:?}",
                    request.kind
                )));
            }
        };

        let server_guard = server.lock().await;
        let randr_state = server_guard.randr_state();

        // Find the CRTC (for now, assume screen 0)
        let screen = randr_state
            .get_screen(0)
            .ok_or_else(|| X11Error::Protocol("Screen not found".to_string()))?;

        let crtc = screen
            .crtcs
            .iter()
            .find(|c| c.id == crtc_request.crtc)
            .ok_or_else(|| X11Error::Protocol(format!("CRTC {} not found", crtc_request.crtc)))?;

        // Calculate reply length
        let num_outputs = crtc.outputs.len() as u16;
        let num_possible = 0; // For now, no possible outputs
        let reply_length = (32 + num_outputs * 4 + num_possible * 4) / 4;

        // Create response
        let mut writer = ByteOrderWriter::new(ByteOrder::LittleEndian);
        writer.write_u8(1); // Reply
        writer.write_u8(0); // Unused
        writer.write_u16(request.sequence_number); // Sequence number
        writer.write_u32(reply_length as u32); // Reply length
        writer.write_u8(1); // Status (success)
        writer.write_u16(crtc.width); // Width
        writer.write_u16(crtc.height); // Height
        writer.write_u16(crtc.x as u16); // X position
        writer.write_u16(crtc.y as u16); // Y position
        writer.write_u32(crtc.mode_id.unwrap_or(0)); // Mode ID
        writer.write_u16(crtc.rotation.to_u16()); // Rotation
        writer.write_u16(num_outputs); // Number of outputs
        writer.write_u16(num_possible); // Number of possible outputs

        // Write outputs
        for output_id in &crtc.outputs {
            writer.write_u32(*output_id);
        }

        // Write possible outputs (none for now)
        // (empty)

        Ok(Some(writer.into_vec()))
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (self.major_opcode, Some(RandrOpcode::GetCrtcInfo.to_u8()))
    }

    fn name(&self) -> &'static str {
        "RandrGetCrtcInfo"
    }
}

/// Handler for RandrGetScreenSizeRange requests (minor opcode 18)
pub struct RandrGetScreenSizeRangeHandler {
    major_opcode: u8,
}

impl RandrGetScreenSizeRangeHandler {
    pub fn new(major_opcode: u8) -> Self {
        Self { major_opcode }
    }
}

#[async_trait]
impl RequestHandler for RandrGetScreenSizeRangeHandler {
    async fn handle_request(
        &self,
        _client_id: ClientId,
        request: &Request,
        server: Arc<Mutex<Server>>,
    ) -> HandlerResult<Option<Vec<u8>>> {
        let _size_request = match &request.kind {
            RequestKind::RandrGetScreenSizeRange(req) => req,
            _ => {
                return Err(X11Error::Protocol(format!(
                    "Invalid request type for RandrGetScreenSizeRange: {:?}",
                    request.kind
                )));
            }
        };

        let server_guard = server.lock().await;
        let randr_state = server_guard.randr_state();

        // Get screen info (for now, assume screen 0)
        let screen = randr_state
            .get_screen(0)
            .ok_or_else(|| X11Error::Protocol("Screen not found".to_string()))?;

        // Create response
        let mut writer = ByteOrderWriter::new(ByteOrder::LittleEndian);
        writer.write_u8(1); // Reply
        writer.write_u8(0); // Unused
        writer.write_u16(request.sequence_number); // Sequence number
        writer.write_u32(0); // Reply length
        writer.write_u16(screen.min_width); // Min width
        writer.write_u16(screen.min_height); // Min height
        writer.write_u16(screen.max_width); // Max width
        writer.write_u16(screen.max_height); // Max height
        writer.write_padding(16); // Padding to 32 bytes

        Ok(Some(writer.into_vec()))
    }

    fn opcode(&self) -> (u8, Option<u8>) {
        (
            self.major_opcode,
            Some(RandrOpcode::GetScreenSizeRange.to_u8()),
        )
    }

    fn name(&self) -> &'static str {
        "RandrGetScreenSizeRange"
    }
}
