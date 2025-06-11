//! Graphics context management
//!
//! This module manages graphics contexts used for drawing operations.

use crate::protocol::types::*;

/// Graphics context state
#[derive(Debug, Clone)]
pub struct GraphicsContext {
    /// Graphics context ID
    pub id: GContext,
    /// Foreground color
    pub foreground: u32,
    /// Background color
    pub background: u32,
    /// Line width
    pub line_width: u16,
    /// Line style
    pub line_style: LineStyle,
    /// Cap style for line ends
    pub cap_style: CapStyle,
    /// Join style for line corners
    pub join_style: JoinStyle,
    /// Fill style
    pub fill_style: FillStyle,
    /// Fill rule
    pub fill_rule: FillRule,
    /// Current font
    pub font: Option<Font>,
    /// Clipping rectangles
    pub clip_rects: Vec<Rectangle>,
}

/// Line style enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    Solid = 0,
    OnOffDash = 1,
    DoubleDash = 2,
}

/// Cap style enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapStyle {
    NotLast = 0,
    Butt = 1,
    Round = 2,
    Projecting = 3,
}

/// Join style enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinStyle {
    Miter = 0,
    Round = 1,
    Bevel = 2,
}

/// Fill style enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillStyle {
    Solid = 0,
    Tiled = 1,
    Stippled = 2,
    OpaqueStippled = 3,
}

/// Fill rule enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    EvenOdd = 0,
    Winding = 1,
}

impl Default for GraphicsContext {
    fn default() -> Self {
        Self {
            id: 0,
            foreground: 0x000000, // Black
            background: 0xFFFFFF, // White
            line_width: 0,
            line_style: LineStyle::Solid,
            cap_style: CapStyle::Butt,
            join_style: JoinStyle::Miter,
            fill_style: FillStyle::Solid,
            fill_rule: FillRule::EvenOdd,
            font: None,
            clip_rects: Vec::new(),
        }
    }
}

impl GraphicsContext {
    /// Create a new graphics context with the given ID
    pub fn new(id: GContext) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    /// Set the foreground color
    pub fn set_foreground(&mut self, color: u32) {
        self.foreground = color;
    }

    /// Set the background color
    pub fn set_background(&mut self, color: u32) {
        self.background = color;
    }

    /// Set the line width
    pub fn set_line_width(&mut self, width: u16) {
        self.line_width = width;
    }

    /// Set the line style
    pub fn set_line_style(&mut self, style: LineStyle) {
        self.line_style = style;
    }

    /// Set the cap style
    pub fn set_cap_style(&mut self, style: CapStyle) {
        self.cap_style = style;
    }

    /// Set the join style
    pub fn set_join_style(&mut self, style: JoinStyle) {
        self.join_style = style;
    }

    /// Set the fill style
    pub fn set_fill_style(&mut self, style: FillStyle) {
        self.fill_style = style;
    }

    /// Set the fill rule
    pub fn set_fill_rule(&mut self, rule: FillRule) {
        self.fill_rule = rule;
    }

    /// Set the current font
    pub fn set_font(&mut self, font: Font) {
        self.font = Some(font);
    }

    /// Add a clipping rectangle
    pub fn add_clip_rect(&mut self, rect: Rectangle) {
        self.clip_rects.push(rect);
    }

    /// Clear all clipping rectangles
    pub fn clear_clip_rects(&mut self) {
        self.clip_rects.clear();
    }

    /// Check if a point is within the clipping region
    pub fn point_in_clip_region(&self, x: i16, y: i16) -> bool {
        if self.clip_rects.is_empty() {
            return true; // No clipping
        }

        for rect in &self.clip_rects {
            if x >= rect.x
                && x < rect.x + rect.width as i16
                && y >= rect.y
                && y < rect.y + rect.height as i16
            {
                return true;
            }
        }

        false
    }

    /// Check if a rectangle intersects with the clipping region
    pub fn rect_in_clip_region(&self, rect: &Rectangle) -> bool {
        if self.clip_rects.is_empty() {
            return true; // No clipping
        }

        for clip_rect in &self.clip_rects {
            if rectangles_intersect(rect, clip_rect) {
                return true;
            }
        }

        false
    }
}

/// Check if two rectangles intersect
fn rectangles_intersect(a: &Rectangle, b: &Rectangle) -> bool {
    let a_right = a.x + a.width as i16;
    let a_bottom = a.y + a.height as i16;
    let b_right = b.x + b.width as i16;
    let b_bottom = b.y + b.height as i16;

    !(a.x >= b_right || b.x >= a_right || a.y >= b_bottom || b.y >= a_bottom)
}
