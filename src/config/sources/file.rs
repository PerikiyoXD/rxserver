//! File-based configuration source

use crate::config::formats::{ConfigFormat, FormatDetector};
use crate::config::sources::ConfigSource;
use crate::config::types::ServerConfig;
use crate::types::{ConfigurationError, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;

/// File-based configuration source
pub struct FileSource {
    path: PathBuf,
    format: Option<ConfigFormat>,
    priority: u32,
}

impl FileSource {
    /// Create a new file source
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Detect format from file extension
        let format = ConfigFormat::from_extension(&path);

        Ok(Self {
            path,
            format,
            priority: 100, // Default priority for file sources
        })
    }

    /// Create a new file source with explicit format
    pub fn with_format<P: AsRef<Path>>(path: P, format: ConfigFormat) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            format: Some(format),
            priority: 100,
        }
    }

    /// Create a new file source with custom priority
    pub fn with_priority<P: AsRef<Path>>(path: P, priority: u32) -> Result<Self> {
        let mut source = Self::new(path)?;
        source.priority = priority;
        Ok(source)
    }

    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Check if the file exists
    pub async fn exists(&self) -> bool {
        fs::metadata(&self.path).await.is_ok()
    }

    /// Get file metadata
    pub async fn metadata(&self) -> Result<std::fs::Metadata> {
        fs::metadata(&self.path).await.map_err(|e| {
            ConfigurationError::FileError {
                path: self.path.clone(),
                message: e.to_string(),
            }
            .into()
        })
    }

    /// Read file content
    async fn read_content(&self) -> Result<String> {
        fs::read_to_string(&self.path).await.map_err(|e| {
            ConfigurationError::FileError {
                path: self.path.clone(),
                message: e.to_string(),
            }
            .into()
        })
    }

    /// Detect format from content if not explicitly set
    fn detect_format(&self, content: &str) -> ConfigFormat {
        if let Some(format) = self.format {
            format
        } else {
            FormatDetector::detect(&self.path, Some(content))
        }
    }
}

#[async_trait]
impl ConfigSource for FileSource {
    async fn load(&self) -> Result<ServerConfig> {
        // Check if file exists
        if !self.exists().await {
            return Err(ConfigurationError::FileError {
                path: self.path.clone(),
                message: "File does not exist".to_string(),
            }
            .into());
        }

        // Read file content
        let content = self.read_content().await?;

        // Detect format
        let format = self.detect_format(&content);

        // Parse configuration
        format.parse(&content)
    }

    fn identifier(&self) -> String {
        format!("file:{}", self.path.display())
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn supports_watch(&self) -> bool {
        true
    }

    async fn start_watch(&self) -> Result<()> {
        // TODO: Implement file watching using tokio::fs::watch or similar
        // For now, just log that watching is not implemented
        log::info!(
            "File watching for '{}' is not yet implemented",
            self.path.display()
        );
        Ok(())
    }

    async fn stop_watch(&self) -> Result<()> {
        // TODO: Implement stopping file watch
        Ok(())
    }
}

/// Multiple file source that loads from multiple files
pub struct MultiFileSource {
    sources: Vec<FileSource>,
    priority: u32,
}

impl MultiFileSource {
    /// Create a new multi-file source
    pub fn new(paths: Vec<PathBuf>) -> Result<Self> {
        let mut sources = Vec::new();

        for path in paths {
            match FileSource::new(path) {
                Ok(source) => sources.push(source),
                Err(e) => log::warn!("Failed to create file source: {}", e),
            }
        }

        Ok(Self {
            sources,
            priority: 100,
        })
    }

    /// Set priority for all file sources
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        for source in &mut self.sources {
            source.priority = priority;
        }
        self
    }

    /// Add a file source
    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let mut source = FileSource::new(path)?;
        source.priority = self.priority;
        self.sources.push(source);
        Ok(())
    }

    /// Get all file sources
    pub fn sources(&self) -> &[FileSource] {
        &self.sources
    }
}

#[async_trait]
impl ConfigSource for MultiFileSource {
    async fn load(&self) -> Result<ServerConfig> {
        let mut config = ServerConfig::default();

        // Load from each file source and merge
        for source in &self.sources {
            match source.load().await {
                Ok(file_config) => {
                    config = config.merge(file_config)?;
                }
                Err(e) => {
                    log::warn!("Failed to load from {}: {}", source.identifier(), e);
                }
            }
        }

        Ok(config)
    }

