//! Component-specific logging utilities
//!
//! This module provides structured logging helpers for different components
//! of the RXServer, making it easy to maintain consistent logging patterns.

use tracing::{error, info, warn};

/// Server component logging utilities
pub struct ServerLogger {
    component: String,
}

impl ServerLogger {
    /// Create a new server logger for a specific component
    pub fn new(component: &str) -> Self {
        Self {
            component: component.to_string(),
        }
    }

    /// Log server startup with standard formatting
    pub fn startup(display_num: u8, config_file: &str) {
        crate::logging::init::log_startup_info(display_num, config_file);
    }

    /// Log startup information for this component
    pub fn log_startup(&self, message: &str, details: &str) {
        info!(
            component = %self.component,
            message = message,
            details = details,
            "Component startup"
        );
    }

    /// Log shutdown information for this component
    pub fn log_shutdown(&self, message: &str) {
        info!(
            component = %self.component,
            message = message,
            "Component shutdown"
        );
    }

    /// Log server shutdown with standard formatting
    pub fn shutdown() {
        crate::logging::init::log_shutdown_info();
    }

    /// Log successful configuration loading
    pub fn config_loaded(path: &str) {
        info!(config_path = path, "Configuration loaded successfully");
    }

    /// Log server binding to address
    pub fn bind_address(address: &str) {
        info!(bind_address = address, "Server bound to address");
    }

    /// Log server error with context
    pub fn error(context: &str, error: &dyn std::error::Error) {
        error!(
            context = context,
            error = %error,
            "Server error occurred"
        );
    }
}

/// Network component logging utilities
pub struct NetworkLogger;

impl NetworkLogger {
    /// Log client connection accepted
    pub fn connection_accepted(client_id: u32, remote_addr: std::net::SocketAddr) {
        crate::log_connection!(accept, client_id, remote_addr);
    }

    /// Log client connection closed
    pub fn connection_closed(client_id: u32, reason: &str) {
        crate::log_connection!(disconnect, client_id, reason);
    }

    /// Log connection error
    pub fn connection_error(client_id: u32, error: &dyn std::error::Error) {
        crate::log_connection!(error, client_id, error);
    }

    /// Log network listener started
    pub fn listener_started(address: &str) {
        info!(listener_address = address, "Network listener started");
    }

    /// Log network listener error
    pub fn listener_error(address: &str, error: &dyn std::error::Error) {
        error!(
            listener_address = address,
            error = %error,
            "Network listener error"
        );
    }
}

/// Protocol component logging utilities
pub struct ProtocolLogger;

impl ProtocolLogger {
    /// Log X11 request received
    pub fn request_received(client_id: u32, request_type: &str) {
        crate::log_x11_protocol!(request, client_id, request_type);
    }

    /// Log X11 response sent
    pub fn response_sent(client_id: u32, response_type: &str) {
        crate::log_x11_protocol!(response, client_id, response_type);
    }

    /// Log X11 event sent
    pub fn event_sent(client_id: u32, event_type: &str) {
        crate::log_x11_protocol!(event, client_id, event_type);
    }

    /// Log X11 protocol error
    pub fn protocol_error(client_id: u32, error_code: u8, error_msg: &str) {
        crate::log_x11_protocol!(error, client_id, error_code, error_msg);
    }

    /// Log successful connection establishment
    pub fn connection_established(client_id: u32) {
        info!(
            client_id = client_id,
            "X11 connection established"
        );
    }

    /// Log connection failure
    pub fn connection_failed(error: &dyn std::error::Error) {
        error!(
            error = %error,
            "X11 connection failed"
        );
    }

    /// Log connection closure
    pub fn connection_closed(client_id: u32, duration: std::time::Duration) {
        info!(
            client_id = client_id,
            duration_ms = duration.as_millis(),
            "X11 connection closed"
        );
    }

    /// Log request processing error
    pub fn request_error(client_id: u32, error: &dyn std::error::Error) {
        error!(
            client_id = client_id,
            error = %error,
            "X11 request processing error"
        );
    }

    /// Log successful X11 handshake
    pub fn handshake_completed(client_id: u32, major: u16, minor: u16) {
        info!(
            client_id = client_id,
            protocol_major = major,
            protocol_minor = minor,
            "X11 handshake completed"
        );
    }

    /// Log protocol parsing error
    pub fn parse_error(client_id: u32, details: &str) {
        warn!(
            client_id = client_id,
            details = details,
            "Protocol parsing error"
        );
    }
}

/// Resource management logging utilities
pub struct ResourceLogger;

impl ResourceLogger {
    /// Log window creation
    pub fn window_created(window_id: u32, client_id: u32) {
        crate::log_resource!(create, "window", window_id, client_id);
    }

    /// Log window destruction
    pub fn window_destroyed(window_id: u32, client_id: u32) {
        crate::log_resource!(destroy, "window", window_id, client_id);
    }

    /// Log pixmap creation
    pub fn pixmap_created(pixmap_id: u32, client_id: u32) {
        crate::log_resource!(create, "pixmap", pixmap_id, client_id);
    }

    /// Log pixmap destruction
    pub fn pixmap_destroyed(pixmap_id: u32, client_id: u32) {
        crate::log_resource!(destroy, "pixmap", pixmap_id, client_id);
    }

    /// Log resource leak detection
    pub fn resource_leak_detected(resource_type: &str, resource_id: u32, client_id: u32) {
        crate::log_resource!(leak, resource_type, resource_id, client_id);
    }

    /// Log resource cleanup on client disconnect
    pub fn cleanup_client_resources(client_id: u32, resource_count: usize) {
        info!(
            client_id = client_id,
            resource_count = resource_count,
            "Cleaned up client resources"
        );
    }
}

/// Performance monitoring logging utilities
pub struct PerformanceLogger;

impl PerformanceLogger {
    /// Log operation timing
    pub fn operation_timing(operation: &str, duration_ms: f64) {
        crate::log_performance!(timing, operation, duration_ms);
    }

    /// Log memory usage
    pub fn memory_usage(component: &str, bytes: usize) {
        crate::log_performance!(memory, component, bytes);
    }

    /// Log request processing rate
    pub fn request_rate(requests_per_second: f64) {
        crate::log_performance!(counter, "requests_per_second", requests_per_second);
    }

    /// Log frame rate for graphics operations
    pub fn frame_rate(fps: f64) {
        crate::log_performance!(counter, "frames_per_second", fps);
    }

    /// Log average response time
    pub fn avg_response_time(avg_ms: f64) {
        crate::log_performance!(timing, "average_response_time", avg_ms);
    }
}
