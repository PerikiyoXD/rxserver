//! Command line argument configuration source

use crate::config::sources::ConfigSource;
use crate::config::types::*;
use crate::types::Result;
use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};
use std::collections::HashMap;

/// Command line argument configuration source
pub struct CommandLineSource {
    args: Option<ArgMatches>,
    priority: u32,
}

impl CommandLineSource {
    /// Create a new command line source
    pub fn new() -> Self {
        Self {
            args: None,
            priority: 300, // Highest priority
        }
    }

    /// Create with pre-parsed arguments
    pub fn with_args(args: ArgMatches) -> Self {
        Self {
            args: Some(args),
            priority: 300,
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Parse command line arguments if not already parsed
    fn ensure_parsed(&mut self) -> Result<()> {
        if self.args.is_none() {
            let app = self.build_cli_app();
            self.args = Some(app.try_get_matches()?);
        }
        Ok(())
    }

    /// Build the CLI application definition
    fn build_cli_app(&self) -> Command {
        Command::new("rxserver")
            .about("RXServer - Modern X11 Server Implementation")
            .version(env!("CARGO_PKG_VERSION"))
            // Configuration file
            .arg(
                Arg::new("config")
                    .short('c')
                    .long("config")
                    .value_name("FILE")
                    .help("Configuration file path"),
            )
            // Server options
            .arg(
                Arg::new("display")
                    .short('d')
                    .long("display")
                    .value_name("NUMBER")
                    .help("Display number"),
            )
            .arg(
                Arg::new("screen-count")
                    .long("screen-count")
                    .value_name("COUNT")
                    .help("Number of screens"),
            )
            // Network options
            .arg(
                Arg::new("bind-address")
                    .short('b')
                    .long("bind-address")
                    .value_name("ADDRESS")
                    .help("TCP bind address"),
            )
            .arg(
                Arg::new("bind-port")
                    .short('p')
                    .long("bind-port")
                    .value_name("PORT")
                    .help("TCP bind port"),
            )
            .arg(
                Arg::new("unix-socket")
                    .short('s')
                    .long("unix-socket")
                    .value_name("PATH")
                    .help("Unix socket path"),
            )
            .arg(
                Arg::new("max-connections")
                    .long("max-connections")
                    .value_name("COUNT")
                    .help("Maximum number of connections"),
            )
            // Display options
            .arg(
                Arg::new("resolution")
                    .short('r')
                    .long("resolution")
                    .value_name("WIDTHxHEIGHT")
                    .help("Screen resolution (e.g., 1920x1080)"),
            )
            .arg(
                Arg::new("color-depth")
                    .long("color-depth")
                    .value_name("BITS")
                    .help("Color depth in bits"),
            )
            .arg(
                Arg::new("dpi")
                    .long("dpi")
                    .value_name("DPI")
                    .help("Dots per inch"),
            )
            .arg(
                Arg::new("refresh-rate")
                    .long("refresh-rate")
                    .value_name("HZ")
                    .help("Refresh rate in Hz"),
            )
            .arg(
                Arg::new("backend")
                    .long("backend")
                    .value_name("NAME")
                    .help("Display backend (software, opengl, vulkan, etc.)"),
            )
            // Logging options
            .arg(
                Arg::new("log-level")
                    .short('l')
                    .long("log-level")
                    .value_name("LEVEL")
                    .help("Log level (error, warn, info, debug, trace)"),
            )
            .arg(
                Arg::new("log-file")
                    .long("log-file")
                    .value_name("PATH")
                    .help("Log file path"),
            )
            .arg(
                Arg::new("structured-logs")
                    .long("structured-logs")
                    .action(clap::ArgAction::SetTrue)
                    .help("Enable structured logging"),
            )
            // Performance options
            .arg(
                Arg::new("threads")
                    .short('t')
                    .long("threads")
                    .value_name("COUNT")
                    .help("Thread pool size"),
            )
            .arg(
                Arg::new("request-queue-size")
                    .long("request-queue-size")
                    .value_name("SIZE")
                    .help("Request queue size"),
            )
            .arg(
                Arg::new("event-queue-size")
                    .long("event-queue-size")
                    .value_name("SIZE")
                    .help("Event queue size"),
            )
            // Feature toggles
            .arg(
                Arg::new("no-extensions")
                    .long("no-extensions")
                    .action(clap::ArgAction::SetTrue)
                    .help("Disable X11 extensions"),
            )
            .arg(
                Arg::new("enable-compositing")
                    .long("enable-compositing")
                    .action(clap::ArgAction::SetTrue)
                    .help("Enable compositing"),
            )
            .arg(
                Arg::new("no-damage-tracking")
                    .long("no-damage-tracking")
                    .action(clap::ArgAction::SetTrue)
                    .help("Disable damage tracking"),
            )
            .arg(
                Arg::new("debug")
                    .long("debug")
                    .action(clap::ArgAction::SetTrue)
                    .help("Enable debug features"),
            )
            .arg(
                Arg::new("performance-monitoring")
                    .long("performance-monitoring")
                    .action(clap::ArgAction::SetTrue)
                    .help("Enable performance monitoring"),
            )
            // Security options
            .arg(
                Arg::new("no-access-control")
                    .long("no-access-control")
                    .action(clap::ArgAction::SetTrue)
                    .help("Disable access control"),
            )
            .arg(
                Arg::new("audit")
                    .long("audit")
                    .action(clap::ArgAction::SetTrue)
                    .help("Enable audit logging"),
            )
            .arg(
                Arg::new("audit-log")
                    .long("audit-log")
                    .value_name("PATH")
                    .help("Audit log file path"),
            )
            // Development options
            .arg(
                Arg::new("preset").long("preset").value_name("NAME").help(
                    "Configuration preset (development, production, embedded, high-performance)",
                ),
            )
            .arg(
                Arg::new("daemonize")
                    .long("daemonize")
                    .action(clap::ArgAction::SetTrue)
                    .help("Run as daemon"),
            )
            .arg(
                Arg::new("pid-file")
                    .long("pid-file")
                    .value_name("PATH")
                    .help("PID file path"),
            )
    }

    /// Convert command line arguments to configuration
    fn args_to_config(&self, args: &ArgMatches) -> ServerConfig {
        let mut config = ServerConfig::default();

        // Server configuration
        if let Some(display) = args.get_one::<String>("display") {
            if let Ok(num) = display.parse() {
                config.server.display_number = num;
            }
        }

        if let Some(screen_count) = args.get_one::<String>("screen-count") {
            if let Ok(count) = screen_count.parse() {
                config.server.screen_count = count;
            }
        }

        // Network configuration
        if let Some(bind_address) = args.get_one::<String>("bind-address") {
            if let Some(bind_port) = args.get_one::<String>("bind-port") {
                if let Ok(addr) = format!("{}:{}", bind_address, bind_port).parse() {
                    config.network.tcp_addresses = vec![addr];
                }
            } else if let Ok(addr) = bind_address.parse() {
                config.network.tcp_addresses = vec![addr];
            }
        } else if let Some(bind_port) = args.get_one::<String>("bind-port") {
            if let Ok(port) = bind_port.parse::<u16>() {
                if let Ok(addr) = format!("127.0.0.1:{}", port).parse() {
                    config.network.tcp_addresses = vec![addr];
                }
            }
        }

        if let Some(unix_socket) = args.get_one::<String>("unix-socket") {
            config.network.unix_sockets = vec![unix_socket.into()];
        }

        if let Some(max_connections) = args.get_one::<String>("max-connections") {
            if let Ok(count) = max_connections.parse() {
                config.network.max_connections = count;
            }
        }

        // Display configuration
        if let Some(resolution) = args.get_one::<String>("resolution") {
            if let Some((width, height)) = self.parse_resolution(resolution) {
                config.display.default_resolution = Resolution { width, height };
            }
        }

        if let Some(color_depth) = args.get_one::<String>("color-depth") {
            if let Ok(depth) = color_depth.parse() {
                config.display.color_depth = depth;
            }
        }

        if let Some(dpi) = args.get_one::<String>("dpi") {
            if let Ok(dpi_val) = dpi.parse() {
                config.display.dpi = dpi_val;
            }
        }

        if let Some(refresh_rate) = args.get_one::<String>("refresh-rate") {
            if let Ok(rate) = refresh_rate.parse() {
                config.display.refresh_rate = rate;
            }
        }

        if let Some(backend) = args.get_one::<String>("backend") {
            config.display.backend = backend.clone();
        }

        // Logging configuration
        if let Some(log_level) = args.get_one::<String>("log-level") {
            config.logging.level = log_level.clone();
        }

        if args.get_flag("structured-logs") {
            config.logging.structured = true;
        }

        if let Some(log_file) = args.get_one::<String>("log-file") {
            config.logging.outputs = vec![LogOutput {
                output_type: "file".to_string(),
                config: {
                    let mut map = HashMap::new();
                    map.insert(
                        "path".to_string(),
                        serde_json::Value::String(log_file.clone()),
                    );
                    map
                },
            }];
        }

        // Performance configuration
        if let Some(threads) = args.get_one::<String>("threads") {
            if let Ok(count) = threads.parse() {
                config.performance.thread_pool_size = Some(count);
            }
        }

        if let Some(request_queue_size) = args.get_one::<String>("request-queue-size") {
            if let Ok(size) = request_queue_size.parse() {
                config.performance.request_queue_size = size;
            }
        }

        if let Some(event_queue_size) = args.get_one::<String>("event-queue-size") {
            if let Ok(size) = event_queue_size.parse() {
                config.performance.event_queue_size = size;
            }
        }

        // Feature toggles
        if args.get_flag("no-extensions") {
            config.features.extensions_enabled = false;
        }

        if args.get_flag("enable-compositing") {
            config.features.compositing_enabled = true;
        }

        if args.get_flag("no-damage-tracking") {
            config.features.damage_tracking_enabled = false;
        }

        if args.get_flag("debug") {
            config.features.debug_features_enabled = true;
        }

        if args.get_flag("performance-monitoring") {
            config.features.performance_monitoring_enabled = true;
        }

        // Security configuration
        if args.get_flag("no-access-control") {
            config.security.access_control_enabled = false;
        }

        if args.get_flag("audit") {
            config.security.audit_enabled = true;
        }

        if let Some(audit_log) = args.get_one::<String>("audit-log") {
            config.security.audit_log_path = Some(audit_log.into());
        }

        config
    }

    /// Parse resolution string (e.g., "1920x1080")
    fn parse_resolution(&self, resolution: &str) -> Option<(u32, u32)> {
        let parts: Vec<&str> = resolution.split('x').collect();
        if parts.len() == 2 {
            if let (Ok(width), Ok(height)) = (parts[0].parse(), parts[1].parse()) {
                return Some((width, height));
            }
        }
        None
    }

    /// Get configuration file path from arguments
    pub fn get_config_file_path(&self) -> Option<String> {
        self.args.as_ref()?.get_one::<String>("config").cloned()
    }

    /// Get preset name from arguments
    pub fn get_preset(&self) -> Option<String> {
        self.args.as_ref()?.get_one::<String>("preset").cloned()
    }

    /// Check if server should run as daemon
    pub fn should_daemonize(&self) -> bool {
        self.args
            .as_ref()
            .map(|args| args.get_flag("daemonize"))
            .unwrap_or(false)
    }

    /// Get PID file path
    pub fn get_pid_file_path(&self) -> Option<String> {
        self.args.as_ref()?.get_one::<String>("pid-file").cloned()
    }
}

impl Default for CommandLineSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConfigSource for CommandLineSource {
    async fn load(&self) -> Result<ServerConfig> {
        // For this implementation, we'll return an empty config
        // since command line parsing typically happens at startup
        // and the actual args would be provided via with_args()

        if let Some(args) = &self.args {
            Ok(self.args_to_config(args))
        } else {
            Ok(ServerConfig::default())
        }
    }

    fn identifier(&self) -> String {
        "command-line".to_string()
    }

    fn priority(&self) -> u32 {
        self.priority
    }
}

/// Command line parsing helper
pub struct CliParser;

impl CliParser {
    /// Parse command line arguments and return source
    pub fn parse() -> Result<CommandLineSource> {
        let app = CommandLineSource::new().build_cli_app();
        let matches = app.try_get_matches()?;
        Ok(CommandLineSource::with_args(matches))
    }

    /// Parse specific arguments
    pub fn parse_args<I, T>(args: I) -> Result<CommandLineSource>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let app = CommandLineSource::new().build_cli_app();
        let matches = app.try_get_matches_from(args)?;
        Ok(CommandLineSource::with_args(matches))
    }

    /// Get help text
    pub fn help_text() -> String {
        CommandLineSource::new()
            .build_cli_app()
            .render_help()
            .to_string()
    }

    /// Get version information
    pub fn version_info() -> String {
        format!("RXServer {}", env!("CARGO_PKG_VERSION"))
    }
}
