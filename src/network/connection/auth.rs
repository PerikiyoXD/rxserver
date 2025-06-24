//! Authentication management for X11 connections
//!
//! Handles various X11 authentication mechanisms and authorization.

use crate::network::ConnectionId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Authentication result
#[derive(Debug, Clone, PartialEq)]
pub enum AuthenticationResult {
    /// Authentication successful
    Success,
    /// Authentication failed
    Failed(String),
    /// Additional authentication data required
    ContinueAuth(Vec<u8>),
    /// Authentication not required
    NotRequired,
}

/// Authentication error
#[derive(Debug, thiserror::Error)]
pub enum AuthenticationError {
    #[error("Invalid protocol: {0}")]
    InvalidProtocol(String),

    #[error("Invalid data format: {0}")]
    InvalidData(String),

    #[error("Authentication failed: {0}")]
    Failed(String),

    #[error("Protocol not supported: {0}")]
    NotSupported(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Authentication protocol types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AuthProtocol {
    /// No authentication
    None,
    /// MIT Magic Cookie authentication
    MitMagicCookie1,
    /// XDM authorization
    XdmAuthorization1,
    /// Kerberos authentication
    Kerberos5,
    /// Custom/unknown protocol
    Custom(String),
}

impl AuthProtocol {
    /// Parse protocol name string
    pub fn from_name(name: &str) -> Self {
        match name {
            "" => AuthProtocol::None,
            "MIT-MAGIC-COOKIE-1" => AuthProtocol::MitMagicCookie1,
            "XDM-AUTHORIZATION-1" => AuthProtocol::XdmAuthorization1,
            "KERBEROS_V5" => AuthProtocol::Kerberos5,
            other => AuthProtocol::Custom(other.to_string()),
        }
    }

    /// Get protocol name string
    pub fn name(&self) -> &str {
        match self {
            AuthProtocol::None => "",
            AuthProtocol::MitMagicCookie1 => "MIT-MAGIC-COOKIE-1",
            AuthProtocol::XdmAuthorization1 => "XDM-AUTHORIZATION-1",
            AuthProtocol::Kerberos5 => "KERBEROS_V5",
            AuthProtocol::Custom(name) => name,
        }
    }
}

/// Authentication request
#[derive(Debug, Clone)]
pub struct AuthRequest {
    /// Connection ID
    pub connection_id: ConnectionId,
    /// Authentication protocol
    pub protocol: AuthProtocol,
    /// Protocol data
    pub data: Vec<u8>,
    /// Client machine/host information
    pub client_machine: Option<String>,
    /// Process ID
    pub client_pid: Option<u32>,
}

/// Authentication context
#[derive(Debug)]
struct AuthContext {
    /// Connection ID
    connection_id: ConnectionId,
    /// Authentication protocol
    protocol: AuthProtocol,
    /// Authentication state
    state: AuthState,
    /// Authentication attempts
    attempts: u32,
    /// Last attempt timestamp
    last_attempt: std::time::SystemTime,
}

/// Authentication state
#[derive(Debug, Clone, PartialEq)]
enum AuthState {
    /// Initial state
    Initial,
    /// In progress
    InProgress,
    /// Completed successfully
    Completed,
    /// Failed
    Failed,
}

/// Authentication manager configuration
#[derive(Debug, Clone)]
pub struct AuthManagerConfig {
    /// Maximum authentication attempts per connection
    pub max_attempts: u32,
    /// Authentication timeout in seconds
    pub auth_timeout: u64,
    /// Allowed authentication protocols
    pub allowed_protocols: Vec<AuthProtocol>,
    /// Require authentication
    pub require_auth: bool,
    /// MIT Magic Cookie file path
    pub cookie_file_path: Option<std::path::PathBuf>,
}

impl Default for AuthManagerConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            auth_timeout: 30,
            allowed_protocols: vec![AuthProtocol::None, AuthProtocol::MitMagicCookie1],
            require_auth: false,
            cookie_file_path: None,
        }
    }
}

