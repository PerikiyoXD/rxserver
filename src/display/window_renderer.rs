//! Window renderer for displaying framebuffers to actual windows
//!
//! This module provides software rendering capabilities to display virtual
//! X11 screens to actual operating system windows using winit and softbuffer.

use crate::{
    display::{framebuffer::Framebuffer, types::DisplaySettings},
    Result,
};
use std::{
    collections::HashMap,
    num::NonZeroU32,
    sync::{Arc, Mutex},
    time::Duration,
};
use tracing::{debug, info, warn};
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

/// Window renderer configuration
#[derive(Debug, Clone)]
pub struct WindowRendererConfig {
    /// Window title
    pub title: String,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Enable VSync
    pub vsync: bool,
    /// Target FPS (for frame limiting)
    pub target_fps: u32,
}

impl Default for WindowRendererConfig {
    fn default() -> Self {
        Self {
            title: "RX Server Display".to_string(),
            width: 1024,
            height: 768,
            vsync: true,
            target_fps: 60,
        }
    }
}

/// Individual window renderer for a single screen
pub struct ScreenWindowRenderer {
    /// The actual OS window
    window: Arc<Window>,
    /// Software buffer for rendering
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    /// Window configuration
    config: WindowRendererConfig,
    /// Screen ID this renderer is for
    screen_id: u32,
    /// Last frame time for FPS limiting
    last_frame_time: std::time::Instant,
}

impl ScreenWindowRenderer {
    /// Create a new screen window renderer
    pub fn new(window: Arc<Window>, config: WindowRendererConfig, screen_id: u32) -> Result<Self> {
        info!(
            "Creating window renderer for screen {} ({}x{})",
            screen_id, config.width, config.height
        );

        // Create software rendering surface
        let context = softbuffer::Context::new(window.clone()).map_err(|e| {
            crate::Error::Display(format!("Failed to create softbuffer context: {}", e))
        })?;

        let surface = softbuffer::Surface::new(&context, window.clone()).map_err(|e| {
            crate::Error::Display(format!("Failed to create softbuffer surface: {}", e))
        })?;

        Ok(Self {
            window,
            surface,
            config,
            screen_id,
            last_frame_time: std::time::Instant::now(),
        })
    }
    
    /// Render a framebuffer to this window
    pub fn render_framebuffer(&mut self, framebuffer: &Framebuffer) -> Result<()> {
        // Check if we should limit FPS
        if !self.config.vsync {
            let target_frame_duration = Duration::from_millis(1000 / self.config.target_fps as u64);
            let elapsed = self.last_frame_time.elapsed();
            if elapsed < target_frame_duration {
                return Ok(());
            }
        }

        let window_size = self.window.inner_size();

        // Resize surface if needed
        if let Err(e) = self.surface.resize(
            NonZeroU32::new(window_size.width).unwrap_or(NonZeroU32::new(1).unwrap()),
            NonZeroU32::new(window_size.height).unwrap_or(NonZeroU32::new(1).unwrap()),
        ) {
            warn!("Failed to resize surface: {}", e);
            return Ok(());
        }

        // Get the software buffer
        let mut buffer = self
            .surface
            .buffer_mut()
            .map_err(|e| crate::Error::Display(format!("Failed to get surface buffer: {}", e)))?;

        // Convert framebuffer data to display format
        Self::copy_framebuffer_to_buffer(
            framebuffer,
            &mut buffer,
            window_size.width,
            window_size.height,
        )?;

        // Present the buffer
        buffer
            .present()
            .map_err(|e| crate::Error::Display(format!("Failed to present buffer: {}", e)))?;

        self.last_frame_time = std::time::Instant::now();

        Ok(())
    }

