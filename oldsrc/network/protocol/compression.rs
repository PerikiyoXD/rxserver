//! Data compression implementation
//!
//! Provides compression and decompression capabilities for network data.

use std::collections::HashMap;
use tracing::{debug, warn};

/// Compression type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionType {
    /// No compression
    None,
    /// DEFLATE compression (zlib)
    Deflate,
    /// LZ4 compression
    Lz4,
    /// Brotli compression
    Brotli,
    /// Custom compression
    Custom,
}

/// Compression error
#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    #[error("Compression failed: {0}")]
    CompressionFailed(String),

    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    #[error("Unsupported compression type: {0:?}")]
    UnsupportedType(CompressionType),

    #[error("Invalid compressed data")]
    InvalidData,

    #[error("Buffer too small: need {0} bytes")]
    BufferTooSmall(usize),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Compression type
    pub compression_type: CompressionType,
    /// Compression level (0-9, meaning depends on algorithm)
    pub level: u32,
    /// Minimum data size to compress
    pub min_compress_size: usize,
    /// Maximum compression ratio (compressed_size / original_size)
    pub max_compression_ratio: f32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            compression_type: CompressionType::None,
            level: 6,
            min_compress_size: 128,
            max_compression_ratio: 1.5,
        }
    }
}

/// Compression statistics
#[derive(Debug, Clone)]
pub struct CompressionStats {
    /// Total bytes compressed
    pub bytes_compressed: u64,
    /// Total bytes decompressed
    pub bytes_decompressed: u64,
    /// Total compression operations
    pub compression_ops: u64,
    /// Total decompression operations
    pub decompression_ops: u64,
    /// Average compression ratio
    pub avg_compression_ratio: f32,
    /// Total time spent compressing (microseconds)
    pub compression_time_us: u64,
    /// Total time spent decompressing (microseconds)
    pub decompression_time_us: u64,
}

impl Default for CompressionStats {
    fn default() -> Self {
        Self {
            bytes_compressed: 0,
            bytes_decompressed: 0,
            compression_ops: 0,
            decompression_ops: 0,
            avg_compression_ratio: 1.0,
            compression_time_us: 0,
            decompression_time_us: 0,
        }
    }
}

/// Compression result
#[derive(Debug)]
pub struct CompressionResult {
    /// Compressed data
    pub data: Vec<u8>,
    /// Original size
    pub original_size: usize,
    /// Compression ratio (compressed_size / original_size)
    pub compression_ratio: f32,
    /// Compression time in microseconds
    pub compression_time_us: u64,
}

/// Decompression result
#[derive(Debug)]
pub struct DecompressionResult {
    /// Decompressed data
    pub data: Vec<u8>,
    /// Decompression time in microseconds
    pub decompression_time_us: u64,
}

/// Compression manager
pub struct CompressionManager {
    /// Compression configurations by type
    configs: HashMap<CompressionType, CompressionConfig>,
    /// Compression statistics by type
    stats: HashMap<CompressionType, CompressionStats>,
}

impl CompressionManager {
    /// Create a new compression manager
    pub fn new() -> Self {
        let mut manager = Self {
            configs: HashMap::new(),
            stats: HashMap::new(),
        };

        // Register default configurations
        manager.register_config(CompressionConfig {
            compression_type: CompressionType::None,
            ..Default::default()
        });

        manager.register_config(CompressionConfig {
            compression_type: CompressionType::Deflate,
            level: 6,
            ..Default::default()
        });

        manager.register_config(CompressionConfig {
            compression_type: CompressionType::Lz4,
            level: 1,
            ..Default::default()
        });

        manager
    }

    /// Register compression configuration
    pub fn register_config(&mut self, config: CompressionConfig) {
        let compression_type = config.compression_type;
        debug!("Registering compression config: {:?}", compression_type);
        self.configs.insert(compression_type, config);
        self.stats
            .insert(compression_type, CompressionStats::default());
    }

    /// Compress data
    pub fn compress(
        &mut self,
        data: &[u8],
        compression_type: CompressionType,
    ) -> Result<CompressionResult, CompressionError> {
        let config = self
            .configs
            .get(&compression_type)
            .ok_or(CompressionError::UnsupportedType(compression_type))?;

        // Check minimum size threshold
        if data.len() < config.min_compress_size {
            debug!(
                "Data too small for compression: {} bytes (min: {})",
                data.len(),
                config.min_compress_size
            );

            return Ok(CompressionResult {
                data: data.to_vec(),
                original_size: data.len(),
                compression_ratio: 1.0,
                compression_time_us: 0,
            });
        }

        let start_time = std::time::Instant::now();

        let compressed_data = match compression_type {
            CompressionType::None => data.to_vec(),
            CompressionType::Deflate => self.compress_deflate(data, config.level)?,
            CompressionType::Lz4 => self.compress_lz4(data)?,
            CompressionType::Brotli => self.compress_brotli(data, config.level)?,
            CompressionType::Custom => {
                return Err(CompressionError::UnsupportedType(compression_type));
            }
        };

        let compression_time = start_time.elapsed();
        let compression_ratio = compressed_data.len() as f32 / data.len() as f32;

        // Check if compression was beneficial
        if compression_ratio > config.max_compression_ratio {
            debug!(
                "Compression not beneficial: ratio={:.2}, using uncompressed data",
                compression_ratio
            );

            return Ok(CompressionResult {
                data: data.to_vec(),
                original_size: data.len(),
                compression_ratio: 1.0,
                compression_time_us: compression_time.as_micros() as u64,
            });
        }

        // Update statistics
        if let Some(stats) = self.stats.get_mut(&compression_type) {
            stats.bytes_compressed += data.len() as u64;
            stats.compression_ops += 1;
            stats.compression_time_us += compression_time.as_micros() as u64;

            // Update average compression ratio
            let total_ops = stats.compression_ops as f32;
            stats.avg_compression_ratio =
                (stats.avg_compression_ratio * (total_ops - 1.0) + compression_ratio) / total_ops;
        }

        debug!(
            "Compressed {} bytes to {} bytes (ratio: {:.2})",
            data.len(),
            compressed_data.len(),
            compression_ratio
        );

        Ok(CompressionResult {
            data: compressed_data,
            original_size: data.len(),
            compression_ratio,
            compression_time_us: compression_time.as_micros() as u64,
        })
    }

