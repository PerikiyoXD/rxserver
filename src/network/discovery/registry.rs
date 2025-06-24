//! Service discovery registry
//!
//! Central registry for managing service discovery and advertisement.

use crate::network::transport::TransportType;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};

/// Discovery error
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Discovery mechanism not available: {0}")]
    MechanismNotAvailable(String),

    #[error("Invalid service information: {0}")]
    InvalidServiceInfo(String),
    #[error("Network error: {0}")]
    Network(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Service information
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Service name/identifier
    pub name: String,
    /// Service type (e.g., "_x11._tcp")
    pub service_type: String,
    /// Host address
    pub host: IpAddr,
    /// Service display address (for X11 compatibility)
    pub address: IpAddr,
    /// Port number
    pub port: u16,
    /// Display number (for X11)
    pub display: String,
    /// Transport type
    pub transport_type: TransportType,
    /// Service capabilities
    pub capabilities: Vec<String>,
    /// Protocol version
    pub protocol_version: String,
    /// Service properties/metadata
    pub properties: HashMap<String, String>,
    /// Service TTL (time to live)
    pub ttl: u32,
    /// Service priority
    pub priority: u16,
    /// Service weight
    pub weight: u16,
}

impl ServiceInfo {
    /// Create a new service info
    pub fn new(
        name: String,
        service_type: String,
        host: IpAddr,
        port: u16,
        transport_type: TransportType,
    ) -> Self {
        Self {
            name,
            service_type,
            host,
            address: host,
            port,
            display: format!(":{}", port - 6000).to_string(), // X11 display convention
            transport_type,
            capabilities: Vec::new(),
            protocol_version: "11.0".to_string(),
            properties: HashMap::new(),
            ttl: 120, // 2 minutes default
            priority: 0,
            weight: 0,
        }
    }

    /// Add a property
    pub fn with_property(mut self, key: String, value: String) -> Self {
        self.properties.insert(key, value);
        self
    }

    /// Set TTL
    pub fn with_ttl(mut self, ttl: u32) -> Self {
        self.ttl = ttl;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u16) -> Self {
        self.priority = priority;
        self
    }

    /// Set weight
    pub fn with_weight(mut self, weight: u16) -> Self {
        self.weight = weight;
        self
    }

    /// Get service address as string
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Get full service name
    pub fn full_name(&self) -> String {
        format!("{}.{}", self.name, self.service_type)
    }
}

/// Discovery event
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    /// Service discovered
    ServiceDiscovered {
        service: ServiceInfo,
        discovery_method: String,
    },
    /// Service updated
    ServiceUpdated {
        service: ServiceInfo,
        discovery_method: String,
    },
    /// Service lost/removed
    ServiceLost {
        service_name: String,
        discovery_method: String,
    },
    /// Discovery error
    DiscoveryError {
        error: String,
        discovery_method: String,
    },
}

/// Discovery mechanism trait
#[async_trait::async_trait]
pub trait DiscoveryMechanism: Send + Sync + std::fmt::Debug {
    /// Start discovery
    async fn start(&mut self) -> Result<(), DiscoveryError>;

    /// Stop discovery
    async fn stop(&mut self) -> Result<(), DiscoveryError>;

    /// Advertise a service
    async fn advertise_service(&mut self, service: ServiceInfo) -> Result<(), DiscoveryError>;

    /// Stop advertising a service
    async fn stop_advertising(&mut self, service_name: &str) -> Result<(), DiscoveryError>;

    /// Get mechanism name
    fn name(&self) -> &str;

    /// Check if mechanism is running
    fn is_running(&self) -> bool;
}

/// Service registry configuration
#[derive(Debug, Clone)]
pub struct ServiceRegistryConfig {
    /// Enable mDNS discovery
    pub enable_mdns: bool,
    /// Enable DNS-SD discovery
    pub enable_dns_sd: bool,
    /// Enable broadcast discovery
    pub enable_broadcast: bool,
    /// Broadcast port for discovery
    pub broadcast_port: u16,
    /// Enable static configuration
    pub enable_static_config: bool,
    /// Static configuration file path
    pub static_config_file: std::path::PathBuf,
    /// Service advertisement interval (seconds)
    pub advertisement_interval: u64,
    /// Service discovery timeout (seconds)
    pub discovery_timeout: u64,
}

impl Default for ServiceRegistryConfig {
    fn default() -> Self {
        Self {
            enable_mdns: true,
            enable_dns_sd: false,
            enable_broadcast: true,
            broadcast_port: 6001,
            enable_static_config: true,
            static_config_file: std::path::PathBuf::from("services.toml"),
            advertisement_interval: 30,
            discovery_timeout: 10,
        }
    }
}

/// Service registry
pub struct ServiceRegistry {
    /// Configuration
    config: ServiceRegistryConfig,
    /// Discovered services
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    /// Advertised services
    advertised_services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    /// Discovery mechanisms
    mechanisms: Vec<Box<dyn DiscoveryMechanism>>,
    /// Event sender
    event_sender: mpsc::UnboundedSender<DiscoveryEvent>,
    /// Running state
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new(config: ServiceRegistryConfig) -> (Self, mpsc::UnboundedReceiver<DiscoveryEvent>) {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let registry = Self {
            config,
            services: Arc::new(RwLock::new(HashMap::new())),
            advertised_services: Arc::new(RwLock::new(HashMap::new())),
            mechanisms: Vec::new(),
            event_sender,
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };

        (registry, event_receiver)
    }

