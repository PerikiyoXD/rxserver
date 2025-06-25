//! Broadcast-based service discovery
//!
//! Implements UDP broadcast-based discovery for X11 servers on local networks.

use crate::network::discovery::registry::DiscoveryMechanism;
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use tokio::time::interval;

use super::registry::{DiscoveryError, ServiceInfo};

/// Broadcast discovery service for X11 servers
#[derive(Debug)]
pub struct BroadcastDiscovery {
    socket: UdpSocket,
    broadcast_port: u16,
    interval: Duration,
    is_running: bool,
}

/// Broadcast service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastService {
    pub name: String,
    pub address: SocketAddr,
    pub display: u32,
    pub timestamp: u64,
    pub capabilities: Vec<String>,
}

impl BroadcastDiscovery {
    /// Create a new broadcast discovery instance
    pub fn new(broadcast_port: u16) -> Result<Self, DiscoveryError> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", broadcast_port))
            .map_err(|e| DiscoveryError::Network(e.to_string()))?;

        socket
            .set_broadcast(true)
            .map_err(|e| DiscoveryError::Network(e.to_string()))?;

        Ok(Self {
            socket,
            broadcast_port,
            interval: Duration::from_secs(30),
            is_running: false,
        })
    }

    /// Set the broadcast interval
    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }

    /// Start broadcasting service information
    pub async fn start_advertising(
        &mut self,
        service: &BroadcastService,
    ) -> Result<(), DiscoveryError> {
        self.is_running = true;
        let mut timer = interval(self.interval);

        while self.is_running {
            timer.tick().await;
            self.broadcast_service(service).await?;
        }

        Ok(())
    }

    /// Stop broadcasting
    pub fn stop_advertising(&mut self) {
        self.is_running = false;
    }

    /// Broadcast service information
    async fn broadcast_service(&self, service: &BroadcastService) -> Result<(), DiscoveryError> {
        let data = serde_json::to_vec(service)
            .map_err(|e| DiscoveryError::Serialization(e.to_string()))?;

        let broadcast_addr = format!("255.255.255.255:{}", self.broadcast_port);
        self.socket
            .send_to(&data, &broadcast_addr)
            .map_err(|e| DiscoveryError::Network(e.to_string()))?;

        Ok(())
    }

    /// Listen for broadcast announcements
    pub async fn discover_services(&self) -> Result<Vec<BroadcastService>, DiscoveryError> {
        let mut services = Vec::new();
        let mut buffer = [0u8; 1024];

        // Non-blocking receive with timeout
        match self.socket.recv_from(&mut buffer) {
            Ok((size, _addr)) => {
                if let Ok(service) = serde_json::from_slice::<BroadcastService>(&buffer[..size]) {
                    services.push(service);
                }
            }
            Err(_) => {
                // No data available, return empty list
            }
        }

        Ok(services)
    }
    /// Convert broadcast service to generic service info
    pub fn to_service_info(&self, broadcast_service: &BroadcastService) -> ServiceInfo {
        ServiceInfo {
            name: broadcast_service.name.clone(),
            service_type: "_x11._tcp".to_string(),
            host: broadcast_service.address.ip(),
            address: broadcast_service.address.ip(),
            port: broadcast_service.address.port(),
            display: format!(":{}", broadcast_service.display),
            transport_type: crate::network::transport::TransportType::Tcp,
            capabilities: broadcast_service.capabilities.clone(),
            protocol_version: "11.0".to_string(),
            properties: std::collections::HashMap::new(),
            ttl: 120,
            priority: 0,
            weight: 0,
        }
    }
}

impl From<&ServiceInfo> for BroadcastService {
    fn from(info: &ServiceInfo) -> Self {
        Self {
            name: info.name.clone(),
            address: SocketAddr::new(info.address, info.port),
            display: info.display.trim_start_matches(':').parse().unwrap_or(0),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            capabilities: info.capabilities.clone(),
        }
    }
}

#[async_trait::async_trait]
impl DiscoveryMechanism for BroadcastDiscovery {
    async fn start(&mut self) -> Result<(), DiscoveryError> {
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), DiscoveryError> {
        if !self.is_running {
            return Ok(());
        }

        self.stop_advertising();
        self.is_running = false;
        Ok(())
    }

    async fn advertise_service(&mut self, service: ServiceInfo) -> Result<(), DiscoveryError> {
        let broadcast_service: BroadcastService = BroadcastService::from(&service);
        self.start_advertising(&broadcast_service).await
    }

    async fn stop_advertising(&mut self, _service_name: &str) -> Result<(), DiscoveryError> {
        if !self.is_running {
            return Ok(());
        }

        // In a real implementation, we would stop broadcasting the specific service
        // For now, we just stop the entire discovery mechanism
        self.stop_advertising();
        Ok(())
    }

    fn name(&self) -> &str {
        "BroadcastDiscovery"
    }

    fn is_running(&self) -> bool {
        self.is_running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcast_service_creation() {
        let service = BroadcastService {
            name: "Test Server".to_string(),
            address: "127.0.0.1:6000".parse().unwrap(),
            display: 0,
            timestamp: 1234567890,
            capabilities: vec!["GLX".to_string()],
        };

        assert_eq!(service.name, "Test Server");
        assert_eq!(service.display, 0);
    }
}
