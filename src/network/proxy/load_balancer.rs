//! Load balancer implementation
//!
//! Provides load balancing capabilities for X11 server connections.

use super::{Proxy, ProxyConnection, ProxyError, ProxyInfo, ProxyType};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio::sync::RwLock;

/// Load balancer configuration
#[derive(Debug, Clone)]
pub struct LoadBalancerConfig {
    pub strategy: BalancingStrategy,
    pub health_check_interval: Duration,
    pub connection_timeout: Duration,
    pub max_retries: u32,
    pub sticky_sessions: bool,
}

/// Load balancing strategies
#[derive(Debug, Clone, PartialEq)]
pub enum BalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    IpHash,
    Random,
}

/// Backend server information
#[derive(Debug, Clone)]
pub struct Backend {
    pub address: SocketAddr,
    pub weight: u32,
    pub max_connections: Option<u32>,
    pub enabled: bool,
}

/// Backend server status
#[derive(Debug, Clone)]
pub struct BackendStatus {
    pub backend: Backend,
    pub active_connections: u32,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub last_health_check: Option<Instant>,
    pub is_healthy: bool,
}

/// Load balancer implementation
pub struct LoadBalancer {
    config: LoadBalancerConfig,
    backends: Arc<RwLock<Vec<BackendStatus>>>,
    current_index: Arc<RwLock<usize>>,
    session_map: Arc<RwLock<HashMap<String, SocketAddr>>>,
}

/// Load balancer connection wrapper
pub struct LoadBalancerConnection {
    stream: TcpStream,
    backend_addr: SocketAddr,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            strategy: BalancingStrategy::RoundRobin,
            health_check_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            max_retries: 3,
            sticky_sessions: false,
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:6000".parse().unwrap(),
            weight: 1,
            max_connections: None,
            enabled: true,
        }
    }
}

#[async_trait::async_trait]
impl Proxy for LoadBalancer {
    type Config = LoadBalancerConfig;
    type Error = ProxyError;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            config,
            backends: Arc::new(RwLock::new(Vec::new())),
            current_index: Arc::new(RwLock::new(0)),
            session_map: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    async fn connect(&self, target: SocketAddr) -> Result<Box<dyn ProxyConnection>, Self::Error> {
        let backend_addr = self.select_backend(target).await?;

        let stream = TcpStream::connect(backend_addr)
            .await
            .map_err(|e| ProxyError::Connection(format!("Failed to connect to backend: {}", e)))?;

        let local_addr = stream.local_addr().map_err(|e| ProxyError::Io(e))?;

        // Update backend statistics
        self.update_backend_stats(backend_addr, true).await;

        let connection = LoadBalancerConnection {
            stream,
            backend_addr,
            local_addr,
            remote_addr: target,
        };

        Ok(Box::new(connection))
    }

    fn info(&self) -> ProxyInfo {
        ProxyInfo {
            name: "Load Balancer".to_string(),
            proxy_type: ProxyType::LoadBalancer,
            version: "1.0".to_string(),
            capabilities: vec![
                "load_balancing".to_string(),
                "health_checking".to_string(),
                "failover".to_string(),
                format!("strategy_{:?}", self.config.strategy).to_lowercase(),
            ],
        }
    }
}

impl LoadBalancer {
    /// Add a backend server
    pub async fn add_backend(&self, backend: Backend) {
        let status = BackendStatus {
            backend,
            active_connections: 0,
            total_requests: 0,
            failed_requests: 0,
            last_health_check: None,
            is_healthy: true,
        };

        self.backends.write().await.push(status);
    }

    /// Remove a backend server
    pub async fn remove_backend(&self, address: SocketAddr) -> bool {
        let mut backends = self.backends.write().await;
        let len_before = backends.len();
        backends.retain(|status| status.backend.address != address);
        backends.len() != len_before
    }

    /// Get all backend statuses
    pub async fn get_backend_statuses(&self) -> Vec<BackendStatus> {
        self.backends.read().await.clone()
    }

    /// Perform health checks on all backends
    pub async fn health_check(&self) -> Result<(), ProxyError> {
        let mut backends = self.backends.write().await;

        for status in backends.iter_mut() {
            if !status.backend.enabled {
                continue;
            }

            let is_healthy = self.check_backend_health(&status.backend).await;
            status.is_healthy = is_healthy;
            status.last_health_check = Some(Instant::now());
        }

        Ok(())
    }

