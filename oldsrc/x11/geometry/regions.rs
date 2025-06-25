//! Complex Region Operations
//!
//! This module provides algorithms for complex region operations used in X11
//! for damage tracking, clipping, and window management.

use crate::x11::geometry::types::{Point, Rectangle};

/// A complex region made up of multiple non-overlapping rectangles
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    rectangles: Vec<Rectangle>,
}

impl Region {
    /// Create an empty region
    pub fn new() -> Self {
        Self {
            rectangles: Vec::new(),
        }
    }

    /// Create a region from a single rectangle
    pub fn from_rect(rect: Rectangle) -> Self {
        Self {
            rectangles: vec![rect],
        }
    }

    /// Create a region from multiple rectangles
    pub fn from_rects(rects: Vec<Rectangle>) -> Self {
        let mut region = Self::new();
        for rect in rects {
            region = region.union_rect(rect);
        }
        region
    }

    /// Check if the region is empty
    pub fn is_empty(&self) -> bool {
        self.rectangles.is_empty()
    }

    /// Get the number of rectangles in the region
    pub fn rect_count(&self) -> usize {
        self.rectangles.len()
    }

    /// Get the rectangles that make up this region
    pub fn rectangles(&self) -> &[Rectangle] {
        &self.rectangles
    }

    /// Get the bounding box of the entire region
    pub fn bounds(&self) -> Option<Rectangle> {
        if self.rectangles.is_empty() {
            return None;
        }

        let mut min_x = i16::MAX;
        let mut min_y = i16::MAX;
        let mut max_x = i16::MIN;
        let mut max_y = i16::MIN;

        for rect in &self.rectangles {
            min_x = min_x.min(rect.x);
            min_y = min_y.min(rect.y);
            max_x = max_x.max(rect.x + rect.width as i16);
            max_y = max_y.max(rect.y + rect.height as i16);
        }

        Some(Rectangle::new(
            min_x,
            min_y,
            (max_x - min_x) as u16,
            (max_y - min_y) as u16,
        ))
    }

    /// Check if a point is contained within the region
    pub fn contains_point(&self, point: Point) -> bool {
        self.rectangles
            .iter()
            .any(|rect| rect.contains_point(point))
    }

    /// Check if a rectangle is entirely contained within the region
    pub fn contains_rect(&self, rect: Rectangle) -> bool {
        // For each point of the rectangle, check if it's in the region
        // This is a simplified check - a more sophisticated implementation
        // would do proper geometric containment
        let corners = [
            Point::new(rect.x, rect.y),
            Point::new(rect.x + rect.width as i16, rect.y),
            Point::new(rect.x, rect.y + rect.height as i16),
            Point::new(rect.x + rect.width as i16, rect.y + rect.height as i16),
        ];

        corners.iter().all(|&corner| self.contains_point(corner))
    }

    /// Union this region with another region
    pub fn union(&self, other: &Region) -> Region {
        let mut result = self.clone();
        for rect in &other.rectangles {
            result = result.union_rect(*rect);
        }
        result
    }

    /// Union this region with a rectangle
    pub fn union_rect(&self, rect: Rectangle) -> Region {
        let mut rectangles = self.rectangles.clone();
        rectangles.push(rect);
        Self::merge_rectangles(rectangles)
    }

    /// Intersect this region with another region
    pub fn intersect(&self, other: &Region) -> Region {
        let mut result_rects = Vec::new();

        for rect1 in &self.rectangles {
            for rect2 in &other.rectangles {
                if let Some(intersection) = rect1.intersection(rect2) {
                    result_rects.push(intersection);
                }
            }
        }

        Self::merge_rectangles(result_rects)
    }

    /// Intersect this region with a rectangle
    pub fn intersect_rect(&self, rect: Rectangle) -> Region {
        let mut result_rects = Vec::new();

        for self_rect in &self.rectangles {
            if let Some(intersection) = self_rect.intersection(&rect) {
                result_rects.push(intersection);
            }
        }

        Region {
            rectangles: result_rects,
        }
    }

