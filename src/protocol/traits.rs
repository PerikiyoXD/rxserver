use crate::{
    protocol::{ClientId, Opcode, Request, Response},
    ServerResult,
};

/// Main protocol handler trait for X11 requests
#[async_trait::async_trait]
pub trait ProtocolHandler: Send + Sync {
    /// Handle an incoming X11 request
    async fn handle_request(
        &mut self,
        client_id: ClientId,
        request: Request,
    ) -> ServerResult<Option<Response>>;

    /// Get supported opcodes
    fn supported_opcodes(&self) -> &[Opcode];

    /// Initialize the handler
    async fn initialize(&mut self) -> ServerResult<()> {
        Ok(())
    }

    /// Shutdown the handler
    async fn shutdown(&mut self) -> ServerResult<()> {
        Ok(())
    }

    /// Handle a client connection (X11 handshake and main loop)
    async fn handle_client(&mut self, _stream: &mut tokio::net::TcpStream) -> ServerResult<()> {
        // Default implementation: not implemented
        Err(crate::ServerError::ProtocolError(
            crate::protocol::ProtocolError::Unimplemented,
        ))
    }
}