    /// Decompress data
    pub fn decompress(
        &mut self,
        data: &[u8],
        compression_type: CompressionType,
        expected_size: Option<usize>,
    ) -> Result<DecompressionResult, CompressionError> {
        if compression_type == CompressionType::None {
            return Ok(DecompressionResult {
                data: data.to_vec(),
                decompression_time_us: 0,
            });
        }

        let start_time = std::time::Instant::now();

        let decompressed_data = match compression_type {
            CompressionType::None => data.to_vec(),
            CompressionType::Deflate => self.decompress_deflate(data, expected_size)?,
            CompressionType::Lz4 => self.decompress_lz4(data, expected_size)?,
            CompressionType::Brotli => self.decompress_brotli(data, expected_size)?,
            CompressionType::Custom => {
                return Err(CompressionError::UnsupportedType(compression_type));
            }
        };

        let decompression_time = start_time.elapsed();

        // Update statistics
        if let Some(stats) = self.stats.get_mut(&compression_type) {
            stats.bytes_decompressed += decompressed_data.len() as u64;
            stats.decompression_ops += 1;
            stats.decompression_time_us += decompression_time.as_micros() as u64;
        }

        debug!(
            "Decompressed {} bytes to {} bytes",
            data.len(),
            decompressed_data.len()
        );

        Ok(DecompressionResult {
            data: decompressed_data,
            decompression_time_us: decompression_time.as_micros() as u64,
        })
    }

    /// DEFLATE compression implementation
    fn compress_deflate(&self, data: &[u8], level: u32) -> Result<Vec<u8>, CompressionError> {
        // TODO: Implement actual DEFLATE compression using flate2 crate
        // For now, return uncompressed data
        warn!("DEFLATE compression not implemented, returning uncompressed data");
        Ok(data.to_vec())
    }

    /// DEFLATE decompression implementation
    fn decompress_deflate(
        &self,
        data: &[u8],
        _expected_size: Option<usize>,
    ) -> Result<Vec<u8>, CompressionError> {
        // TODO: Implement actual DEFLATE decompression using flate2 crate
        // For now, return data as-is
        warn!("DEFLATE decompression not implemented, returning data as-is");
        Ok(data.to_vec())
    }

    /// LZ4 compression implementation
    fn compress_lz4(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // TODO: Implement actual LZ4 compression using lz4 crate
        // For now, return uncompressed data
        warn!("LZ4 compression not implemented, returning uncompressed data");
        Ok(data.to_vec())
    }

    /// LZ4 decompression implementation
    fn decompress_lz4(
        &self,
        data: &[u8],
        _expected_size: Option<usize>,
    ) -> Result<Vec<u8>, CompressionError> {
        // TODO: Implement actual LZ4 decompression using lz4 crate
        // For now, return data as-is
        warn!("LZ4 decompression not implemented, returning data as-is");
        Ok(data.to_vec())
    }

    /// Brotli compression implementation
    fn compress_brotli(&self, data: &[u8], level: u32) -> Result<Vec<u8>, CompressionError> {
        // TODO: Implement actual Brotli compression using brotli crate
        // For now, return uncompressed data
        warn!("Brotli compression not implemented, returning uncompressed data");
        Ok(data.to_vec())
    }

    /// Brotli decompression implementation
    fn decompress_brotli(
        &self,
        data: &[u8],
        _expected_size: Option<usize>,
    ) -> Result<Vec<u8>, CompressionError> {
        // TODO: Implement actual Brotli decompression using brotli crate
        // For now, return data as-is
        warn!("Brotli decompression not implemented, returning data as-is");
        Ok(data.to_vec())
    }

    /// Get compression statistics
    pub fn get_stats(&self, compression_type: CompressionType) -> Option<&CompressionStats> {
        self.stats.get(&compression_type)
    }

    /// Get all compression statistics
    pub fn get_all_stats(&self) -> &HashMap<CompressionType, CompressionStats> {
        &self.stats
    }

    /// Get supported compression types
    pub fn get_supported_types(&self) -> Vec<CompressionType> {
        self.configs.keys().cloned().collect()
    }

    /// Check if compression type is supported
    pub fn is_supported(&self, compression_type: CompressionType) -> bool {
        self.configs.contains_key(&compression_type)
    }
}

impl Default for CompressionManager {
    fn default() -> Self {
        Self::new()
    }
}
