//! Enhanced window renderer test with animated updates
//!
//! This test demonstrates live framebuffer updates being rendered to the window

use rxserver::{
    display::{
        framebuffer::{Framebuffer, FramebufferConfig, PixelFormat},
        window_renderer::{ScreenWindowRenderer, WindowRendererConfig},
    },
    Result,
};
use std::{sync::Arc, time::Instant};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rxserver=info")
        .with_target(true)
        .init();

    println!("🚀 Starting Animated Window Renderer Test");

    // Create event loop (must be on main thread)
    let event_loop = EventLoop::new()
        .map_err(|e| rxserver::Error::Display(format!("Failed to create event loop: {}", e)))?;

    // Create window
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("RX Server - Animated Test")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
            .with_resizable(true)
            .build(&event_loop)
            .map_err(|e| rxserver::Error::Display(format!("Failed to create window: {}", e)))?,
    );

    // Create renderer config
    let config = WindowRendererConfig {
        title: "RX Server - Animated Test".to_string(),
        width: 800,
        height: 600,
        vsync: true,
        target_fps: 60,
    };

    // Create screen renderer
    let mut screen_renderer = ScreenWindowRenderer::new(window.clone(), config, 0)?;

    // Create a test framebuffer
    let fb_config = FramebufferConfig {
        width: 400,
        height: 300,
        bpp: 32,
        stride: 400 * 4, // 4 bytes per pixel for RGBA32
        format: PixelFormat::RGBA32,
        software: true,
        scanline_pad: 32,
        little_endian: true,
    };

    let framebuffer = Framebuffer::new(fb_config)?;

    let mut last_render_time = Instant::now();
    let render_interval = std::time::Duration::from_millis(16); // ~60 FPS
    let mut frame_count = 0u64;
    let start_time = Instant::now();

    println!("🖼️  Starting animated event loop");
    println!("💫 You should see animated patterns updating in real-time");

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("🛑 Window close requested - shutting down");
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                println!("📏 Window resized to {}x{}", size.width, size.height);
            }
            Event::AboutToWait => {
                // Render at regular intervals
                let now = Instant::now();
                if now.duration_since(last_render_time) >= render_interval {
                    // Update framebuffer with animated pattern
                    update_framebuffer_animation(&framebuffer, frame_count);
                    
                    window.request_redraw();
                    last_render_time = now;
                    frame_count += 1;

                    // Log frame rate occasionally
                    if frame_count % 300 == 0 {
                        let elapsed = start_time.elapsed().as_secs_f32();
                        let fps = frame_count as f32 / elapsed;
                        println!("🖼️  Frame {}, FPS: {:.1}, Elapsed: {:.1}s", frame_count, fps, elapsed);
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Render the framebuffer
                if let Err(e) = screen_renderer.render_framebuffer(&framebuffer) {
                    eprintln!("Failed to render framebuffer: {}", e);
                }
            }
            _ => {}
        }
    })
    .map_err(|e| rxserver::Error::Display(format!("Event loop error: {}", e)))?;

    println!("🖼️  Test completed");
    Ok(())
}

/// Update the framebuffer with an animated pattern
fn update_framebuffer_animation(framebuffer: &Framebuffer, frame: u64) {
    let time = frame as f32 * 0.02; // Animation speed

    for y in 0..300 {
        for x in 0..400 {
            let fx = x as f32 / 400.0;
            let fy = y as f32 / 300.0;

            // Create animated plasma effect
            let v1 = (fx * 16.0 + time).sin();
            let v2 = ((fx * 8.0 + fy * 6.0) + time * 1.2).sin();
            let v3 = ((fx - 0.5).powi(2) + (fy - 0.5).powi(2)).sqrt() * 32.0 + time * 2.0;
            let v4 = v3.sin();

            let plasma = (v1 + v2 + v4) / 3.0;

            // Convert to RGB
            let r = ((plasma + 1.0) * 0.5 * 255.0) as u8;
            let g = ((plasma.sin() + 1.0) * 0.5 * 255.0) as u8;
            let b = ((plasma.cos() + 1.0) * 0.5 * 255.0) as u8;

            let color = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            
            if let Err(_) = framebuffer.set_pixel(x, y, color) {
                // Ignore pixel set errors for this demo
            }
        }
    }
}
