//! Graphics types and enums
//!
//! This module defines common graphics types and enumerations.

use serde::{Deserialize, Serialize};

/// Graphics backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphicsBackend {
    #[serde(rename = "software")]
    Software,
    #[serde(rename = "opengl")]
    OpenGL,
    #[serde(rename = "vulkan")]
    Vulkan,
}

impl Default for GraphicsBackend {
    fn default() -> Self {
        GraphicsBackend::Software
    }
}

/// Color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const GREEN: Color = Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn to_u32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn from_u32(color: u32) -> Self {
        Self {
            a: ((color >> 24) & 0xFF) as u8,
            r: ((color >> 16) & 0xFF) as u8,
            g: ((color >> 8) & 0xFF) as u8,
            b: (color & 0xFF) as u8,
        }
    }
}

/// Point
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

impl Point {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

/// Rectangle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

impl Rectangle {
    pub fn new(x: i16, y: i16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width as i16
            && point.y >= self.y
            && point.y < self.y + self.height as i16
    }

    pub fn intersects(&self, other: &Rectangle) -> bool {
        self.x < other.x + other.width as i16
            && self.x + self.width as i16 > other.x
            && self.y < other.y + other.height as i16
            && self.y + self.height as i16 > other.y
    }
}

/// Line style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    Solid,
    OnOffDash,
    DoubleDash,
}

/// Cap style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapStyle {
    NotLast,
    Butt,
    Round,
    Projecting,
}

/// Join style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinStyle {
    Miter,
    Round,
    Bevel,
}

/// Fill style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillStyle {
    Solid,
    Tiled,
    Stippled,
    OpaqueStippled,
}

/// Graphics function
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Function {
    Clear,
    And,
    AndReverse,
    Copy,
    AndInverted,
    NoOp,
    Xor,
    Or,
    Nor,
    Equiv,
    Invert,
    OrReverse,
    CopyInverted,
    OrInverted,
    Nand,
    Set,
}
