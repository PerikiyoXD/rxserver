use std::process;

use clap::Parser;
use log::{error, info};

use rxserver::{config::ServerConfig, server::XServer};

/// RX - Rust X Window System Server
#[derive(Parser)]
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
async fn main() {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose {
        "debug"
    } else {
        "info"
    };
    
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .init();

    info!("Starting RX - Rust X Window System Server");
    info!("Display: {}", args.display);
    info!("Config: {}", args.config);

    // Load configuration
    let config = match ServerConfig::load(&args.config) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    // Create and start the X server
    let mut server = match XServer::new(args.display, config).await {
        Ok(server) => server,
        Err(e) => {
            error!("Failed to create X server: {}", e);
            process::exit(1);
        }
    };

    // Handle graceful shutdown
    tokio::select! {
        result = server.run() => {
            match result {
                Ok(_) => info!("Server shut down gracefully"),
                Err(e) => {
                    error!("Server error: {}", e);
                    process::exit(1);
                }
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
            if let Err(e) = server.shutdown().await {
                error!("Error during shutdown: {}", e);
                process::exit(1);
            }
        }
    }
}
