//! Window renderer for displaying framebuffers to actual windows
//!
//! This module provides software rendering capabilities to display virtual
//! X11 screens to actual operating system windows using winit and softbuffer.

use crate::{
    display::{
        framebuffer::Framebuffer, shared_framebuffers::SharedFramebuffers, types::DisplaySettings,
    },
    Result,
};
use std::{collections::HashMap, num::NonZeroU32, sync::Arc, time::Duration};
use tracing::{debug, info, warn};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
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

    /// Render a default pattern when no framebuffer is available
    pub fn render_default_pattern(&mut self) -> Result<()> {
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

        // Generate a simple pattern to show the display is working
        Self::generate_default_pattern(&mut buffer, window_size.width, window_size.height);

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

    /// Generate a default pattern to display when no framebuffer is available
    fn generate_default_pattern(buffer: &mut [u32], width: u32, height: u32) {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as f32
            * 0.001;

        for y in 0..height {
            for x in 0..width {
                let buffer_index = (y * width + x) as usize;
                if buffer_index < buffer.len() {
                    // Create a simple animated pattern
                    let fx = x as f32 / width as f32;
                    let fy = y as f32 / height as f32;

                    // Create animated checkerboard pattern
                    let checker_size = 32.0;
                    let cx = ((fx * width as f32) / checker_size + time * 2.0) as i32;
                    let cy = ((fy * height as f32) / checker_size + time * 1.5) as i32;
                    let checker = (cx + cy) % 2 == 0;

                    // Create animated colors
                    let r = ((fx + time * 0.5).sin() * 0.5 + 0.5) * 255.0;
                    let g = ((fy + time * 0.7).sin() * 0.5 + 0.5) * 255.0;
                    let b = ((fx + fy + time).sin() * 0.5 + 0.5) * 255.0;

                    let color = if checker {
                        // Bright checker squares
                        0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
                    } else {
                        // Darker checker squares
                        0xFF000000
                            | (((r * 0.3) as u32) << 16)
                            | (((g * 0.3) as u32) << 8)
                            | ((b * 0.3) as u32)
                    };

                    buffer[buffer_index] = color;
                }
            }
        }
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
    /// Screen renderers by screen ID
    screen_renderers: HashMap<u32, ScreenWindowRenderer>,
    /// Indicates if the renderer has been initialized
    initialized: bool,
}

impl WindowRenderer {
    /// Create a new window renderer
    pub fn new() -> Result<Self> {
        info!("Creating window renderer");

        Ok(Self {
            screen_renderers: HashMap::new(),
            initialized: false,
        })
    }

    /// Create windows for screens (to be called from main thread)
    pub fn create_windows(&mut self, display_settings: &DisplaySettings) -> Result<()> {
        info!(
            "Creating windows for {} screen(s)",
            display_settings.screens
        );

        if self.initialized {
            warn!("Windows already created");
            return Ok(());
        }

        let event_loop = EventLoop::new()
            .map_err(|e| crate::Error::Display(format!("Failed to create event loop: {}", e)))?;

        // Create windows for each screen
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
            self.screen_renderers.insert(screen_id, renderer);
        }

        self.initialized = true;
        info!("Windows created successfully");

        Ok(())
    }

    /// Run the window event loop (blocks current thread)
    pub fn run_event_loop(self, display_settings: &DisplaySettings) -> Result<()> {
        info!("Starting window event loop");

        let event_loop = EventLoop::new()
            .map_err(|e| crate::Error::Display(format!("Failed to create event loop: {}", e)))?;

        let mut screen_renderers = HashMap::new();

        // Create windows for each screen
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
            screen_renderers.insert(screen_id, renderer);
        }

        let mut last_render_time = std::time::Instant::now();
        let render_interval = Duration::from_millis(16); // ~60 FPS

        event_loop
            .run(move |event, elwt| {
                elwt.set_control_flow(ControlFlow::Poll);

                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        info!("Window close requested, shutting down renderer");
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
                            for renderer in screen_renderers.values() {
                                renderer.window().request_redraw();
                            }
                            last_render_time = now;
                        }
                    }
                    Event::WindowEvent {
                        event: WindowEvent::RedrawRequested,
                        window_id,
                    } => {
                        // Find the renderer for this window and render a default pattern
                        for renderer in screen_renderers.values_mut() {
                            if renderer.window().id() == window_id {
                                debug!("Redraw requested for screen {}", renderer.screen_id());
                                // Render a default pattern since no framebuffer is connected
                                if let Err(e) = renderer.render_default_pattern() {
                                    warn!("Failed to render default pattern: {}", e);
                                }
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            })
            .map_err(|e| crate::Error::Display(format!("Event loop error: {}", e)))?;
        info!("Window event loop finished");
        Ok(())
    }

    /// Run the window event loop with shared framebuffers (blocks current thread)
    pub fn run_event_loop_with_framebuffers(
        self,
        display_settings: &DisplaySettings,
        shared_framebuffers: SharedFramebuffers,
    ) -> Result<()> {
        info!("Starting window event loop with shared framebuffers");

        let event_loop = EventLoop::new()
            .map_err(|e| crate::Error::Display(format!("Failed to create event loop: {}", e)))?;

        let mut screen_renderers = HashMap::new();
        let framebuffer_handle = shared_framebuffers.clone_handle();

        // Create windows for each screen
        for screen_id in 0..display_settings.screens {
            let config = WindowRendererConfig {
                title: format!("RX Server - Screen {} (Display :0)", screen_id),
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
            screen_renderers.insert(screen_id, renderer);
        }

        let mut last_render_time = std::time::Instant::now();
        let render_interval = Duration::from_millis(16); // ~60 FPS
        let mut frame_count = 0u64;

        info!("🚀 Window event loop starting with framebuffer rendering");

        event_loop
            .run(move |event, elwt| {
                elwt.set_control_flow(ControlFlow::Poll);

                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        info!("🛑 Window close requested - shutting down");
                        elwt.exit();
                    }
                    Event::WindowEvent {
                        event: WindowEvent::Resized(size),
                        window_id,
                    } => {
                        debug!(
                            "📏 Window {:?} resized to {}x{}",
                            window_id, size.width, size.height
                        );
                    }
                    Event::AboutToWait => {
                        // Render at regular intervals
                        let now = std::time::Instant::now();
                        if now.duration_since(last_render_time) >= render_interval {
                            // Request redraw for all windows
                            for renderer in screen_renderers.values() {
                                renderer.window().request_redraw();
                            }
                            last_render_time = now;
                            frame_count += 1;

                            // Log frame rate occasionally
                            if frame_count % 300 == 0 {
                                info!("🖼️  Window renderer: {} frames rendered", frame_count);
                            }
                        }
                    }
                    Event::WindowEvent {
                        event: WindowEvent::RedrawRequested,
                        window_id,
                    } => {
                        // Find the renderer for this window and render shared framebuffer
                        for renderer in screen_renderers.values_mut() {
                            if renderer.window().id() == window_id {
                                // Get framebuffer from shared storage
                                if let Ok(fb_map) = framebuffer_handle.read() {
                                    if let Some(fb_arc) = fb_map.get(&renderer.screen_id()) {
                                        if let Ok(framebuffer) = fb_arc.read() {
                                            if let Err(e) =
                                                renderer.render_framebuffer(&*framebuffer)
                                            {
                                                warn!("Failed to render framebuffer: {}", e);
                                            }
                                        } else {
                                            // Fallback to default pattern if can't read framebuffer
                                            if let Err(e) = renderer.render_default_pattern() {
                                                warn!("Failed to render default pattern: {}", e);
                                            }
                                        }
                                    } else {
                                        // Fallback to default pattern if screen not found
                                        if let Err(e) = renderer.render_default_pattern() {
                                            warn!("Failed to render default pattern: {}", e);
                                        }
                                    }
                                } else {
                                    // Fallback to default pattern if can't read framebuffer map
                                    if let Err(e) = renderer.render_default_pattern() {
                                        warn!("Failed to render default pattern: {}", e);
                                    }
                                }
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            })
            .map_err(|e| crate::Error::Display(format!("Event loop error: {}", e)))?;

        info!("🖼️  Window event loop finished");
        Ok(())
    }

    /// Render a framebuffer to the appropriate screen window
    pub fn render_screen(&mut self, screen_id: u32, framebuffer: &Framebuffer) -> Result<()> {
        if let Some(renderer) = self.screen_renderers.get_mut(&screen_id) {
            renderer.render_framebuffer(framebuffer)?;
        } else {
            warn!("No renderer found for screen {}", screen_id);
        }

        Ok(())
    }

    /// Check if the renderer is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get the number of active screen renderers
    pub fn screen_count(&self) -> usize {
        self.screen_renderers.len()
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
