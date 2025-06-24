//! Stunnel proxy implementation
//!
//! Provides SSL/TLS tunnel capabilities for X11 connections using stunnel.

use super::{Proxy, ProxyConnection, ProxyError, ProxyInfo, ProxyType};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;

/// Stunnel proxy configuration
#[derive(Debug, Clone)]
pub struct StunnelConfig {
    pub local_host: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub cert_file: Option<PathBuf>,
    pub key_file: Option<PathBuf>,
    pub ca_file: Option<PathBuf>,
    pub verify_peer: bool,
    pub protocol_version: TlsVersion,
    pub ciphers: Option<String>,
    pub compression: bool,
}

/// TLS/SSL protocol versions
#[derive(Debug, Clone, PartialEq)]
pub enum TlsVersion {
    TlsV1_2,
    TlsV1_3,
    All,
}

/// Stunnel proxy implementation
pub struct StunnelProxy {
    config: StunnelConfig,
}

/// Stunnel proxy connection wrapper
pub struct StunnelConnection {
    stream: TcpStream,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
}

impl Default for StunnelConfig {
    fn default() -> Self {
        Self {
            local_host: "localhost".to_string(),
            local_port: 6000,
            remote_host: "localhost".to_string(),
            remote_port: 6001,
            cert_file: None,
            key_file: None,
            ca_file: None,
            verify_peer: true,
            protocol_version: TlsVersion::TlsV1_3,
            ciphers: None,
            compression: false,
        }
    }
}

impl std::fmt::Display for TlsVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TlsVersion::TlsV1_2 => write!(f, "TLSv1.2"),
            TlsVersion::TlsV1_3 => write!(f, "TLSv1.3"),
            TlsVersion::All => write!(f, "all"),
        }
    }
}

#[async_trait::async_trait]
impl Proxy for StunnelProxy {
    type Config = StunnelConfig;
    type Error = ProxyError;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        if config.local_host.is_empty() {
            return Err(ProxyError::Configuration(
                "Local host cannot be empty".to_string(),
            ));
        }

        if config.remote_host.is_empty() {
            return Err(ProxyError::Configuration(
                "Remote host cannot be empty".to_string(),
            ));
        }

        // Validate certificate files if provided
        if let Some(ref cert_file) = config.cert_file {
            if !cert_file.exists() {
                return Err(ProxyError::Configuration(format!(
                    "Certificate file not found: {}",
                    cert_file.display()
                )));
            }
        }

        if let Some(ref key_file) = config.key_file {
            if !key_file.exists() {
                return Err(ProxyError::Configuration(format!(
                    "Key file not found: {}",
                    key_file.display()
                )));
            }
        }

        Ok(Self { config })
    }

    async fn connect(&self, target: SocketAddr) -> Result<Box<dyn ProxyConnection>, Self::Error> {
        // In a real implementation, this would:
        // 1. Start a stunnel process with the appropriate configuration
        // 2. Connect to the local stunnel port
        // 3. Return a connection that tunnels through SSL/TLS to the target

        // For now, we'll create a mock connection to the local stunnel port
        let local_addr = format!("{}:{}", self.config.local_host, self.config.local_port);
        let stream = TcpStream::connect(&local_addr)
            .await
            .map_err(|e| ProxyError::Connection(format!("Failed to connect to stunnel: {}", e)))?;

        let local_addr = stream.local_addr().map_err(|e| ProxyError::Io(e))?;

        let connection = StunnelConnection {
            stream,
            local_addr,
            remote_addr: target,
        };

        Ok(Box::new(connection))
    }

    fn info(&self) -> ProxyInfo {
        ProxyInfo {
            name: "Stunnel Proxy".to_string(),
            proxy_type: ProxyType::Stunnel,
            version: "5.0".to_string(),
            capabilities: vec![
                "ssl_encryption".to_string(),
                "tls_encryption".to_string(),
                "certificate_auth".to_string(),
                format!("protocol_{}", self.config.protocol_version),
            ],
        }
    }
}

