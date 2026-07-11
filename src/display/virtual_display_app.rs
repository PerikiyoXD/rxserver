use std::{collections::HashSet, sync::Arc};

use softbuffer::{Context, Surface};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::Instant,
};
use tracing::{debug, error, info};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
};

use crate::{
    display::{
        config::DisplayConfig,
        types::{DisplayCallbackMessage, DisplayMessage},
    },
    server::window_system::Window,
};

pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Virtual display application handler for winit
///
/// Allows spawning a native OS window to display X11 server output
///
/// Handles window management, rendering, and interaction with the X11 server
pub struct VirtualDisplayApp {
    window: Option<Arc<winit::window::Window>>,
    context: Option<Context<Arc<winit::window::Window>>>,
    surface: Option<Surface<Arc<winit::window::Window>, Arc<winit::window::Window>>>,
    framebuffer: Vec<u32>,
    config: DisplayConfig,
    last_resize_time: Instant,
    message_receiver: UnboundedReceiver<DisplayMessage>,
    callback_sender: Option<UnboundedSender<DisplayCallbackMessage>>,
    // Window rendering state
    windows: Vec<Window>,
    mapped_windows: HashSet<u32>, // WindowId
    closed: bool,
}

impl VirtualDisplayApp {
    pub fn new(
        config: DisplayConfig,
        message_receiver: UnboundedReceiver<DisplayMessage>,
        callback_sender: Option<UnboundedSender<DisplayCallbackMessage>>,
    ) -> Self {
        let resolution: [u32; 2] = config.resolution;
        let framebuffer_size: usize = (resolution[0] * resolution[1]) as usize;
        let mut mapped_windows: HashSet<u32> = HashSet::new();

        // Root window (ID 1) should be mapped by default
        mapped_windows.insert(1);

        Self {
            window: None,
            context: None,
            surface: None,
            framebuffer: vec![0x00000000; framebuffer_size], // Start with black background
            config,
            last_resize_time: Instant::now(),
            message_receiver,
            callback_sender,
            windows: Vec::new(),
            mapped_windows,
            closed: false,
        }
    }

    /// Render all windows to the framebuffer
    fn render_windows(&mut self) {
        let dimensions = self.config.resolution;
        let width = dimensions[0];
        let height = dimensions[1];

        // Clear framebuffer with default background
        let default_bg_color = 0x00000000; // Black background
        self.framebuffer.fill(default_bg_color);

        // Create a list of windows to render in proper order
        let mut windows_to_render = Vec::new();

        // Find and add root window first (should always be rendered, even if not explicitly mapped)
        if let Some(root_window) = self.windows.iter().find(|w| w.parent.is_none()) {
            debug!(
                "Found root window {} ({}x{} at {},{}) - always rendering",
                root_window.id, root_window.width, root_window.height, root_window.x, root_window.y
            );
            windows_to_render.push((root_window.clone(), 0, 0, true));
        } else {
            debug!(
                "No root window found in window list (count: {})",
                self.windows.len()
            );
        }

        // Add all other mapped windows in hierarchical order
        for window in &self.windows {
            if window.parent.is_some() {
                let is_mapped = self.mapped_windows.contains(&window.id);
                debug!("Child window {} - mapped: {}", window.id, is_mapped);
                if is_mapped {
                    windows_to_render.push((window.clone(), 0, 0, false));
                }
            }
        }

        debug!("Rendering {} windows total", windows_to_render.len());

        // Render all windows
        for (window, parent_x, parent_y, is_root) in windows_to_render {
            self.render_window(&window, parent_x, parent_y, width, height, is_root);
        }

        // Draw dev-time server info overlay (window count/status dots), opt-in only
        if self.config.debug_overlay {
            self.draw_server_info();
        }
    }

