//! Virtual Display Protocol Handler
//!
//! A specialized protocol handler that creates a native window to display
//! the X11 server's framebuffer content. Uses winit + softbuffer for cross-platform display.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, error, info};

// Windows-specific winit imports
#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopBuilderExtWindows;

// Import ApplicationHandler and window creation types
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use crate::{
    ServerError, ServerResult,
    graphics::Renderer,
    plugins::{PluginRegistry, WindowPlugin},
    protocol::{ClientId, Opcode, ProtocolHandler, Request, Response},
};

/// Messages for communicating with the virtual display thread
#[derive(Debug)]
pub enum DisplayMessage {
    UpdateFramebuffer(Vec<u32>),
    Refresh,
    Resize(u32, u32),         // width, height
    ResizeRenderer(u32, u32), // width, height - requests renderer resize
    Shutdown,
}

/// Messages sent from the virtual display thread back to the protocol handler
#[derive(Debug)]
pub enum DisplayCallbackMessage {
    WindowResized(u32, u32), // width, height
}

/// Application handler for the virtual display window
/// This implements the ApplicationHandler trait required by winit 0.30.x
struct VirtualDisplayApp {
    window: Option<Arc<Window>>,
    context: Option<softbuffer::Context<Arc<Window>>>,
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
    current_framebuffer: Vec<u32>,
    current_width: u32,
    current_height: u32,
    last_resize_time: std::time::Instant,
    receiver: mpsc::UnboundedReceiver<DisplayMessage>,
    callback_sender: Option<mpsc::UnboundedSender<DisplayCallbackMessage>>,
}

impl VirtualDisplayApp {
    fn new(
        width: u32,
        height: u32,
        receiver: mpsc::UnboundedReceiver<DisplayMessage>,
        callback_sender: Option<mpsc::UnboundedSender<DisplayCallbackMessage>>,
    ) -> Self {
        Self {
            window: None,
            context: None,
            surface: None,
            current_framebuffer: vec![0u32; (width * height) as usize],
            current_width: width,
            current_height: height,
            last_resize_time: std::time::Instant::now(),
            receiver,
            callback_sender,
        }
    }
}

