//! X11 Wire Protocol Implementation
//!
//! This module handles the X11 wire protocol, including message parsing,
//! serialization, and protocol validation.

pub mod endianness;
pub mod errors;
pub mod opcodes;
pub mod parser;
pub mod request;
pub mod responses;
pub mod serializer;
pub mod types;
pub mod validation;
pub mod wire;

// Re-export commonly used types
pub use errors::{ProtocolError, X11Error};
pub use opcodes::Opcode;
pub use types::*;
pub use wire::{MessageHeader, WireFormat};

use crate::fonts::FontRegistry;
use crate::network::ConnectionId;
use crate::server::configuration::ServerConfig;
use crate::x11::protocol::parser::RequestParser;
use crate::x11::resources::types::atom::AtomRegistry;
use crate::x11::security::SecurityManager;
use std::sync::{Arc, Mutex};

/// X11 Protocol handler for processing requests and generating responses
#[derive(Debug)]
pub struct RequestProcessor {
    /// Connection setup state
    setup_complete: std::collections::HashMap<ConnectionId, bool>,
    /// Protocol version for each connection
    versions: std::collections::HashMap<ConnectionId, (u16, u16)>,
    /// Protocol parsers for each connection (different byte orders)
    parsers: std::collections::HashMap<ConnectionId, RequestParser>,
    /// Server configuration for generating setup responses
    server_config: Option<Arc<ServerConfig>>,
    /// Atom registry for managing atoms
    atom_registry: Arc<Mutex<AtomRegistry>>,
    /// Font registry for managing fonts
    font_registry: Arc<Mutex<FontRegistry>>,
    /// Security manager for authentication and authorization
    security_manager: Arc<Mutex<SecurityManager>>,
    /// Reference to subsystems for request handling
    subsystems: Option<Arc<SubsystemReferences>>,
}

/// References to all server subsystems for protocol handling
#[derive(Debug)]
pub struct SubsystemReferences {
    pub display_manager: Arc<tokio::sync::RwLock<crate::display::DisplayManager>>,
    pub font_system: Arc<tokio::sync::RwLock<crate::fonts::FontSystem>>,
    pub input_system: Arc<tokio::sync::RwLock<crate::input::InputSystem>>,
    pub x11_state: Arc<tokio::sync::RwLock<crate::x11::state::X11ServerState>>,
}

impl RequestProcessor {
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
            subsystems: None,
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
            subsystems: None,
        }
    }

    /// Set the subsystem references for protocol handling
    pub fn set_subsystems(&mut self, subsystems: SubsystemReferences) {
        self.subsystems = Some(Arc::new(subsystems));
    }

    /// Get subsystem references if available
    pub fn get_subsystems(&self) -> Option<Arc<SubsystemReferences>> {
        self.subsystems.clone()
    }

    /// Set the server configuration
    pub fn set_config(&mut self, config: Arc<ServerConfig>) {
        self.server_config = Some(config);
    }

    /// Handle requests
    pub async fn handle(
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

            let request_parser = self
                .parsers
                .get(&connection_id)
                .ok_or(ProtocolError::InvalidFormat)?;

            tracing::trace!(
                "Request from {} at offset {}: opcode={:?} ({}), length={} ({} bytes)",
                connection_id,
                offset,
                opcode,
                header.opcode,
                header.length,
                request_size_bytes
            );

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
}

impl Default for RequestProcessor {
    fn default() -> Self {
        Self::new()
    }
}
