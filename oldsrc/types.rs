//! Core types and shared data structures for the X11 server
//!
//! This module defines fundamental types used throughout the server implementation,
//! following CLEAN architecture principles with clear domain boundaries.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// Server-wide result type with standardized error handling
pub type Result<T> = std::result::Result<T, Error>;

/// Comprehensive error type for the X11 server
#[derive(Debug, Clone)]
pub enum Error {
    /// X11 protocol-specific errors
    Protocol(ProtocolError),
    /// Network communication errors
    Network(NetworkError),
    /// Display backend errors
    Display(DisplayError),
    /// Platform-specific errors
    Platform(PlatformError),
    /// Configuration errors
    Configuration(ConfigurationError),
    /// Security and authentication errors
    Security(SecurityError),
    /// Resource management errors
    Resource(ResourceError),
    /// Input system errors
    Input(InputError),
    /// Font system errors
    Font(FontError),
    /// I/O errors (converted to string to maintain Clone)
    Io(String),
    /// Internal server errors
    Internal(String),
}

/// Error codes for X11 protocol and server operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// Success (not actually an error)
    Success = 0,
    /// Bad request - invalid or malformed request
    BadRequest = 1,
    /// Bad value - parameter out of range
    BadValue = 2,
    /// Bad window - invalid window ID
    BadWindow = 3,
    /// Bad pixmap - invalid pixmap ID
    BadPixmap = 4,
    /// Bad atom - invalid atom value
    BadAtom = 5,
    /// Bad cursor - invalid cursor ID
    BadCursor = 6,
    /// Bad font - invalid font ID
    BadFont = 7,
    /// Bad match - incompatible parameter combination
    BadMatch = 8,
    /// Bad drawable - invalid drawable ID
    BadDrawable = 9,
    /// Bad access - insufficient privileges
    BadAccess = 10,
    /// Bad alloc - insufficient resources
    BadAlloc = 11,
    /// Bad color - invalid colormap ID
    BadColor = 12,
    /// Bad GC - invalid graphics context ID
    BadGC = 13,
    /// Bad ID choice - ID already in use
    BadIDChoice = 14,
    /// Bad name - invalid string parameter
    BadName = 15,
    /// Bad length - request length incorrect
    BadLength = 16,
    /// Bad implementation - server cannot handle request
    BadImplementation = 17,
    /// Invalid state - operation not allowed in current state
    InvalidState = 128,
    /// Timeout - operation timed out
    Timeout = 129,
    /// Resource exhausted - server resources depleted
    ResourceExhausted = 130,
}

/// Server error type - for compatibility with logging and other modules
pub type ServerError = Error;

// Re-export ProtocolError from x11::protocol module
pub use crate::x11::protocol::errors::ProtocolError;

/// Network communication error types
#[derive(Debug, Clone)]
pub enum NetworkError {
    /// Connection failed
    ConnectionFailed(String),
    /// Connection lost
    ConnectionLost,
    /// Connection not found
    ConnectionNotFound(u32),
    /// Too many connections
    TooManyConnections,
    /// Authentication failed
    AuthenticationFailed,
    /// Protocol version mismatch
    ProtocolMismatch { expected: u16, received: u16 },
    /// Timeout occurred
    Timeout,
    /// Invalid message format
    InvalidMessage,
    /// Encryption error
    Encryption(String),
    /// Transport layer error
    Transport(String),
}

/// Display backend error types
#[derive(Debug, Clone)]
pub enum DisplayError {
    /// Backend initialization failed
    InitializationFailed(String),
    /// Rendering operation failed
    RenderingFailed(String),
    /// Resource creation failed
    ResourceCreationFailed(String),
    /// Backend not available
    BackendNotAvailable(String),
    /// Insufficient graphics memory
    InsufficientMemory,
    /// Hardware acceleration not available
    HardwareAccelerationUnavailable,
}

/// Platform-specific error types
#[derive(Debug, Clone)]
pub enum PlatformError {
    /// System call failed
    SystemCallFailed(String),
    /// Platform not supported
    PlatformNotSupported,
    /// Missing system dependencies
    MissingDependencies(Vec<String>),
    /// Permission denied
    PermissionDenied(String),
    /// Resource temporarily unavailable
    ResourceUnavailable(String),
}

