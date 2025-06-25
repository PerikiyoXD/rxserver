//! Data encryption implementation
//!
//! Provides encryption and decryption capabilities for network data.

use std::collections::HashMap;
use tracing::{debug, warn};

/// Encryption type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EncryptionType {
    /// No encryption
    None = 0,
    /// AES-256-GCM encryption
    Aes256Gcm = 1,
    /// ChaCha20-Poly1305 encryption
    ChaCha20Poly1305 = 2,
    /// Custom encryption
    Custom = 3,
}

/// Encryption error
#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Unsupported encryption type: {0:?}")]
    UnsupportedType(EncryptionType),

    #[error("Invalid key size: expected {0}, got {1}")]
    InvalidKeySize(usize, usize),

    #[error("Invalid nonce size: expected {0}, got {1}")]
    InvalidNonceSize(usize, usize),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Key not found for connection: {0}")]
    KeyNotFound(crate::network::ConnectionId),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Encryption key
#[derive(Debug, Clone)]
pub struct EncryptionKey {
    /// Key data
    pub key: Vec<u8>,
    /// Key identifier
    pub key_id: String,
    /// Encryption type this key is for
    pub encryption_type: EncryptionType,
    /// Key creation time
    pub created_at: std::time::SystemTime,
}

/// Encryption configuration
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    /// Encryption type
    pub encryption_type: EncryptionType,
    /// Key rotation interval in seconds
    pub key_rotation_interval: u64,
    /// Enable authentication
    pub enable_authentication: bool,
    /// Additional authenticated data
    pub aad: Vec<u8>,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            encryption_type: EncryptionType::None,
            key_rotation_interval: 3600, // 1 hour
            enable_authentication: true,
            aad: Vec::new(),
        }
    }
}

/// Encryption result
#[derive(Debug)]
pub struct EncryptionResult {
    /// Encrypted data
    pub data: Vec<u8>,
    /// Nonce/IV used
    pub nonce: Vec<u8>,
    /// Authentication tag (if applicable)
    pub tag: Vec<u8>,
    /// Encryption time in microseconds
    pub encryption_time_us: u64,
}

/// Decryption result
#[derive(Debug)]
pub struct DecryptionResult {
    /// Decrypted data
    pub data: Vec<u8>,
    /// Decryption time in microseconds
    pub decryption_time_us: u64,
}

/// Encryption statistics
#[derive(Debug, Clone)]
pub struct EncryptionStats {
    /// Total bytes encrypted
    pub bytes_encrypted: u64,
    /// Total bytes decrypted
    pub bytes_decrypted: u64,
    /// Total encryption operations
    pub encryption_ops: u64,
    /// Total decryption operations
    pub decryption_ops: u64,
    /// Total time spent encrypting (microseconds)
    pub encryption_time_us: u64,
    /// Total time spent decrypting (microseconds)
    pub decryption_time_us: u64,
    /// Number of authentication failures
    pub auth_failures: u64,
}

impl Default for EncryptionStats {
    fn default() -> Self {
        Self {
            bytes_encrypted: 0,
            bytes_decrypted: 0,
            encryption_ops: 0,
            decryption_ops: 0,
            encryption_time_us: 0,
            decryption_time_us: 0,
            auth_failures: 0,
        }
    }
}

/// Connection encryption state
struct ConnectionState {
    /// Current encryption key
    key: EncryptionKey,
    /// Nonce counter
    nonce_counter: u64,
    /// Last key rotation time
    last_rotation: std::time::SystemTime,
}

/// Encryption manager
pub struct EncryptionManager {
    /// Encryption configurations by type
    configs: HashMap<EncryptionType, EncryptionConfig>,
    /// Connection states
    connection_states: HashMap<crate::network::ConnectionId, ConnectionState>,
    /// Encryption statistics by type
    stats: HashMap<EncryptionType, EncryptionStats>,
}

impl EncryptionManager {
    /// Create a new encryption manager
    pub fn new() -> Self {
        let mut manager = Self {
            configs: HashMap::new(),
            connection_states: HashMap::new(),
            stats: HashMap::new(),
        };

        // Register default configurations
        manager.register_config(EncryptionConfig {
            encryption_type: EncryptionType::None,
            ..Default::default()
        });

        manager.register_config(EncryptionConfig {
            encryption_type: EncryptionType::Aes256Gcm,
            ..Default::default()
        });

        manager.register_config(EncryptionConfig {
            encryption_type: EncryptionType::ChaCha20Poly1305,
            ..Default::default()
        });

        manager
    }

    /// Register encryption configuration
    pub fn register_config(&mut self, config: EncryptionConfig) {
        debug!(
            "Registering encryption config: {:?}",
            config.encryption_type
        );
        let encryption_type = config.encryption_type;
        self.configs.insert(encryption_type, config);
        self.stats
            .insert(encryption_type, EncryptionStats::default());
    }

