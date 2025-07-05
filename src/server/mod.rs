pub mod config;
pub mod connection;
pub mod pipeline;
pub mod state;
pub mod tcp;

use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
pub use config::*;
pub use connection::*;
pub use state::*;
pub use tcp::*;
use tokio::sync::Mutex;
use tracing::info;

use crate::{
    display::{
        self,
        config::DisplayConfig,
        create_display,
        types::{Display, DisplayTrait},
    },
    transport::TransportInfo,
};

trait Runnable {
    async fn run(&mut self) -> Result<()>;
}

pub struct RX11Server {
    state: Arc<Mutex<ServerState>>,
}

impl RX11Server {
    /// Create a new RX11 server instance
    pub fn new() -> Self {
        let state: Arc<Mutex<ServerState>> = ServerState::new();
        Self { state }
    }

    pub fn initialize(&mut self, config: ServerConfig) -> Result<()> {
        self.state
            .blocking_lock()
            .initialize(config.clone())
            .map_err(|e| anyhow::anyhow!("Failed to initialize server state: {}", e))?;

        let displays = self
            .setup_displays(config.displays)
            .map_err(|e| anyhow::anyhow!("Failed to setup displays: {}", e))?;

        validate_transport_information(&displays)?;

        let transports = self
            .setup_transports(config.transports)
            .map_err(|e| anyhow::anyhow!("Failed to setup transports: {}", e))?;
        Ok(())
    }

    fn setup_displays(
        &mut self,
        display_configs: Vec<DisplayConfig>,
    ) -> Result<HashMap<TransportInfo, Arc<Mutex<Display>>>> {
        let mut displays: HashMap<TransportInfo, Arc<Mutex<Display>>> = HashMap::new();

        for display_config in display_configs {
            info!("Setting up display: {:?}", display_config);

            let transport = display_config.transport;
            let id = display_config.id;
            let name = display_config.name.clone();

            let display: Arc<Mutex<Display>> =
                create_display(display_config).context("Failed to create display")?;

            {
                let mut display_guard = tokio::runtime::Handle::current().block_on(display.lock());
                display_guard
                    .start()
                    .context("Failed to start virtual display")?;
            }

            info!("Virtual display started successfully");
            let display_transport_info: TransportInfo = TransportInfo::new(transport, id);

            if displays.contains_key(&display_transport_info) {
                info!(
                    "Duplicate TransportInfo detected: {:?} for display '{}'. Skipping.",
                    display_transport_info, name
                );
                continue;
            }

            displays.insert(display_transport_info, display);
        }
        Ok(displays)
    }
}

fn validate_transport_information(
    displays: &HashMap<TransportInfo, Arc<Mutex<Display>>>,
) -> Result<()> {
    for (transport_info, display) in displays {}
    Ok(())
}

impl Runnable for RX11Server {
    /// Run the server
    async fn run(&mut self) -> Result<()> {
        info!("Starting X11 RxServer...");
        Ok(())
    }
}
