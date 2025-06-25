//! Geometric Types
//!
//! This module defines the core geometric types used throughout the X11 server.

use std::ops::{Add, Sub};

/// A point in 2D space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

/// A size with width and height
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

/// A rectangle defined by position and size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rectangle {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

/// Re-export region from regions module
pub use crate::x11::geometry::regions::Region;

/// A circle defined by center and radius
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    pub center: Point,
    pub radius: u16,
}

/// An arc defined by center, radii, and angles
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc {
    pub center: Point,
    pub width: u16,
    pub height: u16,
    pub angle1: i16, // Start angle in 64ths of a degree
    pub angle2: i16, // Arc extent in 64ths of a degree
}

/// A line segment between two points
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineSegment {
    pub start: Point,
    pub end: Point,
}

/// A polygon defined by a series of points
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Polygon {
    pub points: Vec<Point>,
}

impl Point {
    /// Create a new point
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    /// The origin point (0, 0)
    pub const ORIGIN: Point = Point { x: 0, y: 0 };

    /// Calculate the distance to another point
    pub fn distance_to(&self, other: Point) -> f64 {
        let dx = (other.x - self.x) as f64;
        let dy = (other.y - self.y) as f64;
        (dx * dx + dy * dy).sqrt()
    }

    /// Calculate the squared distance to another point (faster than distance_to)
    pub fn distance_squared_to(&self, other: Point) -> i32 {
        let dx = (other.x - self.x) as i32;
        let dy = (other.y - self.y) as i32;
        dx * dx + dy * dy
    }

    /// Translate this point by an offset
    pub fn translate(&self, dx: i16, dy: i16) -> Point {
        Point {
            x: self.x.saturating_add(dx),
            y: self.y.saturating_add(dy),
        }
    }

    /// Check if this point is within the given rectangle
    pub fn is_inside(&self, rect: &Rectangle) -> bool {
        rect.contains_point(*self)
    }
}

impl Size {
    /// Create a new size
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    /// Zero size
    pub const ZERO: Size = Size {
        width: 0,
        height: 0,
    };

    /// Check if this size is empty (zero width or height)
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Get the area
    pub fn area(&self) -> u32 {
        self.width as u32 * self.height as u32
    }

    /// Scale the size by a factor
    pub fn scale(&self, factor: f32) -> Size {
        Size {
            width: ((self.width as f32) * factor) as u16,
            height: ((self.height as f32) * factor) as u16,
        }
    }
}

impl Rectangle {
    /// Create a new rectangle
    pub fn new(x: i16, y: i16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a rectangle from two points
    pub fn from_points(p1: Point, p2: Point) -> Self {
        let min_x = p1.x.min(p2.x);
        let min_y = p1.y.min(p2.y);
        let max_x = p1.x.max(p2.x);
        let max_y = p1.y.max(p2.y);

        Self {
            x: min_x,
            y: min_y,
            width: (max_x - min_x) as u16,
            height: (max_y - min_y) as u16,
        }
    }

    /// Create a rectangle from position and size
    pub fn from_position_and_size(position: Point, size: Size) -> Self {
        Self {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        }
    }

    /// Empty rectangle
    pub const EMPTY: Rectangle = Rectangle {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    };

    /// Get the position (top-left corner)
    pub fn position(&self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Get the size
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Get the center point
    pub fn center(&self) -> Point {
        Point::new(
            self.x + (self.width as i16) / 2,
            self.y + (self.height as i16) / 2,
        )
    }

    /// Get the right edge coordinate
    pub fn right(&self) -> i16 {
        self.x + self.width as i16
    }

    /// Get the bottom edge coordinate
    pub fn bottom(&self) -> i16 {
        self.y + self.height as i16
    }

    /// Get the area
    pub fn area(&self) -> u32 {
        self.width as u32 * self.height as u32
    }

    /// Check if this rectangle is empty
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Check if this rectangle contains a point
    pub fn contains_point(&self, point: Point) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && point.x < self.x + self.width as i16
            && point.y < self.y + self.height as i16
    }

    /// Check if this rectangle completely contains another rectangle
    pub fn contains_rect(&self, other: &Rectangle) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    /// Check if this rectangle intersects with another rectangle
    pub fn intersects(&self, other: &Rectangle) -> bool {
        !(self.right() <= other.x
            || other.right() <= self.x
            || self.bottom() <= other.y
            || other.bottom() <= self.y)
    }

    /// Get the intersection of this rectangle with another
    pub fn intersection(&self, other: &Rectangle) -> Option<Rectangle> {
        if !self.intersects(other) {
            return None;
        }

        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        Some(Rectangle::new(
            left,
            top,
            (right - left) as u16,
            (bottom - top) as u16,
        ))
    }

    /// Get the union (bounding box) of this rectangle with another
    pub fn union(&self, other: &Rectangle) -> Rectangle {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }

        let left = self.x.min(other.x);
        let top = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());

