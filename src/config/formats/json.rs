//! JSON configuration format support

use crate::config::types::ServerConfig;
use crate::types::{ConfigurationError, Result};

/// Parse configuration from JSON string
pub fn parse(content: &str) -> Result<ServerConfig> {
    serde_json::from_str(content).map_err(|e| {
        ConfigurationError::ParseError {
            format: "JSON".to_string(),
            message: e.to_string(),
        }
        .into()
    })
}

/// Serialize configuration to JSON string
pub fn serialize(config: &ServerConfig) -> Result<String> {
    serde_json::to_string_pretty(config).map_err(|e| {
        ConfigurationError::SerializationError {
            format: "JSON".to_string(),
            message: e.to_string(),
        }
        .into()
    })
}

/// Parse configuration from JSON with environment variable substitution
pub fn parse_with_env_substitution(content: &str) -> Result<ServerConfig> {
    let substituted = substitute_env_vars(content)?;
    parse(&substituted)
}

/// Substitute environment variables in JSON content
fn substitute_env_vars(content: &str) -> Result<String> {
    let mut result = content.to_string();

    // Simple environment variable substitution for JSON
    // This handles ${VAR} and ${VAR:-default} patterns within quoted strings
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

/// Validate JSON configuration syntax
pub fn validate_syntax(content: &str) -> Result<()> {
    serde_json::from_str::<serde_json::Value>(content).map_err(|e| {
        ConfigurationError::ParseError {
            format: "JSON".to_string(),
            message: e.to_string(),
        }
    })?;
    Ok(())
}

/// Generate JSON template
pub fn generate_template() -> String {
    let config = ServerConfig::default();
    serialize(&config).unwrap_or_else(|_| {
        r#"{
  "server": {
    "name": "RXServer",
    "version": "0.1.0",
    "vendor": "RXServer Team",
    "release": 1,
    "display_number": 0,
    "screen_count": 1
  },
  "network": {
    "tcp_addresses": ["127.0.0.1:6000"],
    "unix_sockets": ["/tmp/.X11-unix/X0"],
    "max_connections": 100,
    "connection_timeout": 30,
    "authentication_enabled": true,
    "auth_methods": ["MIT-MAGIC-COOKIE-1"]
  },
  "display": {
    "default_resolution": {
      "width": 1920,
      "height": 1080
    },
    "supported_resolutions": [
      {"width": 1920, "height": 1080},
      {"width": 1280, "height": 720},
      {"width": 800, "height": 600}
    ],
    "color_depth": 24,
    "dpi": 96.0,
    "refresh_rate": 60,
    "backend": "software",
    "backend_options": {}
  },
  "security": {
    "access_control_enabled": true,
    "allowed_hosts": ["localhost", "127.0.0.1"],
    "audit_enabled": false,
    "audit_log_path": null,
    "policies": []
  },
  "logging": {
    "level": "info",
    "outputs": [
      {
        "output_type": "console",
        "config": {}
      }
    ],
    "structured": false,
    "rotation": null
  },
  "performance": {
    "thread_pool_size": null,
    "request_queue_size": 1000,
    "event_queue_size": 5000,
    "memory_pools": {
      "enabled": true,
      "initial_size": 1048576,
      "max_size": 67108864,
      "block_size": 4096
    },
    "caching": {
      "font_cache_size": 100,
      "pixmap_cache_size": 1000,
      "glyph_cache_size": 10000,
      "ttl": 3600
    }
  },
  "features": {
    "extensions_enabled": true,
    "compositing_enabled": false,
    "damage_tracking_enabled": true,
    "performance_monitoring_enabled": false,
    "debug_features_enabled": false
  },
  "extensions": {}
}"#
        .to_string()
    })
}

/// Minify JSON configuration
pub fn minify(content: &str) -> Result<String> {
    let value: serde_json::Value =
        serde_json::from_str(content).map_err(|e| ConfigurationError::ParseError {
            format: "JSON".to_string(),
            message: e.to_string(),
        })?;

    serde_json::to_string(&value).map_err(|e| {
        ConfigurationError::SerializationError {
            format: "JSON".to_string(),
            message: e.to_string(),
        }
        .into()
    })
}

/// Pretty print JSON configuration
pub fn prettify(content: &str) -> Result<String> {
    let value: serde_json::Value =
        serde_json::from_str(content).map_err(|e| ConfigurationError::ParseError {
            format: "JSON".to_string(),
            message: e.to_string(),
        })?;

    serde_json::to_string_pretty(&value).map_err(|e| {
        ConfigurationError::SerializationError {
            format: "JSON".to_string(),
            message: e.to_string(),
        }
        .into()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_json() {
        let json_content = r#"{
            "server": {
                "name": "TestServer",
                "version": "1.0.0",
                "vendor": "Test Vendor",
                "release": 1,
                "display_number": 0,
                "screen_count": 1
            },
            "network": {
                "tcp_addresses": ["127.0.0.1:6000"],
                "unix_sockets": ["/tmp/.X11-unix/X0"],
                "max_connections": 50,
                "connection_timeout": 30,
                "authentication_enabled": true,
                "auth_methods": ["MIT-MAGIC-COOKIE-1"]
            }
        }"#;

        let result = parse(json_content);
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
    fn test_validate_syntax() {
        let valid_json = r#"{"server": {"name": "test"}}"#;
        assert!(validate_syntax(valid_json).is_ok());

        let invalid_json = r#"{"server": {"name": "test"}"#;
        assert!(validate_syntax(invalid_json).is_err());
    }

    #[test]
    fn test_minify_and_prettify() {
        let json_content = r#"{
            "server": {
                "name": "test"
            }
        }"#;

        let minified = minify(json_content).unwrap();
        assert!(!minified.contains('\n'));
        assert!(!minified.contains("  "));

        let prettified = prettify(&minified).unwrap();
        assert!(prettified.contains('\n'));
        assert!(prettified.contains("  "));
    }
}