    /// Select a backend based on the configured strategy
    async fn select_backend(&self, _client_addr: SocketAddr) -> Result<SocketAddr, ProxyError> {
        let backends = self.backends.read().await;
        let healthy_backends: Vec<&BackendStatus> = backends
            .iter()
            .filter(|s| s.backend.enabled && s.is_healthy)
            .collect();

        if healthy_backends.is_empty() {
            return Err(ProxyError::Network(
                "No healthy backends available".to_string(),
            ));
        }

        let selected_addr = match self.config.strategy {
            BalancingStrategy::RoundRobin => self.select_round_robin(&healthy_backends).await,
            BalancingStrategy::LeastConnections => {
                self.select_least_connections(&healthy_backends)
                    .await
                    .backend
                    .address
            }
            BalancingStrategy::WeightedRoundRobin => {
                self.select_weighted_round_robin(&healthy_backends)
                    .await
                    .backend
                    .address
            }
            BalancingStrategy::Random => self.select_random(&healthy_backends).await,
            BalancingStrategy::IpHash => self.select_ip_hash(&healthy_backends, _client_addr).await,
        };

        Ok(selected_addr)
    }

    /// Round robin selection
    async fn select_round_robin(&self, backends: &[&BackendStatus]) -> SocketAddr {
        let mut index = self.current_index.write().await;
        let selected = backends[*index % backends.len()];
        *index = (*index + 1) % backends.len();
        selected.backend.address
    }

    /// Least connections selection
    async fn select_least_connections<'a>(
        &self,
        backends: &'a [&BackendStatus],
    ) -> &'a BackendStatus {
        let min_index = backends
            .iter()
            .enumerate()
            .min_by_key(|(_, s)| s.active_connections)
            .map(|(i, _)| i)
            .unwrap();
        backends[min_index]
    }

    /// Weighted round robin selection
    async fn select_weighted_round_robin<'a>(
        &self,
        backends: &'a [&'a BackendStatus],
    ) -> &'a BackendStatus {
        // Simplified weighted selection - in practice, this would be more sophisticated
        let total_weight: u32 = backends.iter().map(|s| s.backend.weight).sum();
        if total_weight == 0 {
            return backends[0];
        }

        let mut index = self.current_index.write().await;
        *index = (*index + 1) % total_weight as usize;

        let mut weight_sum = 0;
        for backend in backends {
            weight_sum += backend.backend.weight;
            if *index < weight_sum as usize {
                return backend;
            }
        }

        backends[0]
    }

    /// Random selection
    async fn select_random(&self, backends: &[&BackendStatus]) -> SocketAddr {
        let index = rand::random::<usize>() % backends.len();
        backends[index].backend.address
    }

    /// IP hash selection
    async fn select_ip_hash(
        &self,
        backends: &[&BackendStatus],
        client_addr: SocketAddr,
    ) -> SocketAddr {
        let hash = self.hash_address(client_addr);
        let index = hash % backends.len();
        backends[index].backend.address
    }
    fn hash_address(&self, addr: SocketAddr) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        addr.ip().hash(&mut hasher);
        hasher.finish() as usize
    }

    /// Check if a backend is healthy
    async fn check_backend_health(&self, backend: &Backend) -> bool {
        // Simple TCP connection test
        tokio::time::timeout(
            self.config.connection_timeout,
            TcpStream::connect(backend.address),
        )
        .await
        .is_ok()
    }

    /// Update backend statistics
    async fn update_backend_stats(&self, backend_addr: SocketAddr, success: bool) {
        let mut backends = self.backends.write().await;

        if let Some(status) = backends
            .iter_mut()
            .find(|s| s.backend.address == backend_addr)
        {
            if success {
                status.active_connections += 1;
                status.total_requests += 1;
            } else {
                status.failed_requests += 1;
            }
        }
    }

    /// Start periodic health checks
    pub async fn start_health_checks(&self) {
        let interval = self.config.health_check_interval;
        let load_balancer = self.clone();

        tokio::spawn(async move {
            let mut timer = tokio::time::interval(interval);
            loop {
                timer.tick().await;
                if let Err(e) = load_balancer.health_check().await {
                    eprintln!("Health check failed: {}", e);
                }
            }
        });
    }
}

impl Clone for LoadBalancer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            backends: Arc::clone(&self.backends),
            current_index: Arc::clone(&self.current_index),
            session_map: Arc::clone(&self.session_map),
        }
    }
}

#[async_trait::async_trait]
impl ProxyConnection for LoadBalancerConnection {
    fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        Ok(self.local_addr)
    }

    fn remote_addr(&self) -> Result<SocketAddr, std::io::Error> {
        Ok(self.remote_addr)
    }

    async fn close(&mut self) -> Result<(), std::io::Error> {
        // Could update backend connection count here
        Ok(())
    }
}

impl AsyncRead for LoadBalancerConnection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for LoadBalancerConnection {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_balancer_creation() {
        let config = LoadBalancerConfig::default();
        let lb = LoadBalancer::new(config).unwrap();

        let backend = Backend::default();
        lb.add_backend(backend).await;

        let statuses = lb.get_backend_statuses().await;
        assert_eq!(statuses.len(), 1);
    }

    #[test]
    fn test_balancing_strategy() {
        assert_eq!(BalancingStrategy::RoundRobin, BalancingStrategy::RoundRobin);
        assert_ne!(BalancingStrategy::RoundRobin, BalancingStrategy::Random);
    }
}
