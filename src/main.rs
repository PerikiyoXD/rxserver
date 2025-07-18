// main.rs
use anyhow::{Context, Result};
use rxserver::{
    logging::init_logging,
    server::{RX11Server, config::load_config},
};

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_config(None).context("Failed to load server configuration")?;
    let logging = config.logging.clone();
    init_logging(logging).context("Failed to initialize logging")?;

    let server = RX11Server::new(config).context("Failed to create X11 server")?;
    server.run().await.context("Failed to run X11 server")?;

    Ok(())
}
