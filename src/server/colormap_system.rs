// colormap_system.rs
use crate::protocol::ColormapId;
use std::collections::HashMap;

/// An RGB color entry, as stored/returned by the colormap (16-bit components,
/// matching the X11 wire format for QueryColors/AllocColor replies).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorEntry {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

/// Internal representation of a colormap in the X11 server
#[derive(Debug, Clone)]
pub struct Colormap {
    pub id: ColormapId,
    entries: HashMap<u32, ColorEntry>,
}

impl Colormap {
    fn new(id: ColormapId) -> Self {
        Self {
            id,
            entries: HashMap::new(),
        }
    }

    pub fn get_color(&self, pixel: u32) -> Option<ColorEntry> {
        self.entries.get(&pixel).copied()
    }
}

/// Manages X11 colormaps. Only the always-present default colormap (id 1,
/// as reported in the connection setup reply) is populated for now - real
/// AllocColor/CreateColormap support is future work.
#[derive(Debug)]
pub struct ColormapSystem {
    colormaps: HashMap<ColormapId, Colormap>,
}

/// The colormap ID reported as `default_colormap` in the connection setup
/// reply (`server::connection`). Kept as one constant so the two can't drift.
pub const DEFAULT_COLORMAP_ID: ColormapId = 1;

impl ColormapSystem {
    pub fn new() -> Self {
        let mut default_map = Colormap::new(DEFAULT_COLORMAP_ID);
        // Seed a basic black/white grayscale ramp on the low pixel values so
        // QueryColors has real data to return instead of always failing.
        for pixel in 0u32..256 {
            let level = ((pixel * 0xFFFF) / 255) as u16;
            default_map.entries.insert(
                pixel,
                ColorEntry {
                    red: level,
                    green: level,
                    blue: level,
                },
            );
        }

        let mut colormaps = HashMap::new();
        colormaps.insert(DEFAULT_COLORMAP_ID, default_map);

        Self { colormaps }
    }

    pub fn get_colormap(&self, id: ColormapId) -> Option<&Colormap> {
        self.colormaps.get(&id)
    }
}
