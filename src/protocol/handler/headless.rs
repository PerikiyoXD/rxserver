//! Headless Protocol Handler
//!
//! A specialized protocol handler that operates without any visual display.
//! Suitable for automated testing, server applications, or headless environments.

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info};

use crate::{
    plugins::{PluginRegistry, WindowPlugin},
    protocol::{ClientId, Opcode, ProtocolError, ProtocolHandler, Request, Response},
    ServerError, ServerResult,
};

/// Protocol handler that operates without visual display
pub struct HeadlessProtocolHandler {
    plugins: Arc<PluginRegistry>,
    window_plugin: Arc<WindowPlugin>,
    // No renderer or display components
}

impl HeadlessProtocolHandler {
    pub fn new(plugins: Arc<PluginRegistry>) -> ServerResult<Self> {
        let window_plugin = Arc::new(WindowPlugin::new());

        info!("Initializing headless protocol handler");

        Ok(Self {
            plugins,
            window_plugin,
        })
    }
}

impl Clone for HeadlessProtocolHandler {
    fn clone(&self) -> Self {
        Self {
            plugins: Arc::clone(&self.plugins),
            window_plugin: Arc::clone(&self.window_plugin),
        }
    }
}

#[async_trait]
impl ProtocolHandler for HeadlessProtocolHandler {
    async fn handle_request(
        &mut self,
        client_id: ClientId,
        request: Request,
    ) -> ServerResult<Option<Response>> {
        debug!(
            "HeadlessProtocolHandler handling request from client {}: {:?}",
            client_id,
            request.opcode()
        );

        match request.opcode() {
            // Window operations - handled without actual display
            Opcode::CreateWindow => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
            Opcode::MapWindow | Opcode::UnmapWindow => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
            Opcode::DestroyWindow => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),

            // Graphics operations - silently consumed
            Opcode::CreateGC | Opcode::FreeGC | Opcode::ChangeGC => Err(
                ServerError::ProtocolError(ProtocolError::UnimplementedOpcode(request.opcode())),
            ),

            // Drawing operations - no-op in headless mode
            Opcode::PolyPoint
            | Opcode::PolyLine
            | Opcode::PolySegment
            | Opcode::PolyRectangle
            | Opcode::PolyArc
            | Opcode::FillPoly
            | Opcode::PolyFillRectangle
            | Opcode::PolyFillArc => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),

            _ => Err(ServerError::ProtocolError(
                crate::protocol::ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
        }
    }

    fn supported_opcodes(&self) -> &[Opcode] {
        &[
            // Window management
            Opcode::CreateWindow,
            Opcode::DestroyWindow,
            Opcode::MapWindow,
            Opcode::UnmapWindow,
            Opcode::ConfigureWindow,
            // Graphics contexts
            Opcode::CreateGC,
            Opcode::FreeGC,
            Opcode::ChangeGC,
            // Drawing operations (no-op in headless)
            Opcode::PolyPoint,
            Opcode::PolyLine,
            Opcode::PolySegment,
            Opcode::PolyRectangle,
            Opcode::PolyArc,
            Opcode::FillPoly,
            Opcode::PolyFillRectangle,
            Opcode::PolyFillArc,
        ]
    }
}
