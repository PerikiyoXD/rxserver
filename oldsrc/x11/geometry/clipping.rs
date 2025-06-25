//! Clipping Algorithms
//!
//! This module provides clipping functionality for geometric operations.

use crate::x11::geometry::types::{LineSegment, Point, Rectangle};

/// Cohen-Sutherland line clipping algorithm
pub fn clip_line_to_rect(line: LineSegment, clip_rect: Rectangle) -> Option<LineSegment> {
    // Cohen-Sutherland outcodes
    const INSIDE: u8 = 0; // 0000
    const LEFT: u8 = 1; // 0001
    const RIGHT: u8 = 2; // 0010
    const BOTTOM: u8 = 4; // 0100
    const TOP: u8 = 8; // 1000

    fn compute_out_code(p: &Point, rect: &Rectangle) -> u8 {
        let mut code = INSIDE;
        if p.x < rect.x {
            code |= LEFT;
        } else if p.x >= rect.x + rect.width as i16 {
            code |= RIGHT;
        }
        if p.y < rect.y {
            code |= TOP;
        } else if p.y >= rect.y + rect.height as i16 {
            code |= BOTTOM;
        }
        code
    }

    let mut p0 = line.start;
    let mut p1 = line.end;
    let mut out_code0 = compute_out_code(&p0, &clip_rect);
    let mut out_code1 = compute_out_code(&p1, &clip_rect);

    let x_min = clip_rect.x;
    let x_max = clip_rect.x + clip_rect.width as i16;
    let y_min = clip_rect.y;
    let y_max = clip_rect.y + clip_rect.height as i16;

    loop {
        if (out_code0 | out_code1) == 0 {
            // Both endpoints are inside the rectangle
            return Some(LineSegment { start: p0, end: p1 });
        } else if (out_code0 & out_code1) != 0 {
            // Both endpoints are outside the rectangle in the same region
            return None;
        }

        // At least one endpoint is outside the rectangle; select it
        let out_code_out = if out_code0 != 0 { out_code0 } else { out_code1 };

        let (x, y) = if (out_code_out & TOP) != 0 {
            // Point is above the clip rectangle
            let x = if p1.y != p0.y {
                p0.x + (p1.x - p0.x) * (y_min - p0.y) / (p1.y - p0.y)
            } else {
                p0.x
            };
            (x, y_min)
        } else if (out_code_out & BOTTOM) != 0 {
            // Point is below the clip rectangle
            let x = if p1.y != p0.y {
                p0.x + (p1.x - p0.x) * (y_max - p0.y) / (p1.y - p0.y)
            } else {
                p0.x
            };
            (x, y_max)
        } else if (out_code_out & RIGHT) != 0 {
            // Point is to the right of clip rectangle
            let y = if p1.x != p0.x {
                p0.y + (p1.y - p0.y) * (x_max - p0.x) / (p1.x - p0.x)
            } else {
                p0.y
            };
            (x_max, y)
        } else {
            // Point is to the left of clip rectangle (LEFT)
            let y = if p1.x != p0.x {
                p0.y + (p1.y - p0.y) * (x_min - p0.x) / (p1.x - p0.x)
            } else {
                p0.y
            };
            (x_min, y)
        };

        let new_point = Point { x, y };

        if out_code_out == out_code0 {
            p0 = new_point;
            out_code0 = compute_out_code(&p0, &clip_rect);
        } else {
            p1 = new_point;
            out_code1 = compute_out_code(&p1, &clip_rect);
        }
    }
}

/// Sutherland-Hodgman polygon clipping algorithm
pub fn clip_polygon_to_rect(points: &[Point], clip_rect: Rectangle) -> Vec<Point> {
    if points.is_empty() {
        return Vec::new();
    }

    fn clip_edge<F, G>(input: &[Point], inside: F, intersect: G) -> Vec<Point>
    where
        F: Fn(&Point) -> bool,
        G: Fn(&Point, &Point) -> Point,
    {
        let mut output = Vec::new();
        if input.is_empty() {
            return output;
        }

        let mut prev = &input[input.len() - 1];
        for curr in input {
            let prev_inside = inside(prev);
            let curr_inside = inside(curr);

            if curr_inside {
                if !prev_inside {
                    output.push(intersect(prev, curr));
                }
                output.push(*curr);
            } else if prev_inside {
                output.push(intersect(prev, curr));
            }
            prev = curr;
        }
        output
    }

    let x_min = clip_rect.x;
    let x_max = clip_rect.x + clip_rect.width as i16;
    let y_min = clip_rect.y;
    let y_max = clip_rect.y + clip_rect.height as i16;

    let mut output = points.to_vec();

    // Clip against left edge
    output = clip_edge(
        &output,
        |p| p.x >= x_min,
        |p1, p2| {
            let dx = p2.x - p1.x;
            let dy = p2.y - p1.y;
            let x = x_min;
            let y = if dx != 0 {
                p1.y + dy * (x_min - p1.x) / dx
            } else {
                p1.y
            };
            Point { x, y }
        },
    );

    // Clip against right edge
    output = clip_edge(
        &output,
        |p| p.x <= x_max,
        |p1, p2| {
            let dx = p2.x - p1.x;
            let dy = p2.y - p1.y;
            let x = x_max;
            let y = if dx != 0 {
                p1.y + dy * (x_max - p1.x) / dx
            } else {
                p1.y
            };
            Point { x, y }
        },
    );

    // Clip against top edge
    output = clip_edge(
        &output,
        |p| p.y >= y_min,
        |p1, p2| {
            let dx = p2.x - p1.x;
            let dy = p2.y - p1.y;
            let y = y_min;
            let x = if dy != 0 {
                p1.x + dx * (y_min - p1.y) / dy
            } else {
                p1.x
            };
            Point { x, y }
        },
    );

    // Clip against bottom edge
    output = clip_edge(
        &output,
        |p| p.y <= y_max,
        |p1, p2| {
            let dx = p2.x - p1.x;
            let dy = p2.y - p1.y;
            let y = y_max;
            let x = if dy != 0 {
                p1.x + dx * (y_max - p1.y) / dy
            } else {
                p1.x
            };
            Point { x, y }
        },
    );

    // Only return valid polygons (at least 3 points)
    if output.len() >= 3 {
        output
    } else {
        Vec::new()
    }
}

/// Clip a rectangle to another rectangle
pub fn clip_rect_to_rect(rect: Rectangle, clip_rect: Rectangle) -> Option<Rectangle> {
    rect.intersection(&clip_rect)
}
