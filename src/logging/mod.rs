use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Configuration for the logging behavior of the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub default_level: LoggingLevel,
    pub show_thread_ids: bool,
    pub show_thread_names: bool,
    pub ansi: bool,
    pub show_file_location: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            default_level: LoggingLevel(Level::TRACE),
            show_thread_ids: true,
            show_thread_names: true,
            ansi: true,
            show_file_location: true,
        }
    }
}

/// Wrapper around `tracing::Level` that enables (de)serialization.
#[derive(Debug, Clone)]
pub struct LoggingLevel(pub Level);

impl<'de> Deserialize<'de> for LoggingLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<Level>()
            .map(LoggingLevel)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for LoggingLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.as_str().to_lowercase())
    }
}

/// Initializes the tracing subscriber with a formatted layer and env-filter.
///
/// This supports runtime log-level overrides via `RUST_LOG`.
///
/// # Arguments
/// * `cfg` - Logging configuration parameters.
///
/// # Returns
/// * `Result<()>` - Ok if initialization succeeded or was already set, otherwise error.
///
/// # Example
/// ```
/// use rxserver::{init_logging, LoggingConfig};
/// use tracing::Level;
///
/// let cfg = LoggingConfig {
///     default_level: Level::INFO.into(),
///     show_thread_ids: true,
///     show_thread_names: true,
///     ansi: true,
///     show_file_location: true,
/// };
///
/// init_logging(cfg).expect("Failed to initialize logging");
/// ```
pub fn init_logging(cfg: LoggingConfig) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(cfg.default_level.0.as_str()));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(cfg.show_thread_ids)
        .with_thread_names(cfg.show_thread_names)
        .with_level(true)
        .with_ansi(cfg.ansi)
        .with_file(cfg.show_file_location)
        .with_line_number(cfg.show_file_location);

    let subscriber = Registry::default().with(filter).with(fmt_layer);

    subscriber.try_init().or_else(|e| {
        if e.to_string().contains("already set") {
            Ok(())
        } else {
            Err(e).context("failed to initialize tracing subscriber")
        }
    })
}