/// Configuration error types
#[derive(Debug, Clone)]
pub enum ConfigurationError {
    /// Invalid configuration format
    InvalidFormat(String),
    /// Missing required configuration
    MissingRequired(String),
    /// Invalid configuration value
    InvalidValue {
        key: String,
        value: String,
        reason: String,
    },
    /// Configuration file not found
    FileNotFound(String),
    /// Permission denied reading configuration
    PermissionDenied(String),
    /// Unsupported configuration format
    UnsupportedFormat(String),
    /// Unsupported configuration source
    UnsupportedSource(String),
    /// Parse error
    ParseError { format: String, message: String },
    /// Serialization error
    SerializationError { format: String, message: String },
    /// File operation error
    FileError {
        path: std::path::PathBuf,
        message: String,
    },
    /// Configuration already initialized
    AlreadyInitialized,
    /// Configuration not initialized
    NotInitialized,
}

/// Security and authentication error types
#[derive(Debug, Clone)]
pub enum SecurityError {
    /// Authentication failed
    AuthenticationFailed(String),
    /// Authorization denied
    AuthorizationDenied(String),
    /// Invalid credentials
    InvalidCredentials,
    /// Security policy violation
    PolicyViolation(String),
    /// Encryption/decryption failed
    CryptographyFailed(String),
    /// Certificate validation failed
    CertificateValidationFailed(String),
}

/// Resource management error types
#[derive(Debug, Clone)]
pub enum ResourceError {
    /// Resource not found
    NotFound { resource_type: String, id: u32 },
    /// Resource already exists
    AlreadyExists { resource_type: String, id: u32 },
    /// Resource limit exceeded
    LimitExceeded(String),
    /// Invalid resource state
    InvalidState {
        resource_type: String,
        id: u32,
        state: String,
    },
    /// Resource cleanup failed
    CleanupFailed(String),
}

/// Input system error types
#[derive(Debug, Clone)]
pub enum InputError {
    /// Input device not found
    DeviceNotFound(String),
    /// Invalid input event
    InvalidEvent(String),
    /// Input device busy
    DeviceBusy(String),
    /// Grab failed
    GrabFailed(String),
}

/// Font system error types
#[derive(Debug, Clone)]
pub enum FontError {
    /// Font not found
    FontNotFound(String),
    /// Invalid font format
    InvalidFormat(String),
    /// Font rendering failed
    RenderingFailed(String),
    /// Font cache error
    CacheError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Protocol(e) => write!(f, "Protocol error: {}", e),
            Error::Network(e) => write!(f, "Network error: {}", e),
            Error::Display(e) => write!(f, "Display error: {}", e),
            Error::Platform(e) => write!(f, "Platform error: {}", e),
            Error::Configuration(e) => write!(f, "Configuration error: {}", e),
            Error::Security(e) => write!(f, "Security error: {}", e),
            Error::Resource(e) => write!(f, "Resource error: {}", e),
            Error::Input(e) => write!(f, "Input error: {}", e),
            Error::Font(e) => write!(f, "Font error: {}", e),
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

// Display implementations for all error types
// Note: ProtocolError Display impl is in x11::protocol::errors module

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            NetworkError::ConnectionLost => write!(f, "Connection lost"),
            NetworkError::ConnectionNotFound(id) => write!(f, "Connection not found: {}", id),
            NetworkError::TooManyConnections => write!(f, "Too many connections"),
            NetworkError::AuthenticationFailed => write!(f, "Authentication failed"),
            NetworkError::ProtocolMismatch { expected, received } => {
                write!(
                    f,
                    "Protocol version mismatch: expected {}, received {}",
                    expected, received
                )
            }
            NetworkError::Timeout => write!(f, "Timeout"),
            NetworkError::InvalidMessage => write!(f, "Invalid message"),
            NetworkError::Encryption(msg) => write!(f, "Encryption error: {}", msg),
            NetworkError::Transport(msg) => write!(f, "Transport error: {}", msg),
        }
    }
}