    /// Subtract another region from this region
    pub fn subtract(&self, other: &Region) -> Region {
        let mut result = self.clone();

        for rect in &other.rectangles {
            result = result.subtract_rect(*rect);
        }

        result
    }

    /// Subtract a rectangle from this region
    pub fn subtract_rect(&self, rect: Rectangle) -> Region {
        let mut result_rects = Vec::new();

        for &self_rect in &self.rectangles {
            // Split self_rect by subtracting rect
            let split_rects = subtract_rect_from_rect(self_rect, rect);
            result_rects.extend(split_rects);
        }

        Region {
            rectangles: result_rects,
        }
    }

    /// Translate the region by the given offset
    pub fn translate(&self, dx: i16, dy: i16) -> Region {
        let translated_rects: Vec<Rectangle> = self
            .rectangles
            .iter()
            .map(|rect| Rectangle::new(rect.x + dx, rect.y + dy, rect.width, rect.height))
            .collect();

        Region {
            rectangles: translated_rects,
        }
    }

    /// Merge overlapping rectangles to minimize the number of rectangles
    fn merge_rectangles(mut rectangles: Vec<Rectangle>) -> Region {
        if rectangles.is_empty() {
            return Region::new();
        }

        // Sort rectangles by position for more efficient merging
        rectangles.sort_by(|a, b| a.y.cmp(&b.y).then_with(|| a.x.cmp(&b.x)));

        let mut merged = Vec::new();
        let mut current = rectangles[0];

        for &next in &rectangles[1..] {
            if can_merge_rectangles(current, next) {
                current = merge_two_rectangles(current, next);
            } else {
                merged.push(current);
                current = next;
            }
        }
        merged.push(current);

        // Multiple passes may be needed for complete merging
        if merged.len() < rectangles.len() && merged.len() > 1 {
            Self::merge_rectangles(merged)
        } else {
            Region { rectangles: merged }
        }
    }
}