    /// Render a single window
    fn render_window(
        &mut self,
        window: &Window,
        parent_x: i32,
        parent_y: i32,
        max_width: u32,
        max_height: u32,
        is_root: bool,
    ) {
        // Get display width/height from config
        let (width, height) = (
            self.config.resolution[0] as u32,
            self.config.resolution[1] as u32,
        );

        let abs_x = parent_x + window.x as i32;
        let abs_y = parent_y + window.y as i32;

        // For root window, ensure it covers the entire display regardless of
        // the window's own declared size
        let actual_x = if is_root { 0 } else { abs_x };
        let actual_y = if is_root { 0 } else { abs_y };
        let actual_width = if is_root {
            max_width as i32
        } else {
            window.width as i32
        };
        let actual_height = if is_root {
            max_height as i32
        } else {
            window.height as i32
        };

        let window_rect = Rect {
            x: actual_x,
            y: actual_y,
            width: actual_width as u32,
            height: actual_height as u32,
        };

        // Determine window colors based on type (0x00RRGGBB, softbuffer's format)
        let (bg_color, border_color) = if is_root {
            // Root window - desktop background (dark blue-gray)
            (0x002E3440, 0x003B4252)
        } else {
            // Regular window - light background with visible border
            (0x00ECEFF4, 0x005E81AC) // Light gray with blue border
        };

        // debug!(
        //     "Rendering {} window {} at ({},{}) size {}x{}",
        //     if is_root { "root" } else { "child" },
        //     window.id,
        //     window_rect.x,
        //     window_rect.y,
        //     window_rect.width,
        //     window_rect.height
        // );

        // Draw window background and border
        for y in 0..window_rect.height {
            for x in 0..window_rect.width {
                let screen_x = window_rect.x + x as i32;
                let screen_y = window_rect.y + y as i32;

                if screen_x >= 0
                    && screen_x < width as i32
                    && screen_y >= 0
                    && screen_y < height as i32
                {
                    let index = (screen_y as u32 * width + screen_x as u32) as usize;
                    if index < self.framebuffer.len() {
                        if !is_root
                            && window.border_width > 0
                            && (x < window.border_width as u32
                                || x >= (actual_width - window.border_width as i32) as u32
                                || y < window.border_width as u32
                                || y >= (actual_height - window.border_width as i32) as u32)
                        {
                            // Draw border for non-root windows
                            self.framebuffer[index] = border_color;
                        } else {
                            // Draw background
                            self.framebuffer[index] = bg_color;
                        }
                    }
                }
            }
        }

        // Draw window content for non-root windows
        if !is_root {
            self.draw_window_content(window, actual_x, actual_y);
        } else {
            // For root window, draw pixel data
            self.draw_window_content(window, actual_x, actual_y);
        }
    }

    /// Draw content inside a window
    fn draw_window_content(&mut self, window: &Window, abs_x: i32, abs_y: i32) {
        let (width, height) = (
            self.config.resolution[0] as u32,
            self.config.resolution[1] as u32,
        );

        // Calculate content area (inside border)
        let content_x = abs_x + window.border_width as i32;
        let content_y = abs_y + window.border_width as i32;
        let content_width = window.width.saturating_sub(window.border_width * 2);
        let content_height = window.height.saturating_sub(window.border_width * 2);

        // Draw pixel data from the window
        for y in 0..content_height {
            for x in 0..content_width {
                let screen_x = content_x + x as i32;
                let screen_y = content_y + y as i32;

                if screen_x >= 0
                    && screen_x < width as i32
                    && screen_y >= 0
                    && screen_y < height as i32
                {
                    // Get pixel from window's pixel data (0xAARRGGBB) and drop the
                    // alpha byte to match softbuffer's 0x00RRGGBB framebuffer format
                    if let Some(pixel) = window.get_pixel(x, y) {
                        let index = (screen_y as u32 * width + screen_x as u32) as usize;
                        if index < self.framebuffer.len() {
                            self.framebuffer[index] = pixel & 0x00FF_FFFF;
                        }
                    }
                }
            }
        }
    }
    /// Draw server information overlay
    fn draw_server_info(&mut self) {
        let (width, height) = (
            self.config.resolution[0] as u32,
            self.config.resolution[1] as u32,
        );

        // Draw RX X11 Server text in the top-left corner (0x00RRGGBB, softbuffer has no alpha)
        let text_color = 0x00D8DEE9; // Light color
        let text_bg = 0x002E3440; // Background

        // Simple text rendering - draw "RX X11" as a pattern
        for y in 10..25 {
            for x in 10..120 {
                if x < width && y < height {
                    let index = (y * width + x) as usize;
                    if index < self.framebuffer.len() {
                        // Draw background for text area
                        if x == 10 || x == 119 || y == 10 || y == 24 {
                            self.framebuffer[index] = text_color;
                        } else {
                            self.framebuffer[index] = text_bg;
                        }
                    }
                }
            }
        }

        // Draw window count info
        let window_count = self.windows.len();
        let mapped_count = self.mapped_windows.len();

        // Draw simple indicators for window count (dots)
        for i in 0..window_count.min(10) {
            let dot_x = 10 + i as u32 * 6;
            let dot_y = 30;
            if dot_x < width && dot_y < height {
                let index = (dot_y * width + dot_x) as usize;
                if index < self.framebuffer.len() {
                    self.framebuffer[index] = if i < mapped_count {
                        0x0088C0D0 // Bright blue for mapped windows
                    } else {
                        0x004C566A // Gray for unmapped windows
                    };
                }
            }
        }

        // Draw status indicators for root window and children
        let status_y = 35;
        let status_colors = [
            0x0088C0D0, // Blue for active
            0x00BF616A, // Red for inactive
            0x00A3BE8C, // Green for mapped
            0x00EBCB8B, // Yellow for created
        ];

        // Status line: Root window indicator
        if let Some(_root_window) = self.windows.iter().find(|w| w.parent.is_none()) {
            for i in 0..4 {
                let status_x = 10 + i as u32 * 8;
                if status_x < width && status_y < height {
                    let index = (status_y * width + status_x) as usize;
                    if index < self.framebuffer.len() {
                        self.framebuffer[index] = status_colors[0]; // Root window is always active
                    }
                }
            }
        }

        // Child window status indicators
        let child_windows: Vec<_> = self.windows.iter().filter(|w| w.parent.is_some()).collect();
        for (i, window) in child_windows.iter().take(8).enumerate() {
            let status_x = 50 + i as u32 * 8;
            if status_x < width && status_y < height {
                let index = (status_y * width + status_x) as usize;
                if index < self.framebuffer.len() {
                    self.framebuffer[index] = if self.mapped_windows.contains(&window.id) {
                        status_colors[2] // Green for mapped
                    } else {
                        status_colors[3] // Yellow for created but not mapped
                    };
                }
            }
        }
    }