impl fmt::Display for DisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplayError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            DisplayError::RenderingFailed(msg) => write!(f, "Rendering failed: {}", msg),
            DisplayError::ResourceCreationFailed(msg) => {
                write!(f, "Resource creation failed: {}", msg)
            }
            DisplayError::BackendNotAvailable(msg) => write!(f, "Backend not available: {}", msg),
            DisplayError::InsufficientMemory => write!(f, "Insufficient graphics memory"),
            DisplayError::HardwareAccelerationUnavailable => {
                write!(f, "Hardware acceleration unavailable")
            }
        }
    }
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::SystemCallFailed(msg) => write!(f, "System call failed: {}", msg),
            PlatformError::PlatformNotSupported => write!(f, "Platform not supported"),
            PlatformError::MissingDependencies(deps) => {
                write!(f, "Missing dependencies: {}", deps.join(", "))
            }
            PlatformError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            PlatformError::ResourceUnavailable(msg) => write!(f, "Resource unavailable: {}", msg),
        }
    }
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigurationError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ConfigurationError::MissingRequired(key) => write!(f, "Missing required: {}", key),
            ConfigurationError::InvalidValue { key, value, reason } => {
                write!(f, "Invalid value for {}: {} ({})", key, value, reason)
            }
            ConfigurationError::FileNotFound(path) => write!(f, "File not found: {}", path),
            ConfigurationError::PermissionDenied(path) => write!(f, "Permission denied: {}", path),
            ConfigurationError::UnsupportedFormat(format) => {
                write!(f, "Unsupported format: {}", format)
            }
            ConfigurationError::UnsupportedSource(source) => {
                write!(f, "Unsupported source: {}", source)
            }
            ConfigurationError::ParseError { format, message } => {
                write!(f, "Parse error in {}: {}", format, message)
            }
            ConfigurationError::SerializationError { format, message } => {
                write!(f, "Serialization error in {}: {}", format, message)
            }
            ConfigurationError::FileError { path, message } => {
                write!(f, "File error for {}: {}", path.display(), message)
            }
            ConfigurationError::AlreadyInitialized => {
                write!(f, "Configuration already initialized")
            }
            ConfigurationError::NotInitialized => write!(f, "Configuration not initialized"),
        }
    }
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityError::AuthenticationFailed(msg) => write!(f, "Authentication failed: {}", msg),
            SecurityError::AuthorizationDenied(msg) => write!(f, "Authorization denied: {}", msg),
            SecurityError::InvalidCredentials => write!(f, "Invalid credentials"),
            SecurityError::PolicyViolation(msg) => write!(f, "Policy violation: {}", msg),
            SecurityError::CryptographyFailed(msg) => write!(f, "Cryptography failed: {}", msg),
            SecurityError::CertificateValidationFailed(msg) => {
                write!(f, "Certificate validation failed: {}", msg)
            }
        }
    }
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceError::NotFound { resource_type, id } => {
                write!(f, "{} not found: {}", resource_type, id)
            }
            ResourceError::AlreadyExists { resource_type, id } => {
                write!(f, "{} already exists: {}", resource_type, id)
            }
            ResourceError::LimitExceeded(msg) => write!(f, "Limit exceeded: {}", msg),
            ResourceError::InvalidState {
                resource_type,
                id,
                state,
            } => {
                write!(f, "{} {} in invalid state: {}", resource_type, id, state)
            }
            ResourceError::CleanupFailed(msg) => write!(f, "Cleanup failed: {}", msg),
        }
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputError::DeviceNotFound(name) => write!(f, "Device not found: {}", name),
            InputError::InvalidEvent(msg) => write!(f, "Invalid event: {}", msg),
            InputError::DeviceBusy(name) => write!(f, "Device busy: {}", name),
            InputError::GrabFailed(msg) => write!(f, "Grab failed: {}", msg),
        }
    }
}

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontError::FontNotFound(name) => write!(f, "Font not found: {}", name),
            FontError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            FontError::RenderingFailed(msg) => write!(f, "Rendering failed: {}", msg),
            FontError::CacheError(msg) => write!(f, "Cache error: {}", msg),
        }
    }
}

