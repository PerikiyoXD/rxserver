//! X11 protocol server library

pub mod protocol;
pub mod server;

pub use protocol::*;
pub use server::*;

use anyhow::{Context, Result};
use tracing::Level;
use tracing_subscriber::{
    EnvFilter, Registry,
    fmt::{self, time::ChronoLocal},
    prelude::*,
};

/// Fine-grained logging configuration.
#[derive(Debug)]
pub struct LoggingConfig {
    /// Fallback log level if `RUST_LOG` is missing or invalid.
    pub default_level: Level,
    /// Enable ANSI colors?
    pub ansi: bool,
    /// Show thread IDs?
    pub show_thread_ids: bool,
    /// Show thread names?
    pub show_thread_names: bool,
    /// Show file and line number?
    pub show_file_location: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            default_level: Level::TRACE,
            ansi: true,
            show_thread_ids: true,
            show_thread_names: true,
            show_file_location: true,
        }
    }
}

/// Initialize tracing with a formatted subscriber and env-overrideable filter.
///
/// # Errors
/// Fails only if the subscriber can’t be initialized for reasons _other_
/// than “already set.”
pub fn init_logging(cfg: LoggingConfig) -> Result<()> {
    // 1) Try to read RUST_LOG, otherwise fall back to our default_level
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(cfg.default_level.to_string()));

    // 2) Build a formatting layer
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(cfg.show_thread_ids)
        .with_thread_names(cfg.show_thread_names)
        .with_level(true)
        .with_ansi(cfg.ansi)
        .with_timer(ChronoLocal::rfc_3339())
        .with_file(cfg.show_file_location)
        .with_line_number(cfg.show_file_location);

    // 3) Compose registry + layers
    let subscriber = Registry::default().with(filter).with(fmt_layer);

    // 4) Try init; ignore “already set” errors
    if let Err(e) = subscriber.try_init() {
        let msg = e.to_string();
        if !msg.contains("already set") {
            return Err(e).context("failed to initialize tracing subscriber");
        }
    }

    Ok(())
}
