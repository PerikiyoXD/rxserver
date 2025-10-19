// main.rs
use anyhow::{Context, Result};
use rxserver::{
    logging::init_logging,
    server::{RX11Server, config::load_config},
};

fn main() -> Result<()> {
    // Create a custom tokio runtime with named threads
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4) // Adjust based on your needs
        .thread_name("rxserver-worker")
        .thread_stack_size(3 * 1024 * 1024) // 3MB stack size
        .enable_all()
        .build()
        .context("Failed to create tokio runtime")?;

    rt.block_on(async_main())
}

async fn async_main() -> Result<()> {
    let config = load_config(None).context("Failed to load server configuration")?;
    let logging = config.logging.clone();
    init_logging(logging).context("Failed to initialize logging")?;

    let server = RX11Server::new(config).context("Failed to create X11 server")?;
    server.run().await.context("Failed to run X11 server")?;

    Ok(())
}
