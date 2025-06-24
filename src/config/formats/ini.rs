//! INI configuration format support

use crate::config::types::*;
use crate::types::Result;
use std::collections::HashMap;

/// Parse configuration from INI string
pub fn parse(content: &str) -> Result<ServerConfig> {
    let parsed = parse_ini(content)?;
    ini_to_config(parsed)
}

/// Serialize configuration to INI string
pub fn serialize(config: &ServerConfig) -> Result<String> {
    config_to_ini(config)
}

/// Parse INI content into a section map
fn parse_ini(content: &str) -> Result<HashMap<String, HashMap<String, String>>> {
    let mut sections = HashMap::new();
    let mut current_section = String::from("global");
    let mut current_map = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        // Handle section headers
        if line.starts_with('[') && line.ends_with(']') {
            // Save previous section
            if !current_map.is_empty() {
                sections.insert(current_section.clone(), current_map);
                current_map = HashMap::new();
            }

            current_section = line[1..line.len() - 1].to_string();
            continue;
        }

        // Handle key-value pairs
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_string();
            let value = line[eq_pos + 1..].trim().to_string();
            current_map.insert(key, value);
        }
    }

    // Save the last section
    if !current_map.is_empty() {
        sections.insert(current_section, current_map);
    }

    Ok(sections)
}

/// Convert parsed INI data to ServerConfig
fn ini_to_config(sections: HashMap<String, HashMap<String, String>>) -> Result<ServerConfig> {
    let mut config = ServerConfig::default();

    // Server section
    if let Some(server_section) = sections.get("server") {
        if let Some(name) = server_section.get("name") {
            config.server.name = name.clone();
        }
        if let Some(version) = server_section.get("version") {
            config.server.version = version.clone();
        }
        if let Some(vendor) = server_section.get("vendor") {
            config.server.vendor = vendor.clone();
        }
        if let Some(release) = server_section.get("release") {
            config.server.release = release.parse().unwrap_or(1);
        }
        if let Some(display_number) = server_section.get("display_number") {
            config.server.display_number = display_number.parse().unwrap_or(0);
        }
        if let Some(screen_count) = server_section.get("screen_count") {
            config.server.screen_count = screen_count.parse().unwrap_or(1);
        }
    }

    // Network section
    if let Some(network_section) = sections.get("network") {
        if let Some(max_connections) = network_section.get("max_connections") {
            config.network.max_connections = max_connections.parse().unwrap_or(100);
        }
        if let Some(connection_timeout) = network_section.get("connection_timeout") {
            config.network.connection_timeout = connection_timeout.parse().unwrap_or(30);
        }
        if let Some(auth_enabled) = network_section.get("authentication_enabled") {
            config.network.authentication_enabled = auth_enabled.parse().unwrap_or(true);
        }
        if let Some(tcp_addresses) = network_section.get("tcp_addresses") {
            // Simple comma-separated parsing
            config.network.tcp_addresses = tcp_addresses
                .split(',')
                .filter_map(|addr| addr.trim().parse().ok())
                .collect();
        }
    }

    // Display section
    if let Some(display_section) = sections.get("display") {
        if let Some(color_depth) = display_section.get("color_depth") {
            config.display.color_depth = color_depth.parse().unwrap_or(24);
        }
        if let Some(dpi) = display_section.get("dpi") {
            config.display.dpi = dpi.parse().unwrap_or(96.0);
        }
        if let Some(refresh_rate) = display_section.get("refresh_rate") {
            config.display.refresh_rate = refresh_rate.parse().unwrap_or(60);
        }
        if let Some(backend) = display_section.get("backend") {
            config.display.backend = backend.clone();
        }
    }

    // Security section
    if let Some(security_section) = sections.get("security") {
        if let Some(access_control) = security_section.get("access_control_enabled") {
            config.security.access_control_enabled = access_control.parse().unwrap_or(true);
        }
        if let Some(audit_enabled) = security_section.get("audit_enabled") {
            config.security.audit_enabled = audit_enabled.parse().unwrap_or(false);
        }
        if let Some(allowed_hosts) = security_section.get("allowed_hosts") {
            config.security.allowed_hosts = allowed_hosts
                .split(',')
                .map(|host| host.trim().to_string())
                .collect();
        }
    }

    // Logging section
    if let Some(logging_section) = sections.get("logging") {
        if let Some(level) = logging_section.get("level") {
            config.logging.level = level.clone();
        }
        if let Some(structured) = logging_section.get("structured") {
            config.logging.structured = structured.parse().unwrap_or(false);
        }
    }

    // Performance section
    if let Some(performance_section) = sections.get("performance") {
        if let Some(request_queue_size) = performance_section.get("request_queue_size") {
            config.performance.request_queue_size = request_queue_size.parse().unwrap_or(1000);
        }
        if let Some(event_queue_size) = performance_section.get("event_queue_size") {
            config.performance.event_queue_size = event_queue_size.parse().unwrap_or(5000);
        }
        if let Some(thread_pool_size) = performance_section.get("thread_pool_size") {
            if !thread_pool_size.is_empty() {
                config.performance.thread_pool_size = thread_pool_size.parse().ok();
            }
        }
    }

    // Features section
    if let Some(features_section) = sections.get("features") {
        if let Some(extensions_enabled) = features_section.get("extensions_enabled") {
            config.features.extensions_enabled = extensions_enabled.parse().unwrap_or(true);
        }
        if let Some(compositing_enabled) = features_section.get("compositing_enabled") {
            config.features.compositing_enabled = compositing_enabled.parse().unwrap_or(false);
        }
        if let Some(damage_tracking_enabled) = features_section.get("damage_tracking_enabled") {
            config.features.damage_tracking_enabled =
                damage_tracking_enabled.parse().unwrap_or(true);
        }
        if let Some(performance_monitoring_enabled) =
            features_section.get("performance_monitoring_enabled")
        {
            config.features.performance_monitoring_enabled =
                performance_monitoring_enabled.parse().unwrap_or(false);
        }
        if let Some(debug_features_enabled) = features_section.get("debug_features_enabled") {
            config.features.debug_features_enabled =
                debug_features_enabled.parse().unwrap_or(false);
        }
    }

    Ok(config)
}

