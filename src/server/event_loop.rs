//! Event Loop
//!
//! This module contains the main server event loop.

use crate::server::{ClientManager, ConnectionManager, RequestHandler};
use crate::server::ServerEvent;
use crate::{Error, Result};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Main server event loop
pub struct EventLoop {
    connection_manager: Arc<ConnectionManager>,
    client_manager: Arc<ClientManager>,
    request_handler: Arc<RequestHandler>,
    event_sender: broadcast::Sender<ServerEvent>,
}

impl EventLoop {
    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        client_manager: Arc<ClientManager>,
        request_handler: Arc<RequestHandler>,
        event_sender: broadcast::Sender<ServerEvent>,
    ) -> Self {
        Self {
            connection_manager,
            client_manager,
            request_handler,
            event_sender,
        }
    }

    pub async fn run(&self) -> Result<()> {
        // TODO: Implement main event loop
        Ok(())
    }
}
