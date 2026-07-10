// state.rs - Fixed imports and types
//! Server and client state management for the X11 server

use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::display::config::DisplayConfig;
use crate::protocol::{Atom, CursorId, GContextId, PixmapId, WindowId, XId};
use crate::server::{
    atom_system::AtomSystem,
    client_system::{Client, ClientId, ClientSystem},
    display_system::DisplaySystem,
    gcontext_system::{GraphicsContext, GraphicsContextSystem},
    pixmap_system::PixmapSystem,
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
    pixmaps: PixmapSystem,
    clients: ClientSystem,
    resources: ResourceSystem,
    displays: DisplaySystem,
    gcontexts: GraphicsContextSystem,
    pointer_grab: Option<PointerGrab>,
}

impl Server {
    pub fn new(display_configs: Vec<DisplayConfig>) -> Result<Arc<Mutex<Self>>> {
        let server = Self {
            atoms: AtomSystem::new(),
            windows: WindowSystem::new(),
            pixmaps: PixmapSystem::new(),
            clients: ClientSystem::new(),
            resources: ResourceSystem::new(),
            displays: DisplaySystem::from_configs(display_configs)?,
            gcontexts: GraphicsContextSystem::new(),
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

    // Pixmap management - delegate to PixmapSystem
    pub fn create_pixmap(
        &mut self,
        id: PixmapId,
        width: u16,
        height: u16,
        depth: u8,
        owner: ClientId,
    ) -> Result<(), String> {
        self.pixmaps.create_pixmap(id, width, height, depth, owner)
    }

    pub async fn destroy_window(&mut self, window_id: WindowId) -> Result<(), String> {
        self.windows.destroy_window(window_id)?;
        self.displays.notify_window_destroyed(window_id).await;
        Ok(())
    }

    pub fn get_window(&self, window_id: WindowId) -> Option<&Window> {
        self.windows.get_window(window_id)
    }

    pub fn get_window_mut(&mut self, window_id: WindowId) -> Option<&mut Window> {
        self.windows.get_window_mut(window_id)
    }

    pub fn get_pixmap(&self, pixmap_id: PixmapId) -> Option<&crate::server::pixmap_system::Pixmap> {
        self.pixmaps.get_pixmap(pixmap_id)
    }

    pub fn get_pixmap_mut(&mut self, pixmap_id: PixmapId) -> Option<&mut crate::server::pixmap_system::Pixmap> {
        self.pixmaps.get_pixmap_mut(pixmap_id)
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

    // Graphics context management - delegate to GraphicsContextSystem
    pub fn create_gc(&mut self, id: GContextId, drawable: u32, owner: ClientId) -> Result<(), String> {
        self.gcontexts.create_gc(id, drawable, owner)
    }

    pub fn destroy_gc(&mut self, id: GContextId) -> Result<(), String> {
        self.gcontexts.destroy_gc(id)
    }

    pub fn get_gc(&self, id: GContextId) -> Option<&GraphicsContext> {
        self.gcontexts.get_gc(id)
    }

    pub fn get_gc_mut(&mut self, id: GContextId) -> Option<&mut GraphicsContext> {
        self.gcontexts.get_gc_mut(id)
    }

    pub fn gc_exists(&self, id: GContextId) -> bool {
        self.gcontexts.gc_exists(id)
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

    /// Get screen dimensions for display ID
    pub fn get_screen_size(&self, display_id: usize) -> (u16, u16) {
        if let Some(screen) = self.displays.randr_state().get_screen(display_id as u32) {
            (screen.width, screen.height)
        } else {
            warn!("Display ID {} not found, returning fallback screen size", display_id);
            (1920, 1080) // fallback
        }
    }

    pub async fn shutdown(&self) {
        self.displays.shutdown().await;
    }

    pub async fn sync_windows_to_displays(&self) {
        let windows: Vec<Window> = self.windows.windows().values().cloned().collect();
        self.displays.sync_windows(windows).await;
    }

    // RANDR extension support
    pub fn randr_state(&self) -> &crate::protocol::randr::RandrState {
        self.displays.randr_state()
    }

    pub fn randr_state_mut(&mut self) -> &mut crate::protocol::randr::RandrState {
        self.displays.randr_state_mut()
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

    /// Send an expose event to a client for a window
    pub async fn send_expose_event(&mut self, client_id: ClientId, window_id: u32, x: i16, y: i16, width: u16, height: u16, count: u16) {
        use crate::protocol::events::ExposeEvent;

        if let Some(client_arc) = self.clients.get_client_mut(client_id) {
            let mut client = client_arc.lock().await;
            let event = ExposeEvent::new(window_id, x, y, width, height, count);
            let event_data = event.serialize(client.byte_order());
            client.queue_event(event_data);
        }
    }
}
