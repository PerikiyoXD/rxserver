// SPDX-License-Identifier: Apache-2.0

//! Configuration management for the RX server
//!
//! This module handles loading and managing server configuration from files,
//! environment variables, and command-line arguments.
//! 
//! STATUS: Stable
//!

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::graphics::types::GraphicsBackend;
use crate::utils::LogLevel;
use crate::{Error, Result};

/// Main server configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct ServerConfig {
    pub server: ServerSettings,
    pub transport: TransportSettings,
    pub network: NetworkSettings,
    pub display: DisplaySettings,
    pub graphics: GraphicsSettings,
    pub input: InputSettings,
    pub logging: LoggingSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct ServerSettings {
    /// Maximum number of client connections
    #[serde(default = "default_max_clients")]
    pub max_clients: usize,
    /// Display number (e.g., 0 for :0)
    #[serde(default)]
    pub display_number: u8,
    /// TCP port for X11 connections (6000 + display number)
    #[serde(default = "default_tcp_port_base")]
    pub tcp_port_base: u16,
    /// Whether to enable TCP connections
    #[serde(default)]
    pub enable_tcp: bool,
    /// Unix socket path template
    #[serde(default = "default_unix_socket_path")]
    pub unix_socket_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct TransportSettings {
    /// Maximum buffer size for transport operations
    #[serde(default = "default_max_buffer_size")]
    pub max_buffer_size: usize,
    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,
    /// Enable compression for data transport
    #[serde(default)]
    pub enable_compression: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct NetworkSettings {
    /// Enable IPv6 support
    #[serde(default)]
    pub enable_ipv6: bool,
    /// List of allowed IP addresses (empty = allow all)
    #[serde(default)]
    pub allowed_ips: Vec<String>,
    /// Enable access control
    #[serde(default)]
    pub enable_access_control: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct DisplaySettings {
    /// Screen width in pixels
    #[serde(default = "default_width")]
    pub width: u32,
    /// Screen height in pixels
    #[serde(default = "default_height")]
    pub height: u32,
    /// Color depth in bits per pixel
    #[serde(default = "default_depth")]
    pub depth: u8,
    /// DPI (dots per inch)
    #[serde(default = "default_dpi")]
    pub dpi: u16,
    /// Refresh rate in Hz
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct GraphicsSettings {
    /// Enable hardware acceleration
    #[serde(default = "default_hw_accel")]
    pub hardware_acceleration: bool,
    /// Graphics backend to use
    #[serde(default = "default_backend")]
    pub backend: GraphicsBackend,
    /// Maximum texture size
    #[serde(default = "default_max_texture_size")]
    pub max_texture_size: u32,
    /// Enable VSync
    #[serde(default)]
    pub enable_vsync: bool,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct LoggingSettings {
    /// Log level (error, warn, info, debug, trace)
    #[serde(default = "default_log_level")]
    pub level: LogLevel,
    /// Log file path (optional)
    pub file: Option<PathBuf>,
    /// Whether to log to stdout
    #[serde(default = "default_log_stdout")]
    pub stdout: bool,
    /// Maximum log file size in MB before rotation
    #[serde(default = "default_max_log_size")]
    pub max_file_size: u64,
    /// Number of rotated log files to keep
    #[serde(default = "default_log_rotation_count")]
    pub rotation_count: u32,
}

// Default value functions
fn default_max_clients() -> usize {
    256
}
fn default_tcp_port_base() -> u16 {
    6000
}
fn default_unix_socket_path() -> String {
    "/tmp/.X11-unix/X{display}".to_string()
}
fn default_max_buffer_size() -> usize {
    65536
}
fn default_connection_timeout() -> u64 {
    30
}
fn default_width() -> u32 {
    1920
}
fn default_height() -> u32 {
    1080
}
fn default_depth() -> u8 {
    24
}
fn default_dpi() -> u16 {
    96
}
fn default_refresh_rate() -> u16 {
    60
}
fn default_hw_accel() -> bool {
    true
}
fn default_backend() -> GraphicsBackend {
    GraphicsBackend::Software
}
fn default_max_texture_size() -> u32 {
    4096
}
fn default_kb_repeat_delay() -> u32 {
    500
}
fn default_kb_repeat_rate() -> u32 {
    30
}
fn default_mouse_accel() -> f32 {
    1.0
}
fn default_mouse_sensitivity() -> f32 {
    1.0
}
fn default_log_level() -> LogLevel {
    LogLevel::Info
}
fn default_log_stdout() -> bool {
    true
}
fn default_max_log_size() -> u64 {
    100
}
fn default_log_rotation_count() -> u32 {
    5
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server: ServerSettings::default(),
            transport: TransportSettings::default(),
            network: NetworkSettings::default(),
            display: DisplaySettings::default(),
            graphics: GraphicsSettings::default(),
            input: InputSettings::default(),
            logging: LoggingSettings::default(),
        }
    }
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            max_clients: default_max_clients(),
            display_number: 0,
            tcp_port_base: default_tcp_port_base(),
            enable_tcp: false,
            unix_socket_path: default_unix_socket_path(),
        }
    }
}

impl Default for TransportSettings {
    fn default() -> Self {
        Self {
            max_buffer_size: default_max_buffer_size(),
            connection_timeout: default_connection_timeout(),
            enable_compression: false,
        }
    }
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            enable_ipv6: false,
            allowed_ips: Vec::new(),
            enable_access_control: false,
        }
    }
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            depth: default_depth(),
            dpi: default_dpi(),
            refresh_rate: default_refresh_rate(),
        }
    }
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            hardware_acceleration: default_hw_accel(),
            backend: default_backend(),
            max_texture_size: default_max_texture_size(),
            enable_vsync: false,
        }
    }
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            keyboard_repeat_delay: default_kb_repeat_delay(),
            keyboard_repeat_rate: default_kb_repeat_rate(),
            mouse_acceleration: default_mouse_accel(),
            mouse_sensitivity: default_mouse_sensitivity(),
        }
    }
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: None,
            stdout: default_log_stdout(),
            max_file_size: default_max_log_size(),
            rotation_count: default_log_rotation_count(),
        }
    }
}

