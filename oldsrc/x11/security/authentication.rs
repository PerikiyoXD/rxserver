//! Client authentication mechanisms for X11 connections
//!
//! This module implements various authentication methods used by X11,
//! including MIT-MAGIC-COOKIE-1, host-based authentication, and custom methods.

use crate::x11::protocol::types::ClientId;
use std::collections::HashMap;

/// Authentication manager
#[derive(Debug)]
pub struct AuthenticationManager {
    /// Available authentication methods
    methods: HashMap<String, Box<dyn AuthMethod>>,
    /// Default authentication method
    default_method: String,
    /// Authentication cache
    auth_cache: HashMap<ClientId, AuthResult>,
}

/// Authentication method trait
pub trait AuthMethod: std::fmt::Debug + Send + Sync {
    /// Get the name of this authentication method
    fn name(&self) -> &str;

    /// Authenticate a client with the given data
    fn authenticate(&self, client_id: ClientId, auth_data: &[u8]) -> AuthResult;

    /// Check if this method requires authentication data
    fn requires_auth_data(&self) -> bool;

    /// Get the expected size of authentication data (if fixed)
    fn auth_data_size(&self) -> Option<usize>;
}

/// Authentication result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthResult {
    /// Authentication successful
    Success,
    /// Authentication failed
    Failed,
    /// Authentication requires more data
    NeedsMoreData,
    /// Authentication method not supported
    NotSupported,
}

/// MIT-MAGIC-COOKIE-1 authentication method
#[derive(Debug)]
pub struct MitMagicCookie {
    /// The magic cookie value
    cookie: Vec<u8>,
}

impl MitMagicCookie {
    /// Create a new MIT-MAGIC-COOKIE-1 authenticator with the given cookie
    pub fn new(cookie: Vec<u8>) -> Self {
        Self { cookie }
    }

    /// Generate a random cookie
    pub fn generate_random() -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::SystemTime;

        let mut hasher = DefaultHasher::new();
        SystemTime::now().hash(&mut hasher);
        let hash = hasher.finish();

        let cookie = hash.to_be_bytes().to_vec();
        Self::new(cookie)
    }
}

impl AuthMethod for MitMagicCookie {
    fn name(&self) -> &str {
        "MIT-MAGIC-COOKIE-1"
    }

    fn authenticate(&self, _client_id: ClientId, auth_data: &[u8]) -> AuthResult {
        if auth_data == self.cookie {
            AuthResult::Success
        } else {
            AuthResult::Failed
        }
    }

    fn requires_auth_data(&self) -> bool {
        true
    }

    fn auth_data_size(&self) -> Option<usize> {
        Some(self.cookie.len())
    }
}

/// Host-based authentication (no authentication data required)
#[derive(Debug)]
pub struct HostBasedAuth;

impl AuthMethod for HostBasedAuth {
    fn name(&self) -> &str {
        "HOST"
    }

    fn authenticate(&self, _client_id: ClientId, _auth_data: &[u8]) -> AuthResult {
        // Host-based authentication is handled by access control
        AuthResult::Success
    }

    fn requires_auth_data(&self) -> bool {
        false
    }

    fn auth_data_size(&self) -> Option<usize> {
        Some(0)
    }
}

/// No authentication required
#[derive(Debug)]
pub struct NoAuth;

impl AuthMethod for NoAuth {
    fn name(&self) -> &str {
        ""
    }

    fn authenticate(&self, _client_id: ClientId, _auth_data: &[u8]) -> AuthResult {
        AuthResult::Success
    }

    fn requires_auth_data(&self) -> bool {
        false
    }

    fn auth_data_size(&self) -> Option<usize> {
        Some(0)
    }
}

impl AuthenticationManager {
    /// Create a new authentication manager
    pub fn new() -> Self {
        let mut manager = Self {
            methods: HashMap::new(),
            default_method: String::new(),
            auth_cache: HashMap::new(),
        };

        // Register built-in authentication methods
        manager.register_method(Box::new(NoAuth));
        manager.register_method(Box::new(HostBasedAuth));
        manager.set_default_method("");

        manager
    }

    /// Register an authentication method
    pub fn register_method(&mut self, method: Box<dyn AuthMethod>) {
        let name = method.name().to_string();
        self.methods.insert(name, method);
    }

    /// Set the default authentication method
    pub fn set_default_method(&mut self, method_name: &str) {
        if self.methods.contains_key(method_name) {
            self.default_method = method_name.to_string();
        }
    }

