use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    protocol::{ClientId, Opcode, ProtocolError, ProtocolHandler, Request, Response},
    ServerError, ServerResult,
};

/// Registry that manages and routes requests to specialized protocol handlers
#[derive(Clone)]
pub struct ProtocolHandlerRegistry {
    handlers: Arc<Mutex<HashMap<Opcode, Arc<Mutex<dyn ProtocolHandler + Send + Sync + 'static>>>>>,
    display_config: Option<crate::protocol::DisplayConfig>,
}

impl ProtocolHandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            display_config: None,
        }
    }

    pub fn with_display_config(display_config: crate::protocol::DisplayConfig) -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            display_config: Some(display_config),
        }
    }

    pub fn set_display_config(&mut self, display_config: crate::protocol::DisplayConfig) {
        self.display_config = Some(display_config);
    }

    pub async fn register_handler(
        &mut self,
        handler: impl ProtocolHandler + Send + Sync + 'static,
    ) -> ServerResult<()> {
        let handler_arc = Arc::new(Mutex::new(handler));
        handler_arc.lock().await.initialize().await?;
        let opcodes = handler_arc.lock().await.supported_opcodes().to_vec();

        let mut handlers = self.handlers.lock().await;

        for &opcode in &opcodes {
            if handlers.contains_key(&opcode) {
                return Err(ServerError::ProtocolError(
                    ProtocolError::OpcodeAlreadyRegistered(opcode),
                ));
            }
        }

        for &opcode in &opcodes {
            handlers.insert(opcode, handler_arc.clone());
        }

        Ok(())
    }

    pub async fn shutdown(&self) -> ServerResult<()> {
        let handlers = self.handlers.lock().await;
        let mut unique_handlers = std::collections::HashSet::<*const ()>::new();

        for handler in handlers.values() {
            let ptr = Arc::as_ptr(handler) as *const ();
            if unique_handlers.insert(ptr) {
                handler.lock().await.shutdown().await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl crate::protocol::ProtocolHandler for ProtocolHandlerRegistry {
    async fn handle_request(
        &mut self,
        client_id: ClientId,
        request: Request,
    ) -> ServerResult<Option<Response>> {
        let opcode = request.opcode();
        let handlers = self.handlers.lock().await;
        if let Some(handler) = handlers.get(&opcode) {
            let mut handler = handler.lock().await;
            handler.handle_request(client_id, request).await
        } else {
            Err(crate::ServerError::ProtocolError(
                crate::protocol::ProtocolError::UnimplementedOpcode(opcode),
            ))
        }
    }
    fn supported_opcodes(&self) -> &[crate::protocol::Opcode] {
        // Not meaningful for the registry; return empty slice
        &[]
    }
    async fn handle_client(
        &mut self,
        stream: &mut tokio::net::TcpStream,
    ) -> crate::ServerResult<()> {
        use crate::protocol::wire::{parse_request, write_response};
        use tracing::{debug, error, info, warn};

        debug!("Starting X11 client connection handling via registry");

        // Perform X11 handshake
        let _setup_request =
            crate::protocol::perform_handshake(stream, self.display_config.as_ref()).await?;

        info!("X11 handshake completed, client connection established"); // Generate a unique client ID
        let client_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u32;

        // Initialize sequence number counter for this client
        let mut sequence_counter = 0u16;

        debug!(
            "Starting main request processing loop for client {}",
            client_id
        ); // Main request processing loop
        loop {
            match parse_request(stream, &mut sequence_counter).await {
                Ok(Some(request)) => {
                    debug!(
                        "Received request: opcode={:?}, length={}, seq={}",
                        request.opcode(),
                        request.length,
                        request.sequence_number
                    ); // Handle the request
                    match self.handle_request(client_id, request.clone()).await {
                        Ok(Some(response)) => {
                            debug!(
                                "Sending response: type={}, data_len={}",
                                response.response_type,
                                response.data.len()
                            );
                            if let Err(e) =
                                write_response(stream, &response, request.sequence_number).await
                            {
                                error!("Failed to write response to client {}: {}", client_id, e);
                                break;
                            }
                        }
                        Ok(None) => {
                            debug!("Request completed successfully (no response required)");
                        }
                        Err(e) => {
                            warn!("Request handling failed for client {}: {}", client_id, e);

                            // Send error response
                            let error_response = crate::protocol::Response::new(
                                0,
                                vec![
                                    1, // Error code - generic error for now
                                    0, 0, // Sequence number placeholder
                                    0, 0, 0, 0, // Additional error data
                                ],
                            );

                            if let Err(write_err) =
                                write_response(stream, &error_response, request.sequence_number)
                                    .await
                            {
                                error!(
                                    "Failed to write error response to client {}: {}",
                                    client_id, write_err
                                );
                                break;
                            }
                        }
                    }
                }
                Ok(None) => {
                    info!("Client {} disconnected cleanly", client_id);
                    break;
                }
                Err(e) => {
                    error!("Failed to parse request from client {}: {}", client_id, e);
                    break;
                }
            }
        }

        debug!("Client {} connection handling completed", client_id);
        Ok(())
    }
}
