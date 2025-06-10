//! Window management event handlers
//!
//! This module contains event handlers for window-related operations.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::server::events::{EventHandler, EventResponse, ServerEvent};
use crate::server::resources::ResourceManager;
use crate::protocol::{Response, Event as ProtocolEvent};
use crate::protocol::types::*;
use crate::{Error, Result};

/// Window manager event handler
pub struct WindowHandler {
    /// Resource manager
    resources: Arc<RwLock<ResourceManager>>,
}

impl WindowHandler {
    /// Create a new window handler
    pub fn new(resources: Arc<RwLock<ResourceManager>>) -> Self {
        Self { resources }
    }
}

#[async_trait::async_trait]
impl EventHandler for WindowHandler {
    async fn handle_event(&self, event: &ServerEvent) -> Result<Option<EventResponse>> {
        match event {
            ServerEvent::RequestReceived { client_id, sequence_number, request } => {
                match request {
                    crate::protocol::Request::CreateWindow {
                        depth: _,
                        wid,
                        parent,
                        x,
                        y,
                        width,
                        height,
                        border_width,
                        class,
                        visual: _,
                        value_mask: _,
                        value_list: _,
                    } => {
                        log::info!("Creating window {} ({}x{} at {},{}) parent={}", 
                                 wid, width, height, x, y, parent);

                        // Create the window in resource manager
                        let mut resources = self.resources.write().await;
                        match resources.create_window(*parent, *x, *y, *width, *height, *border_width, *class) {
                            Ok(window_id) => {                                log::debug!("Successfully created window {}", window_id);
                                
                                // Emit window created event
                                let _window_event = ServerEvent::WindowCreated {
                                    window: window_id,
                                    parent: *parent,
                                    geometry: Rectangle {
                                        x: *x,
                                        y: *y,
                                        width: *width,
                                        height: *height,
                                    },
                                    client_id: *client_id,
                                };

                                // Return success response and chain the window created event
                                return Ok(Some(EventResponse::Response(Response::Success)));
                            }
                            Err(e) => {
                                log::error!("Failed to create window: {}", e);
                                return Ok(Some(EventResponse::Response(Response::Error {
                                    error_code: 3, // BadWindow
                                    sequence_number: *sequence_number,
                                    bad_value: *wid,
                                    minor_opcode: 0,
                                    major_opcode: 1, // CreateWindow
                                })));
                            }
                        }
                    }

                    crate::protocol::Request::DestroyWindow { window } => {
                        log::info!("Destroying window {}", window);

                        let mut resources = self.resources.write().await;
                        match resources.destroy_window(*window) {
                            Ok(_) => {
                                log::debug!("Successfully destroyed window {}", window);
                                
                                // Emit protocol event to notify clients
                                let destroy_event = ProtocolEvent::DestroyNotify {
                                    event: *window,
                                    window: *window,
                                };

                                return Ok(Some(EventResponse::ProtocolEvent {
                                    event: destroy_event,
                                    target_clients: vec![], // Broadcast to all
                                }));
                            }
                            Err(e) => {
                                log::error!("Failed to destroy window: {}", e);
                                return Ok(Some(EventResponse::Response(Response::Error {
                                    error_code: 3, // BadWindow
                                    sequence_number: *sequence_number,
                                    bad_value: *window,
                                    minor_opcode: 0,
                                    major_opcode: 4, // DestroyWindow
                                })));
                            }
                        }
                    }

                    crate::protocol::Request::MapWindow { window } => {
                        log::info!("Mapping window {}", window);

                        let mut resources = self.resources.write().await;
                        match resources.map_window(*window) {
                            Ok(_) => {
                                log::debug!("Successfully mapped window {}", window);
                                
                                // Emit protocol event
                                let map_event = ProtocolEvent::MapNotify {
                                    event: *window,
                                    window: *window,
                                    override_redirect: false,
                                };

                                return Ok(Some(EventResponse::ProtocolEvent {
                                    event: map_event,
                                    target_clients: vec![], // Broadcast to all
                                }));
                            }
                            Err(e) => {
                                log::error!("Failed to map window: {}", e);
                                return Ok(Some(EventResponse::Response(Response::Error {
                                    error_code: 3, // BadWindow
                                    sequence_number: *sequence_number,
                                    bad_value: *window,
                                    minor_opcode: 0,
                                    major_opcode: 8, // MapWindow
                                })));
                            }
                        }
                    }

                    crate::protocol::Request::UnmapWindow { window } => {
                        log::info!("Unmapping window {}", window);

                        let mut resources = self.resources.write().await;
                        match resources.unmap_window(*window) {
                            Ok(_) => {
                                log::debug!("Successfully unmapped window {}", window);
                                
                                // Emit protocol event
                                let unmap_event = ProtocolEvent::UnmapNotify {
                                    event: *window,
                                    window: *window,
                                    from_configure: false,
                                };

                                return Ok(Some(EventResponse::ProtocolEvent {
                                    event: unmap_event,
                                    target_clients: vec![], // Broadcast to all
                                }));
                            }
                            Err(e) => {
                                log::error!("Failed to unmap window: {}", e);
                                return Ok(Some(EventResponse::Response(Response::Error {
                                    error_code: 3, // BadWindow
                                    sequence_number: *sequence_number,
                                    bad_value: *window,
                                    minor_opcode: 0,
                                    major_opcode: 10, // UnmapWindow
                                })));
                            }
                        }
                    }

                    crate::protocol::Request::GetWindowAttributes { window } => {
                        log::debug!("Getting window attributes for {}", window);

                        let resources = self.resources.read().await;
                        if let Some(window_resource) = resources.get_window(*window) {
                            // Build window attributes response
                            let mut body = vec![0u8; 44]; // GetWindowAttributes reply is 44 bytes
                            
                            // Fill in basic window attributes
                            body[0] = window_resource.class as u8; // Window class
                            body[1] = 24; // Backing store (NotUseful = 0, WhenMapped = 1, Always = 2)
                            // TODO: Fill in more attributes as needed
                            
                            return Ok(Some(EventResponse::Response(Response::Reply {
                                data: 0,
                                sequence_number: *sequence_number,
                                length: ((body.len() + 3) / 4) as u32, // Length in 4-byte units
                                body,
                            })));
                        } else {
                            return Ok(Some(EventResponse::Response(Response::Error {
                                error_code: 3, // BadWindow
                                sequence_number: *sequence_number,
                                bad_value: *window,
                                minor_opcode: 0,
                                major_opcode: 3, // GetWindowAttributes
                            })));
                        }
                    }

                    _ => {
                        // Not a window-related request, ignore
                    }
                }
            }

            ServerEvent::WindowCreated { window, parent, geometry, client_id } => {
                log::info!("Window {} created by client {} (parent: {}, geometry: {:?})", 
                         window, client_id, parent, geometry);
                
                // Generate CreateNotify event for interested clients
                let create_event = ProtocolEvent::CreateNotify {
                    parent: *parent,
                    window: *window,
                    x: geometry.x,
                    y: geometry.y,
                    width: geometry.width,
                    height: geometry.height,
                    border_width: 0, // TODO: Get from window resource
                    override_redirect: false,
                };

                return Ok(Some(EventResponse::ProtocolEvent {
                    event: create_event,
                    target_clients: vec![], // Broadcast to all interested clients
                }));
            }

            _ => {
                // Not interested in this event
            }
        }

        Ok(None)
    }