/// Authentication manager
pub struct AuthenticationManager {
    /// Configuration
    config: AuthManagerConfig,
    /// Active authentication contexts
    contexts: Arc<RwLock<HashMap<ConnectionId, AuthContext>>>,
    /// Authentication handlers
    handlers: HashMap<AuthProtocol, Box<dyn AuthHandler + Send + Sync>>,
    /// Authentication statistics
    stats: Arc<RwLock<AuthStats>>,
}

/// Authentication statistics
#[derive(Debug, Clone)]
pub struct AuthStats {
    /// Total authentication attempts
    pub total_attempts: u64,
    /// Successful authentications
    pub successful_auths: u64,
    /// Failed authentications
    pub failed_auths: u64,
    /// Authentication attempts by protocol
    pub attempts_by_protocol: HashMap<AuthProtocol, u64>,
}

impl Default for AuthStats {
    fn default() -> Self {
        Self {
            total_attempts: 0,
            successful_auths: 0,
            failed_auths: 0,
            attempts_by_protocol: HashMap::new(),
        }
    }
}

/// Authentication handler trait
pub trait AuthHandler: std::fmt::Debug {
    /// Authenticate a request
    fn authenticate(
        &self,
        request: &AuthRequest,
    ) -> Result<AuthenticationResult, AuthenticationError>;

    /// Get protocol supported by this handler
    fn protocol(&self) -> AuthProtocol;
}

/// No authentication handler
#[derive(Debug)]
struct NoAuthHandler;

impl AuthHandler for NoAuthHandler {
    fn authenticate(
        &self,
        _request: &AuthRequest,
    ) -> Result<AuthenticationResult, AuthenticationError> {
        Ok(AuthenticationResult::Success)
    }

    fn protocol(&self) -> AuthProtocol {
        AuthProtocol::None
    }
}

/// MIT Magic Cookie authentication handler
#[derive(Debug)]
struct MitMagicCookieHandler {
    /// Valid cookies
    cookies: Vec<Vec<u8>>,
}

impl MitMagicCookieHandler {
    fn new() -> Self {
        Self {
            cookies: Vec::new(),
        }
    }

    fn add_cookie(&mut self, cookie: Vec<u8>) {
        self.cookies.push(cookie);
    }

    fn load_cookies_from_file(
        &mut self,
        path: &std::path::Path,
    ) -> Result<(), AuthenticationError> {
        // TODO: Implement Xauth file parsing
        // For now, just add a dummy cookie
        self.cookies.push(vec![0u8; 16]); // 128-bit dummy cookie

        debug!("Loaded cookies from file: {}", path.display());
        Ok(())
    }
}

impl AuthHandler for MitMagicCookieHandler {
    fn authenticate(
        &self,
        request: &AuthRequest,
    ) -> Result<AuthenticationResult, AuthenticationError> {
        if request.data.len() != 16 {
            return Ok(AuthenticationResult::Failed(
                "Invalid cookie length".to_string(),
            ));
        }

        for cookie in &self.cookies {
            if cookie == &request.data {
                return Ok(AuthenticationResult::Success);
            }
        }

        Ok(AuthenticationResult::Failed("Invalid cookie".to_string()))
    }

    fn protocol(&self) -> AuthProtocol {
        AuthProtocol::MitMagicCookie1
    }
}

impl AuthenticationManager {
    /// Create a new authentication manager
    pub fn new(config: AuthManagerConfig) -> Result<Self, AuthenticationError> {
        let mut manager = Self {
            config: config.clone(),
            contexts: Arc::new(RwLock::new(HashMap::new())),
            handlers: HashMap::new(),
            stats: Arc::new(RwLock::new(AuthStats::default())),
        };

        // Register default handlers
        manager.register_handler(Box::new(NoAuthHandler))?;

        // Register MIT Magic Cookie handler
        let mut cookie_handler = MitMagicCookieHandler::new();
        if let Some(cookie_file) = &config.cookie_file_path {
            if cookie_file.exists() {
                cookie_handler.load_cookies_from_file(cookie_file)?;
            }
        }
        manager.register_handler(Box::new(cookie_handler))?;

        info!(
            "Authentication manager initialized with {} handlers",
            manager.handlers.len()
        );

        Ok(manager)
    }