impl Default for Region {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if two rectangles can be merged into a single rectangle
fn can_merge_rectangles(rect1: Rectangle, rect2: Rectangle) -> bool {
    // Check for horizontal adjacency
    if rect1.y == rect2.y && rect1.height == rect2.height {
        if rect1.x + rect1.width as i16 == rect2.x || rect2.x + rect2.width as i16 == rect1.x {
            return true;
        }
    }

    // Check for vertical adjacency
    if rect1.x == rect2.x && rect1.width == rect2.width {
        if rect1.y + rect1.height as i16 == rect2.y || rect2.y + rect2.height as i16 == rect1.y {
            return true;
        }
    }

    // Check for overlap
    rect1.intersects(&rect2)
}

/// Merge two rectangles into one (assumes they can be merged)
fn merge_two_rectangles(rect1: Rectangle, rect2: Rectangle) -> Rectangle {
    let min_x = rect1.x.min(rect2.x);
    let min_y = rect1.y.min(rect2.y);
    let max_x = (rect1.x + rect1.width as i16).max(rect2.x + rect2.width as i16);
    let max_y = (rect1.y + rect1.height as i16).max(rect2.y + rect2.height as i16);

    Rectangle::new(min_x, min_y, (max_x - min_x) as u16, (max_y - min_y) as u16)
}

/// Subtract one rectangle from another, returning the remaining rectangles
fn subtract_rect_from_rect(source: Rectangle, subtract: Rectangle) -> Vec<Rectangle> {
    if let Some(intersection) = source.intersection(&subtract) {
        let mut result = Vec::new();

        // Top rectangle
        if intersection.y > source.y {
            result.push(Rectangle::new(
                source.x,
                source.y,
                source.width,
                (intersection.y - source.y) as u16,
            ));
        }

        // Bottom rectangle
        let intersection_bottom = intersection.y + intersection.height as i16;
        let source_bottom = source.y + source.height as i16;
        if intersection_bottom < source_bottom {
            result.push(Rectangle::new(
                source.x,
                intersection_bottom,
                source.width,
                (source_bottom - intersection_bottom) as u16,
            ));
        }

        // Left rectangle
        if intersection.x > source.x {
            result.push(Rectangle::new(
                source.x,
                intersection.y,
                (intersection.x - source.x) as u16,
                intersection.height,
            ));
        }

        // Right rectangle
        let intersection_right = intersection.x + intersection.width as i16;
        let source_right = source.x + source.width as i16;
        if intersection_right < source_right {
            result.push(Rectangle::new(
                intersection_right,
                intersection.y,
                (source_right - intersection_right) as u16,
                intersection.height,
            ));
        }

        result
    } else {
        // No intersection, return the original rectangle
        vec![source]
    }
}

/// Convert a region to a simplified form with minimal rectangles
pub fn simplify_region(region: &Region) -> Region {
    Region::merge_rectangles(region.rectangles.clone())
}

/// Create a region from a polygon by scanline conversion
pub fn region_from_polygon(points: &[Point]) -> Region {
    if points.len() < 3 {
        return Region::new();
    }

    // Find bounding box
    let min_y = points.iter().map(|p| p.y).min().unwrap();
    let max_y = points.iter().map(|p| p.y).max().unwrap();

    let mut rectangles = Vec::new();

    // Scanline algorithm
    for y in min_y..=max_y {
        let mut intersections = Vec::new();

        // Find intersections with polygon edges
        for i in 0..points.len() {
            let p1 = points[i];
            let p2 = points[(i + 1) % points.len()];

            if (p1.y <= y && p2.y > y) || (p2.y <= y && p1.y > y) {
                // Edge crosses scanline
                let x = if p2.y != p1.y {
                    p1.x + (p2.x - p1.x) * (y - p1.y) / (p2.y - p1.y)
                } else {
                    p1.x
                };
                intersections.push(x);
            }
        }

        // Sort intersections
        intersections.sort();

        // Create rectangles for spans between pairs of intersections
        for chunk in intersections.chunks_exact(2) {
            if let [x1, x2] = chunk {
                if x2 > x1 {
                    rectangles.push(Rectangle::new(*x1, y, (x2 - x1) as u16, 1));
                }
            }
        }
    }

    Region::merge_rectangles(rectangles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_union() {
        let rect1 = Rectangle::new(0, 0, 10, 10);
        let rect2 = Rectangle::new(5, 5, 10, 10);

        let region1 = Region::from_rect(rect1);
        let region2 = Region::from_rect(rect2);

        let union = region1.union(&region2);
        assert!(!union.is_empty());

        // Test points in the union
        assert!(union.contains_point(Point::new(2, 2))); // In rect1
        assert!(union.contains_point(Point::new(12, 12))); // In rect2
        assert!(union.contains_point(Point::new(7, 7))); // In intersection
    }

    #[test]
    fn test_region_intersection() {
        let rect1 = Rectangle::new(0, 0, 10, 10);
        let rect2 = Rectangle::new(5, 5, 10, 10);

        let region1 = Region::from_rect(rect1);
        let region2 = Region::from_rect(rect2);

        let intersection = region1.intersect(&region2);
        assert!(!intersection.is_empty());

        // Test points in the intersection
        assert!(intersection.contains_point(Point::new(7, 7)));
        assert!(!intersection.contains_point(Point::new(2, 2))); // Outside intersection
    }

    #[test]
    fn test_region_subtract() {
        let rect1 = Rectangle::new(0, 0, 10, 10);
        let rect2 = Rectangle::new(3, 3, 4, 4);

        let region1 = Region::from_rect(rect1);
        let region2 = Region::from_rect(rect2);

        let result = region1.subtract(&region2);

        // Should contain corners of original rectangle
        assert!(result.contains_point(Point::new(1, 1)));
        assert!(result.contains_point(Point::new(8, 8)));

        // Should not contain center (subtracted area)
        assert!(!result.contains_point(Point::new(5, 5)));
    }

    #[test]
    fn test_region_translate() {
        let rect = Rectangle::new(0, 0, 10, 10);
        let region = Region::from_rect(rect);

        let translated = region.translate(5, 3);

        assert!(translated.contains_point(Point::new(7, 5)));
        assert!(!translated.contains_point(Point::new(2, 2)));
    }
}