    /// Set encryption key for a connection
    pub fn set_connection_key(
        &mut self,
        connection_id: crate::network::ConnectionId,
        key: EncryptionKey,
    ) -> Result<(), EncryptionError> {
        // Validate key size based on encryption type
        let expected_key_size = match key.encryption_type {
            EncryptionType::None => 0,
            EncryptionType::Aes256Gcm => 32,
            EncryptionType::ChaCha20Poly1305 => 32,
            EncryptionType::Custom => key.key.len(), // Accept any size for custom
        };

        if key.encryption_type != EncryptionType::Custom && key.key.len() != expected_key_size {
            return Err(EncryptionError::InvalidKeySize(
                expected_key_size,
                key.key.len(),
            ));
        }

        let state = ConnectionState {
            key,
            nonce_counter: 0,
            last_rotation: std::time::SystemTime::now(),
        };

        self.connection_states.insert(connection_id, state);

        debug!("Set encryption key for connection {}", connection_id);
        Ok(())
    }

    /// Encrypt data for a connection
    pub fn encrypt(
        &mut self,
        connection_id: crate::network::ConnectionId,
        data: &[u8],
    ) -> Result<EncryptionResult, EncryptionError> {
        // Scope the mutable borrow to only what is needed
        let (encryption_type, key_bytes, nonce_counter) = {
            let state = self
                .connection_states
                .get_mut(&connection_id)
                .ok_or(EncryptionError::KeyNotFound(connection_id))?;

            if state.key.encryption_type == EncryptionType::None {
                return Ok(EncryptionResult {
                    data: data.to_vec(),
                    nonce: Vec::new(),
                    tag: Vec::new(),
                    encryption_time_us: 0,
                });
            }

            let encryption_type = state.key.encryption_type;
            let key_bytes = state.key.key.clone();
            let nonce_counter = state.nonce_counter;
            // Increment nonce_counter
            state.nonce_counter += 1;
            (encryption_type, key_bytes, nonce_counter)
        };

        let start_time = std::time::Instant::now();

        // Generate nonce
        let nonce = self.generate_nonce(&encryption_type, nonce_counter)?;

        // Get config for AAD
        let aad = {
            let config = self
                .configs
                .get(&encryption_type)
                .ok_or(EncryptionError::UnsupportedType(encryption_type))?;
            config.aad.clone()
        };

        let (encrypted_data, tag) = match encryption_type {
            EncryptionType::None => (data.to_vec(), Vec::new()),
            EncryptionType::Aes256Gcm => self.encrypt_aes256_gcm(&key_bytes, &nonce, data, &aad)?,
            EncryptionType::ChaCha20Poly1305 => {
                self.encrypt_chacha20_poly1305(&key_bytes, &nonce, data, &aad)?
            }
            EncryptionType::Custom => {
                return Err(EncryptionError::UnsupportedType(encryption_type));
            }
        };

        let encryption_time = start_time.elapsed();

        // Update statistics
        if let Some(stats) = self.stats.get_mut(&encryption_type) {
            stats.bytes_encrypted += data.len() as u64;
            stats.encryption_ops += 1;
            stats.encryption_time_us += encryption_time.as_micros() as u64;
        }

        debug!(
            "Encrypted {} bytes for connection {}",
            data.len(),
            connection_id
        );

        Ok(EncryptionResult {
            data: encrypted_data,
            nonce,
            tag,
            encryption_time_us: encryption_time.as_micros() as u64,
        })
    }

    /// Decrypt data for a connection
    pub fn decrypt(
        &mut self,
        connection_id: crate::network::ConnectionId,
        data: &[u8],
        nonce: &[u8],
        tag: &[u8],
    ) -> Result<DecryptionResult, EncryptionError> {
        let state = self
            .connection_states
            .get(&connection_id)
            .ok_or(EncryptionError::KeyNotFound(connection_id))?;

        if state.key.encryption_type == EncryptionType::None {
            return Ok(DecryptionResult {
                data: data.to_vec(),
                decryption_time_us: 0,
            });
        }

        let start_time = std::time::Instant::now();

        // Get config for AAD
        let config = self
            .configs
            .get(&state.key.encryption_type)
            .ok_or(EncryptionError::UnsupportedType(state.key.encryption_type))?;

        let decrypted_data = match state.key.encryption_type {
            EncryptionType::None => data.to_vec(),
            EncryptionType::Aes256Gcm => {
                self.decrypt_aes256_gcm(&state.key.key, nonce, data, tag, &config.aad)?
            }
            EncryptionType::ChaCha20Poly1305 => {
                self.decrypt_chacha20_poly1305(&state.key.key, nonce, data, tag, &config.aad)?
            }
            EncryptionType::Custom => {
                return Err(EncryptionError::UnsupportedType(state.key.encryption_type));
            }
        };

        let decryption_time = start_time.elapsed();

        // Update statistics
        if let Some(stats) = self.stats.get_mut(&state.key.encryption_type) {
            stats.bytes_decrypted += decrypted_data.len() as u64;
            stats.decryption_ops += 1;
            stats.decryption_time_us += decryption_time.as_micros() as u64;
        }

        debug!(
            "Decrypted {} bytes for connection {}",
            decrypted_data.len(),
            connection_id
        );

        Ok(DecryptionResult {
            data: decrypted_data,
            decryption_time_us: decryption_time.as_micros() as u64,
        })
    }

