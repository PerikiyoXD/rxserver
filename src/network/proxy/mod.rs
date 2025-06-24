//! Network proxy module
//!
//! Provides various proxy implementations for X11 network connections.

pub mod load_balancer;
pub mod ssh;
pub mod stunnel;
pub mod tcp_proxy;

// Re-export commonly used items
pub use load_balancer::{BalancingStrategy, LoadBalancer, LoadBalancerConfig};
pub use ssh::{SshConfig, SshError, SshProxy};
pub use stunnel::{StunnelConfig, StunnelProxy};
pub use tcp_proxy::{TcpProxy, TcpProxyConfig};

use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncWrite};

/// Common proxy trait for all proxy implementations
#[async_trait::async_trait]
pub trait Proxy: Send + Sync {
    type Config;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create a new proxy instance with the given configuration
    fn new(config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Establish a proxied connection to the target
    async fn connect(&self, target: SocketAddr) -> Result<Box<dyn ProxyConnection>, Self::Error>;

    /// Get proxy information
    fn info(&self) -> ProxyInfo;
}

/// Trait for proxied connections
#[async_trait::async_trait]
pub trait ProxyConnection: AsyncRead + AsyncWrite + Send + Sync + Unpin {
    /// Get the local address of the proxied connection
    fn local_addr(&self) -> Result<SocketAddr, std::io::Error>;

    /// Get the remote address of the proxied connection
    fn remote_addr(&self) -> Result<SocketAddr, std::io::Error>;

    /// Close the proxied connection
    async fn close(&mut self) -> Result<(), std::io::Error>;
}

/// Proxy information
#[derive(Debug, Clone)]
pub struct ProxyInfo {
    pub name: String,
    pub proxy_type: ProxyType,
    pub version: String,
    pub capabilities: Vec<String>,
}

/// Types of supported proxies
#[derive(Debug, Clone, PartialEq)]
pub enum ProxyType {
    Ssh,
    TcpProxy,
    Stunnel,
    LoadBalancer,
}

/// Common proxy errors
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("Connection failed: {0}")]
    Connection(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_info_creation() {
        let info = ProxyInfo {
            name: "Test Proxy".to_string(),
            proxy_type: ProxyType::TcpProxy,
            version: "1.0".to_string(),
            capabilities: vec!["compression".to_string()],
        };

        assert_eq!(info.name, "Test Proxy");
        assert_eq!(info.proxy_type, ProxyType::TcpProxy);
    }
}
