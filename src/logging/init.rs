//! Logging initialization and configuration
//!
//! This module handles the setup and initialization of the unified logging system
//! using the existing LoggingSettings configuration with full file logging support.

use crate::config::types::LoggingSettings;

use std::path::Path;
use std::sync::Once;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static INIT: Once = Once::new();

/// Initialize the centralized logging system using the existing LoggingSettings
pub fn init_logging(config: &LoggingSettings) -> Result<(), Box<dyn std::error::Error>> {
    INIT.call_once(|| {
        // Set the default log level based on the configuration
        let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            let level_str = match config.level {
                crate::logging::types::LogLevel::Error => "error",
                crate::logging::types::LogLevel::Warn => "warn",
                crate::logging::types::LogLevel::Info => "info",
                crate::logging::types::LogLevel::Debug => "debug",
                crate::logging::types::LogLevel::Trace => "trace",
            };
            EnvFilter::new(level_str)
        });

        // Initialize subscriber based on configuration
        match (&config.stdout, &config.file) {
            // Both console and file logging
            (true, Some(file_path)) => {
                if let Ok(file_appender) = create_file_appender(file_path, config) {
                    tracing_subscriber::registry()
                        .with(env_filter)
                        .with(fmt::layer().compact())
                        .with(fmt::layer().with_writer(file_appender).with_ansi(false))
                        .init();
                } else {
                    // Fall back to console only
                    tracing_subscriber::registry()
                        .with(env_filter)
                        .with(fmt::layer().compact())
                        .init();
                }
            }
            // Console only
            (true, None) => {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(fmt::layer().compact())
                    .init();
            }
            // File only
            (false, Some(file_path)) => {
                if let Ok(file_appender) = create_file_appender(file_path, config) {
                    tracing_subscriber::registry()
                        .with(env_filter)
                        .with(fmt::layer().with_writer(file_appender).with_ansi(false))
                        .init();
                } else {
                    // Fall back to console if file setup fails
                    tracing_subscriber::registry()
                        .with(env_filter)
                        .with(fmt::layer().compact())
                        .init();
                }
            }
            // Default to console if nothing specified
            (false, None) => {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(fmt::layer().compact())
                    .init();
            }
        }

        // Initialize the log bridge to capture legacy log crate usage
        if let Err(_) = tracing_log::LogTracer::init() {
            // LogTracer was already initialized, which is fine
            eprintln!("LogTracer already initialized (this is normal)");
        }
    });

    Ok(())
}

/// Create a file appender with rotation support
fn create_file_appender(
    file_path: &Path,
    config: &LoggingSettings,
) -> Result<RollingFileAppender, Box<dyn std::error::Error>> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Determine rotation strategy based on max file size
    let rotation = if config.max_file_size > 0 {
        // Use daily rotation as approximation (tracing-appender doesn't support size-based rotation directly)
        Rotation::DAILY
    } else {
        Rotation::NEVER
    };

    let file_appender = RollingFileAppender::new(
        rotation,
        file_path.parent().unwrap_or_else(|| Path::new(".")),
        file_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("server.log")),
    );

    Ok(file_appender)
}

/// Log server startup information with standard formatting
pub fn log_startup_info(display_num: u8, config_file: &str) {
    tracing::info!("==========================================");
    tracing::info!("   RX - Rust X Window System Server");
    tracing::info!("   Display: :{}", display_num);
    tracing::info!("   Config: {}", config_file);
    tracing::info!("   PID: {}", std::process::id());
    tracing::info!("==========================================");
}

/// Log server shutdown information with standard formatting
pub fn log_shutdown_info() {
    tracing::info!("==========================================");
    tracing::info!("   RX Server shutting down");
    tracing::info!("==========================================");
}

/// Log system information at startup
pub fn log_system_info() {
    tracing::info!("System Information:");
    tracing::info!("  OS: {}", std::env::consts::OS);
    tracing::info!("  Architecture: {}", std::env::consts::ARCH);
    if let Ok(version) = std::env::var("RUSTC_VERSION") {
        tracing::info!("  Rust version: {}", version);
    }
}
