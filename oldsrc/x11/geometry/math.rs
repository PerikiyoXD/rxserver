//! Mathematical Utilities
//!
//! This module provides mathematical functions and utilities used throughout
//! the X11 server for geometric calculations and optimizations.

use crate::x11::geometry::types::Point;

/// Mathematical constants
pub mod constants {
    /// PI constant with sufficient precision for graphics calculations
    pub const PI: f64 = std::f64::consts::PI;

    /// 2 * PI
    pub const TWO_PI: f64 = 2.0 * PI;

    /// PI / 2
    pub const PI_2: f64 = PI / 2.0;

    /// PI / 4
    pub const PI_4: f64 = PI / 4.0;

    /// Square root of 2
    pub const SQRT_2: f64 = std::f64::consts::SQRT_2;

    /// 1 / Square root of 2
    pub const FRAC_1_SQRT_2: f64 = std::f64::consts::FRAC_1_SQRT_2;
}

/// Calculate the distance between two points
pub fn distance(p1: Point, p2: Point) -> f64 {
    let dx = (p2.x - p1.x) as f64;
    let dy = (p2.y - p1.y) as f64;
    (dx * dx + dy * dy).sqrt()
}

/// Calculate the squared distance between two points (avoids sqrt for performance)
pub fn distance_squared(p1: Point, p2: Point) -> i32 {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    (dx as i32) * (dx as i32) + (dy as i32) * (dy as i32)
}

/// Calculate the Manhattan distance between two points
pub fn manhattan_distance(p1: Point, p2: Point) -> i32 {
    (p2.x - p1.x).abs() as i32 + (p2.y - p1.y).abs() as i32
}

/// Linear interpolation between two values
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

/// Linear interpolation between two points
pub fn lerp_point(p1: Point, p2: Point, t: f64) -> Point {
    Point::new(
        lerp(p1.x as f64, p2.x as f64, t) as i16,
        lerp(p1.y as f64, p2.y as f64, t) as i16,
    )
}

/// Clamp a value between min and max
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Clamp a point to be within a rectangle
pub fn clamp_point_to_rect(
    point: Point,
    rect_x: i16,
    rect_y: i16,
    rect_width: u16,
    rect_height: u16,
) -> Point {
    Point::new(
        clamp(point.x, rect_x, rect_x + rect_width as i16 - 1),
        clamp(point.y, rect_y, rect_y + rect_height as i16 - 1),
    )
}

/// Convert degrees to radians
pub fn deg_to_rad(degrees: f64) -> f64 {
    degrees * constants::PI / 180.0
}

/// Convert radians to degrees
pub fn rad_to_deg(radians: f64) -> f64 {
    radians * 180.0 / constants::PI
}

/// Calculate the angle (in radians) from point p1 to point p2
pub fn angle_between_points(p1: Point, p2: Point) -> f64 {
    let dx = (p2.x - p1.x) as f64;
    let dy = (p2.y - p1.y) as f64;
    dy.atan2(dx)
}

/// Normalize an angle to be between 0 and 2π
pub fn normalize_angle(angle: f64) -> f64 {
    let mut normalized = angle % constants::TWO_PI;
    if normalized < 0.0 {
        normalized += constants::TWO_PI;
    }
    normalized
}

/// Calculate the cross product of two 2D vectors (returns scalar)
pub fn cross_product_2d(v1: Point, v2: Point) -> i32 {
    (v1.x as i32) * (v2.y as i32) - (v1.y as i32) * (v2.x as i32)
}

/// Calculate the dot product of two 2D vectors
pub fn dot_product_2d(v1: Point, v2: Point) -> i32 {
    (v1.x as i32) * (v2.x as i32) + (v1.y as i32) * (v2.y as i32)
}

/// Check if a point is to the left of a directed line segment
pub fn point_left_of_line(point: Point, line_start: Point, line_end: Point) -> bool {
    let line_vec = Point::new(line_end.x - line_start.x, line_end.y - line_start.y);
    let point_vec = Point::new(point.x - line_start.x, point.y - line_start.y);
    cross_product_2d(line_vec, point_vec) > 0
}

/// Calculate the area of a triangle formed by three points
pub fn triangle_area(p1: Point, p2: Point, p3: Point) -> f64 {
    let a = distance(p1, p2);
    let b = distance(p2, p3);
    let c = distance(p3, p1);

    // Heron's formula
    let s = (a + b + c) / 2.0;
    (s * (s - a) * (s - b) * (s - c)).sqrt()
}

/// Calculate the area of a triangle using the cross product (signed area)
pub fn triangle_area_signed(p1: Point, p2: Point, p3: Point) -> f64 {
    let v1 = Point::new(p2.x - p1.x, p2.y - p1.y);
    let v2 = Point::new(p3.x - p1.x, p3.y - p1.y);
    cross_product_2d(v1, v2) as f64 / 2.0
}

/// Check if three points are collinear (lie on the same line)
pub fn points_collinear(p1: Point, p2: Point, p3: Point) -> bool {
    triangle_area_signed(p1, p2, p3).abs() < f64::EPSILON
}

/// Find the centroid (center point) of a set of points
pub fn centroid(points: &[Point]) -> Option<Point> {
    if points.is_empty() {
        return None;
    }

    let sum_x: i32 = points.iter().map(|p| p.x as i32).sum();
    let sum_y: i32 = points.iter().map(|p| p.y as i32).sum();
    let count = points.len() as i32;

    Some(Point::new((sum_x / count) as i16, (sum_y / count) as i16))
}