    /// Copy framebuffer data to the window buffer
    fn copy_framebuffer_to_buffer(
        framebuffer: &Framebuffer,
        buffer: &mut [u32],
        window_width: u32,
        window_height: u32,
    ) -> Result<()> {
        let fb_config = framebuffer.config();
        let fb_width = fb_config.width;
        let fb_height = fb_config.height;

        // Calculate scaling factors
        let scale_x = window_width as f32 / fb_width as f32;
        let scale_y = window_height as f32 / fb_height as f32;

        // For better performance, get the entire framebuffer as RGBA32
        let fb_buffer = framebuffer.get_rgba32_buffer();

        // Simple nearest-neighbor scaling
        for window_y in 0..window_height {
            for window_x in 0..window_width {
                let fb_x = (window_x as f32 / scale_x) as u32;
                let fb_y = (window_y as f32 / scale_y) as u32;

                if fb_x < fb_width && fb_y < fb_height {
                    let fb_index = (fb_y * fb_width + fb_x) as usize;
                    let color = fb_buffer.get(fb_index).copied().unwrap_or(0xFF000000);

                    let buffer_index = (window_y * window_width + window_x) as usize;
                    if buffer_index < buffer.len() {
                        buffer[buffer_index] = color;
                    }
                } else {
                    // Fill with black for out-of-bounds
                    let buffer_index = (window_y * window_width + window_x) as usize;
                    if buffer_index < buffer.len() {
                        buffer[buffer_index] = 0xFF000000;
                    }
                }
            }
        }

        Ok(())
    }

    /// Get a pixel from the framebuffer (fallback implementation)
    fn get_framebuffer_pixel(&self, framebuffer: &Framebuffer, x: u32, y: u32) -> Result<u32> {
        framebuffer.get_pixel(x, y)
    }

    /// Get the window for this renderer
    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    /// Get the screen ID
    pub fn screen_id(&self) -> u32 {
        self.screen_id
    }
}

/// Main window renderer managing all screen windows
pub struct WindowRenderer {
    /// Event loop for window management
    event_loop: Option<EventLoop<()>>,
    /// Screen renderers by screen ID
    screen_renderers: Arc<Mutex<HashMap<u32, ScreenWindowRenderer>>>,
    /// Running state
    running: Arc<AtomicBool>,
    /// Renderer thread handle
    renderer_thread: Option<thread::JoinHandle<()>>,
}

impl WindowRenderer {
    /// Create a new window renderer
    pub fn new() -> Result<Self> {
        info!("Creating window renderer");

        let event_loop = EventLoop::new()
            .map_err(|e| crate::Error::Display(format!("Failed to create event loop: {}", e)))?;

        Ok(Self {
            event_loop: Some(event_loop),
            screen_renderers: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(AtomicBool::new(false)),
            renderer_thread: None,
        })
    }

