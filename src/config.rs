//! Configuration management for the RX server
//!
//! This module handles loading and managing server configuration from files,
//! environment variables, and command-line arguments.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::{Error, Result};

/// Main server configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server settings
    pub server: ServerSettings,
    /// Display settings
    pub display: DisplaySettings,
    /// Graphics settings
    pub graphics: GraphicsSettings,
    /// Input settings
    pub input: InputSettings,
    /// Logging settings
    pub logging: LoggingSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    /// Display number (e.g., 0 for :0)
    pub display_number: u8,
    /// Maximum number of client connections
    pub max_clients: usize,
    /// TCP port for X11 connections (6000 + display number)
    pub tcp_port_base: u16,
    /// Whether to enable TCP connections
    pub enable_tcp: bool,
    /// Unix socket path template
    pub unix_socket_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    /// Screen width in pixels
    pub width: u32,
    /// Screen height in pixels
    pub height: u32,
    /// Color depth in bits per pixel
    pub depth: u8,
    /// DPI (dots per inch)
    pub dpi: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsSettings {
    /// Enable hardware acceleration
    pub hardware_acceleration: bool,
    /// Graphics backend to use
    pub backend: String,
    /// Maximum texture size
    pub max_texture_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSettings {
    /// Keyboard repeat delay in milliseconds
    pub keyboard_repeat_delay: u32,
    /// Keyboard repeat rate in Hz
    pub keyboard_repeat_rate: u32,
    /// Mouse acceleration factor
    pub mouse_acceleration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettings {
    /// Log level (error, warn, info, debug, trace)
    pub level: String,
    /// Log file path (optional)
    pub file: Option<String>,
    /// Whether to log to stdout
    pub stdout: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server: ServerSettings {
                display_number: 0,
                max_clients: 256,
                tcp_port_base: 6000,
                enable_tcp: false,
                unix_socket_path: "/tmp/.X11-unix/X{display}".to_string(),
            },
            display: DisplaySettings {
                width: 1920,
                height: 1080,
                depth: 24,
                dpi: 96,
            },
            graphics: GraphicsSettings {
                hardware_acceleration: true,
                backend: "software".to_string(),
                max_texture_size: 4096,
            },
            input: InputSettings {
                keyboard_repeat_delay: 500,
                keyboard_repeat_rate: 30,
                mouse_acceleration: 1.0,
            },
            logging: LoggingSettings {
                level: "info".to_string(),
                file: None,
                stdout: true,
            },
        }
    }
}

impl ServerConfig {
    /// Load configuration from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            log::warn!("Configuration file {:?} not found, using defaults", path);
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;

        let config: ServerConfig = toml::from_str(&content)
            .map_err(|e| Error::Config(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// Save configuration to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, content)
            .map_err(|e| Error::Config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }
}

/// Alias for DisplaySettings for backward compatibility
pub type DisplayConfig = DisplaySettings;