    /// Draw a simple test pattern to the framebuffer (legacy method)
    fn draw_test_pattern(&mut self) {
        // For backward compatibility, this now calls render_windows
        self.render_windows();
    }

    /// The winit window ID for this display, once its window has been created.
    pub fn window_id(&self) -> Option<winit::window::WindowId> {
        self.window.as_ref().map(|w| w.id())
    }

    /// Whether this display's window has been closed (by the user or a
    /// shutdown message) and it no longer needs to receive events.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Create this display's window. Safe to call multiple times; a no-op
    /// once the window already exists.
    pub fn create_window(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!(
            "Creating RX X11 Server virtual display window ({}x{})",
            self.config.resolution[0], self.config.resolution[1]
        );

        let window_attributes = winit::window::Window::default_attributes()
            .with_title("RX X11 Server - Virtual Display")
            .with_inner_size(winit::dpi::PhysicalSize::new(
                self.config.resolution[0] as u32,
                self.config.resolution[1] as u32,
            ))
            .with_resizable(true);

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create virtual display window"),
        );

        // Create softbuffer context and surface for rendering
        let context = Context::new(window.clone()).expect("Failed to create softbuffer context");
        let mut surface =
            Surface::new(&context, window.clone()).expect("Failed to create softbuffer surface");

        // Initialize surface with current configuration
        let width_nz = std::num::NonZeroU32::new(self.config.resolution[0] as u32).unwrap();
        let height_nz = std::num::NonZeroU32::new(self.config.resolution[1] as u32).unwrap();
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

    /// Handle a window event addressed to this display's window.
    pub fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Virtual display window close requested");
                if let Some(ref callback_sender) = self.callback_sender {
                    let _ = callback_sender.send(DisplayCallbackMessage::DisplayClosed);
                }
                self.closed = true;
                let _ = event_loop;
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
                let new_width = size.width as u32;
                let new_height = size.height as u32;

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

                // Update configuration
                self.config.resolution = [new_width, new_height];

                // Resize surface
                if let Some(surface) = &mut self.surface {
                    let width_nz = std::num::NonZeroU32::new(new_width).unwrap();
                    let height_nz = std::num::NonZeroU32::new(new_height).unwrap();
                    if let Err(e) = surface.resize(width_nz, height_nz) {
                        error!("Failed to resize surface: {}", e);
                        return;
                    }
                } // Resize framebuffer
                let new_size = (new_width * new_height) as usize;
                self.framebuffer.resize(new_size, 0x00000000);

