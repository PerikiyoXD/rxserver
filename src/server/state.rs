// state.rs - Fixed imports and types
//! Server and client state management for the X11 server

use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use crate::display::config::DisplayConfig;
use crate::protocol::{Atom, CursorId, WindowId, XId};
use crate::server::{
    atom_system::AtomSystem,
    client_system::{Client, ClientId, ClientSystem},
    display_system::DisplaySystem,
    resource_system::ResourceSystem,
    window_system::{Window, WindowClass, WindowSystem},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerGrab {
    pub grab_window: WindowId,
    pub grabbing_client: ClientId,
    pub owner_events: bool,
    pub event_mask: u32,
    pub confine_to: Option<WindowId>,
    pub cursor: Option<CursorId>,
    pub time: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrabResult {
    Success = 0,
    BadCursor = 1,
    BadTime = 2,
    BadWindow = 3,
    AlreadyGrabbed = 4,
    InvalidTime = 5,
    NotViewable = 6,
    Frozen = 7,
}

impl GrabResult {
    pub fn to_x11_status(self) -> u8 {
        self as u8
    }
}

impl From<u8> for GrabResult {
    fn from(value: u8) -> Self {
        match value {
            0 => GrabResult::Success,
            1 => GrabResult::BadCursor,
            2 => GrabResult::BadTime,
            3 => GrabResult::BadWindow,
            4 => GrabResult::AlreadyGrabbed,
            5 => GrabResult::InvalidTime,
            6 => GrabResult::NotViewable,
            7 => GrabResult::Frozen,
            _ => GrabResult::BadWindow, // Default fallback
        }
    }
}

/// Global server state shared across all connections
#[derive(Debug)]
pub struct Server {
    atoms: AtomSystem,
    windows: WindowSystem,
    clients: ClientSystem,
    resources: ResourceSystem,
    displays: DisplaySystem,
    pointer_grab: Option<PointerGrab>,
}

impl Server {
    pub fn new(display_configs: Vec<DisplayConfig>) -> Result<Arc<Mutex<Self>>> {
        let server = Self {
            atoms: AtomSystem::new(),
            windows: WindowSystem::new(),
            clients: ClientSystem::new(),
            resources: ResourceSystem::new(),
            displays: DisplaySystem::from_configs(display_configs)?,
            pointer_grab: None,
        };
        Ok(Arc::new(Mutex::new(server)))
    }

    // Client management - delegate to ClientSystem
    pub fn register_client(&mut self, address: SocketAddr) -> (ClientId, Arc<Mutex<Client>>) {
        self.clients.register_client(address)
    }

    pub fn unregister_client(&mut self, client_id: ClientId) {
        if let Some(client_state) = self.clients.unregister_client(client_id) {
            if let Ok(client) = client_state.try_lock() {
                debug!(
                    "Cleaning up {} resources for client {}",
                    client.owned_resources().len(),
                    client_id
                );
            }
            self.windows.cleanup_client_windows(client_id);
        }
    }

    pub fn get_client(&self, client_id: ClientId) -> Option<&Arc<Mutex<Client>>> {
        self.clients.get_client(client_id)
    }

    pub fn client_count(&self) -> usize {
        self.clients.client_count()
    }

    // Atom management - delegate to AtomSystem
    pub fn intern_atom(&mut self, name: &str, only_if_exists: bool) -> Option<Atom> {
        self.atoms.intern_atom(name, only_if_exists)
    }

    pub fn get_atom_name(&self, atom_id: Atom) -> Option<&str> {
        self.atoms.get_atom_name(atom_id)
    }

    // Window management - delegate to WindowSystem
    pub async fn create_window(
        &mut self,
        id: WindowId,
        parent: WindowId,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        depth: u8,
        class: WindowClass,
        owner: ClientId,
    ) -> Result<(), String> {
        self.windows.create_window(
            id,
            parent,
            x,
            y,
            width,
            height,
            border_width,
            depth,
            class,
            owner,
        )?;

        // Notify displays of new window
        if let Some(window) = self.windows.get_window(id) {
            self.displays.notify_window_created(window).await;
        }

        Ok(())
    }

    pub async fn destroy_window(&mut self, window_id: WindowId) -> Result<(), String> {
        self.windows.destroy_window(window_id)?;
        self.displays.notify_window_destroyed(window_id).await;
        Ok(())
    }

    pub fn get_window(&self, window_id: WindowId) -> Option<&Window> {
        self.windows.get_window(window_id)
    }

    pub fn get_root_window(&self) -> &Window {
        self.windows.get_root_window()
    }

    pub fn window_exists(&self, window_id: WindowId) -> bool {
        self.windows.window_exists(window_id)
    }

    pub fn is_window_viewable(&self, window_id: WindowId) -> bool {
        self.windows.is_window_viewable(window_id)
    }

    // Window operations with display notifications
    pub async fn map_window(&mut self, window_id: WindowId) -> Result<(), String> {
        if !self.windows.window_exists(window_id) {
            return Err("Window does not exist".to_string());
        }

        debug!("Mapping window {}", window_id);
        self.displays.notify_window_mapped(window_id).await;
        Ok(())
    }

    pub async fn unmap_window(&mut self, window_id: WindowId) -> Result<(), String> {
        if !self.windows.window_exists(window_id) {
            return Err("Window does not exist".to_string());
        }

        debug!("Unmapping window {}", window_id);
        self.displays.notify_window_unmapped(window_id).await;
        Ok(())
    }

    // Resource management - delegate to ResourceSystem
    pub fn allocate_resource_id(&mut self) -> XId {
        self.resources.allocate_resource_id()
    }

    pub fn is_valid_resource_id(&self, resource_id: XId) -> bool {
        self.resources.is_valid_resource_id(resource_id)
    }

    // Display management - delegate to DisplaySystem
    pub fn display_count(&self) -> usize {
        self.displays.display_count()
    }

    pub async fn shutdown(&self) {
        self.displays.shutdown().await;
    }

    pub async fn sync_windows_to_displays(&self) {
        let windows: Vec<Window> = self.windows.windows().values().cloned().collect();
        self.displays.sync_windows(windows).await;
    }

    //  Pointer grab management
    pub fn establish_pointer_grab(&mut self, grab: PointerGrab) -> GrabResult {
        if self.pointer_grab.is_some() {
            return GrabResult::AlreadyGrabbed;
        }
        self.pointer_grab = Some(grab);
        GrabResult::Success
    }

    pub fn release_pointer_grab(&mut self, client_id: ClientId) -> bool {
        if let Some(ref grab) = self.pointer_grab {
            if grab.grabbing_client == client_id {
                self.pointer_grab = None;
                return true;
            }
        }
        false
    }

    pub fn get_pointer_grab(&self) -> Option<&PointerGrab> {
        self.pointer_grab.as_ref()
    }
}
