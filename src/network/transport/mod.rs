//! Transport layer implementations
//!
//! This module provides concrete implementations of various transport protocols
//! for the X11 server network layer.

pub mod tcp;
pub mod traits;
pub mod unix_socket;

#[cfg(windows)]
pub mod named_pipe;

pub mod shared_memory;

// Re-export commonly used items
pub use tcp::TcpTransport;
pub use traits::{
    ConnectionMetadata, Endpoint, Transport, TransportConfig, TransportConnection, TransportError,
    TransportEvent, TransportFactory, TransportStatistics, TransportType,
};
#[cfg(unix)]
pub use unix_socket::UnixSocketTransport;