    /// Get the default authentication method
    pub fn default_method(&self) -> &str {
        &self.default_method
    }

    /// Get available authentication methods
    pub fn available_methods(&self) -> Vec<String> {
        self.methods.keys().cloned().collect()
    }

    /// Authenticate a client using the specified method
    pub fn authenticate_client(
        &mut self,
        client_id: ClientId,
        method_name: &str,
        auth_data: &[u8],
    ) -> AuthResult {
        // Check cache first
        if let Some(cached_result) = self.auth_cache.get(&client_id) {
            return cached_result.clone();
        }

        let result = if let Some(method) = self.methods.get(method_name) {
            method.authenticate(client_id, auth_data)
        } else {
            AuthResult::NotSupported
        };

        // Cache successful authentications
        if result == AuthResult::Success {
            self.auth_cache.insert(client_id, result.clone());
        }

        result
    }

    /// Check if a client is authenticated
    pub fn is_authenticated(&self, client_id: ClientId) -> bool {
        matches!(self.auth_cache.get(&client_id), Some(AuthResult::Success))
    }

    /// Remove authentication for a client
    pub fn deauthenticate_client(&mut self, client_id: ClientId) {
        self.auth_cache.remove(&client_id);
    }

    /// Clear all authentication cache
    pub fn clear_auth_cache(&mut self) {
        self.auth_cache.clear();
    }

    /// Get authentication statistics
    pub fn get_stats(&self) -> AuthStats {
        AuthStats {
            available_methods: self.available_methods().len(),
            authenticated_clients: self.auth_cache.len(),
            default_method: self.default_method.clone(),
        }
    }

    /// Set MIT-MAGIC-COOKIE-1 with the given cookie
    pub fn set_magic_cookie(&mut self, cookie: Vec<u8>) {
        self.register_method(Box::new(MitMagicCookie::new(cookie)));
        self.set_default_method("MIT-MAGIC-COOKIE-1");
    }

    /// Generate and set a random MIT-MAGIC-COOKIE-1
    pub fn generate_magic_cookie(&mut self) {
        self.register_method(Box::new(MitMagicCookie::generate_random()));
        self.set_default_method("MIT-MAGIC-COOKIE-1");
    }
}

/// Authentication statistics
#[derive(Debug, Clone)]
pub struct AuthStats {
    /// Number of available authentication methods
    pub available_methods: usize,
    /// Number of authenticated clients
    pub authenticated_clients: usize,
    /// Default authentication method
    pub default_method: String,
}

impl Default for AuthenticationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_auth() {
        let mut auth_mgr = AuthenticationManager::new();
        let client_id = 1;

        let result = auth_mgr.authenticate_client(client_id, "", &[]);
        assert_eq!(result, AuthResult::Success);
        assert!(auth_mgr.is_authenticated(client_id));
    }

    #[test]
    fn test_host_based_auth() {
        let mut auth_mgr = AuthenticationManager::new();
        let client_id = 1;

        let result = auth_mgr.authenticate_client(client_id, "HOST", &[]);
        assert_eq!(result, AuthResult::Success);
    }

    #[test]
    fn test_magic_cookie() {
        let mut auth_mgr = AuthenticationManager::new();
        let cookie = vec![1, 2, 3, 4, 5, 6, 7, 8];
        auth_mgr.set_magic_cookie(cookie.clone());

        let client_id = 1;

        // Correct cookie should succeed
        let result = auth_mgr.authenticate_client(client_id, "MIT-MAGIC-COOKIE-1", &cookie);
        assert_eq!(result, AuthResult::Success);

        // Wrong cookie should fail
        let wrong_cookie = vec![8, 7, 6, 5, 4, 3, 2, 1];
        let client_id2 = 2;
        let result = auth_mgr.authenticate_client(client_id2, "MIT-MAGIC-COOKIE-1", &wrong_cookie);
        assert_eq!(result, AuthResult::Failed);
    }

    #[test]
    fn test_auth_cache() {
        let mut auth_mgr = AuthenticationManager::new();
        let client_id = 1;

        // Authenticate once
        auth_mgr.authenticate_client(client_id, "", &[]);
        assert!(auth_mgr.is_authenticated(client_id));

        // Should still be authenticated
        assert!(auth_mgr.is_authenticated(client_id));

        // Deauthenticate
        auth_mgr.deauthenticate_client(client_id);
        assert!(!auth_mgr.is_authenticated(client_id));
    }
}