/// Round a floating point value to the nearest integer
pub fn round_to_int(value: f64) -> i32 {
    (value + 0.5).floor() as i32
}

/// Fast integer square root using Newton's method
pub fn isqrt(n: u32) -> u32 {
    if n < 2 {
        return n;
    }

    let mut x = n;
    let mut y = (x + 1) / 2;

    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }

    x
}

/// Check if a number is a power of two
pub fn is_power_of_two(n: u32) -> bool {
    n != 0 && (n & (n - 1)) == 0
}

/// Round up to the next power of two
pub fn next_power_of_two(mut n: u32) -> u32 {
    if n == 0 {
        return 1;
    }

    n -= 1;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n + 1
}

/// Bresenham's line algorithm for integer line drawing
pub fn bresenham_line(start: Point, end: Point) -> Vec<Point> {
    let mut points = Vec::new();

    let dx = (end.x - start.x).abs();
    let dy = (end.y - start.y).abs();
    let sx = if start.x < end.x { 1 } else { -1 };
    let sy = if start.y < end.y { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = start.x;
    let mut y = start.y;

    loop {
        points.push(Point::new(x, y));

        if x == end.x && y == end.y {
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

    points
}

/// Midpoint circle algorithm for integer circle drawing
pub fn midpoint_circle(center: Point, radius: u16) -> Vec<Point> {
    let mut points = Vec::new();

    let mut x = 0i16;
    let mut y = radius as i16;
    let mut d = 1 - radius as i16;

    // Plot initial point
    let plot_points = |points: &mut Vec<Point>, cx: i16, cy: i16, x: i16, y: i16| {
        points.push(Point::new(cx + x, cy + y));
        points.push(Point::new(cx - x, cy + y));
        points.push(Point::new(cx + x, cy - y));
        points.push(Point::new(cx - x, cy - y));
        points.push(Point::new(cx + y, cy + x));
        points.push(Point::new(cx - y, cy + x));
        points.push(Point::new(cx + y, cy - x));
        points.push(Point::new(cx - y, cy - x));
    };

    plot_points(&mut points, center.x, center.y, x, y);

    while x < y {
        if d < 0 {
            d += 2 * x + 3;
        } else {
            d += 2 * (x - y) + 5;
            y -= 1;
        }
        x += 1;

        plot_points(&mut points, center.x, center.y, x, y);
    }

    points
}

/// Fixed-point arithmetic utilities for performance-critical calculations
pub mod fixed_point {
    /// Fixed-point number with 16 bits for fractional part
    pub type Fixed16 = i32;

    /// Number of fractional bits
    pub const FRACTIONAL_BITS: u32 = 16;

    /// Scale factor for conversion
    pub const SCALE: i32 = 1 << FRACTIONAL_BITS;

    /// Convert integer to fixed-point
    pub fn from_int(value: i32) -> Fixed16 {
        value << FRACTIONAL_BITS
    }

    /// Convert fixed-point to integer (truncated)
    pub fn to_int(value: Fixed16) -> i32 {
        value >> FRACTIONAL_BITS
    }

    /// Convert fixed-point to integer (rounded)
    pub fn to_int_round(value: Fixed16) -> i32 {
        (value + (SCALE / 2)) >> FRACTIONAL_BITS
    }

    /// Convert float to fixed-point
    pub fn from_float(value: f64) -> Fixed16 {
        (value * SCALE as f64) as i32
    }

    /// Convert fixed-point to float
    pub fn to_float(value: Fixed16) -> f64 {
        value as f64 / SCALE as f64
    }

    /// Multiply two fixed-point numbers
    pub fn multiply(a: Fixed16, b: Fixed16) -> Fixed16 {
        ((a as i64 * b as i64) >> FRACTIONAL_BITS) as i32
    }
    /// Divide two fixed-point numbers
    pub fn divide(a: Fixed16, b: Fixed16) -> Fixed16 {
        if b == 0 {
            panic!("Division by zero");
        }
        (((a as i64) << FRACTIONAL_BITS) / (b as i64)) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let p1 = Point::new(0, 0);
        let p2 = Point::new(3, 4);
        assert_eq!(distance(p1, p2), 5.0);
        assert_eq!(distance_squared(p1, p2), 25);
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn test_angle_conversion() {
        let angle_deg = 90.0;
        let angle_rad = deg_to_rad(angle_deg);
        assert!((angle_rad - constants::PI_2).abs() < f64::EPSILON);
        assert!((rad_to_deg(angle_rad) - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_triangle_area() {
        let p1 = Point::new(0, 0);
        let p2 = Point::new(1, 0);
        let p3 = Point::new(0, 1);
        assert_eq!(triangle_area_signed(p1, p2, p3), 0.5);
    }

    #[test]
    fn test_power_of_two() {
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(16));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(0));

        assert_eq!(next_power_of_two(3), 4);
        assert_eq!(next_power_of_two(16), 16);
        assert_eq!(next_power_of_two(17), 32);
    }

    #[test]
    fn test_fixed_point() {
        use fixed_point::*;

        let a = from_int(5);
        let b = from_int(2);
        let result = multiply(a, b);
        assert_eq!(to_int(result), 10);

        let float_val = 3.5;
        let fixed_val = from_float(float_val);
        assert!((to_float(fixed_val) - float_val).abs() < 0.001);
    }
}
