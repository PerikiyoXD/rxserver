// SPDX-License-Identifier: Apache-2.0
// RX-Completion-Status: Complete

//! Default configuration values for the RX server
//!
//! This module provides default values for all configuration settings,
//! ensuring that the server can run with minimal configuration while
//! providing sensible defaults for production use.

use crate::{config::types::*, graphics::types::GraphicsBackend, logging::types::LogLevel};

// Server settings defaults
/// Default maximum number of concurrent client connections
pub fn default_max_clients() -> usize {
    256
}

/// Default TCP port base for X11 connections
pub fn default_tcp_port_base() -> u16 {
    6000
}

/// Default Unix socket path template
pub fn default_unix_socket_path() -> String {
    "/tmp/.X11-unix/X{display}".to_string()
}

// Transport settings defaults
/// Default maximum buffer size (64KB)
pub fn default_max_buffer_size() -> usize {
    65536
}

/// Default connection timeout (30 seconds)
pub fn default_connection_timeout() -> u64 {
    30
}

// Display settings defaults
/// Default screen width (1920px - Full HD)
pub fn default_width() -> u32 {
    1920
}

/// Default screen height (1080px - Full HD)
pub fn default_height() -> u32 {
    1080
}

/// Default color depth (24-bit)
pub fn default_depth() -> u8 {
    24
}

/// Default DPI (96 - Windows standard)
pub fn default_dpi() -> u16 {
    96
}

/// Default refresh rate (60Hz)
pub fn default_refresh_rate() -> u16 {
    60
}

// Graphics settings defaults
/// Default hardware acceleration setting
pub fn default_hw_accel() -> bool {
    true
}

/// Default graphics backend
pub fn default_backend() -> GraphicsBackend {
    GraphicsBackend::Software
}

/// Default maximum texture size (4K)
pub fn default_max_texture_size() -> u32 {
    4096
}

// Input settings defaults
/// Default keyboard repeat delay (500ms)
pub fn default_kb_repeat_delay() -> u32 {
    500
}

/// Default keyboard repeat rate (30Hz)
pub fn default_kb_repeat_rate() -> u32 {
    30
}

/// Default mouse acceleration (1.0 - no acceleration)
pub fn default_mouse_accel() -> f32 {
    1.0
}

/// Default mouse sensitivity (1.0 - normal sensitivity)
pub fn default_mouse_sensitivity() -> f32 {
    1.0
}

// Logging settings defaults
/// Default log level (Info)
pub fn default_log_level() -> LogLevel {
    LogLevel::Info
}

/// Default stdout logging (enabled)
pub fn default_log_stdout() -> bool {
    true
}

/// Default maximum log file size (100MB)
pub fn default_max_log_size() -> u64 {
    100
}

/// Default log rotation count (5 files)
pub fn default_log_rotation_count() -> u32 {
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
