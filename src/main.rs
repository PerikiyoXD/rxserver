use clap::Parser;
use rxserver::{
    core::args::CommandlineArgs,
    core::{config::ServerConfig, logging::init_logging},
    server::{DisplayMode, RXServer},
};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() {
    let args: CommandlineArgs = CommandlineArgs::parse();

    let config = match ServerConfig::load(&args.config) {
        Ok(config) => {
            init_logging(Some(&config.logging)).unwrap_or_else(|e| {
                eprintln!("Failed to initialize logging: {}", e);
                std::process::exit(1);
            });
            config
        }
        Err(e) => {
            init_logging(None).unwrap_or_else(|e| {
                eprintln!("Failed to initialize logging: {}", e);
                std::process::exit(1);
            });
            warn!("Config load failed ({}), using default configuration", e);
            ServerConfig::default()
        }
    };

    info!("Starting X server on display {}", args.display);

    // Parse display mode from command line arguments
    let display_mode = match args.mode.as_str() {
        "headless" => {
            info!("Using headless display mode");
            DisplayMode::Headless
        }
        "virtual" => {
            info!(
                "Using virtual display mode ({}x{})",
                args.width, args.height
            );
            DisplayMode::VirtualDisplay {
                width: args.width,
                height: args.height,
            }
        }
        "native" => {
            info!("Using native display mode ({}x{})", args.width, args.height);
            DisplayMode::NativeDisplay {
                width: args.width,
                height: args.height,
            }
        }
        _ => {
            warn!(
                "Unknown display mode '{}', defaulting to headless",
                args.mode
            );
            DisplayMode::Headless
        }
    };

    let server = match RXServer::new_with_display_mode(args.display, config, display_mode).await {
        Ok(server) => server,
        Err(e) => {
            error!("Server creation failed: {}", e);
            std::process::exit(1);
        }
    };

    tokio::select! {
        result = server.run() => {
            if let Err(e) = result {
                error!("Server error: {}", e);
                std::process::exit(1);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received");
        }
    }
}
