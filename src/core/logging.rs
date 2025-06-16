use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

use crate::{core::LoggingConfig, ServerError, ServerResult};

/// Initialize logging based on configuration (or defaults if None)
pub fn init_logging(config: Option<&LoggingConfig>) -> ServerResult<()> {
    let (level, colored, json, file) = match config {
        Some(cfg) => (&cfg.level, cfg.colored, cfg.json, cfg.file.as_ref()),
        None => (&"info".to_string(), true, false, None),
    };

    let env_filter = EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("info"));

    let console_layer = create_console_layer(json, colored);

    if let Some(file_path) = file {
        let file_layer = create_file_layer(file_path)?;
        tracing_subscriber::registry()
            .with(env_filter)
            .with(console_layer)
            .with(file_layer)
            .try_init()
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(console_layer)
            .try_init()
    }
    .map_err(|e| ServerError::LoggingError(format!("Failed to initialize logging: {}", e)))?;

    tracing::info!("Logging initialized with level: {}", level);
    Ok(())
}

fn create_console_layer<S>(json: bool, colored: bool) -> Box<dyn Layer<S> + Send + Sync>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    if json {
        Box::new(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::CLOSE)
                .with_target(false)
                .compact(),
        )
    } else {
        Box::new(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(true)
                .with_line_number(true)
                .with_ansi(colored)
                .compact(),
        )
    }
}

fn create_file_layer<S>(file_path: &str) -> ServerResult<Box<dyn Layer<S> + Send + Sync>>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .map_err(|e| ServerError::LoggingError(format!("Failed to open log file: {}", e)))?;

    Ok(Box::new(
        tracing_subscriber::fmt::layer()
            .with_writer(file)
            .with_ansi(false)
            .compact(),
    ))
}