    /// Initialize discovery mechanisms
    pub async fn initialize(&mut self) -> Result<(), DiscoveryError> {
        info!("Initializing service discovery mechanisms");

        // Initialize mDNS if enabled
        if self.config.enable_mdns {
            match super::mdns::MdnsDiscovery::new(self.event_sender.clone()).await {
                Ok(mdns) => {
                    debug!("Initialized mDNS discovery");
                    self.mechanisms.push(Box::new(mdns));
                }
                Err(e) => {
                    warn!("Failed to initialize mDNS discovery: {}", e);
                }
            }
        }

        // Initialize DNS-SD if enabled
        if self.config.enable_dns_sd {
            match super::dns_sd::DnsSdDiscovery::new(self.event_sender.clone()).await {
                Ok(dns_sd) => {
                    debug!("Initialized DNS-SD discovery");
                    self.mechanisms.push(Box::new(dns_sd));
                }
                Err(e) => {
                    warn!("Failed to initialize DNS-SD discovery: {}", e);
                }
            }
        }

        // Initialize broadcast discovery if enabled
        if self.config.enable_broadcast {
            match super::broadcast::BroadcastDiscovery::new(self.config.broadcast_port) {
                Ok(broadcast) => {
                    debug!("Initialized broadcast discovery");
                    self.mechanisms.push(Box::new(broadcast));
                }
                Err(e) => {
                    warn!("Failed to initialize broadcast discovery: {}", e);
                }
            }
        }

        // Initialize static config discovery if enabled
        if self.config.enable_static_config {
            let static_config =
                super::static_config::StaticConfigDiscovery::new(&self.config.static_config_file);
            debug!("Initialized static config discovery");
            self.mechanisms.push(Box::new(static_config));
        }

        info!("Initialized {} discovery mechanisms", self.mechanisms.len());
        Ok(())
    }

    /// Start service discovery
    pub async fn start(&mut self) -> Result<(), DiscoveryError> {
        if self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        info!("Starting service discovery");

        // Start all mechanisms
        for mechanism in &mut self.mechanisms {
            if let Err(e) = mechanism.start().await {
                error!(
                    "Failed to start discovery mechanism {}: {}",
                    mechanism.name(),
                    e
                );
            } else {
                debug!("Started discovery mechanism: {}", mechanism.name());
            }
        }

        self.is_running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        // Start event processing
        self.start_event_processing().await;

        info!("Service discovery started");
        Ok(())
    }

    /// Stop service discovery
    pub async fn stop(&mut self) -> Result<(), DiscoveryError> {
        if !self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        info!("Stopping service discovery");

        self.is_running
            .store(false, std::sync::atomic::Ordering::SeqCst);

        // Stop all mechanisms
        for mechanism in &mut self.mechanisms {
            if let Err(e) = mechanism.stop().await {
                error!(
                    "Failed to stop discovery mechanism {}: {}",
                    mechanism.name(),
                    e
                );
            } else {
                debug!("Stopped discovery mechanism: {}", mechanism.name());
            }
        }

        info!("Service discovery stopped");
        Ok(())
    }

    /// Advertise a service
    pub async fn advertise_service(&mut self, service: ServiceInfo) -> Result<(), DiscoveryError> {
        let service_name = service.full_name();

        info!("Advertising service: {}", service_name);

        // Add to advertised services
        {
            let mut advertised = self.advertised_services.write().await;
            advertised.insert(service_name.clone(), service.clone());
        }

        // Advertise on all mechanisms
        for mechanism in &mut self.mechanisms {
            if let Err(e) = mechanism.advertise_service(service.clone()).await {
                warn!(
                    "Failed to advertise service {} on {}: {}",
                    service_name,
                    mechanism.name(),
                    e
                );
            } else {
                debug!(
                    "Advertised service {} on {}",
                    service_name,
                    mechanism.name()
                );
            }
        }

        Ok(())
    }

    /// Stop advertising a service
    pub async fn stop_advertising(&mut self, service_name: &str) -> Result<(), DiscoveryError> {
        info!("Stopping advertisement of service: {}", service_name);

        // Remove from advertised services
        {
            let mut advertised = self.advertised_services.write().await;
            advertised.remove(service_name);
        }

        // Stop advertising on all mechanisms
        for mechanism in &mut self.mechanisms {
            if let Err(e) = mechanism.stop_advertising(service_name).await {
                warn!(
                    "Failed to stop advertising service {} on {}: {}",
                    service_name,
                    mechanism.name(),
                    e
                );
            } else {
                debug!(
                    "Stopped advertising service {} on {}",
                    service_name,
                    mechanism.name()
                );
            }
        }

        Ok(())
    }

    /// Get discovered services
    pub async fn get_services(&self) -> HashMap<String, ServiceInfo> {
        let services = self.services.read().await;
        services.clone()
    }

    /// Get a specific service
    pub async fn get_service(&self, service_name: &str) -> Option<ServiceInfo> {
        let services = self.services.read().await;
        services.get(service_name).cloned()
    }

    /// Get advertised services
    pub async fn get_advertised_services(&self) -> HashMap<String, ServiceInfo> {
        let advertised = self.advertised_services.read().await;
        advertised.clone()
    }

    /// Start event processing
    async fn start_event_processing(&self) {
        let services = self.services.clone();
        let is_running = self.is_running.clone();

        // In a real implementation, this would listen to the event receiver
        // and update the services map accordingly
        tokio::spawn(async move {
            while is_running.load(std::sync::atomic::Ordering::SeqCst) {
                // Process discovery events
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.is_running.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Get active mechanisms
    pub fn get_active_mechanisms(&self) -> Vec<String> {
        self.mechanisms
            .iter()
            .filter(|m| m.is_running())
            .map(|m| m.name().to_string())
            .collect()
    }
}
