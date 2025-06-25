//! Default configuration values and templates
//!
//! This module provides default configuration values and configuration templates
//! for different use cases and environments.

use crate::config::types::*;
use std::collections::HashMap;

/// Default configuration provider
pub struct DefaultConfig;

impl DefaultConfig {
    /// Get the default server configuration
    pub fn default_config() -> ServerConfig {
        ServerConfig::default()
    }

    /// Get configuration for development environment
    pub fn development_config() -> ServerConfig {
        let mut config = ServerConfig::default();

        // Enable debug features
        config.features.debug_features_enabled = true;
        config.features.performance_monitoring_enabled = true;

        // More verbose logging
        config.logging.level = "debug".to_string();

        // Smaller limits for development
        config.network.max_connections = 10;
        config.performance.request_queue_size = 100;

        config
    }

    /// Get configuration for production environment
    pub fn production_config() -> ServerConfig {
        let mut config = ServerConfig::default();

        // Disable debug features
        config.features.debug_features_enabled = false;
        config.features.performance_monitoring_enabled = true;

        // Production logging
        config.logging.level = "warn".to_string();
        config.logging.structured = true;

        // Enable log rotation for production
        config.logging.rotation = Some(LogRotation {
            max_size: 100 * 1024 * 1024, // 100MB
            max_files: 10,
            compress: true,
        });

        // Higher limits for production
        config.network.max_connections = 1000;
        config.performance.request_queue_size = 5000;

        // Enable security features
        config.security.audit_enabled = true;
        config.security.audit_log_path = Some("/var/log/rxserver/audit.log".into());

        config
    }

    /// Get minimal configuration for embedded systems
    pub fn embedded_config() -> ServerConfig {
        let mut config = ServerConfig::default();

        // Minimal features
        config.features.extensions_enabled = false;
        config.features.compositing_enabled = false;
        config.features.performance_monitoring_enabled = false;
        config.features.debug_features_enabled = false;

        // Basic logging only
        config.logging.level = "error".to_string();
        config.logging.structured = false;

        // Lower resource usage
        config.network.max_connections = 5;
        config.performance.thread_pool_size = Some(2);
        config.performance.request_queue_size = 50;
        config.performance.event_queue_size = 100;

        // Smaller memory pools
        config.performance.memory_pools.initial_size = 256 * 1024; // 256KB
        config.performance.memory_pools.max_size = 4 * 1024 * 1024; // 4MB

        // Smaller caches
        config.performance.caching.font_cache_size = 10;
        config.performance.caching.pixmap_cache_size = 50;
        config.performance.caching.glyph_cache_size = 1000;

        // Lower resolution defaults
        config.display.default_resolution = Resolution {
            width: 800,
            height: 600,
        };
        config.display.supported_resolutions = vec![
            Resolution {
                width: 800,
                height: 600,
            },
            Resolution {
                width: 640,
                height: 480,
            },
        ];

        config
    }

    /// Get high-performance configuration for server workloads
    pub fn high_performance_config() -> ServerConfig {
        let mut config = ServerConfig::default();

        // Enable all performance features
        config.features.damage_tracking_enabled = true;
        config.features.performance_monitoring_enabled = true;

        // High connection limits
        config.network.max_connections = 10000;

        // Large queues and pools
        config.performance.thread_pool_size = Some(num_cpus::get() * 2);
        config.performance.request_queue_size = 10000;
        config.performance.event_queue_size = 50000;

        // Large memory pools
        config.performance.memory_pools.initial_size = 16 * 1024 * 1024; // 16MB
        config.performance.memory_pools.max_size = 512 * 1024 * 1024; // 512MB

        // Large caches
        config.performance.caching.font_cache_size = 1000;
        config.performance.caching.pixmap_cache_size = 10000;
        config.performance.caching.glyph_cache_size = 100000;
        config.performance.caching.ttl = 7200; // 2 hours

        config
    }

