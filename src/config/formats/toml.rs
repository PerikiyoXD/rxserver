//! TOML configuration format support

use crate::config::types::ServerConfig;
use crate::types::{ConfigurationError, Result};

/// Parse configuration from TOML string
pub fn parse(content: &str) -> Result<ServerConfig> {
    toml::from_str(content).map_err(|e| {
        ConfigurationError::ParseError {
            format: "TOML".to_string(),
            message: e.to_string(),
        }
        .into()
    })
}

/// Serialize configuration to TOML string
pub fn serialize(config: &ServerConfig) -> Result<String> {
    toml::to_string_pretty(config).map_err(|e| {
        ConfigurationError::SerializationError {
            format: "TOML".to_string(),
            message: e.to_string(),
        }
        .into()
    })
}

/// Parse configuration from TOML with environment variable substitution
pub fn parse_with_env_substitution(content: &str) -> Result<ServerConfig> {
    let substituted = substitute_env_vars(content)?;
    parse(&substituted)
}

/// Substitute environment variables in TOML content
fn substitute_env_vars(content: &str) -> Result<String> {
    let mut result = content.to_string();

    // Simple environment variable substitution
    // Supports ${VAR} and ${VAR:-default} syntax
    // This is a basic implementation without regex for now
    let mut start = 0;
    while let Some(pos) = result[start..].find("${") {
        let abs_pos = start + pos;
        if let Some(end_pos) = result[abs_pos..].find('}') {
            let var_expr = &result[abs_pos + 2..abs_pos + end_pos];
            let (var_name, default_value) = if let Some(colon_pos) = var_expr.find(":-") {
                (&var_expr[..colon_pos], &var_expr[colon_pos + 2..])
            } else {
                (var_expr, "")
            };

            let replacement = std::env::var(var_name).unwrap_or_else(|_| default_value.to_string());

            result.replace_range(abs_pos..abs_pos + end_pos + 1, &replacement);
            start = abs_pos + replacement.len();
        } else {
            start = abs_pos + 2;
        }
    }

    Ok(result)
}

/// Validate TOML configuration syntax
pub fn validate_syntax(content: &str) -> Result<()> {
    toml::from_str::<toml::Value>(content).map_err(|e| ConfigurationError::ParseError {
        format: "TOML".to_string(),
        message: e.to_string(),
    })?;
    Ok(())
}

/// Generate TOML template with comments
pub fn generate_template() -> String {
    r#"# RXServer Configuration File (TOML format)
# This file contains the configuration for the RXServer X11 implementation.

[server]
# Server identification
name = "RXServer"
version = "0.1.0"
vendor = "RXServer Team"
release = 1
display_number = 0
screen_count = 1

[network]
# Network configuration
tcp_addresses = ["127.0.0.1:6000"]
unix_sockets = ["/tmp/.X11-unix/X0"]
max_connections = 100
connection_timeout = 30
authentication_enabled = true
auth_methods = ["MIT-MAGIC-COOKIE-1"]

[display]
# Display configuration
[display.default_resolution]
width = 1920
height = 1080

[[display.supported_resolutions]]
width = 1920
height = 1080

[[display.supported_resolutions]]
width = 1280
height = 720

[[display.supported_resolutions]]
width = 800
height = 600

color_depth = 24
dpi = 96.0
refresh_rate = 60
backend = "software"

[security]
# Security configuration
access_control_enabled = true
allowed_hosts = ["localhost", "127.0.0.1"]
audit_enabled = false
# audit_log_path = "/var/log/rxserver/audit.log"

[logging]
# Logging configuration
level = "info"
structured = false

[[logging.outputs]]
output_type = "console"

# Uncomment for file logging
# [[logging.outputs]]
# output_type = "file"
# [logging.outputs.config]
# path = "/var/log/rxserver/server.log"

# Uncomment for log rotation
# [logging.rotation]
# max_size = 104857600  # 100MB
# max_files = 10
# compress = true

[performance]
# Performance configuration
# thread_pool_size = 8  # Auto-detect if not specified
request_queue_size = 1000
event_queue_size = 5000

[performance.memory_pools]
enabled = true
initial_size = 1048576    # 1MB
max_size = 67108864       # 64MB
block_size = 4096         # 4KB

[performance.caching]
font_cache_size = 100
pixmap_cache_size = 1000
glyph_cache_size = 10000
ttl = 3600                # 1 hour

[features]
# Feature toggles
extensions_enabled = true
compositing_enabled = false
damage_tracking_enabled = true
performance_monitoring_enabled = false
debug_features_enabled = false

# Extension-specific configurations can be added here
# [extensions.composite]
# enabled = true
# redirect_automatic = false

# [extensions.damage]
# enabled = true
# report_level = "BoundingBox"
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_toml() {
        let toml_content = r#"
        [server]
        name = "TestServer"
        version = "1.0.0"
        
        [network]
        max_connections = 50
        "#;

        let result = parse(toml_content);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.server.name, "TestServer");
        assert_eq!(config.server.version, "1.0.0");
        assert_eq!(config.network.max_connections, 50);
    }

    #[test]
    fn test_serialize_and_parse() {
        let config = ServerConfig::default();
        let serialized = serialize(&config).unwrap();
        let parsed = parse(&serialized).unwrap();

        assert_eq!(config.server.name, parsed.server.name);
        assert_eq!(
            config.network.max_connections,
            parsed.network.max_connections
        );
    }

    #[test]
    fn test_env_substitution() {
        unsafe {
            std::env::set_var("TEST_VAR", "test_value");
            std::env::set_var("TEST_PORT", "6001");
        }

        let content = r#"
        [server]
        name = "${TEST_VAR:-default}"
        
        [network]
        tcp_addresses = ["127.0.0.1:${TEST_PORT:-6000}"]
        "#;

        let result = parse_with_env_substitution(content);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.server.name, "test_value");
        unsafe {
            std::env::remove_var("TEST_VAR");
            std::env::remove_var("TEST_PORT");
        }
    }

    #[test]
    fn test_validate_syntax() {
        let valid_toml = r#"
        [server]
        name = "test"
        "#;
        assert!(validate_syntax(valid_toml).is_ok());

        let invalid_toml = r#"
        [server
        name = "test"
        "#;
        assert!(validate_syntax(invalid_toml).is_err());
    }
}
