//! Integrated X11 server binary invoking the library

use anyhow::Result;
use rxserver::{init_logging, server::tcp::run};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging(None)?;
    info!("Starting X11 RxServer...");
    run("127.0.0.1:6000").await?;
    Ok(())
}
