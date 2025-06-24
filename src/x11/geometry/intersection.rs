//! Geometric Intersection Algorithms
//!
//! This module provides algorithms for calculating intersections between geometric shapes.

use crate::x11::geometry::types::{Circle, LineSegment, Point, Rectangle};

/// Check if two line segments intersect
pub fn line_segments_intersect(line1: LineSegment, line2: LineSegment) -> bool {
    line_segment_intersection(line1, line2).is_some()
}

/// Find the intersection point of two line segments
pub fn line_segment_intersection(line1: LineSegment, line2: LineSegment) -> Option<Point> {
    let p1 = line1.start;
    let q1 = line1.end;
    let p2 = line2.start;
    let q2 = line2.end;

    // Convert to vectors for calculation
    let s1_x = q1.x - p1.x;
    let s1_y = q1.y - p1.y;
    let s2_x = q2.x - p2.x;
    let s2_y = q2.y - p2.y;

    // Calculate determinant
    let det = -s2_x * s1_y + s1_x * s2_y;

    if det == 0 {
        // Lines are parallel
        return None;
    }

    let s = (-s1_y * (p1.x - p2.x) + s1_x * (p1.y - p2.y)) / det;
    let t = (s2_x * (p1.y - p2.y) - s2_y * (p1.x - p2.x)) / det;

    if s >= 0 && s <= 1 && t >= 0 && t <= 1 {
        // Intersection detected
        let intersection_x = p1.x + (t * s1_x);
        let intersection_y = p1.y + (t * s1_y);
        Some(Point::new(intersection_x, intersection_y))
    } else {
        None
    }
}

/// Check if a line segment intersects a rectangle
pub fn line_intersects_rect(line: LineSegment, rect: Rectangle) -> bool {
    // First check if either endpoint is inside the rectangle
    if rect.contains_point(line.start) || rect.contains_point(line.end) {
        return true;
    }

    // Check intersection with each edge of the rectangle
    let top_left = Point::new(rect.x, rect.y);
    let top_right = Point::new(rect.x + rect.width as i16, rect.y);
    let bottom_left = Point::new(rect.x, rect.y + rect.height as i16);
    let bottom_right = Point::new(rect.x + rect.width as i16, rect.y + rect.height as i16);

    let edges = [
        LineSegment {
            start: top_left,
            end: top_right,
        }, // Top edge
        LineSegment {
            start: top_right,
            end: bottom_right,
        }, // Right edge
        LineSegment {
            start: bottom_right,
            end: bottom_left,
        }, // Bottom edge
        LineSegment {
            start: bottom_left,
            end: top_left,
        }, // Left edge
    ];

    edges
        .iter()
        .any(|&edge| line_segments_intersect(line, edge))
}

/// Check if a circle intersects a rectangle
pub fn circle_intersects_rect(circle: Circle, rect: Rectangle) -> bool {
    // Find the closest point on the rectangle to the circle center
    let closest_x = circle.center.x.clamp(rect.x, rect.x + rect.width as i16);
    let closest_y = circle.center.y.clamp(rect.y, rect.y + rect.height as i16);

    let closest_point = Point::new(closest_x, closest_y);

    // Calculate the distance from the circle center to this closest point
    let distance_squared = circle.center.distance_squared_to(closest_point);
    let radius_squared = (circle.radius as i32) * (circle.radius as i32);

    distance_squared <= radius_squared
}

/// Check if a point is on a line segment (within tolerance)
pub fn point_on_line_segment(point: Point, line: LineSegment, tolerance: i16) -> bool {
    let tolerance_squared = (tolerance as i32) * (tolerance as i32);

    // Check if point is within the bounding box of the line segment
    let min_x = line.start.x.min(line.end.x) - tolerance;
    let max_x = line.start.x.max(line.end.x) + tolerance;
    let min_y = line.start.y.min(line.end.y) - tolerance;
    let max_y = line.start.y.max(line.end.y) + tolerance;

    if point.x < min_x || point.x > max_x || point.y < min_y || point.y > max_y {
        return false;
    }

    // Calculate distance from point to line segment
    let line_length_squared = line.start.distance_squared_to(line.end);

    if line_length_squared == 0 {
        // Line segment is a point
        return point.distance_squared_to(line.start) <= tolerance_squared;
    }

    // Calculate the parameter t for the closest point on the line
    let dx = line.end.x - line.start.x;
    let dy = line.end.y - line.start.y;
    let px = point.x - line.start.x;
    let py = point.y - line.start.y;

    let t = ((px as i32 * dx as i32 + py as i32 * dy as i32) as f64 / line_length_squared as f64)
        .clamp(0.0, 1.0);

    // Find the closest point on the line segment
    let closest_x = line.start.x + (t * dx as f64) as i16;
    let closest_y = line.start.y + (t * dy as f64) as i16;
    let closest_point = Point::new(closest_x, closest_y);

    point.distance_squared_to(closest_point) <= tolerance_squared
}

