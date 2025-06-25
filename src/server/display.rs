//! Virtual Display Management for X11 Server
//! 
//! This module creates a native window using winit to display the X11 server's output.
//! It provides a virtual display that shows the rendered content of the X11 windows.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopBuilderExtWindows;

use crate::protocol::X11Error;

/// Display configuration for the virtual display
#[derive(Debug, Clone)]
pub struct DisplayConfig {
    pub width: u16,
    pub height: u16,
    pub width_mm: u16,  // Physical width in millimeters
    pub height_mm: u16, // Physical height in millimeters
    pub depth: u8,      // Color depth in bits
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            width_mm: 270,  // ~96 DPI
            height_mm: 203, // ~96 DPI
            depth: 24,
        }
    }
}

/// Messages for communicating with the virtual display thread
#[derive(Debug)]
pub enum DisplayMessage {
    UpdateFramebuffer(Vec<u32>),
    Resize(u32, u32),
    Shutdown,
}

/// Messages sent from the virtual display back to the server
#[derive(Debug)]
pub enum DisplayCallbackMessage {
    WindowResized(u32, u32),
    DisplayClosed,
}

/// Virtual display application handler for winit
struct VirtualDisplayApp {
    window: Option<Arc<Window>>,
    context: Option<softbuffer::Context<Arc<Window>>>,
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
    framebuffer: Vec<u32>,
    config: DisplayConfig,
    last_resize_time: Instant,
    message_receiver: mpsc::UnboundedReceiver<DisplayMessage>,
    callback_sender: Option<mpsc::UnboundedSender<DisplayCallbackMessage>>,
}

impl VirtualDisplayApp {
    fn new(
        config: DisplayConfig,
        message_receiver: mpsc::UnboundedReceiver<DisplayMessage>,
        callback_sender: Option<mpsc::UnboundedSender<DisplayCallbackMessage>>,
    ) -> Self {
        let framebuffer_size = (config.width as u32 * config.height as u32) as usize;
        Self {
            window: None,
            context: None,
            surface: None,
            framebuffer: vec![0x000000FF; framebuffer_size], // Start with black background
            config,
            last_resize_time: Instant::now(),
            message_receiver,
            callback_sender,
        }
    }

    /// Draw a simple test pattern to the framebuffer
    fn draw_test_pattern(&mut self) {
        let width = self.config.width as u32;
        let height = self.config.height as u32;
        
        for y in 0..height {
            for x in 0..width {
                let index = (y * width + x) as usize;
                if index < self.framebuffer.len() {
                    // Create a simple gradient pattern
                    let r = ((x * 255) / width) as u8;
                    let g = ((y * 255) / height) as u8;
                    let b = 128u8;
                    self.framebuffer[index] = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32) | 0xFF000000;
                }
            }
        }
        
        // Draw "RX X11 Server" text pattern in the center
        let center_x = width / 2;
        let center_y = height / 2;
        
        // Draw a simple cross pattern to represent the server
        for i in 0..60 {
            let x1 = center_x.saturating_sub(30) + i;
            let y1 = center_y;
            let index1 = (y1 * width + x1) as usize;
            if index1 < self.framebuffer.len() {
                self.framebuffer[index1] = 0xFFFFFFFF; // White
            }
            
            let x2 = center_x;
            let y2 = center_y.saturating_sub(30) + i;
            let index2 = (y2 * width + x2) as usize;
            if index2 < self.framebuffer.len() {
                self.framebuffer[index2] = 0xFFFFFFFF; // White
            }
        }
    }
}

