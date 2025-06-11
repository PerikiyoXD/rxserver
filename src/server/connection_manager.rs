/*!
 * Connection Manager for X11 Server
 *
 * Handles network connections (TCP and Unix domain sockets) and manages
 * the lifecycle of client connections with comprehensive logging.
 */

use crate::{config::ServerConfig, Result};
use crate::logging::NetworkLogger;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
#[cfg(unix)]
use tokio::net::UnixListener;
use tracing::{info, warn, error, debug};

/// Represents a new client connection
pub struct NewConnection {
    pub stream: TcpStream,
    pub addr: SocketAddr,
}

/// Manages network connections to the X server
pub struct ConnectionManager {
    tcp_listener: Option<TcpListener>,
    #[cfg(unix)]
    unix_listener: Option<UnixListener>,
    display_num: u8,
    tcp_port: u16,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub async fn new(config: &ServerConfig) -> Result<Self> {
        let display_num = config.server.display_number;
        let tcp_port = 6000 + display_num as u16;

        info!(
            "Initializing ConnectionManager for display :{}",
            display_num
        );

        Ok(ConnectionManager {
            tcp_listener: None,
            #[cfg(unix)]
            unix_listener: None,
            display_num,
            tcp_port,
        })
    }

    /// Start listening for connections
    pub async fn start_listening(&mut self) -> Result<()> {
        info!("Starting connection listeners for display :{}", self.display_num);

        // Set up TCP listener
        self.setup_tcp_listener().await?;

        // Set up Unix domain socket listener (Unix only)
        #[cfg(unix)]
        self.setup_unix_listener().await?;

        Ok(())
    }

    /// Set up TCP listener
    async fn setup_tcp_listener(&mut self) -> Result<()> {
        let addr = format!("127.0.0.1:{}", self.tcp_port);
        
        match TcpListener::bind(&addr).await {
            Ok(listener) => {
                NetworkLogger::listener_started(&addr);
                info!("X server TCP listener bound to {}", addr);
                self.tcp_listener = Some(listener);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to bind TCP listener to {}: {}", addr, e);
                NetworkLogger::listener_error(&addr, &e);
                error!("{}", error_msg);
                Err(crate::Error::Io(e))
            }
        }
    }

    /// Set up Unix domain socket listener (Unix only)
    #[cfg(unix)]
    async fn setup_unix_listener(&mut self) -> Result<()> {
        let socket_path = format!("/tmp/.X11-unix/X{}", self.display_num);
        
        // Create directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&socket_path).parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                warn!("Failed to create X11 socket directory: {}", e);
            }
        }

        // Remove existing socket file if it exists
        let _ = tokio::fs::remove_file(&socket_path).await;

        match UnixListener::bind(&socket_path) {
            Ok(listener) => {
                NetworkLogger::listener_started(&socket_path);
                info!("X server Unix socket listener bound to {}", socket_path);
                self.unix_listener = Some(listener);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to bind Unix socket to {}: {}", socket_path, e);
                NetworkLogger::listener_error(&socket_path, &e);
                warn!("{}", error_msg);
                // Unix socket failure is not fatal, we can still use TCP
                Ok(())
            }
        }
    }    /// Accept the next incoming connection
    pub async fn accept_connection(&self) -> Result<NewConnection> {
        loop {
            // On Unix systems, accept both TCP and Unix socket connections
            #[cfg(unix)]
            {
                tokio::select! {
                    // Accept TCP connections
                    result = async {
                        if let Some(ref listener) = self.tcp_listener {
                            listener.accept().await
                        } else {
                            // Sleep forever if no TCP listener
                            futures_util::future::pending().await
                        }
                    } => {
                        match result {
                            Ok((stream, addr)) => {
                                debug!("Accepted TCP connection from: {}", addr);
                                return Ok(NewConnection { stream, addr });
                            }
                            Err(e) => {
                                NetworkLogger::listener_error("TCP", &e);
                                warn!("Failed to accept TCP connection: {}", e);
                                // Continue loop to try again
                            }
                        }
                    }

                    // Accept Unix socket connections
                    result = async {
                        if let Some(ref listener) = self.unix_listener {
                            listener.accept().await
                        } else {
                            // Sleep forever if no Unix listener
                            futures_util::future::pending().await
                        }
                    } => {
                        match result {
                            Ok((stream, _addr)) => {
                                debug!("Accepted Unix socket connection");
                                // Convert UnixStream to TcpStream-like interface
                                // For now, we'll need to handle this case differently
                                // This is a TODO for proper Unix socket support
                                warn!("Unix socket connections not fully implemented yet");
                                continue;
                            }
                            Err(e) => {
                                NetworkLogger::listener_error("Unix", &e);
                                warn!("Failed to accept Unix socket connection: {}", e);
                                // Continue loop to try again
                            }
                        }
                    }
                }
            }

            // On non-Unix systems, only accept TCP connections
            #[cfg(not(unix))]
            {
                if let Some(ref listener) = self.tcp_listener {
                    match listener.accept().await {
                        Ok((stream, addr)) => {
                            debug!("Accepted TCP connection from: {}", addr);
                            return Ok(NewConnection { stream, addr });
                        }
                        Err(e) => {
                            NetworkLogger::listener_error("TCP", &e);
                            warn!("Failed to accept TCP connection: {}", e);
                            // Continue loop to try again
                        }
                    }
                } else {
                    // No listener available, wait a bit and try again
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Get the display number
    pub fn display_num(&self) -> u8 {
        self.display_num
    }

    /// Get the TCP port number
    pub fn tcp_port(&self) -> u16 {
        self.tcp_port
    }

    /// Check if TCP listener is active
    pub fn has_tcp_listener(&self) -> bool {
        self.tcp_listener.is_some()
    }

    /// Check if Unix listener is active
    #[cfg(unix)]
    pub fn has_unix_listener(&self) -> bool {
        self.unix_listener.is_some()
    }

    #[cfg(not(unix))]
    pub fn has_unix_listener(&self) -> bool {
        false
    }
}
