//! Host-based access control for X11 connections
//!
//! This module implements access control mechanisms that determine which
//! hosts and clients are allowed to connect to the X11 server.

use std::collections::HashSet;
use std::net::IpAddr;

/// Host access control manager
#[derive(Debug)]
pub struct AccessControl {
    /// List of allowed hosts
    allowed_hosts: HashSet<IpAddr>,
    /// Whether access control is enabled
    enabled: bool,
    /// Default policy when host not in list
    default_policy: AccessPolicy,
}

/// Access control policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPolicy {
    /// Allow access by default
    Allow,
    /// Deny access by default
    Deny,
}

/// Host access information
#[derive(Debug, Clone)]
pub struct HostAccess {
    /// IP address of the host
    pub address: IpAddr,
    /// Whether access is allowed
    pub allowed: bool,
    /// Access policy source
    pub policy_source: PolicySource,
}

/// Source of access policy decision
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicySource {
    /// Explicitly allowed in host list
    ExplicitAllow,
    /// Explicitly denied in host list
    ExplicitDeny,
    /// Default policy applied
    Default,
    /// Local connection (always allowed)
    Local,
}

impl AccessControl {
    /// Create a new access control manager
    pub fn new() -> Self {
        Self {
            allowed_hosts: HashSet::new(),
            enabled: true,
            default_policy: AccessPolicy::Deny,
        }
    }

    /// Enable or disable access control
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if access control is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set the default access policy
    pub fn set_default_policy(&mut self, policy: AccessPolicy) {
        self.default_policy = policy;
    }

    /// Get the default access policy
    pub fn default_policy(&self) -> AccessPolicy {
        self.default_policy
    }

    /// Add a host to the allowed list
    pub fn add_host(&mut self, address: IpAddr) {
        self.allowed_hosts.insert(address);
    }

    /// Remove a host from the allowed list
    pub fn remove_host(&mut self, address: IpAddr) -> bool {
        self.allowed_hosts.remove(&address)
    }

    /// Clear all allowed hosts
    pub fn clear_hosts(&mut self) {
        self.allowed_hosts.clear();
    }

    /// Get all allowed hosts
    pub fn get_allowed_hosts(&self) -> Vec<IpAddr> {
        self.allowed_hosts.iter().copied().collect()
    }

    /// Check if a host is allowed to connect
    pub fn check_host_access(&self, address: IpAddr) -> HostAccess {
        // If access control is disabled, allow all
        if !self.enabled {
            return HostAccess {
                address,
                allowed: true,
                policy_source: PolicySource::Default,
            };
        }

        // Local connections are always allowed
        if address.is_loopback() {
            return HostAccess {
                address,
                allowed: true,
                policy_source: PolicySource::Local,
            };
        }

        // Check if host is explicitly allowed
        if self.allowed_hosts.contains(&address) {
            return HostAccess {
                address,
                allowed: true,
                policy_source: PolicySource::ExplicitAllow,
            };
        }

        // Apply default policy
        let allowed = matches!(self.default_policy, AccessPolicy::Allow);
        HostAccess {
            address,
            allowed,
            policy_source: PolicySource::Default,
        }
    }

    /// Get access statistics
    pub fn get_stats(&self) -> AccessControlStats {
        AccessControlStats {
            enabled: self.enabled,
            allowed_hosts_count: self.allowed_hosts.len(),
            default_policy: self.default_policy,
        }
    }
}

/// Access control statistics
#[derive(Debug, Clone)]
pub struct AccessControlStats {
    /// Whether access control is enabled
    pub enabled: bool,
    /// Number of explicitly allowed hosts
    pub allowed_hosts_count: usize,
    /// Default access policy
    pub default_policy: AccessPolicy,
}

impl Default for AccessControl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_access_control_basic() {
        let mut ac = AccessControl::new();
        let localhost = IpAddr::from_str("127.0.0.1").unwrap();
        let remote = IpAddr::from_str("192.168.1.100").unwrap();

        // Local connections should always be allowed
        let access = ac.check_host_access(localhost);
        assert!(access.allowed);
        assert_eq!(access.policy_source, PolicySource::Local);

        // Remote connections denied by default
        let access = ac.check_host_access(remote);
        assert!(!access.allowed);
        assert_eq!(access.policy_source, PolicySource::Default);

        // Add remote host to allowed list
        ac.add_host(remote);
        let access = ac.check_host_access(remote);
        assert!(access.allowed);
        assert_eq!(access.policy_source, PolicySource::ExplicitAllow);
    }

    #[test]
    fn test_access_control_disabled() {
        let mut ac = AccessControl::new();
        ac.set_enabled(false);

        let remote = IpAddr::from_str("192.168.1.100").unwrap();
        let access = ac.check_host_access(remote);
        assert!(access.allowed);
    }

    #[test]
    fn test_default_policy() {
        let mut ac = AccessControl::new();
        ac.set_default_policy(AccessPolicy::Allow);

        let remote = IpAddr::from_str("192.168.1.100").unwrap();
        let access = ac.check_host_access(remote);
        assert!(access.allowed);
        assert_eq!(access.policy_source, PolicySource::Default);
    }
}