impl ApplicationHandler for VirtualDisplayApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            info!(
                "Creating virtual display window ({}x{})",
                self.current_width, self.current_height
            );

            let window_attributes = Window::default_attributes()
                .with_title("RX X11 Server - Virtual Display")
                .with_inner_size(winit::dpi::PhysicalSize::new(
                    self.current_width,
                    self.current_height,
                ))
                .with_resizable(true);

            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Failed to create window"),
            );

            // Create softbuffer context and surface
            let context = softbuffer::Context::new(window.clone())
                .expect("Failed to create softbuffer context");
            let mut surface = softbuffer::Surface::new(&context, window.clone())
                .expect("Failed to create softbuffer surface");

            // Initialize the surface
            let width_nz = std::num::NonZeroU32::new(self.current_width).unwrap();
            let height_nz = std::num::NonZeroU32::new(self.current_height).unwrap();
            surface
                .resize(width_nz, height_nz)
                .expect("Failed to resize surface");

            self.window = Some(window);
            self.context = Some(context);
            self.surface = Some(surface);
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
                        event_loop.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        // Update the display with current framebuffer
                        if let Some(surface) = &mut self.surface {
                            if let Ok(mut buffer) = surface.buffer_mut() {
                                let copy_len =
                                    std::cmp::min(self.current_framebuffer.len(), buffer.len());
                                buffer[..copy_len]
                                    .copy_from_slice(&self.current_framebuffer[..copy_len]);
                                if let Err(e) = buffer.present() {
                                    error!("Failed to present buffer: {}", e);
                                }
                            }
                        }
                    }
                    WindowEvent::Resized(size) => {
                        info!(
                            "Virtual display received resize event to {}x{}",
                            size.width, size.height
                        );

                        let new_width = size.width;
                        let new_height = size.height;

                        // Skip resize if dimensions are invalid (e.g., window minimized)
                        if new_width == 0 || new_height == 0 {
                            return;
                        }

                        // Skip resize if dimensions are equal to current size
                        if new_width == self.current_width && new_height == self.current_height {
                            return;
                        }

                        // Throttle resize events - only process at most every 50ms
                        const RESIZE_THROTTLE_MS: u64 = 50;
                        let now = std::time::Instant::now();
                        let time_since_last =
                            now.duration_since(self.last_resize_time).as_millis() as u64;
                        if time_since_last < RESIZE_THROTTLE_MS {
                            // Always update current_width/current_height so the last event is kept
                            self.current_width = new_width;
                            self.current_height = new_height;
                            return; // Skip this resize event, but keep last event state
                        }

                        // Update throttling state
                        self.last_resize_time = now;

                        // Update stored dimensions
                        self.current_width = new_width;
                        self.current_height = new_height;

                        info!(
                            "Virtual display resized to {}x{} (throttled)",
                            new_width, new_height
                        );

                        // Resize the surface to match new dimensions
                        if let Some(surface) = &mut self.surface {
                            let width_nz = std::num::NonZeroU32::new(new_width).unwrap();
                            let height_nz = std::num::NonZeroU32::new(new_height).unwrap();
                            if let Err(e) = surface.resize(width_nz, height_nz) {
                                error!("Failed to resize surface: {}", e);
                            }
                        }

                        // Update the current framebuffer size
                        self.current_framebuffer
                            .resize((new_width * new_height) as usize, 0); // Send callback to notify protocol handler about resize
                        if let Some(ref callback_sender) = self.callback_sender {
                            if callback_sender
                                .send(DisplayCallbackMessage::WindowResized(new_width, new_height)).is_err()
                            {
                                debug!("Callback channel closed, resize notification not sent");
                            }
                        }

                        // Request a redraw to update the display
                        window.request_redraw();
                    }
                    _ => {}
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Check for messages from the protocol handler
        while let Ok(message) = self.receiver.try_recv() {
            match message {
                DisplayMessage::UpdateFramebuffer(framebuffer) => {
                    debug!("Updating virtual display framebuffer");
                    let copy_len = std::cmp::min(framebuffer.len(), self.current_framebuffer.len());
                    self.current_framebuffer[..copy_len].copy_from_slice(&framebuffer[..copy_len]);
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                DisplayMessage::Refresh => {
                    debug!("Refreshing virtual display");
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                DisplayMessage::Resize(width, height) => {
                    debug!("Processing resize message: {}x{}", width, height);
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                DisplayMessage::ResizeRenderer(width, height) => {
                    debug!("Renderer resize requested: {}x{}", width, height);
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                DisplayMessage::Shutdown => {
                    info!("Virtual display shutdown requested");
                    event_loop.exit();
                }
            }
        }
    }
}

/// Virtual display manager that runs in a separate thread
/// This is necessary because winit's EventLoop must run on the main thread
/// and cannot be used in an async context directly
pub struct VirtualDisplayManager {
    display_sender: Arc<Mutex<Option<mpsc::UnboundedSender<DisplayMessage>>>>,
    callback_sender: Arc<Mutex<Option<mpsc::UnboundedSender<DisplayCallbackMessage>>>>,
    width: Arc<Mutex<u32>>,
    height: Arc<Mutex<u32>>,
}

impl VirtualDisplayManager {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            display_sender: Arc::new(Mutex::new(None)),
            callback_sender: Arc::new(Mutex::new(None)),
            width: Arc::new(Mutex::new(width)),
            height: Arc::new(Mutex::new(height)),
        }
    }

    /// Start the virtual display in a separate thread
    /// This creates the EventLoop and runs it on a dedicated thread
    pub async fn start_display_thread(&self) -> ServerResult<()> {
        let (sender, receiver) = mpsc::unbounded_channel::<DisplayMessage>();

        // Store the sender for communication
        {
            let mut display_sender = self.display_sender.lock().await;
            *display_sender = Some(sender);
        }

        // Get the callback sender
        let callback_sender = self.callback_sender.lock().await.clone();

        let width = *self.width.lock().await;
        let height = *self.height.lock().await;

        // Spawn the display thread - this must run EventLoop.run_app() which blocks
        std::thread::spawn(move || {
            info!("Starting virtual display thread ({}x{})", width, height);

            // Create the event loop - this must be done on the thread that will run it
            // On Windows, we need to use any_thread() for cross-platform compatibility
            #[cfg(target_os = "windows")]
            let event_loop = EventLoop::builder().with_any_thread(true).build().unwrap();

            #[cfg(not(target_os = "windows"))]
            let event_loop = EventLoop::new().unwrap();

            // Create the application handler
            let mut app = VirtualDisplayApp::new(width, height, receiver, callback_sender);

            // Run the event loop with the application handler
            event_loop
                .run_app(&mut app)
                .expect("Event loop terminated unexpectedly");
        });

        info!("Virtual display thread started");
        Ok(())
    }

    /// Send a framebuffer update to the display
    pub async fn update_framebuffer(&self, framebuffer: Vec<u32>) -> ServerResult<()> {
        debug!("Sending framebuffer update");

        let display_sender = self.display_sender.lock().await;
        if let Some(sender) = display_sender.as_ref() {
            sender
                .send(DisplayMessage::UpdateFramebuffer(framebuffer))
                .map_err(|e| {
                    ServerError::NetworkError(format!("Failed to send framebuffer update: {}", e))
                })?;
        }
        Ok(())
    }

    /// Request a display refresh
    pub async fn refresh_display(&self) -> ServerResult<()> {
        let display_sender = self.display_sender.lock().await;
        if let Some(sender) = display_sender.as_ref() {
            sender
                .send(DisplayMessage::Refresh)
                .map_err(|e| ServerError::NetworkError(format!("Failed to send refresh: {}", e)))?;
        }
        Ok(())
    }

    /// Request a display resize
    pub async fn resize_display(&self, width: u32, height: u32) -> ServerResult<()> {
        let display_sender = self.display_sender.lock().await;
        if let Some(sender) = display_sender.as_ref() {
            sender
                .send(DisplayMessage::Resize(width, height))
                .map_err(|e| ServerError::NetworkError(format!("Failed to send resize: {}", e)))?;
        }
        Ok(())
    }

    /// Shutdown the virtual display
    pub async fn shutdown(&self) -> ServerResult<()> {
        let display_sender = self.display_sender.lock().await;
        if let Some(sender) = display_sender.as_ref() {
            sender.send(DisplayMessage::Shutdown).map_err(|e| {
                ServerError::NetworkError(format!("Failed to send shutdown: {}", e))
            })?;
        }
        Ok(())
    }

    pub async fn dimensions(&self) -> (u16, u16) {
        let width = *self.width.lock().await;
        let height = *self.height.lock().await;
        (width as u16, height as u16)
    }

    /// Update the stored dimensions
    pub async fn update_dimensions(&self, width: u32, height: u32) {
        *self.width.lock().await = width;
        *self.height.lock().await = height;
    }

    /// Set the callback sender for reverse communication
    pub async fn set_callback_sender(&self, sender: mpsc::UnboundedSender<DisplayCallbackMessage>) {
        let mut callback_sender = self.callback_sender.lock().await;
        *callback_sender = Some(sender);
    }
}

