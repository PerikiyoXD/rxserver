use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::transport::tcp::TcpSocketTransport;

pub mod tcp;
#[cfg(unix)]
pub mod unix;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum TransportKind {
    Tcp,
    #[cfg(unix)]
    Unix,
}

pub enum Transport {
    TcpSocket(TcpSocketTransport),
    #[cfg(unix)]
    UnixSocket(UnixSocketTransport),
}

pub trait TransportTrait: Sized {
    async fn new(addr: &str) -> Result<Self>;
    async fn run(&self) -> Result<()>;
}

impl Transport {
    pub async fn new(transport_type: TransportKind, addr: &str) -> Result<Self> {
        match transport_type {
            TransportKind::Tcp => Ok(Self::TcpSocket(TcpSocketTransport::new(addr).await?)),
            #[cfg(unix)]
            TransportKind::Unix => Ok(Self::UnixSocket(UnixSocketTransport::new(addr).await?)),
        }
    }

    pub async fn run(&self) -> Result<()> {
        match self {
            Self::TcpSocket(tcp) => tcp.run().await,
            #[cfg(unix)]
            Self::UnixSocket(unix) => unix.run().await,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransportInfo {
    pub kind: TransportKind,
    pub address: usize,
}

impl TransportInfo {
    pub fn new(kind: TransportKind, bind_index: usize) -> Self {
        Self {
            kind,
            address: bind_index,
        }
    }
}