    fn name(&self) -> &str {
        "WindowHandler"
    }
}

/// Graphics event handler
pub struct GraphicsHandler {
    /// Resource manager
    resources: Arc<RwLock<ResourceManager>>,
}

impl GraphicsHandler {
    /// Create a new graphics handler
    pub fn new(resources: Arc<RwLock<ResourceManager>>) -> Self {
        Self { resources }
    }
}

#[async_trait::async_trait]
impl EventHandler for GraphicsHandler {
    async fn handle_event(&self, event: &ServerEvent) -> Result<Option<EventResponse>> {
        match event {
            ServerEvent::GraphicsOperation { operation, client_id } => {
                log::debug!("Processing graphics operation from client {}: {:?}", client_id, operation);
                
                match operation {
                    crate::server::events::GraphicsOp::ClearArea { window, rect } => {
                        log::info!("Clearing area in window {}: {:?}", window, rect);
                        
                        // Generate expose event to trigger redraw
                        let expose_event = ProtocolEvent::Expose {
                            window: *window,
                            x: rect.x as u16,
                            y: rect.y as u16,
                            width: rect.width,
                            height: rect.height,
                            count: 0, // No more expose events follow
                        };

                        return Ok(Some(EventResponse::ProtocolEvent {
                            event: expose_event,
                            target_clients: vec![], // Broadcast to all
                        }));
                    }
                    
                    crate::server::events::GraphicsOp::DrawPoint { window, x, y, gc } => {
                        log::debug!("Drawing point at ({}, {}) in window {} with GC {}", x, y, window, gc);
                        // TODO: Implement actual drawing
                        return Ok(Some(EventResponse::Response(Response::Success)));
                    }
                    
                    crate::server::events::GraphicsOp::DrawLine { window, x1, y1, x2, y2, gc } => {
                        log::debug!("Drawing line from ({}, {}) to ({}, {}) in window {} with GC {}", 
                                  x1, y1, x2, y2, window, gc);
                        // TODO: Implement actual drawing
                        return Ok(Some(EventResponse::Response(Response::Success)));
                    }
                    
                    _ => {
                        log::debug!("Graphics operation not yet implemented: {:?}", operation);
                        return Ok(Some(EventResponse::Response(Response::Success)));
                    }                }
            }

            ServerEvent::RequestReceived { client_id, sequence_number: _, request } => {
                match request {
                    crate::protocol::Request::ClearArea { window, x, y, width, height, .. } => {
                        log::info!("Clear area request for window {}", window);
                        
                        // Convert to graphics operation event
                        let graphics_op = crate::server::events::GraphicsOp::ClearArea {
                            window: *window,
                            rect: Rectangle {
                                x: *x,
                                y: *y,
                                width: *width,
                                height: *height,
                            },
                        };

                        let graphics_event = ServerEvent::GraphicsOperation {
                            operation: graphics_op,
                            client_id: *client_id,
                        };

                        return Ok(Some(EventResponse::ServerEvent(graphics_event)));
                    }
                    _ => {
                        // Not a graphics-related request
                    }
                }
            }

            _ => {
                // Not interested in this event
            }
        }

        Ok(None)
    }

    fn name(&self) -> &str {
        "GraphicsHandler"
    }
}
