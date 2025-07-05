use std::sync::Arc;

use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{debug, error, info};

pub async fn run(addr: &str) -> Result<()> {
    // Map the root window by default
    if let Err(e) = state.map_window(1) {
        debug!("Failed to map root window: {}", e);
    }

    // Send initial window state to display (including root window)
    state.sync_windows_to_display();
    info!("Initial window state synced to display");

    info!("Initialized X11 server state");

    let listener: TcpListener = TcpListener::bind(addr).await?;
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
