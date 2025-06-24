//! Configuration type definitions
//!
//! This module defines the core configuration structures used throughout the server.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;

/// Main server configuration structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server identification and metadata
    pub server: ServerInfo,
    /// Network configuration
    pub network: NetworkConfig,
    /// Display configuration
    pub display: DisplayConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Feature toggles
    pub features: FeatureConfig,
    /// Extension-specific configurations
    pub extensions: HashMap<String, serde_json::Value>,
}

/// Server identification and metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// Vendor information
    pub vendor: String,
    /// Release number
    pub release: u32,
    /// Display number
    pub display_number: u16,
    /// Screen count
    pub screen_count: u8,
}

/// Network configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// TCP bind addresses
    pub tcp_addresses: Vec<SocketAddr>,
    /// Unix socket paths
    pub unix_sockets: Vec<PathBuf>,
    /// Maximum connections
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Enable authentication
    pub authentication_enabled: bool,
    /// Authentication methods
    pub auth_methods: Vec<String>,
}

/// Display configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Default screen resolution
    pub default_resolution: Resolution,
    /// Supported resolutions
    pub supported_resolutions: Vec<Resolution>,
    /// Color depth
    pub color_depth: u8,
    /// DPI (dots per inch)
    pub dpi: f32,
    /// Refresh rate in Hz
    pub refresh_rate: u32,
    /// Display backend to use
    pub backend: String,
    /// Backend-specific options
    pub backend_options: HashMap<String, serde_json::Value>,
}

/// Screen resolution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

/// Security configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable access control
    pub access_control_enabled: bool,
    /// Allowed hosts
    pub allowed_hosts: Vec<String>,
    /// Enable audit logging
    pub audit_enabled: bool,
    /// Audit log path
    pub audit_log_path: Option<PathBuf>,
    /// Security policies
    pub policies: Vec<String>,
}

/// Logging configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (error, warn, info, debug, trace)
    pub level: String,
    /// Log output destinations
    pub outputs: Vec<LogOutput>,
    /// Enable structured logging
    pub structured: bool,
    /// Log rotation settings
    pub rotation: Option<LogRotation>,
}

/// Log output configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogOutput {
    /// Output type (console, file, syslog, etc.)
    pub output_type: String,
    /// Output-specific configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Log rotation configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogRotation {
    /// Maximum file size in bytes
    pub max_size: u64,
    /// Maximum number of archived files
    pub max_files: u32,
    /// Compression enabled
    pub compress: bool,
}

/// Performance configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Thread pool size
    pub thread_pool_size: Option<usize>,
    /// Request queue size
    pub request_queue_size: u32,
    /// Event queue size
    pub event_queue_size: u32,
    /// Memory pool settings
    pub memory_pools: MemoryPoolConfig,
    /// Caching settings
    pub caching: CacheConfig,
}

/// Memory pool configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryPoolConfig {
    /// Enable memory pooling
    pub enabled: bool,
    /// Initial pool size
    pub initial_size: usize,
    /// Maximum pool size
    pub max_size: usize,
    /// Block size
    pub block_size: usize,
}

/// Cache configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Font cache size
    pub font_cache_size: usize,
    /// Pixmap cache size
    pub pixmap_cache_size: usize,
    /// Glyph cache size
    pub glyph_cache_size: usize,
    /// Cache TTL in seconds
    pub ttl: u64,
}

/// Feature toggles
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// Enable X11 extensions
    pub extensions_enabled: bool,
    /// Enable compositing
    pub compositing_enabled: bool,
    /// Enable damage tracking
    pub damage_tracking_enabled: bool,
    /// Enable performance monitoring
    pub performance_monitoring_enabled: bool,
    /// Enable debugging features
    pub debug_features_enabled: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server: ServerInfo::default(),
            network: NetworkConfig::default(),
            display: DisplayConfig::default(),
            security: SecurityConfig::default(),
            logging: LoggingConfig::default(),
            performance: PerformanceConfig::default(),
            features: FeatureConfig::default(),
            extensions: HashMap::new(),
        }
    }
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            name: "RXServer".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            vendor: "RXServer Team".to_string(),
            release: 1,
            display_number: 0,
            screen_count: 1,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            tcp_addresses: vec!["127.0.0.1:6000".parse().unwrap()],
            unix_sockets: vec![PathBuf::from("/tmp/.X11-unix/X0")],
            max_connections: 100,
            connection_timeout: 30,
            authentication_enabled: true,
            auth_methods: vec!["MIT-MAGIC-COOKIE-1".to_string()],
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            default_resolution: Resolution {
                width: 1920,
                height: 1080,
            },
            supported_resolutions: vec![
                Resolution {
                    width: 1920,
                    height: 1080,
                },
                Resolution {
                    width: 1280,
                    height: 720,
                },
                Resolution {
                    width: 800,
                    height: 600,
                },
            ],
            color_depth: 24,
            dpi: 96.0,
            refresh_rate: 60,
            backend: "software".to_string(),
            backend_options: HashMap::new(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            access_control_enabled: true,
            allowed_hosts: vec!["localhost".to_string(), "127.0.0.1".to_string()],
            audit_enabled: false,
            audit_log_path: None,
            policies: Vec::new(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "trace".to_string(),
            outputs: vec![LogOutput {
                output_type: "console".to_string(),
                config: HashMap::new(),
            }],
            structured: false,
            rotation: None,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            thread_pool_size: None, // Auto-detect based on CPU cores
            request_queue_size: 1000,
            event_queue_size: 5000,
            memory_pools: MemoryPoolConfig::default(),
            caching: CacheConfig::default(),
        }
    }
}

impl Default for MemoryPoolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_size: 1024 * 1024,  // 1MB
            max_size: 64 * 1024 * 1024, // 64MB
            block_size: 4096,           // 4KB
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            font_cache_size: 100,
            pixmap_cache_size: 1000,
            glyph_cache_size: 10000,
            ttl: 3600, // 1 hour
        }
    }
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            extensions_enabled: true,
            compositing_enabled: false,
            damage_tracking_enabled: true,
            performance_monitoring_enabled: false,
            debug_features_enabled: cfg!(debug_assertions),
        }
    }
}

impl ServerConfig {
    /// Merge this configuration with another, with the other taking precedence
    pub fn merge(mut self, other: ServerConfig) -> crate::types::Result<Self> {
        // Simple field-by-field merge for now
        // TODO: Implement more sophisticated merging logic

        self.server = other.server;
        self.network = other.network;
        self.display = other.display;
        self.security = other.security;
        self.logging = other.logging;
        self.performance = other.performance;
        self.features = other.features;

        // Merge extensions map
        for (key, value) in other.extensions {
            self.extensions.insert(key, value);
        }

        Ok(self)
    }
}
