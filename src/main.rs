//! Main entrypoint for the RxServer X11 implementation
//!
//! This is the primary executable entry point for the RxServer.
//! It handles command-line argument parsing, configuration loading,
//! and server initialization with comprehensive health monitoring.

use clap::Parser;
use std::process;
use std::time::Duration;
use tokio::signal;
use tracing::{error, info, warn};

use rxserver::{
    diagnostics::health::{HealthCommand, HealthSeverity},
    display::DisplayConfig,
    input::InputConfiguration,
    logging::{manager::LogManager, types::LogLevel},
    platform::Platform,
    server::{Server, ServerBuilder, configuration::ServerConfig},
};

/// Command line arguments for the RxServer
#[derive(Parser, Debug)]
#[command(
    name = "rxserver",
    about = "A modern X11 server implementation in Rust",
    version = env!("CARGO_PKG_VERSION"),
    author = "RxServer Contributors"
)]
struct Args {
    /// Display number to use (e.g., :0, :1)
    #[arg(short = 'D', long, default_value = ":0")]
    display: String,

    /// Configuration file path
    #[arg(short, long, default_value = "rxserver.toml")]
    config: String,

    /// Display mode (headless, virtual, native)
    #[arg(short, long, default_value = "virtual")]
    mode: String,

    /// Display width for virtual/native modes
    #[arg(long, default_value = "1920")]
    width: u32,

    /// Display height for virtual/native modes
    #[arg(long, default_value = "1080")]
    height: u32,
    /// Reduce logging verbosity (default: TRACE, -v: DEBUG, -vv: INFO, -vvv: WARN, -vvvv: ERROR)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Run in daemon mode
    #[arg(short, long)]
    daemon: bool,

    /// Health check interval in seconds
    #[arg(long, default_value = "30")]
    health_interval: u64,

    /// Enable health monitoring
    #[arg(long, default_value = "true")]
    enable_health: bool,

    /// Pidfile location (daemon mode only)
    #[arg(long)]
    pidfile: Option<String>,
}

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging first
    if let Err(e) = init_logging(&args) {
        eprintln!("Failed to initialize logging: {}", e);
        process::exit(1);
    }

    info!(
        "Starting RxServer X11 implementation v{}",
        env!("CARGO_PKG_VERSION")
    );
    info!(
        "Display: {}, Mode: {}, Resolution: {}x{}",
        args.display, args.mode, args.width, args.height
    );

    // Load configuration
    let config = match load_configuration(&args).await {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    // Initialize health monitoring if enabled
    if args.enable_health {
        if let Err(e) = rxserver::init_health_monitoring().await {
            error!("Failed to initialize health monitoring: {}", e);
            process::exit(1);
        }

        // Configure health check interval
        let interval = Duration::from_secs(args.health_interval);
        if let Err(e) = rxserver::send_health_command(HealthCommand::SetInterval { interval }) {
            warn!("Failed to set health check interval: {}", e);
        }
    } // Create and initialize server with complete initialization
    let mut server = match rxserver::init_complete_server(config).await {
        Ok(server) => server,
        Err(e) => {
            error!("Failed to create server: {}", e);
            process::exit(1);
        }
    };

    // Set up signal handlers for graceful shutdown
    setup_signal_handlers(&mut server).await;

    // Start the server
    match server.start().await {
        Ok(_) => {
            info!("RxServer started successfully");
        }
        Err(e) => {
            error!("Server startup failed: {}", e);
            process::exit(1);
        }
    }

    // Main server loop with health monitoring
    if let Err(e) = run_server_loop(&mut server, &args).await {
        error!("Server runtime error: {}", e);
        process::exit(1);
    }

    // Graceful shutdown
    info!("Shutting down RxServer");
    if let Err(e) = server.stop().await {
        error!("Error during shutdown: {}", e);
        process::exit(1);
    }

    info!("RxServer shutdown complete");
}