/// Protocol handler that displays content in a native window
pub struct VirtualDisplayProtocolHandler {
    plugins: Arc<PluginRegistry>,
    window_plugin: Arc<WindowPlugin>,
    renderer: Arc<Mutex<Renderer>>,
    virtual_display_manager: Arc<VirtualDisplayManager>,
}

impl VirtualDisplayProtocolHandler {
    pub fn new(plugins: Arc<PluginRegistry>, width: u32, height: u32) -> ServerResult<Self> {
        let window_plugin = Arc::new(WindowPlugin::new());
        let renderer = Arc::new(Mutex::new(Renderer::new(width, height, 24)));
        let virtual_display_manager = Arc::new(VirtualDisplayManager::new(width, height));

        info!(
            "Initializing virtual display protocol handler ({}x{})",
            width, height
        );

        Ok(Self {
            plugins,
            window_plugin,
            renderer,
            virtual_display_manager,
        })
    }
    /// Start the virtual display (call this after creation)
    pub async fn start_display(&self) -> ServerResult<()> {
        // Set up callback channel for receiving resize notifications
        let (callback_sender, mut callback_receiver) =
            mpsc::unbounded_channel::<DisplayCallbackMessage>();
        self.virtual_display_manager
            .set_callback_sender(callback_sender)
            .await;

        // Start the display thread
        self.virtual_display_manager.start_display_thread().await?;
        info!("Virtual display started");

        // Spawn a task to handle callback messages from the display thread
        let display_manager = Arc::clone(&self.virtual_display_manager);
        let renderer = Arc::clone(&self.renderer);
        tokio::spawn(async move {
            while let Some(message) = callback_receiver.recv().await {
                match message {
                    DisplayCallbackMessage::WindowResized(width, height) => {
                        info!("Received window resize callback: {}x{}", width, height);

                        // Update the display manager dimensions
                        display_manager.update_dimensions(width, height).await;

                        // Resize the renderer and redraw the pattern
                        {
                            let mut renderer = renderer.lock().await;
                            renderer.resize(width, height);
                            renderer.draw_rx_pattern(); // Redraw the pattern for the new size
                        }

                        // Update the framebuffer with the new renderer content
                        let framebuffer = {
                            let renderer = renderer.lock().await;
                            renderer.framebuffer().to_vec()
                        };

                        if let Err(e) = display_manager.update_framebuffer(framebuffer).await {
                            error!("Failed to update framebuffer after resize: {}", e);
                        }
                    }
                }
            }
        });

        // Initial refresh to show the default "rx" pattern
        self.refresh_display().await?;
        info!("Virtual display refreshed with initial pattern");
        Ok(())
    }

