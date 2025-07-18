// mod.rs - Final clean version
pub mod atom_system;
pub mod client_system;
pub mod config;
pub mod connection;
pub mod display_system;
pub mod resource_system;
pub mod state;
pub mod window_system;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub use config::*;
pub use connection::*;
pub use state::*;

pub struct RX11Server {
    state: Arc<Mutex<Server>>,
}

impl RX11Server {
    pub fn new(config: ServerConfig) -> Result<Self> {
        let state = Server::new(config.displays)?;
        info!("Server initialized successfully");
        Ok(Self { state })
    }

    pub fn state(&self) -> &Arc<Mutex<Server>> {
        &self.state
    }

    pub async fn run(self) -> Result<()> {
        info!("Starting X11 RxServer...");
        
        let display_count = {
            let server = self.state.lock().await;
            server.display_count()
        };
        info!("Server running with {} display(s)", display_count);

        tokio::signal::ctrl_c().await?;

        info!("Received shutdown signal, stopping server...");
        {
            let server = self.state.lock().await;
            server.shutdown().await;
        }
        info!("X11 RxServer stopped");
        Ok(())
    }
}