    fn identifier(&self) -> String {
        let paths: Vec<String> = self
            .sources
            .iter()
            .map(|s| s.path().display().to_string())
            .collect();
        format!("multi-file:[{}]", paths.join(","))
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn supports_watch(&self) -> bool {
        true
    }

    async fn start_watch(&self) -> Result<()> {
        for source in &self.sources {
            source.start_watch().await?;
        }
        Ok(())
    }

    async fn stop_watch(&self) -> Result<()> {
        for source in &self.sources {
            source.stop_watch().await?;
        }
        Ok(())
    }
}

/// Directory-based configuration source that loads all config files from a directory
pub struct DirectorySource {
    directory: PathBuf,
    pattern: String,
    recursive: bool,
    priority: u32,
}

impl DirectorySource {
    /// Create a new directory source
    pub fn new<P: AsRef<Path>>(directory: P) -> Self {
        Self {
            directory: directory.as_ref().to_path_buf(),
            pattern: "*.toml".to_string(),
            recursive: false,
            priority: 100,
        }
    }

    /// Set file pattern to match
    pub fn with_pattern(mut self, pattern: String) -> Self {
        self.pattern = pattern;
        self
    }

    /// Enable recursive directory scanning
    pub fn recursive(mut self) -> Self {
        self.recursive = true;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Find configuration files in the directory (iterative to avoid async recursion)
    async fn find_config_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut dirs_to_visit = vec![self.directory.clone()];

        while let Some(current_dir) = dirs_to_visit.pop() {
            if !current_dir.exists() {
                continue;
            }
            let mut dir_reader = match fs::read_dir(&current_dir).await {
                Ok(reader) => reader,
                Err(e) => {
                    return Err(crate::types::Error::Configuration(
                        ConfigurationError::FileError {
                            path: current_dir.clone(),
                            message: e.to_string(),
                        },
                    ));
                }
            };

            while let Some(entry) =
                dir_reader
                    .next_entry()
                    .await
                    .map_err(|e| ConfigurationError::FileError {
                        path: current_dir.clone(),
                        message: e.to_string(),
                    })?
            {
                let path = entry.path();

                if path.is_file() {
                    // Check if file matches pattern (simple glob matching)
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if self.matches_pattern(name) {
                            files.push(path);
                        }
                    }
                } else if path.is_dir() && self.recursive {
                    dirs_to_visit.push(path);
                }
            }
        }

        // Sort files for consistent ordering
        files.sort();
        Ok(files)
    }

    /// Simple pattern matching (supports * wildcard)
    fn matches_pattern(&self, filename: &str) -> bool {
        if self.pattern == "*" {
            return true;
        }

        if let Some(star_pos) = self.pattern.find('*') {
            let prefix = &self.pattern[..star_pos];
            let suffix = &self.pattern[star_pos + 1..];

            filename.starts_with(prefix) && filename.ends_with(suffix)
        } else {
            filename == self.pattern
        }
    }
}

#[async_trait]
impl ConfigSource for DirectorySource {
    async fn load(&self) -> Result<ServerConfig> {
        let files = self.find_config_files().await?;

        if files.is_empty() {
            return Ok(ServerConfig::default());
        }

        let multi_source = MultiFileSource::new(files)?.with_priority(self.priority);

        multi_source.load().await
    }

    fn identifier(&self) -> String {
        format!("directory:{}", self.directory.display())
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn supports_watch(&self) -> bool {
        true
    }

    async fn start_watch(&self) -> Result<()> {
        // TODO: Implement directory watching
        log::info!(
            "Directory watching for '{}' is not yet implemented",
            self.directory.display()
        );
        Ok(())
    }

    async fn stop_watch(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_file_source() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
        [server]
        name = "TestServer"
        
        [network]
        max_connections = 42
        "#
        )
        .unwrap();

        let source = FileSource::new(temp_file.path()).unwrap();
        assert!(source.exists().await);

        let config = source.load().await.unwrap();
        assert_eq!(config.server.name, "TestServer");
        assert_eq!(config.network.max_connections, 42);
    }

    #[test]
    fn test_pattern_matching() {
        let source = DirectorySource::new("/tmp").with_pattern("*.toml".to_string());

        assert!(source.matches_pattern("config.toml"));
        assert!(source.matches_pattern("test.toml"));
        assert!(!source.matches_pattern("config.json"));
        assert!(!source.matches_pattern("config"));

        let source = DirectorySource::new("/tmp").with_pattern("config.*".to_string());
        assert!(source.matches_pattern("config.toml"));
        assert!(source.matches_pattern("config.json"));
        assert!(!source.matches_pattern("other.toml"));
    }
}
