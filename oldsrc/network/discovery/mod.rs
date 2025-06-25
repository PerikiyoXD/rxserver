//! Service discovery module
//!
//! Provides various service discovery mechanisms for finding and advertising X11 servers.

pub mod broadcast;
pub mod dns_sd;
pub mod mdns;
pub mod registry;
pub mod static_config;

// Re-export commonly used items
pub use broadcast::{BroadcastDiscovery, BroadcastService};
pub use dns_sd::{DnsSdDiscovery, DnsSdService};
pub use mdns::{MdnsDiscovery, MdnsService};
pub use registry::{DiscoveryError, ServiceInfo, ServiceRegistry};
pub use static_config::{StaticConfigDiscovery, StaticService};
