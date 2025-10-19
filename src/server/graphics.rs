// graphics.rs
//! Graphics rendering operations for X11 server

use crate::protocol::{Arc, Point, Rectangle};
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

pub fn draw_line(window: &mut Window, points: &[Point], color: u32) {
    if points.len() < 2 {
        return;
    }

    // Draw lines between consecutive points
    for i in 0..(points.len() - 1) {
        let p1 = &points[i];
        let p2 = &points[i + 1];
        draw_line_segment(window, p1.x, p1.y, p2.x, p2.y, color);
    }
}

pub fn fill_rectangle(window: &mut Window, rect: &Rectangle, color: u32) {
    for y in rect.y..(rect.y + rect.height as i16) {
        for x in rect.x..(rect.x + rect.width as i16) {
            if x >= 0 && y >= 0 {
                window.set_pixel(x as u16, y as u16, color);
            }
        }
    }
}

fn draw_line_segment(window: &mut Window, x1: i16, y1: i16, x2: i16, y2: i16, color: u32) {
    // Bresenham's line algorithm
    let mut x = x1;
    let mut y = y1;
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        if x >= 0 && y >= 0 {
            window.set_pixel(x as u16, y as u16, color);
        }

        if x == x2 && y == y2 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}