        Rectangle::new(left, top, (right - left) as u16, (bottom - top) as u16)
    }

    /// Translate this rectangle by an offset
    pub fn translate(&self, dx: i16, dy: i16) -> Rectangle {
        Rectangle::new(
            self.x.saturating_add(dx),
            self.y.saturating_add(dy),
            self.width,
            self.height,
        )
    }

    /// Expand this rectangle by a margin
    pub fn expand(&self, margin: u16) -> Rectangle {
        let margin_i16 = margin as i16;
        Rectangle::new(
            self.x.saturating_sub(margin_i16),
            self.y.saturating_sub(margin_i16),
            self.width.saturating_add(margin * 2),
            self.height.saturating_add(margin * 2),
        )
    }

    /// Shrink this rectangle by a margin
    pub fn shrink(&self, margin: u16) -> Rectangle {
        let margin_i16 = margin as i16;
        let margin_u16_2 = margin.saturating_mul(2);

        if self.width <= margin_u16_2 || self.height <= margin_u16_2 {
            return Rectangle::EMPTY;
        }

        Rectangle::new(
            self.x.saturating_add(margin_i16),
            self.y.saturating_add(margin_i16),
            self.width.saturating_sub(margin_u16_2),
            self.height.saturating_sub(margin_u16_2),
        )
    }
}

// Arithmetic operations for Point
impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point::new(
            self.x.saturating_add(other.x),
            self.y.saturating_add(other.y),
        )
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point::new(
            self.x.saturating_sub(other.x),
            self.y.saturating_sub(other.y),
        )
    }
}

// Arithmetic operations for Size
impl Add for Size {
    type Output = Size;

    fn add(self, other: Size) -> Size {
        Size::new(
            self.width.saturating_add(other.width),
            self.height.saturating_add(other.height),
        )
    }
}

impl Sub for Size {
    type Output = Size;

    fn sub(self, other: Size) -> Size {
        Size::new(
            self.width.saturating_sub(other.width),
            self.height.saturating_sub(other.height),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_operations() {
        let p1 = Point::new(10, 20);
        let p2 = Point::new(5, 8);

        assert_eq!(p1 + p2, Point::new(15, 28));
        assert_eq!(p1 - p2, Point::new(5, 12));
        assert_eq!(p1.distance_squared_to(p2), 25 + 144);
    }

    #[test]
    fn test_rectangle_operations() {
        let rect1 = Rectangle::new(0, 0, 10, 10);
        let rect2 = Rectangle::new(5, 5, 10, 10);

        assert!(rect1.contains_point(Point::new(5, 5)));
        assert!(!rect1.contains_point(Point::new(15, 15)));
        assert!(rect1.intersects(&rect2));

        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection, Rectangle::new(5, 5, 5, 5));

        let union = rect1.union(&rect2);
        assert_eq!(union, Rectangle::new(0, 0, 15, 15));
    }
}
