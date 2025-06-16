//! Surface Protocol Handler
//!
//! Handles X11 surface and graphics-related requests.

use async_trait::async_trait;
use std::sync::Arc;
use tracing::debug;

use crate::{
    graphics::Renderer,
    protocol::{ClientId, Opcode, ProtocolError, ProtocolHandler, Request, Response},
    ServerError, ServerResult,
};

/// Protocol handler specialized for surface/graphics operations
pub struct SurfaceProtocolHandler {
    renderer: Arc<tokio::sync::Mutex<Renderer>>,
}

impl SurfaceProtocolHandler {
    pub fn new(width: u32, height: u32, depth: u8) -> Self {
        Self {
            renderer: Arc::new(tokio::sync::Mutex::new(Renderer::new(width, height, depth))),
        }
    }
}

#[async_trait]
impl ProtocolHandler for SurfaceProtocolHandler {
    async fn handle_request(
        &mut self,
        client_id: ClientId,
        request: Request,
    ) -> ServerResult<Option<Response>> {
        debug!(
            "SurfaceProtocolHandler handling request from client {}: {:?}",
            client_id,
            request.opcode()
        );

        match request.opcode() {
            Opcode::CreateGC => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
            Opcode::FreeGC => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
            Opcode::ChangeGC => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
            Opcode::CopyGC => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
            Opcode::SetDashes => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
            Opcode::SetClipRectangles => Err(ServerError::ProtocolError(
                ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
            _ => Err(ServerError::ProtocolError(
                crate::protocol::ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
        }
    }

    fn supported_opcodes(&self) -> &[Opcode] {
        &[
            Opcode::CreateGC,
            Opcode::FreeGC,
            Opcode::ChangeGC,
            Opcode::CopyGC,
            Opcode::SetDashes,
            Opcode::SetClipRectangles,
        ]
    }
}
