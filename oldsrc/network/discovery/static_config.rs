//! Static configuration-based service discovery
//!
//! Provides service discovery through static configuration files and manual entries.

use crate::network::discovery::registry::DiscoveryMechanism;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tokio::fs;

use super::registry::{DiscoveryError, ServiceInfo};

/// Static configuration discovery service
#[derive(Debug, Clone)]
pub struct StaticConfigDiscovery {
    config_file: PathBuf,
    services: HashMap<String, StaticService>,
    auto_reload: bool,
}

/// Static service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticService {
    pub name: String,
    pub address: SocketAddr,
    pub display: u32,
    pub capabilities: Vec<String>,
    pub protocol_version: String,
    pub enabled: bool,
    pub priority: u32,
    pub description: Option<String>,
}

/// Configuration file format
#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    version: String,
    services: Vec<StaticService>,
}

impl StaticConfigDiscovery {
    /// Create a new static config discovery instance
    pub fn new<P: AsRef<Path>>(config_file: P) -> Self {
        Self {
            config_file: config_file.as_ref().to_path_buf(),
            services: HashMap::new(),
            auto_reload: true,
        }
    }

    /// Enable or disable automatic configuration reloading
    pub fn set_auto_reload(&mut self, enabled: bool) {
        self.auto_reload = enabled;
    }

    /// Load services from configuration file
    pub async fn load_config(&mut self) -> Result<(), DiscoveryError> {
        if !self.config_file.exists() {
            return self.create_default_config().await;
        }

        let content = fs::read_to_string(&self.config_file)
            .await
            .map_err(|e| DiscoveryError::Io(e.to_string()))?;

        let config: ConfigFile = serde_json::from_str(&content)
            .map_err(|e| DiscoveryError::Serialization(e.to_string()))?;

        self.services.clear();
        for service in config.services {
            if service.enabled {
                self.services.insert(service.name.clone(), service);
            }
        }

        Ok(())
    }

    /// Save current services to configuration file
    pub async fn save_config(&self) -> Result<(), DiscoveryError> {
        let services: Vec<StaticService> = self.services.values().cloned().collect();
        let config = ConfigFile {
            version: "1.0".to_string(),
            services,
        };

        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| DiscoveryError::Serialization(e.to_string()))?;

        fs::write(&self.config_file, content)
            .await
            .map_err(|e| DiscoveryError::Io(e.to_string()))?;

        Ok(())
    }

    /// Create a default configuration file
    async fn create_default_config(&mut self) -> Result<(), DiscoveryError> {
        let default_service = StaticService {
            name: "Local X11 Server".to_string(),
            address: "127.0.0.1:6000".parse().unwrap(),
            display: 0,
            capabilities: vec!["GLX".to_string(), "RANDR".to_string()],
            protocol_version: "11.0".to_string(),
            enabled: true,
            priority: 1,
            description: Some("Default local X11 server".to_string()),
        };

        self.services
            .insert(default_service.name.clone(), default_service);
        self.save_config().await
    }

    /// Add a static service
    pub async fn add_service(&mut self, service: StaticService) -> Result<(), DiscoveryError> {
        self.services.insert(service.name.clone(), service);
        if self.auto_reload {
            self.save_config().await?;
        }
        Ok(())
    }

    /// Remove a static service
    pub async fn remove_service(&mut self, name: &str) -> Result<bool, DiscoveryError> {
        let removed = self.services.remove(name).is_some();
        if removed && self.auto_reload {
            self.save_config().await?;
        }
        Ok(removed)
    }

    /// Update a static service
    pub async fn update_service(
        &mut self,
        name: &str,
        service: StaticService,
    ) -> Result<bool, DiscoveryError> {
        let exists = self.services.contains_key(name);
        if exists {
            self.services.insert(name.to_string(), service);
            if self.auto_reload {
                self.save_config().await?;
            }
        }
        Ok(exists)
    }

    /// Get all available services
    pub fn get_services(&self) -> Vec<&StaticService> {
        let mut services: Vec<&StaticService> = self.services.values().collect();
        services.sort_by(|a, b| b.priority.cmp(&a.priority));
        services
    }

    /// Find a service by name
    pub fn find_service(&self, name: &str) -> Option<&StaticService> {
        self.services.get(name)
    }
    /// Convert static service to generic service info
    pub fn to_service_info(&self, static_service: &StaticService) -> ServiceInfo {
        ServiceInfo {
            name: static_service.name.clone(),
            service_type: "_x11._tcp".to_string(),
            host: static_service.address.ip(),
            address: static_service.address.ip(),
            port: static_service.address.port(),
            display: format!(":{}", static_service.display),
            transport_type: crate::network::transport::TransportType::Tcp,
            capabilities: static_service.capabilities.clone(),
            protocol_version: static_service.protocol_version.clone(),
            properties: HashMap::new(),
            ttl: 120,
            priority: static_service.priority as u16,
            weight: 0,
        }
    }

    /// Get all services as ServiceInfo
    pub fn get_service_infos(&self) -> Vec<ServiceInfo> {
        self.get_services()
            .into_iter()
            .map(|s| self.to_service_info(s))
            .collect()
    }
}

impl From<&ServiceInfo> for StaticService {
    fn from(info: &ServiceInfo) -> Self {
        Self {
            name: info.name.clone(),
            address: SocketAddr::new(info.address, info.port),
            display: info.display.trim_start_matches(':').parse().unwrap_or(0),
            capabilities: info.capabilities.clone(),
            protocol_version: info.protocol_version.clone(),
            enabled: true,
            priority: 1,
            description: None,
        }
    }
}

#[async_trait::async_trait]
impl DiscoveryMechanism for StaticConfigDiscovery {
    async fn start(&mut self) -> Result<(), DiscoveryError> {
        todo!()
    }

    async fn stop(&mut self) -> Result<(), DiscoveryError> {
        todo!()
    }

    async fn advertise_service(&mut self, _service: ServiceInfo) -> Result<(), DiscoveryError> {
        todo!()
    }

    async fn stop_advertising(&mut self, _service_name: &str) -> Result<(), DiscoveryError> {
        todo!()
    }

    fn name(&self) -> &str {
        todo!()
    }

    fn is_running(&self) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_static_config_creation() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.json");

        let mut discovery = StaticConfigDiscovery::new(&config_path);
        discovery.load_config().await.unwrap();

        assert!(!discovery.services.is_empty());
    }

    #[test]
    fn test_static_service_creation() {
        let service = StaticService {
            name: "Test Server".to_string(),
            address: "127.0.0.1:6000".parse().unwrap(),
            display: 0,
            capabilities: vec!["GLX".to_string()],
            protocol_version: "11.0".to_string(),
            enabled: true,
            priority: 1,
            description: Some("Test description".to_string()),
        };

        assert_eq!(service.name, "Test Server");
        assert!(service.enabled);
    }
}