/// Check if two rectangles intersect
pub fn rectangles_intersect(rect1: &Rectangle, rect2: &Rectangle) -> bool {
    rect1.intersects(rect2)
}

/// Check if two circles intersect
pub fn circles_intersect(circle1: Circle, circle2: Circle) -> bool {
    let distance_squared = circle1.center.distance_squared_to(circle2.center);
    let radii_sum = circle1.radius as i32 + circle2.radius as i32;
    let radii_sum_squared = radii_sum * radii_sum;

    distance_squared <= radii_sum_squared
}

/// Find all intersection points between a circle and a line segment
pub fn circle_line_intersections(circle: Circle, line: LineSegment) -> Vec<Point> {
    let mut intersections = Vec::new();

    let dx = line.end.x - line.start.x;
    let dy = line.end.y - line.start.y;
    let fx = line.start.x - circle.center.x;
    let fy = line.start.y - circle.center.y;

    let a = dx as f64 * dx as f64 + dy as f64 * dy as f64;
    let b = 2.0 * (fx as f64 * dx as f64 + fy as f64 * dy as f64);
    let c = (fx as f64 * fx as f64 + fy as f64 * fy as f64)
        - (circle.radius as f64 * circle.radius as f64);

    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        // No intersection
        return intersections;
    }

    let sqrt_discriminant = discriminant.sqrt();

    if discriminant == 0.0 {
        // One intersection (tangent)
        let t = -b / (2.0 * a);
        if t >= 0.0 && t <= 1.0 {
            let x = line.start.x + (t * dx as f64) as i16;
            let y = line.start.y + (t * dy as f64) as i16;
            intersections.push(Point::new(x, y));
        }
    } else {
        // Two intersections
        let t1 = (-b - sqrt_discriminant) / (2.0 * a);
        let t2 = (-b + sqrt_discriminant) / (2.0 * a);

        if t1 >= 0.0 && t1 <= 1.0 {
            let x = line.start.x + (t1 * dx as f64) as i16;
            let y = line.start.y + (t1 * dy as f64) as i16;
            intersections.push(Point::new(x, y));
        }

        if t2 >= 0.0 && t2 <= 1.0 {
            let x = line.start.x + (t2 * dx as f64) as i16;
            let y = line.start.y + (t2 * dy as f64) as i16;
            intersections.push(Point::new(x, y));
        }
    }

    intersections
}

#[cfg(test)]
mod tests {
    use crate::x11::geometry::clipping::{clip_line_to_rect, clip_polygon_to_rect};

    use super::*;

    #[test]
    fn test_line_segment_intersection() {
        let line1 = LineSegment {
            start: Point::new(0, 0),
            end: Point::new(10, 10),
        };
        let line2 = LineSegment {
            start: Point::new(0, 10),
            end: Point::new(10, 0),
        };

        let intersection = line_segment_intersection(line1, line2);
        assert!(intersection.is_some());
        let point = intersection.unwrap();
        assert_eq!(point, Point::new(5, 5));
    }

    #[test]
    fn test_line_intersects_rect() {
        let line = LineSegment {
            start: Point::new(-5, 5),
            end: Point::new(15, 5),
        };
        let rect = Rectangle::new(0, 0, 10, 10);

        assert!(line_intersects_rect(line, rect));

        let line_outside = LineSegment {
            start: Point::new(-5, -5),
            end: Point::new(-2, -2),
        };
        assert!(!line_intersects_rect(line_outside, rect));
    }

    #[test]
    fn test_circle_intersects_rect() {
        let circle = Circle {
            center: Point::new(5, 5),
            radius: 3,
        };
        let rect = Rectangle::new(0, 0, 10, 10);

        assert!(circle_intersects_rect(circle, rect));

        let circle_outside = Circle {
            center: Point::new(20, 20),
            radius: 3,
        };
        assert!(!circle_intersects_rect(circle_outside, rect));
    }

    #[test]
    fn test_clip_line_to_rect() {
        let line = LineSegment {
            start: Point::new(-5, 5),
            end: Point::new(15, 5),
        };
        let rect = Rectangle::new(0, 0, 10, 10);

        let clipped = clip_line_to_rect(line, rect);
        assert!(clipped.is_some());
        let clipped_line = clipped.unwrap();
        assert_eq!(clipped_line.start, Point::new(0, 5));
        assert_eq!(clipped_line.end, Point::new(10, 5));
    }

    #[test]
    fn test_clip_polygon_to_rect() {
        let triangle = vec![Point::new(-5, 5), Point::new(15, 5), Point::new(5, 15)];
        let rect = Rectangle::new(0, 0, 10, 10);

        let clipped = clip_polygon_to_rect(&triangle, rect);
        assert!(!clipped.is_empty());
        assert!(clipped.len() >= 3);
    }
}
