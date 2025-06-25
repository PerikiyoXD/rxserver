//! Integrated X11 server binary invoking the library

use anyhow::{Context, Result};
use rxserver::{LoggingConfig, init_logging, server::tcp::run};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging(LoggingConfig::default()).context("Failed to initialize logging")?;
    info!("Starting X11 RxServer...");
    run("127.0.0.1:6000").await?;
    Ok(())
}
