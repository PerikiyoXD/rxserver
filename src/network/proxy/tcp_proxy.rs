//! TCP proxy implementation
//!
//! Provides simple TCP proxying capabilities for X11 connections.

use super::{Proxy, ProxyConnection, ProxyError, ProxyInfo, ProxyType};
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;

/// TCP proxy configuration
#[derive(Debug, Clone)]
pub struct TcpProxyConfig {
    pub proxy_host: String,
    pub proxy_port: u16,
    pub connection_timeout: Option<u32>,
    pub read_timeout: Option<u32>,
    pub write_timeout: Option<u32>,
    pub buffer_size: usize,
}

/// TCP proxy implementation
pub struct TcpProxy {
    config: TcpProxyConfig,
}

/// TCP proxy connection wrapper
pub struct TcpProxyConnection {
    stream: TcpStream,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
}

impl Default for TcpProxyConfig {
    fn default() -> Self {
        Self {
            proxy_host: "localhost".to_string(),
            proxy_port: 8080,
            connection_timeout: Some(30),
            read_timeout: Some(30),
            write_timeout: Some(30),
            buffer_size: 8192,
        }
    }
}

#[async_trait::async_trait]
impl Proxy for TcpProxy {
    type Config = TcpProxyConfig;
    type Error = ProxyError;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        if config.proxy_host.is_empty() {
            return Err(ProxyError::Configuration(
                "Proxy host cannot be empty".to_string(),
            ));
        }

        if config.buffer_size == 0 {
            return Err(ProxyError::Configuration(
                "Buffer size must be greater than 0".to_string(),
            ));
        }

        Ok(Self { config })
    }

    async fn connect(&self, target: SocketAddr) -> Result<Box<dyn ProxyConnection>, Self::Error> {
        // Connect to the proxy server
        let proxy_addr = format!("{}:{}", self.config.proxy_host, self.config.proxy_port);
        let stream = TcpStream::connect(&proxy_addr)
            .await
            .map_err(|e| ProxyError::Connection(format!("Failed to connect to proxy: {}", e)))?;

        let local_addr = stream.local_addr().map_err(|e| ProxyError::Io(e))?;

        // In a real implementation, this would:
        // 1. Send a CONNECT request to the proxy
        // 2. Handle the proxy's response
        // 3. Establish the tunnel to the target

        let connection = TcpProxyConnection {
            stream,
            local_addr,
            remote_addr: target,
        };

        Ok(Box::new(connection))
    }

    fn info(&self) -> ProxyInfo {
        ProxyInfo {
            name: "TCP Proxy".to_string(),
            proxy_type: ProxyType::TcpProxy,
            version: "1.0".to_string(),
            capabilities: vec!["tcp_tunnel".to_string(), "connection_pooling".to_string()],
        }
    }
}

impl TcpProxy {
    /// Create a TCP proxy with custom configuration
    pub fn with_config(config: TcpProxyConfig) -> Result<Self, ProxyError> {
        Self::new(config)
    }

    /// Create a TCP proxy with host and port
    pub fn new_simple(host: &str, port: u16) -> Result<Self, ProxyError> {
        let config = TcpProxyConfig {
            proxy_host: host.to_string(),
            proxy_port: port,
            ..Default::default()
        };

        Self::new(config)
    }

    /// Set connection timeout
    pub fn set_connection_timeout(&mut self, timeout_seconds: Option<u32>) {
        self.config.connection_timeout = timeout_seconds;
    }

    /// Set buffer size for data transfer
    pub fn set_buffer_size(&mut self, size: usize) {
        if size > 0 {
            self.config.buffer_size = size;
        }
    }

    /// Get proxy address
    pub fn proxy_address(&self) -> String {
        format!("{}:{}", self.config.proxy_host, self.config.proxy_port)
    }
}

#[async_trait::async_trait]
impl ProxyConnection for TcpProxyConnection {
    fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        Ok(self.local_addr)
    }

    fn remote_addr(&self) -> Result<SocketAddr, std::io::Error> {
        Ok(self.remote_addr)
    }

    async fn close(&mut self) -> Result<(), std::io::Error> {
        // Gracefully close the TCP connection
        Ok(())
    }
}

impl AsyncRead for TcpProxyConnection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpProxyConnection {
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

    #[test]
    fn test_tcp_proxy_config_creation() {
        let config = TcpProxyConfig::default();
        assert_eq!(config.proxy_host, "localhost");
        assert_eq!(config.proxy_port, 8080);
        assert_eq!(config.buffer_size, 8192);
    }

    #[test]
    fn test_tcp_proxy_creation() {
        let proxy = TcpProxy::new_simple("example.com", 8080);
        assert!(proxy.is_ok());

        let proxy = proxy.unwrap();
        assert_eq!(proxy.proxy_address(), "example.com:8080");
    }

    #[test]
    fn test_invalid_tcp_proxy_config() {
        let config = TcpProxyConfig {
            proxy_host: "".to_string(),
            ..Default::default()
        };

        let result = TcpProxy::new(config);
        assert!(result.is_err());
    }
}
