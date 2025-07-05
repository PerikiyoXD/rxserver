//! X11 protocol server library

pub mod protocol;
pub mod server;

pub use server::*;

use anyhow::{Context, Result};
use tracing_subscriber::{EnvFilter, Registry, fmt, prelude::*};

/// Initialize tracing with a formatted subscriber and env-overrideable filter.
///
/// # Errors
/// Fails only if the subscriber can’t be initialized for reasons _other_
/// than “already set.”
pub fn init_logging(cfg: LoggingConfig) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(cfg.default_level.to_string()));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(cfg.show_thread_ids)
        .with_thread_names(cfg.show_thread_names)
        .with_level(true)
        .with_ansi(cfg.ansi)
        .with_file(cfg.show_file_location)
        .with_line_number(cfg.show_file_location);

    let subscriber = Registry::default().with(filter).with(fmt_layer);

    if let Err(e) = subscriber.try_init() {
        if !e.to_string().contains("already set") {
            return Err(e).context("failed to initialize tracing subscriber");
        }
    }

    Ok(())
}
