//! Window Protocol Handler
//!
//! Handles X11 window-related requests using the Window plugin.

use async_trait::async_trait;
use std::sync::Arc;
use tracing::debug;

use crate::{
    plugins::WindowPlugin,
    protocol::{ClientId, Opcode, ProtocolError, ProtocolHandler, Request, Response},
    ServerError, ServerResult,
};

/// Protocol handler specialized for window operations
pub struct WindowProtocolHandler {
    window_plugin: Arc<WindowPlugin>,
}

impl WindowProtocolHandler {
    pub fn new(window_plugin: Arc<WindowPlugin>) -> Self {
        Self { window_plugin }
    }
}

#[async_trait]
impl ProtocolHandler for WindowProtocolHandler {
    async fn handle_request(
        &mut self,
        client_id: ClientId,
        request: Request,
    ) -> ServerResult<Option<Response>> {
        debug!(
            "WindowProtocolHandler handling request from client {}: {:?}",
            client_id,
            request.opcode()
        );

        match request.opcode() {
            Opcode::CreateWindow => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
            Opcode::DestroyWindow => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
            Opcode::MapWindow => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
            Opcode::UnmapWindow => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
            Opcode::ConfigureWindow => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
            Opcode::GetGeometry => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
            _ => Err(ServerError::ProtocolError(
                crate::protocol::ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
        }
    }

    fn supported_opcodes(&self) -> &[Opcode] {
        &[
            Opcode::CreateWindow,
            Opcode::DestroyWindow,
            Opcode::MapWindow,
            Opcode::UnmapWindow,
            Opcode::ConfigureWindow,
            Opcode::GetGeometry,
        ]
    }
}