impl ApplicationHandler for VirtualDisplayApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            info!(
                "Creating RX X11 Server virtual display window ({}x{})",
                self.config.width, self.config.height
            );

            let window_attributes = Window::default_attributes()
                .with_title("RX X11 Server - Virtual Display")
                .with_inner_size(winit::dpi::PhysicalSize::new(
                    self.config.width as u32,
                    self.config.height as u32,
                ))
                .with_resizable(true);

            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Failed to create virtual display window"),
            );

            // Create softbuffer context and surface for rendering
            let context = softbuffer::Context::new(window.clone())
                .expect("Failed to create softbuffer context");
            let mut surface = softbuffer::Surface::new(&context, window.clone())
                .expect("Failed to create softbuffer surface");

            // Initialize surface with current configuration
            let width_nz = std::num::NonZeroU32::new(self.config.width as u32).unwrap();
            let height_nz = std::num::NonZeroU32::new(self.config.height as u32).unwrap();
            surface
                .resize(width_nz, height_nz)
                .expect("Failed to resize surface");

            // Draw initial test pattern
            self.draw_test_pattern();

            self.window = Some(window);
            self.context = Some(context);
            self.surface = Some(surface);

            info!("Virtual display window created successfully");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(window) = &self.window {
            if window.id() == window_id {
                match event {
                    WindowEvent::CloseRequested => {
                        info!("Virtual display window close requested");
                        if let Some(ref callback_sender) = self.callback_sender {
                            let _ = callback_sender.send(DisplayCallbackMessage::DisplayClosed);
                        }
                        event_loop.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        // Render the current framebuffer to the window
                        if let Some(surface) = &mut self.surface {
                            if let Ok(mut buffer) = surface.buffer_mut() {
                                let copy_len = std::cmp::min(self.framebuffer.len(), buffer.len());
                                buffer[..copy_len].copy_from_slice(&self.framebuffer[..copy_len]);
                                if let Err(e) = buffer.present() {
                                    error!("Failed to present framebuffer: {}", e);
                                }
                            }
                        }
                    }
                    WindowEvent::Resized(size) => {
                        let new_width = size.width;
                        let new_height = size.height;

                        // Skip invalid sizes
                        if new_width == 0 || new_height == 0 {
                            return;
                        }

                        // Throttle resize events
                        let now = Instant::now();
                        if now.duration_since(self.last_resize_time).as_millis() < 50 {
                            return;
                        }
                        self.last_resize_time = now;

                        info!("Virtual display resized to {}x{}", new_width, new_height);

                        // Update configuration
                        self.config.width = new_width as u16;
                        self.config.height = new_height as u16;
                        self.config.width_mm = ((new_width as f32 / 96.0) * 25.4) as u16;
                        self.config.height_mm = ((new_height as f32 / 96.0) * 25.4) as u16;

                        // Resize surface
                        if let Some(surface) = &mut self.surface {
                            let width_nz = std::num::NonZeroU32::new(new_width).unwrap();
                            let height_nz = std::num::NonZeroU32::new(new_height).unwrap();
                            if let Err(e) = surface.resize(width_nz, height_nz) {
                                error!("Failed to resize surface: {}", e);
                                return;
                            }
                        }                        // Resize framebuffer
                        let new_size = (new_width * new_height) as usize;
                        self.framebuffer.resize(new_size, 0x000000FF);
                        
                        // Notify the server about the resize
                        if let Some(ref callback_sender) = self.callback_sender {
                            let _ = callback_sender.send(DisplayCallbackMessage::WindowResized(new_width, new_height));
                        }

                        // Draw test pattern and request redraw outside the borrow scope
                        let should_redraw = true;
                        if should_redraw {
                            self.draw_test_pattern();
                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Process messages from the X11 server
        while let Ok(message) = self.message_receiver.try_recv() {
            match message {
                DisplayMessage::UpdateFramebuffer(new_framebuffer) => {
                    debug!("Updating virtual display framebuffer");
                    let copy_len = std::cmp::min(new_framebuffer.len(), self.framebuffer.len());
                    self.framebuffer[..copy_len].copy_from_slice(&new_framebuffer[..copy_len]);
                    
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                DisplayMessage::Resize(width, height) => {
                    debug!("Received resize request: {}x{}", width, height);
                    // Let the window system handle resizing naturally
                    if let Some(window) = &self.window {
                        let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(width, height));
                    }
                }
                DisplayMessage::Shutdown => {
                    info!("Virtual display shutdown requested");
                    if let Some(ref callback_sender) = self.callback_sender {
                        let _ = callback_sender.send(DisplayCallbackMessage::DisplayClosed);
                    }
                    _event_loop.exit();
                }
            }
        }
    }
}

/// Virtual display manager that handles the display thread
#[derive(Debug)]
pub struct VirtualDisplay {
    config: Arc<Mutex<DisplayConfig>>,
    message_sender: Option<mpsc::UnboundedSender<DisplayMessage>>,
    callback_receiver: Option<mpsc::UnboundedReceiver<DisplayCallbackMessage>>,
}

impl VirtualDisplay {
    /// Create a new virtual display with the given configuration
    pub fn new(config: DisplayConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            message_sender: None,
            callback_receiver: None,
        }
    }

    /// Start the virtual display in a separate thread
    pub fn start(&mut self) -> Result<(), X11Error> {
        let (message_sender, message_receiver) = mpsc::unbounded_channel();
        let (callback_sender, callback_receiver) = mpsc::unbounded_channel();

        let config = {
            let config_guard = self.config.lock().unwrap();
            config_guard.clone()
        };

        // Store the channels
        self.message_sender = Some(message_sender);
        self.callback_receiver = Some(callback_receiver);

        // Spawn the display thread
        std::thread::spawn(move || {
            info!("Starting virtual display thread");

            // Create event loop
            #[cfg(target_os = "windows")]
            let event_loop = EventLoop::builder()
                .with_any_thread(true)
                .build()
                .expect("Failed to create event loop");

            #[cfg(not(target_os = "windows"))]
            let event_loop = EventLoop::new().expect("Failed to create event loop");

            // Create application
            let mut app = VirtualDisplayApp::new(config, message_receiver, Some(callback_sender));

            // Run the event loop
            if let Err(e) = event_loop.run_app(&mut app) {
                error!("Virtual display event loop error: {}", e);
            }

            info!("Virtual display thread terminated");
        });

        info!("Virtual display started successfully");
        Ok(())
    }

    /// Get the current display configuration
    pub fn get_config(&self) -> DisplayConfig {
        let config_guard = self.config.lock().unwrap();
        config_guard.clone()
    }

    /// Update the display configuration
    pub fn update_config(&self, new_config: DisplayConfig) {
        let mut config_guard = self.config.lock().unwrap();
        *config_guard = new_config;
    }

    /// Send a framebuffer update to the display
    pub fn update_framebuffer(&self, framebuffer: Vec<u32>) -> Result<(), X11Error> {
        if let Some(ref sender) = self.message_sender {
            sender
                .send(DisplayMessage::UpdateFramebuffer(framebuffer))
                .map_err(|e| X11Error::Protocol(format!("Failed to send framebuffer update: {}", e)))?;
        }
        Ok(())
    }

    /// Request a display resize
    pub fn resize(&self, width: u32, height: u32) -> Result<(), X11Error> {
        if let Some(ref sender) = self.message_sender {
            sender
                .send(DisplayMessage::Resize(width, height))
                .map_err(|e| X11Error::Protocol(format!("Failed to send resize: {}", e)))?;
        }
        Ok(())
    }

    /// Shutdown the virtual display
    pub fn shutdown(&self) -> Result<(), X11Error> {
        if let Some(ref sender) = self.message_sender {
            sender
                .send(DisplayMessage::Shutdown)
                .map_err(|e| X11Error::Protocol(format!("Failed to send shutdown: {}", e)))?;
        }
        Ok(())
    }

    /// Check for callback messages (non-blocking)
    pub fn try_recv_callback(&mut self) -> Option<DisplayCallbackMessage> {
        if let Some(ref mut receiver) = self.callback_receiver {
            receiver.try_recv().ok()
        } else {
            None
        }
    }
}

impl Drop for VirtualDisplay {
    fn drop(&mut self) {
        if let Err(e) = self.shutdown() {
            warn!("Error shutting down virtual display: {}", e);
        }
    }
}
