//! Integrated X11 server binary invoking the library

use anyhow::{Context, Result};
use rxserver::{
    LoggingConfig, init_logging,
    logging::init_logging,
    server::{RX11Server, ServerConfig, config::load_config},
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let config: ServerConfig = load_config(None).context("Failed to load server configuration")?;
    init_logging(config.logging).context("Failed to initialize logging")?;
    let mut server = RX11Server::new().context("Failed to create X11 server")?;
    server
        .initialize(config)
        .await
        .context("Failed to initialize X11 server")?;
    server.run().await.context("Failed to run X11 server")?;

    Ok(())
}
