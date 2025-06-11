// SPDX-License-Identifier: Apache-2.0
// RX-Completion-Status: Complete

//! Configuration type definitions for the RX server
//!
//! This module defines all configuration structures used throughout the server,
//! with support for TOML serialization/deserialization and comprehensive defaults.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{graphics::types::GraphicsBackend, logging::types::LogLevel};

use super::defaults::*;

/// Main server configuration structure
///
/// This structure contains all configuration sections for the RX server,
/// organizing settings by functional area for better maintainability.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct ServerConfig {
    /// Core server settings (ports, connections, etc.)
    pub server: ServerSettings,
    /// Transport layer configuration
    pub transport: TransportSettings,
    /// Network and security settings  
    pub network: NetworkSettings,
    /// Display and screen settings
    pub display: DisplaySettings,
    /// Graphics rendering configuration
    pub graphics: GraphicsSettings,
    /// Input device settings
    pub input: InputSettings,
    /// Logging and debugging configuration
    pub logging: LoggingSettings,
}

/// Core server operation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct ServerSettings {
    /// Maximum number of concurrent client connections    #[serde(default = "default_max_clients")]
    pub max_clients: usize,
    /// X11 display number (e.g., 0 for :0)
    #[serde(default)]
    pub display_number: u8,
    /// Base TCP port for X11 connections (actual port = base + display_number)
    #[serde(default = "default_tcp_port_base")]
    pub tcp_port_base: u16,
    /// Whether to enable TCP connections (Unix sockets are always enabled)
    #[serde(default)]
    pub enable_tcp: bool,
    /// Unix socket path template with {display} placeholder
    #[serde(default = "default_unix_socket_path")]
    pub unix_socket_path: String,
}

/// Transport layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct TransportSettings {
    /// Maximum buffer size for transport operations (bytes)
    #[serde(default = "default_max_buffer_size")]
    pub max_buffer_size: usize,
    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,
    /// Enable compression for data transport
    #[serde(default)]
    pub enable_compression: bool,
}

/// Network and security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct NetworkSettings {
    /// Enable IPv6 support
    #[serde(default)]
    pub enable_ipv6: bool,
    /// List of allowed IP addresses (empty = allow all)
    #[serde(default)]    pub allowed_ips: Vec<String>,
    /// Enable access control mechanisms
    #[serde(default)]
    pub enable_access_control: bool,
}

/// Display and screen configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct DisplaySettings {
    /// Screen width in pixels
    #[serde(default = "default_width")]
    pub width: u32,
    /// Screen height in pixels
    #[serde(default = "default_height")]
    pub height: u32,
    /// Color depth in bits per pixel (8, 16, 24, or 32)
    #[serde(default = "default_depth")]
    pub depth: u8,
    /// DPI (dots per inch) for font rendering
    #[serde(default = "default_dpi")]
    pub dpi: u16,
    /// Refresh rate in Hz
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate: u16,
}

/// Graphics rendering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct GraphicsSettings {
    /// Enable hardware acceleration if available
    #[serde(default = "default_hw_accel")]
    pub hardware_acceleration: bool,
    /// Graphics backend to use (Software, OpenGL, Vulkan, etc.)
    #[serde(default = "default_backend")]
    pub backend: GraphicsBackend,
    /// Maximum texture size for hardware acceleration
    #[serde(default = "default_max_texture_size")]
    pub max_texture_size: u32,
    /// Enable vertical synchronization    #[serde(default)]
    pub enable_vsync: bool,
}

/// Input device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct InputSettings {
    /// Keyboard repeat delay in milliseconds
    #[serde(default = "default_kb_repeat_delay")]
    pub keyboard_repeat_delay: u32,
    /// Keyboard repeat rate in Hz
    #[serde(default = "default_kb_repeat_rate")]
    pub keyboard_repeat_rate: u32,
    /// Mouse acceleration factor
    #[serde(default = "default_mouse_accel")]
    pub mouse_acceleration: f32,
    /// Mouse sensitivity multiplier
    #[serde(default = "default_mouse_sensitivity")]
    pub mouse_sensitivity: f32,
}

/// Logging and debugging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct LoggingSettings {
    /// Log level (error, warn, info, debug, trace)
    #[serde(default = "default_log_level")]
    pub level: LogLevel,
    /// Optional log file path
    pub file: Option<PathBuf>,
    /// Whether to log to stdout/stderr
    #[serde(default = "default_log_stdout")]
    pub stdout: bool,
    /// Maximum log file size in MB before rotation
    #[serde(default = "default_max_log_size")]
    pub max_file_size: u64,
    /// Number of rotated log files to keep
    #[serde(default = "default_log_rotation_count")]
    pub rotation_count: u32,
}