    /// Register an authentication handler
    pub fn register_handler(
        &mut self,
        handler: Box<dyn AuthHandler + Send + Sync>,
    ) -> Result<(), AuthenticationError> {
        let protocol = handler.protocol();

        debug!(
            "Registering authentication handler for protocol: {:?}",
            protocol
        );
        self.handlers.insert(protocol, handler);

        Ok(())
    }

    /// Authenticate a connection
    pub async fn authenticate(
        &self,
        request: AuthRequest,
    ) -> Result<AuthenticationResult, AuthenticationError> {
        let connection_id = request.connection_id;

        // Check if protocol is allowed
        if !self.config.allowed_protocols.contains(&request.protocol) {
            return Err(AuthenticationError::NotSupported(format!(
                "Protocol {:?} not allowed",
                request.protocol
            )));
        }

        // Check authentication context
        {
            let mut contexts = self.contexts.write().await;
            let context = contexts
                .entry(connection_id)
                .or_insert_with(|| AuthContext {
                    connection_id,
                    protocol: request.protocol.clone(),
                    state: AuthState::Initial,
                    attempts: 0,
                    last_attempt: std::time::SystemTime::now(),
                });

            // Check attempt limit
            if context.attempts >= self.config.max_attempts {
                context.state = AuthState::Failed;
                return Ok(AuthenticationResult::Failed(
                    "Too many attempts".to_string(),
                ));
            }

            // Check timeout
            let elapsed = std::time::SystemTime::now()
                .duration_since(context.last_attempt)
                .unwrap_or_default();

            if elapsed.as_secs() > self.config.auth_timeout {
                context.state = AuthState::Failed;
                return Ok(AuthenticationResult::Failed(
                    "Authentication timeout".to_string(),
                ));
            }

            context.attempts += 1;
            context.last_attempt = std::time::SystemTime::now();
            context.state = AuthState::InProgress;
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_attempts += 1;
            *stats
                .attempts_by_protocol
                .entry(request.protocol.clone())
                .or_insert(0) += 1;
        }

        // Find and use appropriate handler
        let result = if let Some(handler) = self.handlers.get(&request.protocol) {
            debug!(
                "Authenticating connection {} with protocol {:?}",
                connection_id, request.protocol
            );

            handler.authenticate(&request).unwrap_or_else(|e| {
                error!("Authentication handler error: {}", e);
                AuthenticationResult::Failed("Internal authentication error".to_string())
            })
        } else {
            AuthenticationResult::Failed(format!("No handler for protocol {:?}", request.protocol))
        };

        // Update context based on result
        {
            let mut contexts = self.contexts.write().await;
            if let Some(context) = contexts.get_mut(&connection_id) {
                match &result {
                    AuthenticationResult::Success => {
                        context.state = AuthState::Completed;
                        info!("Authentication successful for connection {}", connection_id);
                    }
                    AuthenticationResult::Failed(reason) => {
                        context.state = AuthState::Failed;
                        warn!(
                            "Authentication failed for connection {}: {}",
                            connection_id, reason
                        );
                    }
                    AuthenticationResult::ContinueAuth(_) => {
                        // Keep state as InProgress
                    }
                    AuthenticationResult::NotRequired => {
                        context.state = AuthState::Completed;
                    }
                }
            }
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            match &result {
                AuthenticationResult::Success | AuthenticationResult::NotRequired => {
                    stats.successful_auths += 1;
                }
                AuthenticationResult::Failed(_) => {
                    stats.failed_auths += 1;
                }
                _ => {}
            }
        }

        Ok(result)
    }

    /// Remove authentication context for a connection
    pub async fn remove_context(&self, connection_id: ConnectionId) {
        let mut contexts = self.contexts.write().await;
        contexts.remove(&connection_id);

        debug!(
            "Removed authentication context for connection {}",
            connection_id
        );
    }

    /// Check if connection is authenticated
    pub async fn is_authenticated(&self, connection_id: ConnectionId) -> bool {
        let contexts = self.contexts.read().await;

        if let Some(context) = contexts.get(&connection_id) {
            context.state == AuthState::Completed
        } else {
            false
        }
    }

    /// Get authentication statistics
    pub async fn get_statistics(&self) -> AuthStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Get supported protocols
    pub fn get_supported_protocols(&self) -> Vec<AuthProtocol> {
        self.handlers.keys().cloned().collect()
    }
}