/// X11 Resource ID - unique identifier for X11 resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);

impl ResourceId {
    /// Create a new resource ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub fn raw(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:08x}", self.0)
    }
}

/// X11 Atom - unique identifier for interned strings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Atom(pub u32);

impl Atom {
    /// Create a new atom
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw atom value
    pub fn raw(&self) -> u32 {
        self.0
    }

    /// Predefined atoms from the X11 specification
    pub const NONE: Atom = Atom(0);
    pub const ANY_PROPERTY_TYPE: Atom = Atom(0);
    pub const PRIMARY: Atom = Atom(1);
    pub const SECONDARY: Atom = Atom(2);
    pub const ARC: Atom = Atom(3);
    pub const ATOM: Atom = Atom(4);
    pub const BITMAP: Atom = Atom(5);
    pub const CARDINAL: Atom = Atom(6);
    pub const COLORMAP: Atom = Atom(7);
    pub const CURSOR: Atom = Atom(8);
    pub const CUT_BUFFER0: Atom = Atom(9);
    pub const CUT_BUFFER1: Atom = Atom(10);
    pub const CUT_BUFFER2: Atom = Atom(11);
    pub const CUT_BUFFER3: Atom = Atom(12);
    pub const CUT_BUFFER4: Atom = Atom(13);
    pub const CUT_BUFFER5: Atom = Atom(14);
    pub const CUT_BUFFER6: Atom = Atom(15);
    pub const CUT_BUFFER7: Atom = Atom(16);
    pub const DRAWABLE: Atom = Atom(17);
    pub const FONT: Atom = Atom(18);
    pub const INTEGER: Atom = Atom(19);
    pub const PIXMAP: Atom = Atom(20);
    pub const POINT: Atom = Atom(21);
    pub const RECTANGLE: Atom = Atom(22);
    pub const RESOURCE_MANAGER: Atom = Atom(23);
    pub const RGB_COLOR_MAP: Atom = Atom(24);
    pub const RGB_BEST_MAP: Atom = Atom(25);
    pub const RGB_BLUE_MAP: Atom = Atom(26);
    pub const RGB_DEFAULT_MAP: Atom = Atom(27);
    pub const RGB_GRAY_MAP: Atom = Atom(28);
    pub const RGB_GREEN_MAP: Atom = Atom(29);
    pub const RGB_RED_MAP: Atom = Atom(30);
    pub const STRING: Atom = Atom(31);
    pub const VISUALID: Atom = Atom(32);
    pub const WINDOW: Atom = Atom(33);
    pub const WM_COMMAND: Atom = Atom(34);
    pub const WM_HINTS: Atom = Atom(35);
    pub const WM_CLIENT_MACHINE: Atom = Atom(36);
    pub const WM_ICON_NAME: Atom = Atom(37);
    pub const WM_ICON_SIZE: Atom = Atom(38);
    pub const WM_NAME: Atom = Atom(39);
    pub const WM_NORMAL_HINTS: Atom = Atom(40);
    pub const WM_SIZE_HINTS: Atom = Atom(41);
    pub const WM_ZOOM_HINTS: Atom = Atom(42);
    pub const MIN_SPACE: Atom = Atom(43);
    pub const NORM_SPACE: Atom = Atom(44);
    pub const MAX_SPACE: Atom = Atom(45);
    pub const END_SPACE: Atom = Atom(46);
    pub const SUPERSCRIPT_X: Atom = Atom(47);
    pub const SUPERSCRIPT_Y: Atom = Atom(48);
    pub const SUBSCRIPT_X: Atom = Atom(49);
    pub const SUBSCRIPT_Y: Atom = Atom(50);
    pub const UNDERLINE_POSITION: Atom = Atom(51);
    pub const UNDERLINE_THICKNESS: Atom = Atom(52);
    pub const STRIKEOUT_ASCENT: Atom = Atom(53);
    pub const STRIKEOUT_DESCENT: Atom = Atom(54);
    pub const ITALIC_ANGLE: Atom = Atom(55);
    pub const X_HEIGHT: Atom = Atom(56);
    pub const QUAD_WIDTH: Atom = Atom(57);
    pub const WEIGHT: Atom = Atom(58);
    pub const POINT_SIZE: Atom = Atom(59);
    pub const RESOLUTION: Atom = Atom(60);
    pub const COPYRIGHT: Atom = Atom(61);
    pub const NOTICE: Atom = Atom(62);
    pub const FONT_NAME: Atom = Atom(63);
    pub const FAMILY_NAME: Atom = Atom(64);
    pub const FULL_NAME: Atom = Atom(65);
    pub const CAP_HEIGHT: Atom = Atom(66);
    pub const WM_CLASS: Atom = Atom(67);
    pub const WM_TRANSIENT_FOR: Atom = Atom(68);
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Atom({})", self.0)
    }
}

