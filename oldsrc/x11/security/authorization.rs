//! Operation authorization for X11 requests
//!
//! This module implements authorization mechanisms that determine whether
//! authenticated clients are allowed to perform specific operations.

use crate::x11::protocol::types::{ClientId, XId};
use std::collections::{HashMap, HashSet};

/// Authorization manager
#[derive(Debug)]
pub struct AuthorizationManager {
    /// Per-client permissions
    client_permissions: HashMap<ClientId, HashSet<Permission>>,
    /// Global permission policies
    global_policies: HashMap<Permission, PermissionPolicy>,
    /// Resource-specific permissions
    resource_permissions: HashMap<XId, HashMap<ClientId, HashSet<ResourcePermission>>>,
}

/// System permission types
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Permission {
    /// Connect to the server
    Connect,
    /// Create windows
    CreateWindow,
    /// Modify windows
    ModifyWindow,
    /// Destroy windows
    DestroyWindow,
    /// Create graphics contexts
    CreateGC,
    /// Modify graphics contexts
    ModifyGC,
    /// Create pixmaps
    CreatePixmap,
    /// Create fonts
    CreateFont,
    /// Grab pointer
    GrabPointer,
    /// Grab keyboard
    GrabKeyboard,
    /// Access server information
    QueryServer,
    /// Kill other clients
    KillClient,
    /// Change host access list
    ChangeHosts,
    /// Manage extensions
    ManageExtensions,
    /// Debug server state
    Debug,
    /// Administrative operations
    Admin,
}

/// Resource-specific permission types
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ResourcePermission {
    /// Read access to resource
    Read,
    /// Write access to resource
    Write,
    /// Execute/use access to resource
    Execute,
    /// Delete access to resource
    Delete,
    /// Change properties
    ChangeProperties,
    /// Get properties
    GetProperties,
}

/// Permission policy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionPolicy {
    /// Allow by default
    Allow,
    /// Deny by default
    Deny,
    /// Require explicit grant
    RequireGrant,
}

/// Authorization context for requests
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Client making the request
    pub client_id: ClientId,
    /// Operation being requested
    pub operation: String,
    /// Target resource (if applicable)
    pub target_resource: Option<XId>,
    /// Additional context data
    pub context_data: HashMap<String, String>,
}

/// Authorization result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthResult {
    /// Operation is allowed
    Allow,
    /// Operation is denied
    Deny,
    /// Need more information to decide
    NeedMoreInfo,
}

impl AuthorizationManager {
    /// Create a new authorization manager
    pub fn new() -> Self {
        let mut manager = Self {
            client_permissions: HashMap::new(),
            global_policies: HashMap::new(),
            resource_permissions: HashMap::new(),
        };

        // Set default policies
        manager.set_default_policies();
        manager
    }

    /// Set default permission policies
    fn set_default_policies(&mut self) {
        use Permission::*;
        use PermissionPolicy::*;

        // Basic operations allowed by default
        self.global_policies.insert(Connect, Allow);
        self.global_policies.insert(CreateWindow, Allow);
        self.global_policies.insert(ModifyWindow, Allow);
        self.global_policies.insert(CreateGC, Allow);
        self.global_policies.insert(CreatePixmap, Allow);
        self.global_policies.insert(CreateFont, Allow);
        self.global_policies.insert(QueryServer, Allow);

        // Privileged operations require explicit grant
        self.global_policies.insert(GrabPointer, RequireGrant);
        self.global_policies.insert(GrabKeyboard, RequireGrant);
        self.global_policies.insert(KillClient, Deny);
        self.global_policies.insert(ChangeHosts, Deny);
        self.global_policies.insert(ManageExtensions, Deny);
        self.global_policies.insert(Debug, Deny);
        self.global_policies.insert(Admin, Deny);
    }

    /// Grant a permission to a client
    pub fn grant_permission(&mut self, client_id: ClientId, permission: Permission) {
        self.client_permissions
            .entry(client_id)
            .or_insert_with(HashSet::new)
            .insert(permission);
    }

    /// Revoke a permission from a client
    pub fn revoke_permission(&mut self, client_id: ClientId, permission: Permission) {
        if let Some(permissions) = self.client_permissions.get_mut(&client_id) {
            permissions.remove(&permission);
            if permissions.is_empty() {
                self.client_permissions.remove(&client_id);
            }
        }
    }

    /// Check if a client has a specific permission
    pub fn has_permission(&self, client_id: ClientId, permission: &Permission) -> bool {
        self.client_permissions
            .get(&client_id)
            .map(|perms| perms.contains(permission))
            .unwrap_or(false)
    }

    /// Set global policy for a permission
    pub fn set_global_policy(&mut self, permission: Permission, policy: PermissionPolicy) {
        self.global_policies.insert(permission, policy);
    }

    /// Authorize an operation
    pub fn authorize_operation(&self, context: &AuthContext) -> AuthResult {
        let permission = self.map_operation_to_permission(&context.operation);

        // Check explicit permission grants first
        if let Some(perm) = &permission {
            if self.has_permission(context.client_id, perm) {
                return AuthResult::Allow;
            }
        }

        // Check global policy
        if let Some(perm) = &permission {
            match self.global_policies.get(perm) {
                Some(PermissionPolicy::Allow) => AuthResult::Allow,
                Some(PermissionPolicy::Deny) => AuthResult::Deny,
                Some(PermissionPolicy::RequireGrant) => {
                    if self.has_permission(context.client_id, perm) {
                        AuthResult::Allow
                    } else {
                        AuthResult::Deny
                    }
                }
                None => AuthResult::Deny, // Default to deny if no policy
            }
        } else {
            // Unknown operation, default to deny
            AuthResult::Deny
        }
    }

