//! X11 protocol server library

pub mod protocol;
pub mod server;

pub use protocol::*;
pub use server::*;

use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt};

/// Initialize tracing logging
pub fn init_logging(log_level: Option<&str>) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level.unwrap_or("trace")));

    fmt().with_env_filter(filter).with_target(false).init();

    Ok(())
}
