//! Shared framebuffer for communication between X11 server and window renderer
//!
//! This module provides thread-safe shared framebuffer access that allows
//! the X11 server to update framebuffer content while the window renderer
//! displays it on the main thread.

use crate::{
    display::{framebuffer::Framebuffer, types::DisplaySettings},
    Result,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tracing::{debug, info};

/// Shared framebuffer manager for cross-thread communication
pub struct SharedFramebuffers {
    /// Framebuffers by screen ID, protected by RwLock for thread safety
    framebuffers: Arc<RwLock<HashMap<u32, Arc<RwLock<Framebuffer>>>>>,
    /// Display settings
    settings: DisplaySettings,
}

impl SharedFramebuffers {
    /// Create a new shared framebuffer manager
    pub fn new(settings: DisplaySettings) -> Result<Self> {
        info!("Creating shared framebuffers for {} screens", settings.screens);
        
        let mut framebuffers = HashMap::new();
        
        // Create framebuffers for each screen
        for screen_id in 0..settings.screens {
            let framebuffer = Framebuffer::from_settings(
                settings.width,
                settings.height,
                &settings.framebuffer,
            )?;
            
            // Clear with a default pattern
            framebuffer.clear(0x002E3440)?; // Dark blue-gray
            
            framebuffers.insert(screen_id, Arc::new(RwLock::new(framebuffer)));
            debug!("Created shared framebuffer for screen {}", screen_id);
        }
        
        Ok(Self {
            framebuffers: Arc::new(RwLock::new(framebuffers)),
            settings,
        })
    }
    
    /// Get a framebuffer handle for reading/writing
    pub fn get_framebuffer(&self, screen_id: u32) -> Option<Arc<RwLock<Framebuffer>>> {
        self.framebuffers
            .read()
            .ok()?
            .get(&screen_id)
            .cloned()
    }
    
    /// Get all framebuffer handles
    pub fn get_all_framebuffers(&self) -> Vec<(u32, Arc<RwLock<Framebuffer>>)> {
        self.framebuffers
            .read()
            .map(|fb_map| {
                fb_map
                    .iter()
                    .map(|(&id, fb)| (id, fb.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Update a pixel in a framebuffer (thread-safe)
    pub fn set_pixel(&self, screen_id: u32, x: u32, y: u32, color: u32) -> Result<()> {
        if let Some(fb) = self.get_framebuffer(screen_id) {
            let framebuffer = fb.write().map_err(|_| {
                crate::Error::Display("Failed to acquire framebuffer write lock".to_string())
            })?;
            framebuffer.set_pixel(x, y, color)?;
        }
        Ok(())
    }
    
    /// Read a pixel from a framebuffer (thread-safe)
    pub fn get_pixel(&self, screen_id: u32, x: u32, y: u32) -> Result<u32> {
        if let Some(fb) = self.get_framebuffer(screen_id) {
            let framebuffer = fb.read().map_err(|_| {
                crate::Error::Display("Failed to acquire framebuffer read lock".to_string())
            })?;
            return framebuffer.get_pixel(x, y);
        }
        Err(crate::Error::Display(format!("Framebuffer not found for screen {}", screen_id)))
    }
    
    /// Clear a framebuffer (thread-safe)
    pub fn clear_screen(&self, screen_id: u32, color: u32) -> Result<()> {
        if let Some(fb) = self.get_framebuffer(screen_id) {
            let framebuffer = fb.write().map_err(|_| {
                crate::Error::Display("Failed to acquire framebuffer write lock".to_string())
            })?;
            framebuffer.clear(color)?;
        }
        Ok(())
    }
    
    /// Get display settings
    pub fn get_settings(&self) -> &DisplaySettings {
        &self.settings
    }
    
    /// Get the shared framebuffers handle for cloning to other threads
    pub fn clone_handle(&self) -> Arc<RwLock<HashMap<u32, Arc<RwLock<Framebuffer>>>>> {
        self.framebuffers.clone()
    }
}
