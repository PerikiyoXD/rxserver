// SPDX-License-Identifier: Apache-2.0
// RX-Completion-Status: Complete

//! Configuration management for the RX server
//!
//! This module handles loading and managing server configuration from files,
//! environment variables, and command-line arguments. It provides a comprehensive
//! configuration system with validation, defaults, and multiple input sources.
//!
//! # Examples
//!
//! ```rust
//! use rxserver::config::ServerConfig;
//!
//! // Load from file with fallback to defaults
//! let config = ServerConfig::load("config.toml")?;
//!
//! // Create from command-line arguments
//! let args = vec!["--display".to_string(), "1".to_string()];
//! let config = ServerConfig::from_args(&args)?;
//!
//! // Save current configuration
//! config.save("output.toml")?;
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::config::types::ServerConfig;
use crate::graphics::types::GraphicsBackend;
use crate::logging::types::LogLevel;
use crate::{Error, Result};

impl ServerConfig {
    /// Load configuration from a TOML file with environment variable support
    ///
    /// This method loads configuration from a TOML file, applies environment variable
    /// overrides, validates the result, and returns a complete configuration.
    /// If the file doesn't exist, it returns default configuration.
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// * `Result<Self>` - The loaded and validated configuration
    ///
    /// # Examples
    /// ```rust
    /// let config = ServerConfig::load("rxserver.toml")?;
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            tracing::warn!("Configuration file {:?} not found, using defaults", path);
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

    /// Create a new configuration with minimal settings for testing
    ///
    /// This method creates a basic configuration suitable for testing environments
    /// with reduced buffer sizes and timeouts for faster test execution.
    pub fn for_testing() -> Self {
        let mut config = Self::default();
        config.server.max_clients = 10;
        config.transport.max_buffer_size = 4096;
        config.transport.connection_timeout = 5;
        config.display.width = 800;
        config.display.height = 600;
        config.logging.level = LogLevel::Debug;
        config
    }

    /// Check if the configuration is using default values
    pub fn is_default(&self) -> bool {
        let default_config = Self::default();
        std::ptr::eq(self, &default_config) // Simple comparison
    }

    /// Get a summary of the current configuration
    pub fn summary(&self) -> String {
        format!(
            "RX Server Config: Display {}x{}@{}bit, {} max clients, {} backend, TCP: {}",
            self.display.width,
            self.display.height,
            self.display.depth,
            self.server.max_clients,
            match self.graphics.backend {
                GraphicsBackend::Software => "Software",
                GraphicsBackend::OpenGL => "OpenGL",
                GraphicsBackend::Vulkan => "Vulkan",
            },
            if self.server.enable_tcp {
                "enabled"
            } else {
                "disabled"
            }
        )
    }

    /// Log the current configuration settings
    pub fn log_config(&self) {
        tracing::info!("=== RX Server Configuration ===");
        tracing::info!(
            "Server: display={}, max_clients={}, tcp={}",
            self.server.display_number,
            self.server.max_clients,
            self.server.enable_tcp
        );
        tracing::info!(
            "Display: {}x{}@{}bit, {}dpi, {}Hz",
            self.display.width,
            self.display.height,
            self.display.depth,
            self.display.dpi,
            self.display.refresh_rate
        );
        tracing::info!(
            "Graphics: backend={:?}, hw_accel={}, vsync={}",
            self.graphics.backend,
            self.graphics.hardware_acceleration,
            self.graphics.enable_vsync
        );
        tracing::info!(
            "Transport: buffer={}KB, timeout={}s, compression={}",
            self.transport.max_buffer_size / 1024,
            self.transport.connection_timeout,
            self.transport.enable_compression
        );
        tracing::info!(
            "Logging: level={:?}, stdout={}, file={:?}",
            self.logging.level,
            self.logging.stdout,
            self.logging.file
        );
        tracing::info!("===============================");
    }
}
