// graphics.rs
//! Graphics rendering operations for X11 server

use crate::protocol::{Arc, Point, Rectangle};
use crate::server::pixmap_system::Pixmap;
use crate::server::window_system::{Background, Window};

/// Common surface drawing operations write to. xproto's DRAWABLE type is a
/// union of WINDOW and PIXMAP (encoding.xml, types:DRAWABLE) - requests like
/// PolyFillRectangle accept either, so handlers shouldn't have to duplicate
/// their fill logic per drawable kind. `Window`/`Pixmap` already have
/// identical `set_pixel` primitives; this just lets code written against one
/// work against both.
pub trait Drawable {
    fn set_pixel(&mut self, x: u16, y: u16, color: u32);
}

impl Drawable for Window {
    fn set_pixel(&mut self, x: u16, y: u16, color: u32) {
        Window::set_pixel(self, x, y, color);
    }
}

impl Drawable for Pixmap {
    fn set_pixel(&mut self, x: u16, y: u16, color: u32) {
        Pixmap::set_pixel(self, x, y, color);
    }
}

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

pub fn fill_rectangle(drawable: &mut impl Drawable, rect: &Rectangle, color: u32) {
    for y in rect.y..(rect.y + rect.height as i16) {
        for x in rect.x..(rect.x + rect.width as i16) {
            if x >= 0 && y >= 0 {
                drawable.set_pixel(x as u16, y as u16, color);
            }
        }
    }
}

/// ClearArea's fill logic (xproto sect1-9.xml, ClearArea): x/y are relative
/// to the window origin; a zero width/height is replaced with "to the far
/// edge of the window" (not "zero pixels"); the fill itself follows the
/// window's resolved `Background` - `None` means leave pixels untouched,
/// `Pixmap` tiles the background pixmap starting at the window origin (not
/// at the cleared rectangle's origin - that's what makes it a consistent
/// tile across separate clears), and `Pixel`/`ParentRelative`-resolved-to-Pixel
/// is a flat fill.
///
/// `background_pixmap` must be `Some` exactly when `background` is
/// `Background::Pixmap(_)` - the caller looks it up by ID before calling
/// this, since `PixmapSystem` lives outside `Window`.
pub fn clear_area(
    window: &mut Window,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    background: Background,
    background_pixmap: Option<&Pixmap>,
) {
    let effective_width = if width == 0 {
        (window.width as i16 - x).max(0) as u16
    } else {
        width
    };
    let effective_height = if height == 0 {
        (window.height as i16 - y).max(0) as u16
    } else {
        height
    };

    match background {
        Background::None => {
            // Contents left unchanged, per spec.
        }
        Background::ParentRelative => {
            // Callers resolve ParentRelative to a concrete background via
            // WindowSystem::resolve_background before reaching here - this
            // arm exists so the match stays exhaustive if that invariant is
            // ever violated, rather than silently doing the wrong fill.
            debug_assert!(
                false,
                "clear_area received unresolved Background::ParentRelative"
            );
        }
        Background::Pixel(color) => {
            let rect = Rectangle {
                x,
                y,
                width: effective_width,
                height: effective_height,
            };
            fill_rectangle(window, &rect, color);
        }
        Background::Pixmap(_) => {
            let Some(pixmap) = background_pixmap else {
                return;
            };
            for py in y..(y + effective_height as i16) {
                for px in x..(x + effective_width as i16) {
                    if px < 0 || py < 0 {
                        continue;
                    }
                    // Tile relative to the window origin (0,0), not the
                    // cleared rectangle, so repeated ClearArea calls on
                    // different sub-rectangles show one continuous tiling.
                    let tile_x = (px as u32) % (pixmap.width as u32);
                    let tile_y = (py as u32) % (pixmap.height as u32);
                    let index = tile_y as usize * pixmap.width as usize + tile_x as usize;
                    if let Some(&color) = pixmap.pixel_data.get(index) {
                        window.set_pixel(px as u16, py as u16, color);
                    }
                }
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
