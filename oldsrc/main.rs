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
    config,
    diagnostics::{
        self,
        health::{HealthCommand, HealthSeverity},
    },
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
    }
    // Create and initialize server using ServerBuilder
    let mut server = match create_server(config, &args).await {
        Ok(server) => server,
        Err(e) => {
            error!("Failed to create server: {}", e);
            process::exit(1);
        }
    }; // Prepare signal handling
    prepare_signal_handling(&mut server).await; // Start the server
    match server.start().await {
        Ok(_) => {
            info!("RxServer started successfully");

            // Demonstrate service registry usage
            info!("=== Service Registry Status ===");
            match server.get_service_registry_status().await {
                Ok(status) => {
                    for line in status.lines() {
                        info!("{}", line);
                    }
                }
                Err(e) => {
                    warn!("Failed to get service registry status: {}", e);
                }
            }

            // List registered services
            match server.list_registered_services().await {
                Ok(services) => {
                    info!("Registered services:");
                    for service in services {
                        info!("  - {}", service);
                    }
                }
                Err(e) => {
                    warn!("Failed to list services: {}", e);
                }
            }

            // Find core services
            match server.find_services_by_tag("core").await {
                Ok(core_services) => {
                    info!("Core services: {:?}", core_services);
                }
                Err(e) => {
                    warn!("Failed to find core services: {}", e);
                }
            }
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
    // Initialize global systems first if needed
    if args.enable_health {
        if let Err(e) = async move { config::init_global_config() }.await {
            warn!("Failed to initialize global config: {}", e);
        }

        if let Err(e) = async move { diagnostics::init_global_diagnostics() }.await {
            warn!("Failed to initialize global diagnostics: {}", e);
        }
    }

    let mut builder = ServerBuilder::new().with_config(config);

    // Configure health monitoring if enabled
    if args.enable_health {
        let health_config = rxserver::diagnostics::health::HealthMonitor::new();
        builder = builder.with_health_config(health_config);
    }

    let mut server = builder.build()?;

    // Initialize the server with all subsystems
    server.initialize().await?;

    info!("Server created and initialized successfully");
    Ok(server)
}

/// Prepare server for signal handling (signal handling is now done in main loop)
async fn prepare_signal_handling(server: &mut Server) {
    // Perform any necessary setup for signal handling
    // Signal handling itself is now centralized in run_server_loop

    // We could use this function to set up platform-specific signal handling
    // or to configure the server for graceful shutdown procedures

    info!("Signal handling prepared - signals will be handled in main server loop");
}

/// Represents different reasons for server shutdown
#[derive(Debug, Clone)]
enum ShutdownReason {
    Signal,
    HealthFailure(String),
    AdminRequest,
    InternalError(String),
}

/// Runtime configuration and state for the server loop
struct ServerRuntime {
    health_enabled: bool,
    health_interval: Duration,
}

impl ServerRuntime {
    fn new(args: &Args) -> Self {
        Self {
            health_enabled: args.enable_health,
            health_interval: Duration::from_secs(args.health_interval),
        }
    }
}

/// Handle health check and return whether to continue running
async fn handle_health_check(server: &mut Server) -> Result<bool, Box<dyn std::error::Error>> {
    match server.get_health_status().await {
        Ok(health) => {
            match health.severity() {
                HealthSeverity::Healthy => Ok(true),
                HealthSeverity::Warning => {
                    warn!("Server health warning: {}", health.message);
                    Ok(true)
                }
                HealthSeverity::Critical => {
                    error!("Server health critical: {}", health.message);
                    // Could implement automatic recovery here
                    Ok(true)
                }
                HealthSeverity::Fatal => {
                    error!("Server health fatal: {}", health.message);
                    Ok(false) // Signal shutdown needed
                }
            }
        }
        Err(e) => {
            warn!("Failed to get health status: {}", e);
            Ok(true) // Continue despite health check failure
        }
    }
}

/// Create a shutdown signal handler that returns when shutdown is requested
async fn create_shutdown_signal() -> Result<ShutdownReason, Box<dyn std::error::Error>> {
    tokio::select! {
        result = signal::ctrl_c() => {
            match result {
                Ok(()) => {
                    info!("Received SIGINT signal");
                    Ok(ShutdownReason::Signal)
                }
                Err(e) => {
                    error!("Failed to listen for shutdown signal: {}", e);
                    Err(e.into())
                }
            }
        }
        // Could add other shutdown triggers here (e.g., admin API, IPC, etc.)
    }
}

/// Main server runtime loop with health monitoring
async fn run_server_loop(
    server: &mut Server,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = ServerRuntime::new(args);
    let mut health_check_interval = tokio::time::interval(runtime.health_interval);

    info!(
        "Starting server runtime loop (health_enabled: {})",
        runtime.health_enabled
    );

    loop {
        tokio::select! {
            // Periodic health checks
            _ = health_check_interval.tick(), if runtime.health_enabled => {
                match handle_health_check(server).await {
                    Ok(should_continue) => {
                        if !should_continue {
                            info!("Health check indicates shutdown required");
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Health check error: {}", e);
                        // Continue running despite health check errors
                    }
                }
            }

            // Handle shutdown signals
            shutdown_result = create_shutdown_signal() => {
                match shutdown_result {
                    Ok(reason) => {
                        info!("Shutdown requested: {:?}", reason);
                        break;
                    }
                    Err(e) => {
                        error!("Shutdown signal error: {}", e);
                        return Err(e);
                    }
                }
            }            // Server main processing loop
            // Let the server handle X11 protocol messages, client connections, and core operations
            _ = server.run() => {
                info!("Server main loop completed");
                break;
            }
        }
    }

    info!("Server runtime loop completed successfully");
    Ok(())
}