/// X11 Timestamp - time in milliseconds since server start
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(pub u32);

impl Timestamp {
    /// Create a new timestamp
    pub fn new(time: u32) -> Self {
        Self(time)
    }

    /// Get the raw timestamp value
    pub fn raw(&self) -> u32 {
        self.0
    }

    /// Current time constant
    pub const CURRENT_TIME: Timestamp = Timestamp(0);
}

// Re-export geometric types from x11::geometry module
pub use crate::x11::geometry::types::{Point, Rectangle, Size};

// Re-export Color from x11::protocol module
pub use crate::x11::protocol::types::Color;

/// Configuration properties
pub type Properties = HashMap<String, String>;

/// Reference-counted configuration
pub type SharedConfig = Arc<Properties>;

/// Client identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(pub u32);

impl ClientId {
    /// Create a new client ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get raw client ID
    pub fn raw(&self) -> u32 {
        self.0
    }
}

/// Extension identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtensionId(pub String);

impl ExtensionId {
    /// Create a new extension ID
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get extension name
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// Version information
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    /// Major version
    pub major: u16,
    /// Minor version
    pub minor: u16,
}

impl Version {
    /// Create a new version
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

// From trait implementations for error conversions

// Configuration error conversions
impl From<ConfigurationError> for Error {
    fn from(err: ConfigurationError) -> Self {
        Error::Configuration(err)
    }
}

impl From<std::io::Error> for ConfigurationError {
    fn from(err: std::io::Error) -> Self {
        ConfigurationError::FileError {
            path: std::path::PathBuf::new(),
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for ConfigurationError {
    fn from(err: serde_json::Error) -> Self {
        ConfigurationError::ParseError {
            format: "json".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<toml::de::Error> for ConfigurationError {
    fn from(err: toml::de::Error) -> Self {
        ConfigurationError::ParseError {
            format: "toml".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<toml::ser::Error> for ConfigurationError {
    fn from(err: toml::ser::Error) -> Self {
        ConfigurationError::SerializationError {
            format: "toml".to_string(),
            message: err.to_string(),
        }
    }
}

// Command line parsing error conversion
impl From<clap::error::Error> for Error {
    fn from(err: clap::error::Error) -> Self {
        Error::Configuration(ConfigurationError::ParseError {
            format: "command_line".to_string(),
            message: err.to_string(),
        })
    }
}

// Network error conversions
impl From<NetworkError> for Error {
    fn from(err: NetworkError) -> Self {
        Error::Network(err)
    }
}

// Display error conversions
impl From<DisplayError> for Error {
    fn from(err: DisplayError) -> Self {
        Error::Display(err)
    }
}

// Platform error conversions
impl From<PlatformError> for Error {
    fn from(err: PlatformError) -> Self {
        Error::Platform(err)
    }
}

// Security error conversions
impl From<SecurityError> for Error {
    fn from(err: SecurityError) -> Self {
        Error::Security(err)
    }
}

// Resource error conversions
impl From<ResourceError> for Error {
    fn from(err: ResourceError) -> Self {
        Error::Resource(err)
    }
}

// Input error conversions
impl From<InputError> for Error {
    fn from(err: InputError) -> Self {
        Error::Input(err)
    }
}

// Font error conversions
impl From<FontError> for Error {
    fn from(err: FontError) -> Self {
        Error::Font(err)
    }
}

// I/O error conversions
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}
