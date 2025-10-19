// graphics.rs
//! Graphics rendering operations for X11 server

use crate::protocol::Arc;
use crate::server::window_system::Window;

/// Simple arc drawing functions
pub fn draw_arc(window: &mut Window, arc: &Arc, color: u32) {
    // Simple circle outline approximation
    let center_x = arc.x + arc.width as i16 / 2;
    let center_y = arc.y + arc.height as i16 / 2;
    let radius = (arc.width.min(arc.height) / 2) as i16;

    // Draw a simple circle outline using Bresenham's algorithm approximation
    for angle in 0..360 {
        let rad = angle as f32 * std::f32::consts::PI / 180.0;
        let x = center_x + (rad.cos() * radius as f32) as i16;
        let y = center_y + (rad.sin() * radius as f32) as i16;
        if x >= 0 && y >= 0 {
            window.set_pixel(x as u16, y as u16, color);
        }
    }
}

pub fn fill_arc(window: &mut Window, arc: &Arc, color: u32) {
    // Simple filled circle approximation
    let center_x = arc.x + arc.width as i16 / 2;
    let center_y = arc.y + arc.height as i16 / 2;
    let radius = (arc.width.min(arc.height) / 2) as i16;

    // Fill a simple circle using the midpoint circle algorithm
    for y in (center_y - radius)..=(center_y + radius) {
        for x in (center_x - radius)..=(center_x + radius) {
            let dx = x - center_x;
            let dy = y - center_y;
            if dx * dx + dy * dy <= radius * radius {
                if x >= 0 && y >= 0 {
                    window.set_pixel(x as u16, y as u16, color);
                }
            }
        }
    }
}