    /// Get display configuration for this virtual display
    pub async fn get_display_config(&self) -> crate::protocol::DisplayConfig {
        let (width, height) = self.virtual_display_manager.dimensions().await;
        crate::protocol::DisplayConfig {
            width,
            height,
            // Calculate millimeters assuming 96 DPI
            width_mm: (width as f32 / 96.0 * 25.4) as u16,
            height_mm: (height as f32 / 96.0 * 25.4) as u16,
            depth: 24, // Standard 24-bit color depth
        }
    }
    /// Refresh the virtual display with current renderer state
    async fn refresh_display(&self) -> ServerResult<()> {
        let renderer = self.renderer.lock().await;
        let framebuffer = renderer.framebuffer().to_vec();
        drop(renderer);

        self.virtual_display_manager
            .update_framebuffer(framebuffer)
            .await?;
        Ok(())
    }
}

impl Clone for VirtualDisplayProtocolHandler {
    fn clone(&self) -> Self {
        Self {
            plugins: Arc::clone(&self.plugins),
            window_plugin: Arc::clone(&self.window_plugin),
            renderer: Arc::clone(&self.renderer),
            virtual_display_manager: Arc::clone(&self.virtual_display_manager),
        }
    }
}

#[async_trait]
impl ProtocolHandler for VirtualDisplayProtocolHandler {
    async fn handle_request(
        &mut self,
        client_id: ClientId,
        request: Request,
    ) -> ServerResult<Option<Response>> {
        debug!(
            "VirtualDisplayProtocolHandler handling request from client {}: {:?}",
            client_id,
            request.opcode()
        );

        match request.opcode() {
            Opcode::CreateWindow => {
                debug!("Creating window for virtual display");
                let window_id: u32 = 0x1000001;

                // Update the virtual display to show the new window
                self.refresh_display().await?;

                let mut response_data = vec![0u8; 32];
                response_data[0..4].copy_from_slice(&window_id.to_le_bytes());
                Ok(Some(Response::new(1, response_data)))
            }
            Opcode::MapWindow => {
                debug!("Mapping window to virtual display");
                // Make window visible on the virtual display
                self.refresh_display().await?;
                // MapWindow doesn't return a response
                Ok(None)
            }
            Opcode::UnmapWindow => {
                debug!("Unmapping window from virtual display");
                // Hide window from virtual display
                self.refresh_display().await?;
                // UnmapWindow doesn't return a response
                Ok(None)
            }
            Opcode::DestroyWindow => {
                debug!("Destroying window on virtual display");
                // Remove window from virtual display
                self.refresh_display().await?;
                // DestroyWindow doesn't return a response
                Ok(None)
            }
            // Graphics operations - update the renderer
            Opcode::CreateGC => {
                debug!("Creating graphics context for virtual display");
                // CreateGC doesn't return a response - it's a request-only operation
                // The GC ID is provided by the client in the request
                Ok(None)
            }
            // Drawing operations - render to framebuffer and update display
            Opcode::PolyPoint
            | Opcode::PolyLine
            | Opcode::PolySegment
            | Opcode::PolyRectangle
            | Opcode::PolyArc
            | Opcode::FillPoly
            | Opcode::PolyFillRectangle
            | Opcode::PolyFillArc => {
                debug!("Drawing operation on virtual display");

                // TODO: Parse drawing parameters from request
                // TODO: Update renderer framebuffer based on request
                {
                    let _renderer = self.renderer.lock().await;
                    // Example: renderer.draw_something(...);
                    debug!("Drawing command processed");
                }

                // Refresh the virtual display with updated framebuffer
                self.refresh_display().await?;

                // Drawing operations don't return responses
                Ok(None)
            }

            _ => Err(ServerError::ProtocolError(
                crate::protocol::ProtocolError::UnimplementedOpcode(request.opcode()),
            )),
        }
    }

    fn supported_opcodes(&self) -> &[Opcode] {
        &[
            // Window management
            Opcode::CreateWindow,
            Opcode::DestroyWindow,
            Opcode::MapWindow,
            Opcode::UnmapWindow,
            Opcode::ConfigureWindow,
            // Graphics contexts
            Opcode::CreateGC,
            Opcode::FreeGC,
            Opcode::ChangeGC,
            // Drawing operations (rendered to virtual display)
            Opcode::PolyPoint,
            Opcode::PolyLine,
            Opcode::PolySegment,
            Opcode::PolyRectangle,
            Opcode::PolyArc,
            Opcode::FillPoly,
            Opcode::PolyFillRectangle,
            Opcode::PolyFillArc,
            // Text operations
            Opcode::PolyText8,
            Opcode::PolyText16,
            Opcode::ImageText8,
            Opcode::ImageText16,
        ]
    }
}
