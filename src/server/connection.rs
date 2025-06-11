//! Client connection management
//!
//! This module provides simple data structures for representing client connections.
//! Connection handling logic is implemented in dedicated handler modules.

use std::net::SocketAddr;
use tokio::net::TcpStream;

/// Represents a new connection from the connection manager
pub struct NewConnection {
    pub stream: TcpStream,
    pub addr: SocketAddr,
}
