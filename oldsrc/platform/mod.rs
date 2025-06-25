//! Platform detection and abstraction layer
//!
//! This module provides platform-specific implementations and abstractions
//! for cross-platform compatibility.

use std::fmt;

pub mod common;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "freebsd")]
pub mod freebsd;

/// Supported platform types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Windows,
    MacOS,
    FreeBSD,
    Unknown,
}

impl Platform {
    /// Detect the current platform
    pub fn current() -> Self {
        #[cfg(target_os = "linux")]
        return Platform::Linux;

        #[cfg(target_os = "windows")]
        return Platform::Windows;

        #[cfg(target_os = "macos")]
        return Platform::MacOS;

        #[cfg(target_os = "freebsd")]
        return Platform::FreeBSD;

        #[cfg(all(
            not(target_os = "linux"),
            not(target_os = "windows"),
            not(target_os = "macos"),
            not(target_os = "freebsd")
        ))]
        Platform::Unknown
    }

    /// Get platform-specific capabilities
    pub fn capabilities(&self) -> PlatformCapabilities {
        match self {
            Platform::Linux => PlatformCapabilities {
                unix_sockets: true,
                tcp_sockets: true,
                shared_memory: true,
                hardware_acceleration: true,
                window_system: WindowSystem::X11,
            },
            Platform::Windows => PlatformCapabilities {
                unix_sockets: false,
                tcp_sockets: true,
                shared_memory: true,
                hardware_acceleration: true,
                window_system: WindowSystem::Win32,
            },
            Platform::MacOS => PlatformCapabilities {
                unix_sockets: true,
                tcp_sockets: true,
                shared_memory: true,
                hardware_acceleration: true,
                window_system: WindowSystem::Quartz,
            },
            Platform::FreeBSD => PlatformCapabilities {
                unix_sockets: true,
                tcp_sockets: true,
                shared_memory: true,
                hardware_acceleration: true,
                window_system: WindowSystem::X11,
            },
            Platform::Unknown => PlatformCapabilities {
                unix_sockets: false,
                tcp_sockets: true,
                shared_memory: false,
                hardware_acceleration: false,
                window_system: WindowSystem::None,
            },
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::Linux => write!(f, "Linux"),
            Platform::Windows => write!(f, "Windows"),
            Platform::MacOS => write!(f, "macOS"),
            Platform::FreeBSD => write!(f, "FreeBSD"),
            Platform::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Platform capabilities
#[derive(Debug, Clone)]
pub struct PlatformCapabilities {
    pub unix_sockets: bool,
    pub tcp_sockets: bool,
    pub shared_memory: bool,
    pub hardware_acceleration: bool,
    pub window_system: WindowSystem,
}

/// Native window system types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowSystem {
    X11,
    Win32,
    Quartz,
    None,
}

/// Platform abstraction traits
pub trait PlatformAbstraction {
    /// Initialize platform-specific resources
    fn initialize(&mut self) -> Result<(), PlatformError>;

    /// Cleanup platform-specific resources
    fn cleanup(&mut self) -> Result<(), PlatformError>;

    /// Get platform information
    fn platform_info(&self) -> PlatformInfo;
}

/// Platform information
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub platform: Platform,
    pub version: String,
    pub architecture: String,
    pub capabilities: PlatformCapabilities,
}

/// Platform-specific errors
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("Platform not supported: {0}")]
    Unsupported(String),

    #[error("Platform initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Platform feature not available: {0}")]
    FeatureUnavailable(String),

    #[error("Platform I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Platform factory for creating platform-specific implementations
pub struct PlatformFactory;

impl PlatformFactory {
    /// Create platform-specific implementation
    pub fn create() -> Result<Box<dyn PlatformAbstraction>, PlatformError> {
        let platform = Platform::current();

        match platform {
            #[cfg(target_os = "linux")]
            Platform::Linux => Ok(Box::new(linux::LinuxPlatform::new()?)),

            #[cfg(target_os = "windows")]
            Platform::Windows => Ok(Box::new(windows::WindowsPlatform::new()?)),

            #[cfg(target_os = "macos")]
            Platform::MacOS => Ok(Box::new(macos::MacOSPlatform::new()?)),

            #[cfg(target_os = "freebsd")]
            Platform::FreeBSD => Ok(Box::new(freebsd::FreeBSDPlatform::new()?)),

            _ => Err(PlatformError::Unsupported(format!(
                "Platform {} not supported",
                platform
            ))),
        }
    }
}

/// Get current platform information
pub fn get_platform_info() -> PlatformInfo {
    let platform = Platform::current();
    let capabilities = platform.capabilities();

    PlatformInfo {
        platform,
        version: get_platform_version(),
        architecture: get_architecture(),
        capabilities,
    }
}

/// Get platform version string
pub fn get_platform_version() -> String {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/version")
            .unwrap_or_else(|_| "Unknown Linux".to_string())
            .trim()
            .to_string()
    }

    #[cfg(target_os = "windows")]
    {
        use winapi::um::sysinfoapi::GetVersionExW;
        use winapi::um::winnt::OSVERSIONINFOW;
        let mut osvi: OSVERSIONINFOW = unsafe { std::mem::zeroed() };
        osvi.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;
        unsafe {
            if GetVersionExW(&mut osvi) == 0 {
                panic!("Failed to get Windows version");
            }
        }
        format!(
            "{}.{}.{}",
            osvi.dwMajorVersion, osvi.dwMinorVersion, osvi.dwBuildNumber
        )
    }

    #[cfg(target_os = "macos")]
    {
        // Use sysctl to get macOS version
        use std::process::Command;
        let output = Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .unwrap_or_else(|_| panic!("Failed to get macOS version"));
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    #[cfg(target_os = "freebsd")]
    {
        // Use sysctl to get FreeBSD version
        use std::process::Command;
        let output = Command::new("uname")
            .arg("-r")
            .output()
            .unwrap_or_else(|_| panic!("Failed to get FreeBSD version"));
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos",
        target_os = "freebsd"
    )))]
    {
        "Unknown".to_string()
    }
}

/// Get architecture string
pub fn get_architecture() -> String {
    std::env::consts::ARCH.to_string()
}
