//! Structured logging macros for consistent formatting
//!
//! This module provides standardized logging macros for different types of events
//! throughout the RXServer codebase.

/// Log connection events with structured data
#[macro_export]
macro_rules! log_connection {
    (accept, $client_id:expr, $addr:expr) => {
        tracing::info!(
            client_id = $client_id,
            address = %$addr,
            "Client connected"
        );
    };
    (disconnect, $client_id:expr, $reason:expr) => {
        tracing::info!(
            client_id = $client_id,
            reason = $reason,
            "Client disconnected"
        );
    };
    (error, $client_id:expr, $error:expr) => {
        tracing::error!(
            client_id = $client_id,
            error = %$error,
            "Connection error"
        );
    };
}

/// Log X11 protocol events with structured data
#[macro_export]
macro_rules! log_x11_protocol {
    (request, $client_id:expr, $request_type:expr) => {
        tracing::debug!(
            client_id = $client_id,
            request_type = $request_type,
            "X11 request received"
        );
    };
    (response, $client_id:expr, $response_type:expr) => {
        tracing::debug!(
            client_id = $client_id,
            response_type = $response_type,
            "X11 response sent"
        );
    };
    (event, $client_id:expr, $event_type:expr) => {
        tracing::debug!(
            client_id = $client_id,
            event_type = $event_type,
            "X11 event sent"
        );
    };
    (error, $client_id:expr, $error_code:expr, $error_msg:expr) => {
        tracing::warn!(
            client_id = $client_id,
            error_code = $error_code,
            error_message = $error_msg,
            "X11 protocol error"
        );
    };
}

/// Log resource management events
#[macro_export]
macro_rules! log_resource {
    (create, $resource_type:expr, $resource_id:expr, $client_id:expr) => {
        tracing::debug!(
            resource_type = $resource_type,
            resource_id = $resource_id,
            client_id = $client_id,
            "Resource created"
        );
    };
    (destroy, $resource_type:expr, $resource_id:expr, $client_id:expr) => {
        tracing::debug!(
            resource_type = $resource_type,
            resource_id = $resource_id,
            client_id = $client_id,
            "Resource destroyed"
        );
    };
    (leak, $resource_type:expr, $resource_id:expr, $client_id:expr) => {
        tracing::warn!(
            resource_type = $resource_type,
            resource_id = $resource_id,
            client_id = $client_id,
            "Resource leak detected"
        );
    };
}

/// Log performance metrics
#[macro_export]
macro_rules! log_performance {
    (timing, $operation:expr, $duration_ms:expr) => {
        tracing::debug!(
            operation = $operation,
            duration_ms = $duration_ms,
            "Performance timing"
        );
    };
    (memory, $component:expr, $bytes:expr) => {
        tracing::debug!(
            component = $component,
            bytes = $bytes,
            "Memory usage"
        );
    };
    (counter, $metric:expr, $value:expr) => {
        tracing::debug!(
            metric = $metric,
            value = $value,
            "Performance counter"
        );
    };
}

/// Log implementation status with categorization
#[macro_export]
macro_rules! log_implementation {
    (complete, $component:expr, $description:expr) => {
        tracing::info!(
            component = $component,
            status = "complete",
            "✅ {}: {}",
            $component,
            $description
        );
    };
    (partial, $component:expr, $description:expr) => {
        tracing::warn!(
            component = $component,
            status = "partial",
            "⚠️  {}: {}",
            $component,
            $description
        );
    };
    (todo, $component:expr, $description:expr) => {
        tracing::warn!(
            component = $component,
            status = "todo",
            "🚧 {}: {}",
            $component,
            $description
        );
    };
    (missing, $component:expr, $description:expr) => {
        tracing::error!(
            component = $component,
            status = "missing",
            "❌ {}: {}",
            $component,
            $description
        );
    };
}
