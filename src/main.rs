use std::process;

use clap::Parser as ArgsParser;
use tracing::{error, info};

use rxserver::{
    config::ServerConfig,
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
async fn main() {
    let args = Args::parse();
    // Initialize tracing/logging
    let log_level = if args.verbose { "debug" } else { "info" };

    // Set up tracing subscriber with proper formatting
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(log_level))
        .with_target(false)
        .with_level(true)
        .with_thread_ids(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    // Initialize tracing-log bridge to capture log crate messages
    tracing_log::LogTracer::init().expect("Failed to set up log bridge");

    // Log implementation status for awareness
    log_implementation_status();

    // Load configuration
    let config = match ServerConfig::load(&args.config) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            on_exit();
            unreachable!();
        }
    };

    // init_logging(&config.logging.level, log_to_file, log_to_stdout);
    // log_startup_info(config.server.display_number, &args.config);

    // Create and start the X server
    let server = match XServer::new(args.display, config).await {
        Ok(server) => server,
        Err(e) => {
            error!("Failed to create X server: {}", e);
            on_exit();
            unreachable!();
        }
    };

    // Handle graceful shutdown
    tokio::select! {
        result = server.run() => {
            match result {
                Ok(_) => info!("Server shut down gracefully"),
                Err(e) => {
                    error!("Server error: {}", e);
                    on_exit();
                }
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
            if let Err(e) = server.shutdown().await {
                error!("Error during shutdown: {}", e);
                on_exit();
            }
        }
    }
}

fn on_exit() {
    log_shutdown_info();
    process::exit(0);
}
