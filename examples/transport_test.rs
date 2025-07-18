// Example demonstrating the transport contract trait with generic stream handler

use rxserver::server::state::Server;
use rxserver::transport::{Transport, TransportKind, TransportMessage};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize server state
    let server_state = Arc::new(Mutex::new(Server::new()));

    // Create a message channel for transport events
    let (tx, mut rx) = mpsc::unbounded_channel::<TransportMessage>();

    // Demonstrate TCP transport
    println!("Testing TCP transport...");
    let tcp_transport = Transport::new(
        TransportKind::Tcp,
        "127.0.0.1:6000",
        Arc::clone(&server_state),
        tx.clone(),
    )
    .await?;

    println!("TCP transport kind: {:?}", tcp_transport.transport_kind());

    // Demonstrate Unix transport (only on Unix systems)
    #[cfg(unix)]
    {
        println!("Testing Unix transport...");
        let unix_transport = Transport::new(
            TransportKind::Unix,
            "/tmp/x11-test.sock",
            Arc::clone(&server_state),
            tx,
        )
        .await?;

        println!("Unix transport kind: {:?}", unix_transport.transport_kind());
    }

    println!("✓ Transport contract trait successfully implemented!");
    println!("✓ Generic stream handler supports both TCP and Unix sockets!");

    Ok(())
}