    /// Get configuration template with environment variable substitution
    pub fn template_config() -> String {
        r#"
[server]
name = "${RXSERVER_NAME:-RXServer}"
version = "${RXSERVER_VERSION:-0.1.0}"
vendor = "${RXSERVER_VENDOR:-RXServer Team}"
release = ${RXSERVER_RELEASE:-1}
display_number = ${DISPLAY_NUMBER:-0}
screen_count = ${SCREEN_COUNT:-1}

[network]
tcp_addresses = ["${BIND_ADDRESS:-127.0.0.1}:${BIND_PORT:-6000}"]
unix_sockets = ["${UNIX_SOCKET:-/tmp/.X11-unix/X0}"]
max_connections = ${MAX_CONNECTIONS:-100}
connection_timeout = ${CONNECTION_TIMEOUT:-30}
authentication_enabled = ${AUTH_ENABLED:-true}
auth_methods = ["${AUTH_METHOD:-MIT-MAGIC-COOKIE-1}"]

[display]
default_resolution = { width = ${SCREEN_WIDTH:-1920}, height = ${SCREEN_HEIGHT:-1080} }
color_depth = ${COLOR_DEPTH:-24}
dpi = ${DPI:-96.0}
refresh_rate = ${REFRESH_RATE:-60}
backend = "${DISPLAY_BACKEND:-software}"

[security]
access_control_enabled = ${ACCESS_CONTROL:-true}
allowed_hosts = ["${ALLOWED_HOSTS:-localhost,127.0.0.1}"]
audit_enabled = ${AUDIT_ENABLED:-false}

[logging]
level = "${LOG_LEVEL:-info}"
structured = ${STRUCTURED_LOGS:-false}

[performance]
thread_pool_size = ${THREAD_POOL_SIZE:-}
request_queue_size = ${REQUEST_QUEUE_SIZE:-1000}
event_queue_size = ${EVENT_QUEUE_SIZE:-5000}

[performance.memory_pools]
enabled = ${MEMORY_POOLS_ENABLED:-true}
initial_size = ${MEMORY_POOL_INITIAL_SIZE:-1048576}
max_size = ${MEMORY_POOL_MAX_SIZE:-67108864}
block_size = ${MEMORY_POOL_BLOCK_SIZE:-4096}

[performance.caching]
font_cache_size = ${FONT_CACHE_SIZE:-100}
pixmap_cache_size = ${PIXMAP_CACHE_SIZE:-1000}
glyph_cache_size = ${GLYPH_CACHE_SIZE:-10000}
ttl = ${CACHE_TTL:-3600}

[features]
extensions_enabled = ${EXTENSIONS_ENABLED:-true}
compositing_enabled = ${COMPOSITING_ENABLED:-false}
damage_tracking_enabled = ${DAMAGE_TRACKING_ENABLED:-true}
performance_monitoring_enabled = ${PERFORMANCE_MONITORING_ENABLED:-false}
debug_features_enabled = ${DEBUG_FEATURES_ENABLED:-false}
"#
        .trim()
        .to_string()
    }

    /// Get all available configuration presets
    pub fn available_presets() -> HashMap<String, ServerConfig> {
        let mut presets = HashMap::new();

        presets.insert("default".to_string(), Self::default_config());
        presets.insert("development".to_string(), Self::development_config());
        presets.insert("production".to_string(), Self::production_config());
        presets.insert("embedded".to_string(), Self::embedded_config());
        presets.insert(
            "high-performance".to_string(),
            Self::high_performance_config(),
        );

        presets
    }

    /// Get a preset configuration by name
    pub fn get_preset(name: &str) -> Option<ServerConfig> {
        match name {
            "default" => Some(Self::default_config()),
            "development" | "dev" => Some(Self::development_config()),
            "production" | "prod" => Some(Self::production_config()),
            "embedded" => Some(Self::embedded_config()),
            "high-performance" | "high-perf" => Some(Self::high_performance_config()),
            _ => None,
        }
    }
}