                // Notify the server about the resize
                if let Some(ref callback_sender) = self.callback_sender {
                    let _ = callback_sender
                        .send(DisplayCallbackMessage::WindowResized(new_width, new_height));
                }

                // Draw test pattern and request redraw outside the borrow scope
                self.draw_test_pattern();
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(ref callback_sender) = self.callback_sender {
                    let _ = callback_sender.send(DisplayCallbackMessage::PointerMoved(
                        position.x as i32,
                        position.y as i32,
                    ));
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                // X11 button numbering: 1=left, 2=middle, 3=right, 4/5=wheel.
                // Wheel deltas arrive via WindowEvent::MouseWheel, not here -
                // MouseInput only covers "real" buttons winit models this way.
                let x11_button = match button {
                    MouseButton::Left => 1,
                    MouseButton::Middle => 2,
                    MouseButton::Right => 3,
                    MouseButton::Back => 8,
                    MouseButton::Forward => 9,
                    MouseButton::Other(code) => code as u8,
                };
                if let Some(ref callback_sender) = self.callback_sender {
                    let message = match state {
                        ElementState::Pressed => {
                            DisplayCallbackMessage::PointerButtonPressed(x11_button)
                        }
                        ElementState::Released => {
                            DisplayCallbackMessage::PointerButtonReleased(x11_button)
                        }
                    };
                    let _ = callback_sender.send(message);
                }
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(key_code),
                    state,
                    ..
                },
                ..
            } => {
                if let Some(ref callback_sender) = self.callback_sender {
                    let scancode = key_code as u32;
                    let message = match state {
                        ElementState::Pressed => DisplayCallbackMessage::KeyPressed(scancode),
                        ElementState::Released => DisplayCallbackMessage::KeyReleased(scancode),
                    };
                    let _ = callback_sender.send(message);
                }
            }
            _ => {}
        }
    }

    /// Drain pending `DisplayMessage`s and apply them. Exposed so a
    /// multi-display manager can pump every registered display each time the
    /// event loop is idle.
    pub fn pump_messages(&mut self) {
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
                DisplayMessage::UpdateWindows(new_windows) => {
                    debug!(
                        "Updating virtual display windows (count: {})",
                        new_windows.len()
                    );
                    self.windows = new_windows;
                    self.render_windows();

                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                DisplayMessage::WindowCreated(window_state) => {
                    debug!("Window created: ID {}", window_state.id);
                    self.windows.push(window_state);
                    self.render_windows();

                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                DisplayMessage::WindowMapped(window_id) => {
                    debug!("Window mapped: ID {}", window_id);
                    self.mapped_windows.insert(window_id);
                    self.render_windows();

                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                DisplayMessage::WindowUnmapped(window_id) => {
                    debug!("Window unmapped: ID {}", window_id);
                    self.mapped_windows.remove(&window_id);
                    self.render_windows();

                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                DisplayMessage::WindowDestroyed(window_id) => {
                    debug!("Window destroyed: ID {}", window_id);
                    self.windows.retain(|w| w.id != window_id);
                    self.mapped_windows.remove(&window_id);
                    self.render_windows();

                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                DisplayMessage::Resize(width, height) => {
                    debug!("Received resize request: {}x{}", width, height);
                    // Let the window system handle resizing naturally
                    if let Some(window) = &self.window {
                        let _ =
                            window.request_inner_size(winit::dpi::PhysicalSize::new(width, height));
                    }
                }
                DisplayMessage::Shutdown => {
                    info!("Virtual display shutdown requested");
                    if let Some(ref callback_sender) = self.callback_sender {
                        let _ = callback_sender.send(DisplayCallbackMessage::DisplayClosed);
                    }
                    self.closed = true;
                }
            }
        }
    }
}

impl ApplicationHandler for VirtualDisplayApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_window(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if self.window_id() != Some(window_id) {
            return;
        }
        self.handle_window_event(event_loop, event);
        if self.closed {
            event_loop.exit();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.pump_messages();
        if self.closed {
            event_loop.exit();
        }
    }
}
