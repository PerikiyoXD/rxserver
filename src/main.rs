use std::process;

use clap::Parser as ArgsParser;
use tracing::{error, info, warn};

use rxserver::{
    config::ServerConfig,
    logging::{init_logging, log_shutdown_info, log_system_info, ServerLogger, log_memory_usage, with_context},
    server::XServer,
    utils::log_implementation_status,
};

/// RX - Rust X Window System Server
#[derive(ArgsParser)]
#[command(name = "rxserver")]
#[command(about = "A modern, safe, and efficient Rust implementation of the X11 protocol")]
#[command(version)]
struct Args {
    /// Display number (e.g., :0, :1)
    #[arg(short, long, default_value = ":0")]
    display: String,

    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Run in foreground (don't daemonize)
    #[arg(short, long)]
    foreground: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Load configuration first
    let config = match ServerConfig::load(&args.config) {
        Ok(config) => {
            info!("Configuration loaded successfully from: {}", args.config);
            config
        }
        Err(e) => {
            eprintln!("Failed to load configuration from '{}': {}", args.config, e);
            eprintln!("Consider creating a config file or using --config to specify a different path");
            std::process::exit(1);
        }
    };    // Initialize logging system with comprehensive setup
    with_context("logging initialization", || {
        if let Err(e) = init_logging(&config.logging) {
            eprintln!("Failed to initialize logging: {}", e);
            warn!("Logging initialization failed, using fallback logging");
        }
    });

    // Log system and server information
    log_system_info();
    log_memory_usage();
    
    // Log implementation status for awareness
    log_implementation_status();
    
    // Log server startup with configuration details
    ServerLogger::startup(config.server.display_number, &args.config);
    config.log_config();

    // Log runtime environment
    info!("Runtime Environment:");
    info!("  Verbose mode: {}", args.verbose);
    info!("  Foreground mode: {}", args.foreground);
    info!("  Working directory: {:?}", std::env::current_dir().unwrap_or_default());    // Create and start the X server
    info!("Creating X server for display :{}", args.display);
    let server = match XServer::new(args.display, config).await {
        Ok(server) => {
            info!("X server created successfully");
            server
        }
        Err(e) => {
            error!("Failed to create X server: {}", e);
            std::process::exit(1);
        }
    };

    // Handle graceful shutdown with detailed logging
    tokio::select! {
        result = server.run() => {
            match result {
                Ok(_) => {
                    info!("Server completed normally");
                    on_exit();
                    Ok(())
                }
                Err(e) => {
                    error!("Server terminated with error: {}", e);
                    error!("Error details: {:?}", e);
                    on_exit_with_error(&e);
                    Ok(())
                }
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C signal, initiating graceful shutdown...");
            on_exit();
            Ok(())
        }
    }
}fn on_exit() {
    info!("Performing graceful shutdown...");
    log_memory_usage();
    log_shutdown_info();
    info!("Goodbye!");
    process::exit(0);
}

fn on_exit_with_error(error: &dyn std::error::Error) {
    error!("Performing emergency shutdown due to error: {}", error);
    log_memory_usage();
    log_shutdown_info();
    error!("Server terminated with errors!");
    process::exit(1);
}