impl StunnelProxy {
    /// Create stunnel proxy with SSL server configuration
    pub fn new_server(
        local_port: u16,
        remote_host: &str,
        remote_port: u16,
        cert_file: PathBuf,
        key_file: PathBuf,
    ) -> Result<Self, ProxyError> {
        let config = StunnelConfig {
            local_port,
            remote_host: remote_host.to_string(),
            remote_port,
            cert_file: Some(cert_file),
            key_file: Some(key_file),
            verify_peer: false,
            ..Default::default()
        };

        Self::new(config)
    }

    /// Create stunnel proxy with SSL client configuration
    pub fn new_client(
        local_port: u16,
        remote_host: &str,
        remote_port: u16,
        ca_file: Option<PathBuf>,
    ) -> Result<Self, ProxyError> {
        let config = StunnelConfig {
            local_port,
            remote_host: remote_host.to_string(),
            remote_port,
            ca_file,
            verify_peer: true,
            ..Default::default()
        };

        Self::new(config)
    }

    /// Set TLS protocol version
    pub fn set_protocol_version(&mut self, version: TlsVersion) {
        self.config.protocol_version = version;
    }

    /// Set cipher suites
    pub fn set_ciphers(&mut self, ciphers: Option<String>) {
        self.config.ciphers = ciphers;
    }

    /// Enable or disable peer verification
    pub fn set_verify_peer(&mut self, verify: bool) {
        self.config.verify_peer = verify;
    }

    /// Generate stunnel configuration file content
    pub fn generate_config(&self) -> String {
        let mut config = String::new();

        config.push_str(&format!("pid = /var/run/stunnel.pid\n"));
        config.push_str(&format!("debug = 7\n"));
        config.push_str(&format!("output = /var/log/stunnel.log\n\n"));

        config.push_str(&format!("[x11-tunnel]\n"));
        config.push_str(&format!(
            "accept = {}:{}\n",
            self.config.local_host, self.config.local_port
        ));
        config.push_str(&format!(
            "connect = {}:{}\n",
            self.config.remote_host, self.config.remote_port
        ));

        if let Some(ref cert_file) = self.config.cert_file {
            config.push_str(&format!("cert = {}\n", cert_file.display()));
        }

        if let Some(ref key_file) = self.config.key_file {
            config.push_str(&format!("key = {}\n", key_file.display()));
        }

        if let Some(ref ca_file) = self.config.ca_file {
            config.push_str(&format!("CAfile = {}\n", ca_file.display()));
        }

        if self.config.verify_peer {
            config.push_str("verify = 2\n");
        } else {
            config.push_str("verify = 0\n");
        }

        config.push_str(&format!("sslVersion = {}\n", self.config.protocol_version));

        if let Some(ref ciphers) = self.config.ciphers {
            config.push_str(&format!("ciphers = {}\n", ciphers));
        }

        if self.config.compression {
            config.push_str("compression = zlib\n");
        }

        config
    }
}

#[async_trait::async_trait]
impl ProxyConnection for StunnelConnection {
    fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        Ok(self.local_addr)
    }

    fn remote_addr(&self) -> Result<SocketAddr, std::io::Error> {
        Ok(self.remote_addr)
    }

    async fn close(&mut self) -> Result<(), std::io::Error> {
        // In a real implementation, this might also signal stunnel to close
        Ok(())
    }
}

impl AsyncRead for StunnelConnection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for StunnelConnection {
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
    fn test_stunnel_config_creation() {
        let config = StunnelConfig::default();
        assert_eq!(config.local_host, "localhost");
        assert_eq!(config.protocol_version, TlsVersion::TlsV1_3);
        assert!(config.verify_peer);
    }

    #[test]
    fn test_tls_version_display() {
        assert_eq!(TlsVersion::TlsV1_2.to_string(), "TLSv1.2");
        assert_eq!(TlsVersion::TlsV1_3.to_string(), "TLSv1.3");
        assert_eq!(TlsVersion::All.to_string(), "all");
    }
}