    /// Generate nonce for encryption type
    fn generate_nonce(
        &mut self,
        encryption_type: &EncryptionType,
        counter: u64,
    ) -> Result<Vec<u8>, EncryptionError> {
        match encryption_type {
            EncryptionType::None => Ok(Vec::new()),
            EncryptionType::Aes256Gcm => {
                // AES-GCM typically uses 12-byte nonces
                let mut nonce = vec![0u8; 12];
                nonce[4..12].copy_from_slice(&counter.to_le_bytes());
                Ok(nonce)
            }
            EncryptionType::ChaCha20Poly1305 => {
                // ChaCha20-Poly1305 uses 12-byte nonces
                let mut nonce = vec![0u8; 12];
                nonce[4..12].copy_from_slice(&counter.to_le_bytes());
                Ok(nonce)
            }
            EncryptionType::Custom => Err(EncryptionError::UnsupportedType(*encryption_type)),
        }
    }

    /// AES-256-GCM encryption implementation
    fn encrypt_aes256_gcm(
        &mut self,
        key: &[u8],
        _nonce: &[u8],
        data: &[u8],
        _aad: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        // TODO: Implement actual AES-256-GCM encryption using aes-gcm crate
        // For now, return unencrypted data
        warn!("AES-256-GCM encryption not implemented, returning unencrypted data");
        Ok((data.to_vec(), vec![0u8; 16])) // 16-byte dummy tag
    }

    /// AES-256-GCM decryption implementation
    fn decrypt_aes256_gcm(
        &self,
        key: &[u8],
        _nonce: &[u8],
        data: &[u8],
        _tag: &[u8],
        _aad: &[u8],
    ) -> Result<Vec<u8>, EncryptionError> {
        // TODO: Implement actual AES-256-GCM decryption using aes-gcm crate
        // For now, return data as-is
        warn!("AES-256-GCM decryption not implemented, returning data as-is");
        Ok(data.to_vec())
    }

    /// ChaCha20-Poly1305 encryption implementation
    fn encrypt_chacha20_poly1305(
        &mut self,
        _key: &[u8],
        _nonce: &[u8],
        data: &[u8],
        _aad: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>), EncryptionError> {
        // TODO: Implement actual ChaCha20-Poly1305 encryption using chacha20poly1305 crate
        // For now, return unencrypted data
        warn!("ChaCha20-Poly1305 encryption not implemented, returning unencrypted data");
        Ok((data.to_vec(), vec![0u8; 16])) // 16-byte dummy tag
    }

    /// ChaCha20-Poly1305 decryption implementation
    fn decrypt_chacha20_poly1305(
        &self,
        key: &[u8],
        _nonce: &[u8],
        data: &[u8],
        _tag: &[u8],
        _aad: &[u8],
    ) -> Result<Vec<u8>, EncryptionError> {
        // TODO: Implement actual ChaCha20-Poly1305 decryption using chacha20poly1305 crate
        // For now, return data as-is
        warn!("ChaCha20-Poly1305 decryption not implemented, returning data as-is");
        Ok(data.to_vec())
    }

    /// Remove connection encryption state
    pub fn remove_connection(&mut self, connection_id: crate::network::ConnectionId) {
        self.connection_states.remove(&connection_id);
        debug!("Removed encryption state for connection {}", connection_id);
    }

    /// Check if connection has encryption enabled
    pub fn is_encrypted(&self, connection_id: crate::network::ConnectionId) -> bool {
        self.connection_states
            .get(&connection_id)
            .map(|state| state.key.encryption_type != EncryptionType::None)
            .unwrap_or(false)
    }

    /// Get encryption statistics
    pub fn get_stats(&self, encryption_type: EncryptionType) -> Option<&EncryptionStats> {
        self.stats.get(&encryption_type)
    }

    /// Get all encryption statistics
    pub fn get_all_stats(&self) -> &HashMap<EncryptionType, EncryptionStats> {
        &self.stats
    }

    /// Get supported encryption types
    pub fn get_supported_types(&self) -> Vec<EncryptionType> {
        self.configs.keys().cloned().collect()
    }
}

impl Default for EncryptionManager {
    fn default() -> Self {
        Self::new()
    }
}
