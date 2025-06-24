//! Coordinate Transformations
//!
//! This module provides coordinate transformation utilities for X11.

use crate::x11::geometry::types::{Point, Rectangle};

/// A 2D transformation matrix
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub m11: f32,
    pub m12: f32,
    pub m13: f32,
    pub m21: f32,
    pub m22: f32,
    pub m23: f32,
}

impl Transform {
    /// Identity transformation
    pub const IDENTITY: Transform = Transform {
        m11: 1.0,
        m12: 0.0,
        m13: 0.0,
        m21: 0.0,
        m22: 1.0,
        m23: 0.0,
    };

    /// Create a translation transformation
    pub fn translate(dx: f32, dy: f32) -> Self {
        Transform {
            m11: 1.0,
            m12: 0.0,
            m13: dx,
            m21: 0.0,
            m22: 1.0,
            m23: dy,
        }
    }

    /// Create a scale transformation
    pub fn scale(sx: f32, sy: f32) -> Self {
        Transform {
            m11: sx,
            m12: 0.0,
            m13: 0.0,
            m21: 0.0,
            m22: sy,
            m23: 0.0,
        }
    }

    /// Transform a point
    pub fn transform_point(&self, point: Point) -> Point {
        let x = self.m11 * point.x as f32 + self.m12 * point.y as f32 + self.m13;
        let y = self.m21 * point.x as f32 + self.m22 * point.y as f32 + self.m23;

        Point::new(x as i16, y as i16)
    }

    /// Transform a rectangle
    pub fn transform_rectangle(&self, rect: Rectangle) -> Rectangle {
        let top_left = self.transform_point(Point::new(rect.x, rect.y));
        let bottom_right = self.transform_point(Point::new(
            rect.x + rect.width as i16,
            rect.y + rect.height as i16,
        ));

        Rectangle::from_points(top_left, bottom_right)
    }
}