    /// Start the window renderer
    pub fn start(&mut self, display_settings: &DisplaySettings) -> Result<()> {
        info!(
            "Starting window renderer with {} screen(s)",
            display_settings.screens
        );

        if self.running.load(Ordering::Relaxed) {
            warn!("Window renderer already running");
            return Ok(());
        }

        self.running.store(true, Ordering::Relaxed);

        // Create windows for each screen
        let event_loop = self
            .event_loop
            .take()
            .ok_or_else(|| crate::Error::Display("Event loop already taken".to_string()))?;

        let screen_renderers = self.screen_renderers.clone();
        let running = self.running.clone();

        // Create initial windows
        for screen_id in 0..display_settings.screens {
            let config = WindowRendererConfig {
                title: format!("RX Server - Screen {}", screen_id),
                width: display_settings.width,
                height: display_settings.height,
                ..Default::default()
            };

            let window = Arc::new(
                WindowBuilder::new()
                    .with_title(&config.title)
                    .with_inner_size(winit::dpi::LogicalSize::new(config.width, config.height))
                    .with_resizable(true)
                    .build(&event_loop)
                    .map_err(|e| {
                        crate::Error::Display(format!("Failed to create window: {}", e))
                    })?,
            );

            let renderer = ScreenWindowRenderer::new(window, config, screen_id)?;
            screen_renderers.lock().unwrap().insert(screen_id, renderer);
        }

        // Start the event loop in a separate thread
        let renderer_thread = thread::spawn(move || {
            info!("Window renderer thread started");

            let mut last_render_time = std::time::Instant::now();
            let render_interval = Duration::from_millis(16); // ~60 FPS

            event_loop
                .run(move |event, elwt| {
                    elwt.set_control_flow(ControlFlow::Poll);

                    if !running.load(Ordering::Relaxed) {
                        elwt.exit();
                        return;
                    }

                    match event {
                        Event::WindowEvent {
                            event: WindowEvent::CloseRequested,
                            ..
                        } => {
                            info!("Window close requested, shutting down renderer");
                            running.store(false, Ordering::Relaxed);
                            elwt.exit();
                        }
                        Event::WindowEvent {
                            event: WindowEvent::Resized(size),
                            window_id,
                        } => {
                            debug!("Window {:?} resized to {:?}", window_id, size);
                            // Handle window resize if needed
                        }
                        Event::AboutToWait => {
                            // Render at regular intervals
                            let now = std::time::Instant::now();
                            if now.duration_since(last_render_time) >= render_interval {
                                // Request redraw for all windows
                                let renderers = screen_renderers.lock().unwrap();
                                for renderer in renderers.values() {
                                    renderer.window().request_redraw();
                                }
                                last_render_time = now;
                            }
                        }
                        Event::WindowEvent {
                            event: WindowEvent::RedrawRequested,
                            window_id,
                        } => {
                            // Find the renderer for this window and render
                            let renderers = screen_renderers.lock().unwrap();
                            for renderer in renderers.values() {
                                if renderer.window().id() == window_id {
                                    // TODO: Get the actual framebuffer for this screen
                                    // For now, we'll just trigger a render with dummy data
                                    debug!("Redraw requested for screen {}", renderer.screen_id());
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                })
                .map_err(|e| {
                    error!("Event loop error: {}", e);
                })
                .ok();

            info!("Window renderer thread finished");
        });

        self.renderer_thread = Some(renderer_thread);
        info!("Window renderer started successfully");

        Ok(())
    }

    /// Render a framebuffer to the appropriate screen window
    pub fn render_screen(&self, screen_id: u32, framebuffer: &Framebuffer) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        let mut renderers = self.screen_renderers.lock().unwrap();
        if let Some(renderer) = renderers.get_mut(&screen_id) {
            renderer.render_framebuffer(framebuffer)?;
        } else {
            warn!("No renderer found for screen {}", screen_id);
        }

        Ok(())
    }

    /// Stop the window renderer
    pub fn stop(&mut self) -> Result<()> {
        info!("Stopping window renderer");

        if !self.running.load(Ordering::Relaxed) {
            debug!("Window renderer already stopped");
            return Ok(());
        }

        self.running.store(false, Ordering::Relaxed);

        // Wait for renderer thread to finish
        if let Some(thread) = self.renderer_thread.take() {
            thread
                .join()
                .map_err(|_| crate::Error::Display("Failed to join renderer thread".to_string()))?;
        }

        // Clear screen renderers
        self.screen_renderers.lock().unwrap().clear();

        info!("Window renderer stopped successfully");
        Ok(())
    }

    /// Check if the renderer is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Get the number of active screen renderers
    pub fn screen_count(&self) -> usize {
        self.screen_renderers.lock().unwrap().len()
    }
}

impl Drop for WindowRenderer {
    fn drop(&mut self) {
        if self.is_running() {
            let _ = self.stop();
        }
    }
}

/// Create a default window renderer configuration from display settings
impl From<&DisplaySettings> for WindowRendererConfig {
    fn from(settings: &DisplaySettings) -> Self {
        Self {
            title: "RX Server Display".to_string(),
            width: settings.width,
            height: settings.height,
            vsync: true,
            target_fps: 60,
        }
    }
}
