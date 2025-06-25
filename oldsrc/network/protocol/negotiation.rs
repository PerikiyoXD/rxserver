//! Protocol negotiation implementation
//!
//! Handles protocol version negotiation and capability exchange.

use crate::network::ConnectionId;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Protocol negotiation result
#[derive(Debug, Clone)]
pub enum NegotiationResult {
    /// Negotiation successful
    Success(NegotiatedProtocol),
    /// Negotiation failed
    Failed(String),
    /// Continue negotiation with additional data
    Continue(Vec<u8>),
}

/// Negotiation error
#[derive(Debug, thiserror::Error)]
pub enum NegotiationError {
    #[error("Unsupported protocol version: {0}.{1}")]
    UnsupportedVersion(u16, u16),

    #[error("Invalid negotiation data: {0}")]
    InvalidData(String),

    #[error("Negotiation timeout")]
    Timeout,

    #[error("Protocol mismatch: {0}")]
    ProtocolMismatch(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Negotiated protocol information
#[derive(Debug, Clone)]
pub struct NegotiatedProtocol {
    /// Protocol name
    pub name: String,
    /// Major version
    pub major_version: u16,
    /// Minor version
    pub minor_version: u16,
    /// Agreed capabilities
    pub capabilities: Vec<String>,
    /// Protocol-specific options
    pub options: HashMap<String, String>,
}

/// Protocol version information
#[derive(Debug, Clone)]
pub struct ProtocolVersion {
    /// Major version
    pub major: u16,
    /// Minor version
    pub minor: u16,
}

impl ProtocolVersion {
    /// Create a new protocol version
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Check if this version is compatible with another
    pub fn is_compatible_with(&self, other: &ProtocolVersion) -> bool {
        // Major version must match, minor version can be different
        self.major == other.major
    }

    /// Get the minimum compatible version
    pub fn min_compatible(&self, other: &ProtocolVersion) -> Option<ProtocolVersion> {
        if self.is_compatible_with(other) {
            Some(ProtocolVersion::new(
                self.major,
                self.minor.min(other.minor),
            ))
        } else {
            None
        }
    }
}

/// Protocol capability
#[derive(Debug, Clone)]
pub struct ProtocolCapability {
    /// Capability name
    pub name: String,
    /// Capability version
    pub version: String,
    /// Required flag
    pub required: bool,
}

/// Protocol negotiation request
#[derive(Debug)]
pub struct NegotiationRequest {
    /// Connection ID
    pub connection_id: ConnectionId,
    /// Requested protocol name
    pub protocol_name: String,
    /// Client protocol version
    pub client_version: ProtocolVersion,
    /// Client capabilities
    pub client_capabilities: Vec<ProtocolCapability>,
    /// Additional negotiation data
    pub data: Vec<u8>,
}

/// Protocol negotiator
pub struct ProtocolNegotiator {
    /// Supported protocols
    supported_protocols: HashMap<String, SupportedProtocol>,
    /// Active negotiations
    active_negotiations: HashMap<ConnectionId, NegotiationState>,
    /// Negotiation timeout in seconds
    negotiation_timeout: u64,
}

/// Supported protocol configuration
#[derive(Debug, Clone)]
struct SupportedProtocol {
    /// Protocol name
    name: String,
    /// Supported versions
    versions: Vec<ProtocolVersion>,
    /// Supported capabilities
    capabilities: Vec<ProtocolCapability>,
    /// Default options
    default_options: HashMap<String, String>,
}

/// Negotiation state
#[derive(Debug)]
struct NegotiationState {
    /// Connection ID
    connection_id: ConnectionId,
    /// Protocol being negotiated
    protocol_name: String,
    /// Negotiation start time
    started_at: std::time::SystemTime,
    /// Current negotiation step
    step: NegotiationStep,
}

/// Negotiation steps
#[derive(Debug, Clone)]
enum NegotiationStep {
    /// Initial version exchange
    VersionExchange,
    /// Capability negotiation
    CapabilityNegotiation,
    /// Option configuration
    OptionConfiguration,
    /// Completed
    Completed,
    /// Failed
    Failed,
}

impl ProtocolNegotiator {
    /// Create a new protocol negotiator
    pub fn new() -> Self {
        let mut negotiator = Self {
            supported_protocols: HashMap::new(),
            active_negotiations: HashMap::new(),
            negotiation_timeout: 30,
        };

        // Register X11 protocol support
        negotiator.register_x11_protocol();

        negotiator
    }

    /// Register X11 protocol support
    fn register_x11_protocol(&mut self) {
        let x11_protocol = SupportedProtocol {
            name: "X11".to_string(),
            versions: vec![ProtocolVersion::new(11, 0)],
            capabilities: vec![
                ProtocolCapability {
                    name: "SHAPE".to_string(),
                    version: "1.1".to_string(),
                    required: false,
                },
                ProtocolCapability {
                    name: "RENDER".to_string(),
                    version: "0.11".to_string(),
                    required: false,
                },
                ProtocolCapability {
                    name: "RANDR".to_string(),
                    version: "1.5".to_string(),
                    required: false,
                },
            ],
            default_options: HashMap::new(),
        };

        self.supported_protocols
            .insert("X11".to_string(), x11_protocol);

        debug!("Registered X11 protocol support");
    }

    /// Start protocol negotiation
    pub fn start_negotiation(
        &mut self,
        request: NegotiationRequest,
    ) -> Result<NegotiationResult, NegotiationError> {
        let connection_id = request.connection_id;

        // Check if protocol is supported
        let supported = self
            .supported_protocols
            .get(&request.protocol_name)
            .ok_or_else(|| {
                NegotiationError::ProtocolMismatch(format!(
                    "Protocol {} not supported",
                    request.protocol_name
                ))
            })?;

        debug!(
            "Starting protocol negotiation for connection {} (protocol: {})",
            connection_id, request.protocol_name
        );

        // Find compatible version
        let compatible_version = supported
            .versions
            .iter()
            .find_map(|v| v.min_compatible(&request.client_version))
            .ok_or_else(|| {
                NegotiationError::UnsupportedVersion(
                    request.client_version.major,
                    request.client_version.minor,
                )
            })?;

        info!(
            "Compatible protocol version found: {}.{}",
            compatible_version.major, compatible_version.minor
        );

        // Clone supported to avoid borrow conflict
        let supported_cloned = supported.clone();

        // Create negotiation state
        let state = NegotiationState {
            connection_id,
            protocol_name: request.protocol_name.clone(),
            started_at: std::time::SystemTime::now(),
            step: NegotiationStep::VersionExchange,
        };

        self.active_negotiations.insert(connection_id, state);

        // Start with version confirmation
        self.handle_version_exchange(
            connection_id,
            &request,
            &compatible_version,
            &supported_cloned,
        )
    }

    /// Handle version exchange step
    fn handle_version_exchange(
        &mut self,
        connection_id: ConnectionId,
        request: &NegotiationRequest,
        version: &ProtocolVersion,
        supported: &SupportedProtocol,
    ) -> Result<NegotiationResult, NegotiationError> {
        // Update negotiation step
        if let Some(state) = self.active_negotiations.get_mut(&connection_id) {
            state.step = NegotiationStep::CapabilityNegotiation;
        }

        // Negotiate capabilities
        let agreed_capabilities =
            self.negotiate_capabilities(&request.client_capabilities, &supported.capabilities);

        debug!(
            "Agreed on {} capabilities for connection {}",
            agreed_capabilities.len(),
            connection_id
        );

        // Complete negotiation
        self.complete_negotiation(connection_id, version, &agreed_capabilities, supported)
    }

    /// Negotiate capabilities
    fn negotiate_capabilities(
        &self,
        client_caps: &[ProtocolCapability],
        server_caps: &[ProtocolCapability],
    ) -> Vec<String> {
        let mut agreed = Vec::new();

        for client_cap in client_caps {
            if let Some(_server_cap) = server_caps.iter().find(|c| c.name == client_cap.name) {
                // Simple capability matching - in reality this would be more complex
                agreed.push(client_cap.name.clone());
                debug!("Agreed on capability: {}", client_cap.name);
            } else if client_cap.required {
                warn!("Required capability {} not supported", client_cap.name);
            }
        }

        agreed
    }

    /// Complete negotiation
    fn complete_negotiation(
        &mut self,
        connection_id: ConnectionId,
        version: &ProtocolVersion,
        capabilities: &[String],
        supported: &SupportedProtocol,
    ) -> Result<NegotiationResult, NegotiationError> {
        // Update negotiation step
        if let Some(state) = self.active_negotiations.get_mut(&connection_id) {
            state.step = NegotiationStep::Completed;
        }

        let negotiated = NegotiatedProtocol {
            name: supported.name.clone(),
            major_version: version.major,
            minor_version: version.minor,
            capabilities: capabilities.to_vec(),
            options: supported.default_options.clone(),
        };

        info!(
            "Protocol negotiation completed for connection {}: {}.{}",
            connection_id, version.major, version.minor
        );

        Ok(NegotiationResult::Success(negotiated))
    }

    /// Continue ongoing negotiation
    pub fn continue_negotiation(
        &mut self,
        connection_id: ConnectionId,
        data: Vec<u8>,
    ) -> Result<NegotiationResult, NegotiationError> {
        let state = self
            .active_negotiations
            .get(&connection_id)
            .ok_or_else(|| NegotiationError::Internal("No active negotiation".to_string()))?;

        // Check timeout
        let elapsed = std::time::SystemTime::now()
            .duration_since(state.started_at)
            .unwrap_or_default();

        if elapsed.as_secs() > self.negotiation_timeout {
            self.active_negotiations.remove(&connection_id);
            return Err(NegotiationError::Timeout);
        }

        // Handle based on current step
        match &state.step {
            NegotiationStep::VersionExchange => {
                // Continue with version exchange
                Err(NegotiationError::Internal(
                    "Version exchange continuation not implemented".to_string(),
                ))
            }
            NegotiationStep::CapabilityNegotiation => {
                // Continue with capability negotiation
                Err(NegotiationError::Internal(
                    "Capability negotiation continuation not implemented".to_string(),
                ))
            }
            NegotiationStep::OptionConfiguration => {
                // Continue with option configuration
                Err(NegotiationError::Internal(
                    "Option configuration continuation not implemented".to_string(),
                ))
            }
            NegotiationStep::Completed => Err(NegotiationError::Internal(
                "Negotiation already completed".to_string(),
            )),
            NegotiationStep::Failed => Err(NegotiationError::Internal(
                "Negotiation already failed".to_string(),
            )),
        }
    }

    /// Finish negotiation for a connection
    pub fn finish_negotiation(&mut self, connection_id: ConnectionId) {
        self.active_negotiations.remove(&connection_id);
        debug!("Finished negotiation for connection {}", connection_id);
    }

    /// Get supported protocols
    pub fn get_supported_protocols(&self) -> Vec<String> {
        self.supported_protocols.keys().cloned().collect()
    }

    /// Check if negotiation is active
    pub fn is_negotiating(&self, connection_id: ConnectionId) -> bool {
        self.active_negotiations.contains_key(&connection_id)
    }
}

impl Default for ProtocolNegotiator {
    fn default() -> Self {
        Self::new()
    }
}
