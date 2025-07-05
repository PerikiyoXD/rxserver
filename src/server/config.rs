use tracing::Level;

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
