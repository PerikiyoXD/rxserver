//! Server event system
//!
//! This module defines internal server events that are used for communication
//! between different components of the server.

use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::protocol::types::*;
use crate::protocol::{responses::Event as ProtocolEvent, Request, Response};
use crate::{Error, Result};

/// Internal server events
#[derive(Debug, Clone)]
pub enum ServerEvent {
    /// Client connected
    ClientConnected { client_id: u32, address: String },
    /// Client disconnected
    ClientDisconnected { client_id: u32 },
    /// Request received from client
    RequestReceived {
        client_id: u32,
        sequence_number: u16,
        request: Request,
    },
    /// Window created
    WindowCreated {
        window: Window,
        parent: Window,
        geometry: Rectangle,
        client_id: u32,
    },
    /// Window destroyed
    WindowDestroyed { window: Window, client_id: u32 },
    /// Window mapped
    WindowMapped { window: Window, client_id: u32 },
    /// Window unmapped
    WindowUnmapped {
        window: Window,
        geometry: Rectangle,
        client_id: u32,
    },
    /// Graphics operation requested
    GraphicsOperation {
        operation: GraphicsOp,
        client_id: u32,
    },
    /// Input event occurred
    InputEvent { event: InputEventType },
    /// Server is shutting down
    ServerShuttingDown,
}

/// Graphics operations
#[derive(Debug, Clone)]
pub enum GraphicsOp {
    DrawPoint {
        window: Window,
        x: i16,
        y: i16,
        gc: GContext,
    },
    DrawLine {
        window: Window,
        x1: i16,
        y1: i16,
        x2: i16,
        y2: i16,
        gc: GContext,
    },
    DrawRectangle {
        window: Window,
        rect: Rectangle,
        gc: GContext,
    },
    FillRectangle {
        window: Window,
        rect: Rectangle,
        gc: GContext,
    },
    CopyArea {
        src_window: Window,
        dst_window: Window,
        src_rect: Rectangle,
        dst_point: Point,
        gc: GContext,
    },
    ClearArea {
        window: Window,
        rect: Rectangle,
    },
}

/// Input event types
#[derive(Debug, Clone)]
pub enum InputEventType {
    KeyPress {
        keycode: KeyCode,
        timestamp: Timestamp,
    },
    KeyRelease {
        keycode: KeyCode,
        timestamp: Timestamp,
    },
    ButtonPress {
        button: u8,
        x: i16,
        y: i16,
        timestamp: Timestamp,
    },
    ButtonRelease {
        button: u8,
        x: i16,
        y: i16,
        timestamp: Timestamp,
    },
    MotionNotify {
        x: i16,
        y: i16,
        timestamp: Timestamp,
    },
}

/// Event handler trait
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle a server event
    async fn handle_event(&self, event: &ServerEvent) -> Result<Option<EventResponse>>;

    /// Get the handler's name for debugging
    fn name(&self) -> &str;
}

/// Response from an event handler
#[derive(Debug)]
pub enum EventResponse {
    /// Send a response to the client
    Response(Response),
    /// Send a protocol event to clients
    ProtocolEvent {
        event: ProtocolEvent,
        target_clients: Vec<u32>, // Empty means broadcast to all
    },
    /// Chain another server event
    ServerEvent(ServerEvent),
}

/// Event bus for managing server events
pub struct EventBus {
    /// Event sender
    sender: broadcast::Sender<ServerEvent>,
    /// Event handlers
    handlers: Arc<RwLock<Vec<Arc<dyn EventHandler>>>>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);

        Self {
            sender,
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register an event handler
    pub async fn register_handler(&self, handler: Arc<dyn EventHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.push(handler);
        log::debug!("Registered event handler");
    }

    /// Emit an event
    pub async fn emit(&self, event: ServerEvent) -> Result<Vec<EventResponse>> {
        log::debug!("Emitting event: {:?}", event);

        // Send event through broadcast channel
        if let Err(e) = self.sender.send(event.clone()) {
            log::warn!("Failed to broadcast event: {}", e);
        }

        // Process event through handlers
        let mut responses = Vec::new();
        let handlers = self.handlers.read().await;

        for handler in handlers.iter() {
            match handler.handle_event(&event).await {
                Ok(Some(response)) => {
                    log::debug!("Handler '{}' produced response", handler.name());
                    responses.push(response);
                }
                Ok(None) => {
                    // Handler didn't produce a response, that's fine
                }
                Err(e) => {
                    log::error!("Handler '{}' failed: {}", handler.name(), e);
                }
            }
        }

        Ok(responses)
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.sender.subscribe()
    }

    /// Get the number of registered handlers
    pub async fn handler_count(&self) -> usize {
        self.handlers.read().await.len()
    }
}

/// Default event handler implementation
pub struct DefaultEventHandler {
    name: String,
    event_receiver: broadcast::Receiver<ServerEvent>,
}

impl DefaultEventHandler {
    /// Create a new default event handler
    pub fn new(name: String, event_receiver: broadcast::Receiver<ServerEvent>) -> Self {
        Self {
            name,
            event_receiver,
        }
    }

    /// Start the event handler loop
    pub async fn start(&mut self) -> Result<()> {
        use crate::todo_high;

        loop {
            match self.event_receiver.recv().await {
                Ok(event) => {
                    if let Some(_response) = self.handle_event(&event).await? {
                        todo_high!("event_handler", "Event response handling not implemented");
                    }
                }
                Err(broadcast::error::RecvError::Closed) => {
                    log::info!("Event channel closed, stopping event handler");
                    break;
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    log::warn!("Event handler lagged, some events may have been missed");
                    continue;
                }
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl EventHandler for DefaultEventHandler {
    async fn handle_event(&self, event: &ServerEvent) -> Result<Option<EventResponse>> {
        use crate::todo_medium;

        log::debug!("Handling event: {:?}", event);

        match event {
            ServerEvent::ClientConnected { client_id, address } => {
                log::info!("Client {} connected from {}", client_id, address);
                Ok(None)
            }
            ServerEvent::ClientDisconnected { client_id } => {
                log::info!("Client {} disconnected", client_id);
                Ok(None)
            }
            ServerEvent::RequestReceived {
                client_id,
                sequence_number,
                request,
            } => {
                todo_medium!(
                    "event_handler",
                    "Request processing not implemented for client {} seq {} request {:?}",
                    client_id,
                    sequence_number,
                    request
                );
                Ok(None)
            }
            ServerEvent::WindowCreated {
                window,
                parent,
                geometry,
                client_id,
            } => {
                log::info!(
                    "Window {} created by client {} (parent: {}, geometry: {:?})",
                    window,
                    client_id,
                    parent,
                    geometry
                );
                Ok(None)
            }
            ServerEvent::WindowDestroyed { window, client_id } => {
                log::info!("Window {} destroyed by client {}", window, client_id);
                Ok(None)
            }
            ServerEvent::ServerShuttingDown => {
                log::info!("Server shutting down");
                Ok(None)
            }
            _ => {
                todo_medium!("event_handler", "Unhandled event type: {:?}", event);
                Ok(None)
            }
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}