/// Initialize logging based on command line arguments
fn init_logging(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for the tracing macros used throughout the code
    let log_level = match args.verbose {
        0 => tracing::Level::TRACE, // Default to most verbose
        1 => tracing::Level::DEBUG,
        2 => tracing::Level::INFO,
        3 => tracing::Level::WARN,
        _ => tracing::Level::ERROR,
    };

    tracing_subscriber::fmt().with_max_level(log_level).init();

    // Also initialize our custom LogManager for application-specific logging
    let log_manager = LogManager::new()?;

    // Configure the log manager
    {
        let mut manager = log_manager
            .lock()
            .map_err(|_| "Failed to lock log manager")?;
        let rxserver_log_level = match args.verbose {
            0 => "trace", // Default to most verbose
            1 => "debug",
            2 => "info",
            3 => "warn",
            _ => "error",
        };
        manager.set_level(LogLevel::from(rxserver_log_level));
    }

    Ok(())
}

/// Load server configuration from file and command line
async fn load_configuration(args: &Args) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let mut config = match ServerConfig::load(&args.config) {
        Ok(config) => {
            info!("Loaded configuration from: {}", args.config);
            config
        }
        Err(e) => {
            warn!(
                "Failed to load config file '{}': {}, generating default config",
                args.config, e
            );

            // Save file with default config
            let default_config = ServerConfig::default();
            default_config.save(&args.config)?;
            info!("Generated default config and saved to: {}", args.config);
            default_config
        }
    };

    // Log platform information
    let platform = Platform::current();
    info!("Running on platform: {:?}", platform);

    // Override config with command line arguments
    if let Some(ref mut display_config) = config.display {
        // Update display resolution from CLI args
        display_config.default_resolution =
            rxserver::display::types::Resolution::new(args.width, args.height);

        // Set display mode based on CLI arg
        match args.mode.as_str() {
            "headless" => {
                display_config.preferred_backend = rxserver::display::BackendType::Headless
            }
            "virtual" => display_config.preferred_backend = rxserver::display::BackendType::Virtual,
            "native" => display_config.preferred_backend = rxserver::display::BackendType::Native,
            _ => warn!("Unknown display mode '{}', using default", args.mode),
        }
    }

    Ok(config)
}

/// Create and configure the server
async fn create_server(
    config: ServerConfig,
    args: &Args,
) -> Result<Server, Box<dyn std::error::Error>> {
    let mut builder = ServerBuilder::new().with_config(config);

    // Configure health monitoring if enabled
    if args.enable_health {
        let health_config = rxserver::diagnostics::health::HealthMonitor::new();
        builder = builder.with_health_config(health_config);
    }

    let server = builder.build()?;
    info!("Server created successfully");

    Ok(server)
}

/// Set up signal handlers for graceful shutdown
async fn setup_signal_handlers(server: &mut Server) {
    let server_health_sender = server.get_health_command_sender();

    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("Received SIGINT, initiating graceful shutdown");

                // Send health check to verify server state before shutdown
                if let Some(sender) = server_health_sender {
                    if let Err(e) = sender.send(HealthCommand::RunChecks) {
                        warn!("Failed to run final health check: {}", e);
                    }
                }
            }
            Err(err) => {
                error!("Failed to listen for SIGINT: {}", err);
            }
        }
    });
}

/// Main server runtime loop with health monitoring
async fn run_server_loop(
    server: &mut Server,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut health_check_interval =
        tokio::time::interval(Duration::from_secs(args.health_interval));

    loop {
        tokio::select! {
            // Periodic health checks
            _ = health_check_interval.tick() => {
                if args.enable_health {
                    match server.get_health_status().await {
                        Ok(health) => {
                            match health.severity() {
                                HealthSeverity::Healthy => {
                                    // All good, continue
                                }
                                HealthSeverity::Warning => {
                                    warn!("Server health warning: {}", health.message);
                                }
                                HealthSeverity::Critical => {
                                    error!("Server health critical: {}", health.message);
                                    // Could implement automatic recovery here
                                }
                                HealthSeverity::Fatal => {
                                    error!("Server health fatal: {}", health.message);
                                    return Err("Fatal health status detected".into());
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to get health status: {}", e);
                        }
                    }
                }
            }

            // Handle shutdown signals
            _ = signal::ctrl_c() => {
                info!("Received shutdown signal");
                break;
            }

            // Server could have additional async operations here
            // For now, just continue the loop
        }
    }

    Ok(())
}
