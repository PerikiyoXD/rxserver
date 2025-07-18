// mod.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

use crate::server::state::Server;

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

#[derive(Debug, Clone)]
pub struct ConnectionEvent {
    pub client_addr: String,
    pub transport_kind: TransportKind,
}

#[derive(Debug)]
pub enum TransportMessage {
    ConnectionAccepted(ConnectionEvent),
    Error(String),
    Shutdown,
}

#[allow(async_fn_in_trait)]
pub trait TransportContract: Send + Sync {
    async fn start(&self) -> Result<()>;
    fn stop(&self);
    fn transport_kind(&self) -> TransportKind;
}

pub enum Transport {
    Tcp(tcp::TcpTransport),
    #[cfg(unix)]
    Unix(unix::UnixTransport),
}

impl Transport {
    pub async fn new(
        transport_type: TransportKind,
        addr: &str,
        server_state: Arc<Mutex<Server>>,
        tx: mpsc::UnboundedSender<TransportMessage>,
    ) -> Result<Self> {
        match transport_type {
            TransportKind::Tcp => Ok(Self::Tcp(
                tcp::TcpTransport::new(addr, server_state, tx).await?,
            )),
            #[cfg(unix)]
            TransportKind::Unix => Ok(Self::Unix(
                unix::UnixTransport::new(addr, server_state, tx).await?,
            )),
        }
    }

    pub async fn start(&self) -> Result<()> {
        match self {
            Self::Tcp(transport) => transport.start().await,
            #[cfg(unix)]
            Self::Unix(transport) => transport.start().await,
        }
    }

    pub fn stop(&self) {
        match self {
            Self::Tcp(transport) => transport.stop(),
            #[cfg(unix)]
            Self::Unix(transport) => transport.stop(),
        }
    }

    pub fn transport_kind(&self) -> TransportKind {
        match self {
            Self::Tcp(transport) => transport.transport_kind(),
            #[cfg(unix)]
            Self::Unix(transport) => transport.transport_kind(),
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
