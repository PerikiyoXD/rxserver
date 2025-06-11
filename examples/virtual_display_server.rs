//! Virtual Display X11 Server
//!
//! This example demonstrates a complete X11 server that provides a virtual display
//! with visual output rendered to a window. It properly handles winit's main thread
//! requirement and can display X11 applications in a virtual screen environment.

use rxserver::{
    config::ServerConfig,
    display::{
        shared_framebuffers::SharedFramebuffers,
        types::{DisplaySettings, FramebufferSettings, VisualClass},
    },
    server::XServer,
    Result,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tracing::{error, info, warn};

fn main() -> Result<()> {
    // Initialize logging with detailed trace information
    tracing_subscriber::fmt()
        .with_env_filter("rxserver=trace,tokio=info,winit=info")
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    info!("🚀 Starting Virtual Display RX Server");

    // Create display settings
    let display_settings = DisplaySettings {
        width: 1024,
        height: 768,
        screens: 1,
        depth: 24,
        dpi: 96,
        visual_class: VisualClass::TrueColor,
        hw_acceleration: false,
        framebuffer: FramebufferSettings {
            software: true,
            bpp: 32,
            scanline_pad: 32,
            little_endian: true,
        },
    };

    // Create shared framebuffers for communication between server and renderer
    let shared_framebuffers = SharedFramebuffers::new(display_settings.clone())?;
    info!("📺 Shared framebuffers created for inter-thread communication");

    // Create shutdown coordination
    let running = Arc::new(AtomicBool::new(true));
    let running_server = running.clone();
    let running_demo = running.clone();    // Clone framebuffers for different threads
    let _demo_framebuffers = shared_framebuffers.clone_handle();
    let _server_framebuffers = shared_framebuffers.clone_handle();

    // Create tokio runtime for background tasks
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Start X11 server in background thread
    let server_handle = rt.spawn(async move {
        info!("🔧 Starting X11 server on background thread");

        let config = ServerConfig::default();

        let server = match XServer::new(":0".to_string(), config).await {
            Ok(server) => {
                info!("✅ X11 server created successfully on display :0");
                info!("🏃 X11 server listening on TCP port 6000");
                server
            }
            Err(e) => {
                error!("❌ Failed to create X11 server: {}", e);
                running_server.store(false, Ordering::Relaxed);
                return Err(e);
            }
        };

        // Run the server
        let result = server.run().await;
        running_server.store(false, Ordering::Relaxed);

        match &result {
            Ok(_) => info!("✅ X11 server completed normally"),
            Err(e) => error!("❌ X11 server error: {}", e),
        }

        result
    });    // Start demo content generation in background thread (DISABLED FOR TESTING)
    /*
    let demo_display_settings = display_settings.clone();
    let demo_handle = rt.spawn(async move {
        info!("🎨 Starting animated demo content generator");

        let mut frame_count = 0u32;
        let start_time = Instant::now();

        while running_demo.load(Ordering::Relaxed) {
            // Generate animated demo content
            if let Err(e) =
                generate_demo_content(&demo_framebuffers, frame_count, &demo_display_settings)
            {
                warn!("Demo content generation error: {}", e);
            }

            frame_count = frame_count.wrapping_add(1);

            // Update at ~30 FPS
            tokio::time::sleep(Duration::from_millis(33)).await;

            // Log stats every 5 seconds
            if frame_count % 150 == 0 {
                let elapsed = start_time.elapsed().as_secs_f32();
                let fps = frame_count as f32 / elapsed;
                info!(
                    "📊 Demo stats: Frame {}, FPS: {:.1}, Running: {:.1}s",
                    frame_count, fps, elapsed
                );
            }
        }

        info!("🎨 Demo content generator stopped");
    });
    */
    
    // Create a dummy handle for demo that just sleeps
    let demo_handle = rt.spawn(async move {
        info!("🎨 Demo content generator DISABLED - will show X11 content only");
        while running_demo.load(Ordering::Relaxed) {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        info!("🎨 Demo content generator stopped");
    });    info!("🖼️  Starting window renderer on main thread");
    info!(
        "   📱 Resolution: {}x{}",
        display_settings.width, display_settings.height
    );
    info!("   🎨 Color depth: {} bits", display_settings.depth);
    info!("   📺 Screens: {}", display_settings.screens);
    info!("");
    info!("💡 Demo content DISABLED - window will show X11 content only");
    info!("💡 X11 applications can connect to display :0");
    info!("💡 Try: xcalc.exe -display 127.0.0.1:0");
    info!("💡 Close the window or press Ctrl+C to exit");
    info!("");

    // Run window renderer on main thread (required by winit)
    let window_result = run_main_window_loop(shared_framebuffers, &display_settings);

    // Signal shutdown to background tasks
    running.store(false, Ordering::Relaxed);

    info!("🔄 Shutting down background tasks...");

    // Wait for background tasks to complete
    rt.block_on(async {
        tokio::select! {
            result = server_handle => {
                match result {
                    Ok(Ok(_)) => info!("✅ X11 server shutdown completed"),
                    Ok(Err(e)) => error!("❌ X11 server shutdown error: {}", e),
                    Err(e) => error!("❌ Failed to join server task: {}", e),
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(3)) => {
                warn!("⏰ Server shutdown timeout");
            }
        }

        tokio::select! {
            _ = demo_handle => {
                info!("✅ Demo content generator stopped");
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                warn!("⏰ Demo shutdown timeout");
            }
        }
    });

    match window_result {
        Ok(_) => info!("🏁 Virtual Display RX Server shutdown successful"),
        Err(e) => error!("❌ Window renderer error: {}", e),
    }

    Ok(())
}

/*
/// Generate animated demo content in the shared framebuffers  
fn generate_demo_content(
    framebuffers: &Arc<
        std::sync::RwLock<std::collections::HashMap<u32, Arc<std::sync::RwLock<Framebuffer>>>>,
    >,
    frame: u32,
    settings: &DisplaySettings,
) -> Result<()> {
    let time = frame as f32 * 0.03; // Animation speed

    // Get framebuffer for screen 0
    if let Ok(fb_map) = framebuffers.read() {
        if let Some(fb_arc) = fb_map.get(&0) {
            if let Ok(framebuffer) = fb_arc.write() {
                // Generate animated plasma-like pattern
                for y in 0..settings.height {
                    for x in 0..settings.width {
                        let fx = x as f32 / settings.width as f32;
                        let fy = y as f32 / settings.height as f32;

                        // Create animated plasma effect
                        let v1 = (fx * 16.0 + time).sin();
                        let v2 = ((fx * 8.0 + fy * 6.0) + time * 1.2).sin();
                        let v3 =
                            ((fx - 0.5).powi(2) + (fy - 0.5).powi(2)).sqrt() * 20.0 + time * 0.8;
                        let v4 = (v1 + v2 + v3.sin()).sin();

                        let intensity = (v4 + 1.0) * 0.5; // Normalize to 0-1

                        // Create color based on position and time
                        let r = ((intensity * 255.0) * (1.0 + (time * 0.1).sin()) * 0.5) as u8;
                        let g =
                            ((intensity * 255.0) * (1.0 + (time * 0.13 + 2.0).sin()) * 0.5) as u8;
                        let b =
                            ((intensity * 255.0) * (1.0 + (time * 0.17 + 4.0).sin()) * 0.5) as u8;

                        // Create RGBA32 color
                        let color =
                            0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);                        let _ = framebuffer.set_pixel(x, y, color);
                    }
                }
            }
        }
    }

    Ok(())
}
*/

/// Main window rendering loop (must run on main thread)
fn run_main_window_loop(
    framebuffers: SharedFramebuffers,
    display_settings: &DisplaySettings,
) -> Result<()> {
    use std::{collections::HashMap, sync::Arc as StdArc};
    use winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };

    info!("🖼️  Creating window event loop");

    let event_loop = EventLoop::new()
        .map_err(|e| rxserver::Error::Display(format!("Failed to create event loop: {}", e)))?;

    let mut screen_renderers = HashMap::new();
    let framebuffer_handle = framebuffers.clone_handle();

    // Create windows for each screen
    for screen_id in 0..display_settings.screens {
        let window = StdArc::new(
            WindowBuilder::new()
                .with_title(&format!("RX Server - Screen {} (Display :0)", screen_id))
                .with_inner_size(winit::dpi::LogicalSize::new(
                    display_settings.width,
                    display_settings.height,
                ))
                .with_resizable(true)
                .build(&event_loop)
                .map_err(|e| rxserver::Error::Display(format!("Failed to create window: {}", e)))?,
        );

        let config = rxserver::display::window_renderer::WindowRendererConfig {
            title: format!("RX Server - Screen {}", screen_id),
            width: display_settings.width,
            height: display_settings.height,
            vsync: true,
            target_fps: 60,
        };

        let renderer = rxserver::display::window_renderer::ScreenWindowRenderer::new(
            window, config, screen_id,
        )?;

        screen_renderers.insert(screen_id, renderer);
        info!("✅ Created window for screen {}", screen_id);
    }

    let mut last_render_time = Instant::now();
    let render_interval = Duration::from_millis(16); // ~60 FPS
    let mut frame_count = 0u64;

    info!("🚀 Window event loop starting - windows should be visible now!");
    info!("💫 You should see animated plasma effects in the window");

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
                    info!(
                        "📏 Window {:?} resized to {}x{}",
                        window_id, size.width, size.height
                    );
                }
                Event::AboutToWait => {
                    // Render at regular intervals
                    let now = Instant::now();
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
                                        if let Err(e) = renderer.render_framebuffer(&*framebuffer) {
                                            warn!("Failed to render framebuffer: {}", e);
                                        }
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
                _ => {}
            }
        })
        .map_err(|e| rxserver::Error::Display(format!("Event loop error: {}", e)))?;

    info!("🖼️  Window event loop finished");
    Ok(())
}
