//! SSH proxy implementation
//!
//! Provides SSH tunneling capabilities for X11 connections.

use super::{Proxy, ProxyConnection, ProxyInfo, ProxyType};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;

/// SSH proxy configuration
#[derive(Debug, Clone)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub authentication: SshAuthentication,
    pub compression: bool,
    pub keep_alive: Option<u32>,
    pub connection_timeout: Option<u32>,
}

/// SSH authentication methods
#[derive(Debug, Clone)]
pub enum SshAuthentication {
    Password(String),
    PublicKey {
        private_key_path: PathBuf,
        passphrase: Option<String>,
    },
    Agent,
    Interactive,
}

/// SSH proxy implementation
pub struct SshProxy {
    config: SshConfig,
}

/// SSH proxy connection wrapper
pub struct SshConnection {
    stream: TcpStream,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
}

/// SSH-specific errors
#[derive(Debug, thiserror::Error)]
pub enum SshError {
    #[error("SSH connection failed: {0}")]
    ConnectionFailed(String),

    #[error("SSH authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("SSH tunnel creation failed: {0}")]
    TunnelFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 22,
            username: "user".to_string(),
            authentication: SshAuthentication::Agent,
            compression: true,
            keep_alive: Some(30),
            connection_timeout: Some(10),
        }
    }
}

#[async_trait::async_trait]
impl Proxy for SshProxy {
    type Config = SshConfig;
    type Error = SshError;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        // Validate configuration
        if config.host.is_empty() {
            return Err(SshError::InvalidConfig("Host cannot be empty".to_string()));
        }

        if config.username.is_empty() {
            return Err(SshError::InvalidConfig(
                "Username cannot be empty".to_string(),
            ));
        }

        Ok(Self { config })
    }

    async fn connect(&self, target: SocketAddr) -> Result<Box<dyn ProxyConnection>, Self::Error> {
        // In a real implementation, this would:
        // 1. Establish SSH connection to the proxy host
        // 2. Authenticate using the configured method
        // 3. Create a tunnel to the target address
        // 4. Return a ProxyConnection that wraps the tunneled stream

        // For now, we'll create a mock connection
        let stream = TcpStream::connect(&self.config.host)
            .await
            .map_err(|e| SshError::ConnectionFailed(e.to_string()))?;

        let local_addr = stream.local_addr().map_err(|e| SshError::Io(e))?;

        let connection = SshConnection {
            stream,
            local_addr,
            remote_addr: target,
        };

        Ok(Box::new(connection))
    }

    fn info(&self) -> ProxyInfo {
        ProxyInfo {
            name: "SSH Proxy".to_string(),
            proxy_type: ProxyType::Ssh,
            version: "2.0".to_string(),
            capabilities: vec![
                "compression".to_string(),
                "encryption".to_string(),
                "authentication".to_string(),
            ],
        }
    }
}

impl SshProxy {
    /// Create SSH proxy with password authentication
    pub fn with_password(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> Result<Self, SshError> {
        let config = SshConfig {
            host: host.to_string(),
            port,
            username: username.to_string(),
            authentication: SshAuthentication::Password(password.to_string()),
            ..Default::default()
        };

        Self::new(config)
    }

    /// Create SSH proxy with public key authentication
    pub fn with_public_key(
        host: &str,
        port: u16,
        username: &str,
        private_key_path: PathBuf,
        passphrase: Option<String>,
    ) -> Result<Self, SshError> {
        let config = SshConfig {
            host: host.to_string(),
            port,
            username: username.to_string(),
            authentication: SshAuthentication::PublicKey {
                private_key_path,
                passphrase,
            },
            ..Default::default()
        };

        Self::new(config)
    }

    /// Enable or disable compression
    pub fn set_compression(&mut self, enabled: bool) {
        self.config.compression = enabled;
    }

    /// Set keep-alive interval in seconds
    pub fn set_keep_alive(&mut self, seconds: Option<u32>) {
        self.config.keep_alive = seconds;
    }
}

#[async_trait::async_trait]
impl ProxyConnection for SshConnection {
    fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        Ok(self.local_addr)
    }

    fn remote_addr(&self) -> Result<SocketAddr, std::io::Error> {
        Ok(self.remote_addr)
    }

    async fn close(&mut self) -> Result<(), std::io::Error> {
        // In a real implementation, this would cleanly close the SSH tunnel
        Ok(())
    }
}

impl AsyncRead for SshConnection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for SshConnection {
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
    fn test_ssh_config_creation() {
        let config = SshConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 22);
        assert!(config.compression);
    }

    #[test]
    fn test_ssh_proxy_creation() {
        let proxy = SshProxy::with_password("example.com", 22, "user", "password");
        assert!(proxy.is_ok());
    }

    #[test]
    fn test_invalid_ssh_config() {
        let config = SshConfig {
            host: "".to_string(),
            ..Default::default()
        };

        let result = SshProxy::new(config);
        assert!(result.is_err());
    }
}
