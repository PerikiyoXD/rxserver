use std::sync::Arc;

use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{debug, error, info};

use crate::server::{ConnectionHandler, DisplayConfig, ServerState, VirtualDisplay};

pub async fn run(addr: &str) -> Result<()> {
    // Initialize virtual display with default configuration
    let display_config = DisplayConfig::default();
    info!("Creating virtual display with config: {:?}", display_config);

    let mut virtual_display = VirtualDisplay::new(display_config.clone());
    virtual_display.start()?;
    info!("Virtual display started successfully");

    // Initialize global server state
    let server_state = ServerState::new();

    // Set the virtual display in the server state
    {
        let mut state = server_state.lock().unwrap();
        state.set_virtual_display(virtual_display);
        info!("Virtual display integrated with server state");

        // Map the root window by default
        if let Err(e) = state.map_window(1) {
            debug!("Failed to map root window: {}", e);
        }

        // Send initial window state to display (including root window)
        state.sync_windows_to_display();
        info!("Initial window state synced to display");
    }

    info!("Initialized X11 server state");

    let listener = TcpListener::bind(addr).await?;
    info!("X11 server listening on {}", addr);

    {
        let state = server_state.lock().unwrap();
        debug!(
            "Server state initialized with {} predefined atoms",
            state.atom_registry.len()
        );
    }

    loop {
        let (socket, client_addr) = listener.accept().await?;
        info!("Accepting connection from {}", client_addr);

        let server_state = Arc::clone(&server_state);
        tokio::spawn(async move {
            match ConnectionHandler::new(server_state, socket) {
                Ok(handler) => {
                    if let Err(e) = handler.handle().await {
                        error!("Connection handler error for {}: {:?}", client_addr, e);
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to create connection handler for {}: {:?}",
                        client_addr, e
                    );
                }
            }
        });
    }
}