    /// Map operation string to permission enum
    fn map_operation_to_permission(&self, operation: &str) -> Option<Permission> {
        match operation {
            "CreateWindow" => Some(Permission::CreateWindow),
            "DestroyWindow" => Some(Permission::DestroyWindow),
            "ConfigureWindow" | "ChangeWindowAttributes" => Some(Permission::ModifyWindow),
            "CreateGC" => Some(Permission::CreateGC),
            "ChangeGC" => Some(Permission::ModifyGC),
            "CreatePixmap" => Some(Permission::CreatePixmap),
            "OpenFont" => Some(Permission::CreateFont),
            "GrabPointer" => Some(Permission::GrabPointer),
            "GrabKeyboard" => Some(Permission::GrabKeyboard),
            "QueryTree" | "GetGeometry" | "QueryFont" => Some(Permission::QueryServer),
            "KillClient" => Some(Permission::KillClient),
            "ChangeHosts" => Some(Permission::ChangeHosts),
            "QueryExtension" => Some(Permission::ManageExtensions),
            _ => None, // Unknown operation
        }
    }

    /// Grant resource-specific permission
    pub fn grant_resource_permission(
        &mut self,
        resource_id: XId,
        client_id: ClientId,
        permission: ResourcePermission,
    ) {
        self.resource_permissions
            .entry(resource_id)
            .or_insert_with(HashMap::new)
            .entry(client_id)
            .or_insert_with(HashSet::new)
            .insert(permission);
    }

    /// Check resource-specific permission
    pub fn has_resource_permission(
        &self,
        resource_id: XId,
        client_id: ClientId,
        permission: &ResourcePermission,
    ) -> bool {
        self.resource_permissions
            .get(&resource_id)
            .and_then(|client_perms| client_perms.get(&client_id))
            .map(|perms| perms.contains(permission))
            .unwrap_or(false)
    }

    /// Get all permissions for a client
    pub fn get_client_permissions(&self, client_id: ClientId) -> HashSet<Permission> {
        self.client_permissions
            .get(&client_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Remove all permissions for a client
    pub fn remove_client_permissions(&mut self, client_id: ClientId) {
        self.client_permissions.remove(&client_id);

        // Remove from resource permissions as well
        for resource_perms in self.resource_permissions.values_mut() {
            resource_perms.remove(&client_id);
        }
    }

    /// Get authorization statistics
    pub fn get_stats(&self) -> AuthorizationStats {
        AuthorizationStats {
            clients_with_permissions: self.client_permissions.len(),
            total_permission_grants: self
                .client_permissions
                .values()
                .map(|perms| perms.len())
                .sum(),
            resources_with_permissions: self.resource_permissions.len(),
        }
    }
}

/// Authorization statistics
#[derive(Debug, Clone)]
pub struct AuthorizationStats {
    /// Number of clients with explicit permissions
    pub clients_with_permissions: usize,
    /// Total number of permission grants
    pub total_permission_grants: usize,
    /// Number of resources with specific permissions
    pub resources_with_permissions: usize,
}

impl Default for AuthorizationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_authorization() {
        let mut authz = AuthorizationManager::new();
        let client_id = 1;

        // CreateWindow should be allowed by default
        let context = AuthContext {
            client_id,
            operation: "CreateWindow".to_string(),
            target_resource: None,
            context_data: HashMap::new(),
        };

        assert_eq!(authz.authorize_operation(&context), AuthResult::Allow);
    }

    #[test]
    fn test_privileged_operation() {
        let authz = AuthorizationManager::new();
        let client_id = 1;

        // KillClient should be denied by default
        let context = AuthContext {
            client_id,
            operation: "KillClient".to_string(),
            target_resource: None,
            context_data: HashMap::new(),
        };

        assert_eq!(authz.authorize_operation(&context), AuthResult::Deny);
    }

    #[test]
    fn test_explicit_permission_grant() {
        let mut authz = AuthorizationManager::new();
        let client_id = 1;

        // Grant KillClient permission
        authz.grant_permission(client_id, Permission::KillClient);

        let context = AuthContext {
            client_id,
            operation: "KillClient".to_string(),
            target_resource: None,
            context_data: HashMap::new(),
        };

        assert_eq!(authz.authorize_operation(&context), AuthResult::Allow);
    }

    #[test]
    fn test_resource_permissions() {
        let mut authz = AuthorizationManager::new();
        let client_id = 1;
        let resource_id = 100;

        // Grant read permission on specific resource
        authz.grant_resource_permission(resource_id, client_id, ResourcePermission::Read);

        assert!(authz.has_resource_permission(resource_id, client_id, &ResourcePermission::Read));
        assert!(!authz.has_resource_permission(resource_id, client_id, &ResourcePermission::Write));
    }

    #[test]
    fn test_permission_revocation() {
        let mut authz = AuthorizationManager::new();
        let client_id = 1;

        // Grant and then revoke permission
        authz.grant_permission(client_id, Permission::Admin);
        assert!(authz.has_permission(client_id, &Permission::Admin));

        authz.revoke_permission(client_id, Permission::Admin);
        assert!(!authz.has_permission(client_id, &Permission::Admin));
    }
}