impl ServerConfig {
    /// Load configuration from a file with environment variable support
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            log::warn!("Configuration file {:?} not found, using defaults", path);
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;

        let mut config: ServerConfig = toml::from_str(&content)
            .map_err(|e| Error::Config(format!("Failed to parse config file: {}", e)))?;

        // Apply environment variable overrides
        config.apply_env_overrides();

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Save configuration to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        // Validate before saving
        self.validate()?;

        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, content)
            .map_err(|e| Error::Config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate transport settings
        if self.transport.max_buffer_size < 1024 {
            return Err(Error::Config(
                "Buffer size must be at least 1024 bytes".to_string(),
            ));
        }

        if self.transport.connection_timeout == 0 {
            return Err(Error::Config(
                "Connection timeout must be greater than 0".to_string(),
            ));
        }

        // Validate display settings
        if self.display.width == 0 || self.display.height == 0 {
            return Err(Error::Config(
                "Display dimensions must be greater than 0".to_string(),
            ));
        }

        if ![8, 16, 24, 32].contains(&self.display.depth) {
            return Err(Error::Config(
                "Color depth must be 8, 16, 24, or 32 bits".to_string(),
            ));
        }

        // Validate server settings
        if self.server.max_clients == 0 {
            return Err(Error::Config(
                "max_clients must be greater than 0".to_string(),
            ));
        }

        // Validate graphics settings
        if self.graphics.max_texture_size < 256 || !self.graphics.max_texture_size.is_power_of_two()
        {
            return Err(Error::Config(
                "Max texture size must be a power of 2 and at least 256".to_string(),
            ));
        }

        // Validate input settings
        if self.input.keyboard_repeat_delay == 0 || self.input.keyboard_repeat_delay > 2000 {
            return Err(Error::Config(
                "Keyboard repeat delay must be between 1 and 2000 ms".to_string(),
            ));
        }

        if self.input.keyboard_repeat_rate == 0 || self.input.keyboard_repeat_rate > 100 {
            return Err(Error::Config(
                "Keyboard repeat rate must be between 1 and 100 Hz".to_string(),
            ));
        }

        if self.input.mouse_acceleration < 0.1 || self.input.mouse_acceleration > 10.0 {
            return Err(Error::Config(
                "Mouse acceleration must be between 0.1 and 10.0".to_string(),
            ));
        }

        if self.input.mouse_sensitivity < 0.1 || self.input.mouse_sensitivity > 10.0 {
            return Err(Error::Config(
                "Mouse sensitivity must be between 0.1 and 10.0".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the computed TCP port for X11 connections
    pub fn tcp_port(&self) -> u16 {
        self.server.tcp_port_base + self.server.display_number as u16
    }

    /// Get the Unix socket path with display number substituted
    pub fn unix_socket_path(&self) -> String {
        self.server
            .unix_socket_path
            .replace("{display}", &self.server.display_number.to_string())
    }

    /// Apply environment variable overrides to configuration
    fn apply_env_overrides(&mut self) {
        use std::env;
        use std::str::FromStr;

        // Server settings
        if let Ok(val) = env::var("RX_MAX_CLIENTS") {
            if let Ok(max_clients) = val.parse() {
                self.server.max_clients = max_clients;
            }
        }
        if let Ok(val) = env::var("RX_DISPLAY_NUMBER") {
            if let Ok(display_number) = val.parse() {
                self.server.display_number = display_number;
            }
        }
        if let Ok(val) = env::var("RX_ENABLE_TCP") {
            if let Ok(enable_tcp) = val.parse() {
                self.server.enable_tcp = enable_tcp;
            }
        }

        // Display settings
        if let Ok(val) = env::var("RX_DISPLAY_WIDTH") {
            if let Ok(width) = val.parse() {
                self.display.width = width;
            }
        }
        if let Ok(val) = env::var("RX_DISPLAY_HEIGHT") {
            if let Ok(height) = val.parse() {
                self.display.height = height;
            }
        }
        if let Ok(val) = env::var("RX_DISPLAY_DEPTH") {
            if let Ok(depth) = val.parse() {
                self.display.depth = depth;
            }
        }

        // Graphics settings
        if let Ok(val) = env::var("RX_GRAPHICS_BACKEND") {
            if let Ok(backend) = GraphicsBackend::from_str(&val) {
                self.graphics.backend = backend;
            }
        }
        if let Ok(val) = env::var("RX_HARDWARE_ACCELERATION") {
            if let Ok(hw_accel) = val.parse() {
                self.graphics.hardware_acceleration = hw_accel;
            }
        }

        // Logging settings
        if let Ok(val) = env::var("RX_LOG_LEVEL") {
            if let Ok(level) = LogLevel::from_str(&val) {
                self.logging.level = level;
            }
        }
        if let Ok(val) = env::var("RX_LOG_FILE") {
            self.logging.file = Some(PathBuf::from(val));
        }
    }

    /// Create configuration from command-line arguments
    pub fn from_args(args: &[String]) -> Result<Self> {
        let mut config = Self::default();
        let mut i = 0;

        while i < args.len() {
            match args[i].as_str() {
                "--display" | "-d" => {
                    if i + 1 < args.len() {
                        config.server.display_number = args[i + 1]
                            .parse()
                            .map_err(|_| Error::Config("Invalid display number".to_string()))?;
                        i += 2;
                    } else {
                        return Err(Error::Config("Missing display number".to_string()));
                    }
                }
                "--width" | "-w" => {
                    if i + 1 < args.len() {
                        config.display.width = args[i + 1]
                            .parse()
                            .map_err(|_| Error::Config("Invalid width".to_string()))?;
                        i += 2;
                    } else {
                        return Err(Error::Config("Missing width value".to_string()));
                    }
                }
                "--height" | "-h" => {
                    if i + 1 < args.len() {
                        config.display.height = args[i + 1]
                            .parse()
                            .map_err(|_| Error::Config("Invalid height".to_string()))?;
                        i += 2;
                    } else {
                        return Err(Error::Config("Missing height value".to_string()));
                    }
                }
                "--backend" | "-b" => {
                    if i + 1 < args.len() {
                        config.graphics.backend = GraphicsBackend::from_str(&args[i + 1])
                            .map_err(|_| Error::Config("Invalid graphics backend".to_string()))?;
                        i += 2;
                    } else {
                        return Err(Error::Config("Missing backend value".to_string()));
                    }
                }
                "--enable-tcp" => {
                    config.server.enable_tcp = true;
                    i += 1;
                }
                "--log-level" => {
                    if i + 1 < args.len() {
                        config.logging.level = LogLevel::from_str(&args[i + 1])
                            .map_err(|_| Error::Config("Invalid log level".to_string()))?;
                        i += 2;
                    } else {
                        return Err(Error::Config("Missing log level".to_string()));
                    }
                }
                _ => i += 1,
            }
        }

        config.validate()?;
        Ok(config)
    }
}

/// Alias for DisplaySettings for backward compatibility
pub type DisplayConfig = DisplaySettings;
