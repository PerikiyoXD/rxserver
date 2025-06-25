//! Configuration Schema Definitions
//!
//! This module defines the JSON schema for configuration validation.

use serde_json::Value;

/// Get the JSON schema for server configuration
pub fn get_config_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "network": {
                "type": "object",
                "properties": {
                    "bind_address": { "type": "string" },
                    "max_connections": { "type": "integer", "minimum": 1 },
                    "connection_timeout": { "type": "integer", "minimum": 0 },
                    "tcp_keepalive": { "type": "boolean" }
                },
                "required": ["bind_address", "max_connections"]
            },
            "security": {
                "type": "object",
                "properties": {
                    "enable_host_access_control": { "type": "boolean" },
                    "allowed_hosts": { "type": "array", "items": { "type": "string" } },
                    "enable_authentication": { "type": "boolean" },
                    "max_auth_attempts": { "type": "integer", "minimum": 1 }
                }
            },
            "performance": {
                "type": "object",
                "properties": {
                    "worker_threads": { "type": "integer", "minimum": 1 },
                    "request_queue_size": { "type": "integer", "minimum": 1 },
                    "enable_batching": { "type": "boolean" }
                }
            }
        },
        "required": ["network", "security", "performance"]
    })
}
