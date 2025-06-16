//! Graphics context management
//!
//! This module handles graphics contexts (GCs) which define drawing parameters.

use super::types::*;
use crate::core::error::ServerResult;
use std::collections::HashMap;

/// Graphics context identifier
pub type GContextId = u32;

/// Graphics context
#[derive(Debug, Clone)]
pub struct GraphicsContext {
    pub id: GContextId,
    pub foreground: Color,
    pub background: Color,
    pub line_width: u16,
    pub line_style: LineStyle,
    pub cap_style: CapStyle,
    pub join_style: JoinStyle,
    pub fill_style: FillStyle,
    pub function: Function,
    pub plane_mask: u32,
    pub clip_region: Option<Rectangle>,
}

impl Default for GraphicsContext {
    fn default() -> Self {
        Self {
            id: 0,
            foreground: Color::BLACK,
            background: Color::WHITE,
            line_width: 0,
            line_style: LineStyle::Solid,
            cap_style: CapStyle::Butt,
            join_style: JoinStyle::Miter,
            fill_style: FillStyle::Solid,
            function: Function::Copy,
            plane_mask: 0xFFFFFFFF,
            clip_region: None,
        }
    }
}

/// Graphics context manager
pub struct GraphicsContextManager {
    contexts: HashMap<GContextId, GraphicsContext>,
    next_id: GContextId,
}

impl GraphicsContextManager {
    /// Create a new graphics context manager
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a new graphics context
    pub fn create_gc(&mut self) -> ServerResult<GContextId> {
        let gc_id = self.next_id;
        self.next_id += 1;

        let gc = GraphicsContext {
            id: gc_id,
            ..Default::default()
        };

        self.contexts.insert(gc_id, gc);
        Ok(gc_id)
    }

    /// Create graphics context from template
    pub fn create_gc_from(&mut self, template: &GraphicsContext) -> ServerResult<GContextId> {
        let gc_id = self.next_id;
        self.next_id += 1;

        let mut gc = template.clone();
        gc.id = gc_id;

        self.contexts.insert(gc_id, gc);
        Ok(gc_id)
    }

    /// Get graphics context by ID
    pub fn get_gc(&self, gc_id: GContextId) -> Option<&GraphicsContext> {
        self.contexts.get(&gc_id)
    }

    /// Get mutable graphics context by ID
    pub fn get_gc_mut(&mut self, gc_id: GContextId) -> Option<&mut GraphicsContext> {
        self.contexts.get_mut(&gc_id)
    }

    /// Update graphics context
    pub fn update_gc(&mut self, gc_id: GContextId, updates: GCUpdates) -> ServerResult<()> {
        if let Some(gc) = self.contexts.get_mut(&gc_id) {
            if let Some(foreground) = updates.foreground {
                gc.foreground = foreground;
            }
            if let Some(background) = updates.background {
                gc.background = background;
            }
            if let Some(line_width) = updates.line_width {
                gc.line_width = line_width;
            }
            if let Some(line_style) = updates.line_style {
                gc.line_style = line_style;
            }
            if let Some(cap_style) = updates.cap_style {
                gc.cap_style = cap_style;
            }
            if let Some(join_style) = updates.join_style {
                gc.join_style = join_style;
            }
            if let Some(fill_style) = updates.fill_style {
                gc.fill_style = fill_style;
            }
            if let Some(function) = updates.function {
                gc.function = function;
            }
            if let Some(plane_mask) = updates.plane_mask {
                gc.plane_mask = plane_mask;
            }
            if let Some(clip_region) = updates.clip_region {
                gc.clip_region = clip_region;
            }
        }
        Ok(())
    }

    /// Copy graphics context values
    pub fn copy_gc(
        &mut self,
        src_id: GContextId,
        dst_id: GContextId,
        mask: u32,
    ) -> ServerResult<()> {
        if let (Some(src), Some(dst)) = (
            self.contexts.get(&src_id).cloned(),
            self.contexts.get_mut(&dst_id),
        ) {
            if mask & 0x01 != 0 {
                dst.function = src.function;
            }
            if mask & 0x02 != 0 {
                dst.plane_mask = src.plane_mask;
            }
            if mask & 0x04 != 0 {
                dst.foreground = src.foreground;
            }
            if mask & 0x08 != 0 {
                dst.background = src.background;
            }
            if mask & 0x10 != 0 {
                dst.line_width = src.line_width;
            }
            if mask & 0x20 != 0 {
                dst.line_style = src.line_style;
            }
            if mask & 0x40 != 0 {
                dst.cap_style = src.cap_style;
            }
            if mask & 0x80 != 0 {
                dst.join_style = src.join_style;
            }
            if mask & 0x100 != 0 {
                dst.fill_style = src.fill_style;
            }
        }
        Ok(())
    }

    /// Free graphics context
    pub fn free_gc(&mut self, gc_id: GContextId) -> ServerResult<()> {
        self.contexts.remove(&gc_id);
        Ok(())
    }

    /// Set clipping rectangle
    pub fn set_clip_region(
        &mut self,
        gc_id: GContextId,
        region: Option<Rectangle>,
    ) -> ServerResult<()> {
        if let Some(gc) = self.contexts.get_mut(&gc_id) {
            gc.clip_region = region;
        }
        Ok(())
    }
}

/// Graphics context update structure
#[derive(Debug, Default)]
pub struct GCUpdates {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub line_width: Option<u16>,
    pub line_style: Option<LineStyle>,
    pub cap_style: Option<CapStyle>,
    pub join_style: Option<JoinStyle>,
    pub fill_style: Option<FillStyle>,
    pub function: Option<Function>,
    pub plane_mask: Option<u32>,
    pub clip_region: Option<Option<Rectangle>>,
}

impl Default for GraphicsContextManager {
    fn default() -> Self {
        Self::new()
    }
}