/// Convert ServerConfig to INI format
fn config_to_ini(config: &ServerConfig) -> Result<String> {
    let mut ini = String::new();

    ini.push_str("# RXServer Configuration File (INI format)\n");
    ini.push_str("# This file contains the configuration for the RXServer X11 implementation.\n\n");

    // Server section
    ini.push_str("[server]\n");
    ini.push_str(&format!("name = {}\n", config.server.name));
    ini.push_str(&format!("version = {}\n", config.server.version));
    ini.push_str(&format!("vendor = {}\n", config.server.vendor));
    ini.push_str(&format!("release = {}\n", config.server.release));
    ini.push_str(&format!(
        "display_number = {}\n",
        config.server.display_number
    ));
    ini.push_str(&format!("screen_count = {}\n", config.server.screen_count));
    ini.push_str("\n");

    // Network section
    ini.push_str("[network]\n");
    let tcp_addresses: Vec<String> = config
        .network
        .tcp_addresses
        .iter()
        .map(|addr| addr.to_string())
        .collect();
    ini.push_str(&format!("tcp_addresses = {}\n", tcp_addresses.join(",")));
    ini.push_str(&format!(
        "max_connections = {}\n",
        config.network.max_connections
    ));
    ini.push_str(&format!(
        "connection_timeout = {}\n",
        config.network.connection_timeout
    ));
    ini.push_str(&format!(
        "authentication_enabled = {}\n",
        config.network.authentication_enabled
    ));
    ini.push_str("\n");

    // Display section
    ini.push_str("[display]\n");
    ini.push_str(&format!("color_depth = {}\n", config.display.color_depth));
    ini.push_str(&format!("dpi = {}\n", config.display.dpi));
    ini.push_str(&format!("refresh_rate = {}\n", config.display.refresh_rate));
    ini.push_str(&format!("backend = {}\n", config.display.backend));
    ini.push_str("\n");

    // Security section
    ini.push_str("[security]\n");
    ini.push_str(&format!(
        "access_control_enabled = {}\n",
        config.security.access_control_enabled
    ));
    ini.push_str(&format!(
        "audit_enabled = {}\n",
        config.security.audit_enabled
    ));
    ini.push_str(&format!(
        "allowed_hosts = {}\n",
        config.security.allowed_hosts.join(",")
    ));
    ini.push_str("\n");

    // Logging section
    ini.push_str("[logging]\n");
    ini.push_str(&format!("level = {}\n", config.logging.level));
    ini.push_str(&format!("structured = {}\n", config.logging.structured));
    ini.push_str("\n");

    // Performance section
    ini.push_str("[performance]\n");
    if let Some(thread_pool_size) = config.performance.thread_pool_size {
        ini.push_str(&format!("thread_pool_size = {}\n", thread_pool_size));
    } else {
        ini.push_str("# thread_pool_size = (auto-detect)\n");
    }
    ini.push_str(&format!(
        "request_queue_size = {}\n",
        config.performance.request_queue_size
    ));
    ini.push_str(&format!(
        "event_queue_size = {}\n",
        config.performance.event_queue_size
    ));
    ini.push_str("\n");

    // Features section
    ini.push_str("[features]\n");
    ini.push_str(&format!(
        "extensions_enabled = {}\n",
        config.features.extensions_enabled
    ));
    ini.push_str(&format!(
        "compositing_enabled = {}\n",
        config.features.compositing_enabled
    ));
    ini.push_str(&format!(
        "damage_tracking_enabled = {}\n",
        config.features.damage_tracking_enabled
    ));
    ini.push_str(&format!(
        "performance_monitoring_enabled = {}\n",
        config.features.performance_monitoring_enabled
    ));
    ini.push_str(&format!(
        "debug_features_enabled = {}\n",
        config.features.debug_features_enabled
    ));

    Ok(ini)
}

/// Generate INI template
pub fn generate_template() -> String {
    let config = ServerConfig::default();
    serialize(&config).unwrap_or_else(|_| {
        r#"# RXServer Configuration File (INI format)
# This file contains the configuration for the RXServer X11 implementation.

[server]
name = RXServer
version = 0.1.0
vendor = RXServer Team
release = 1
display_number = 0
screen_count = 1

[network]
tcp_addresses = 127.0.0.1:6000
max_connections = 100
connection_timeout = 30
authentication_enabled = true

[display]
color_depth = 24
dpi = 96.0
refresh_rate = 60
backend = software

[security]
access_control_enabled = true
audit_enabled = false
allowed_hosts = localhost,127.0.0.1

[logging]
level = info
structured = false

[performance]
# thread_pool_size = (auto-detect)
request_queue_size = 1000
event_queue_size = 5000

[features]
extensions_enabled = true
compositing_enabled = false
damage_tracking_enabled = true
performance_monitoring_enabled = false
debug_features_enabled = false
"#
        .to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_ini() {
        let ini_content = r#"
        [server]
        name = TestServer
        version = 1.0.0
        
        [network]
        max_connections = 50
        "#;

        let result = parse(ini_content);
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
}
