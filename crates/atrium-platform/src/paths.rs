//! Platform-specific standard directories.

use std::path::PathBuf;

use crate::Os;

/// Resolved platform-specific standard paths.
#[derive(Debug, Clone)]
pub struct PlatformPaths {
    /// User home directory.
    pub home: PathBuf,
    /// Config directory (e.g. `~/.config` on Linux, `~/AppData/Roaming` on Windows).
    pub config: PathBuf,
    /// Data directory (e.g. `~/.local/share` on Linux, `~/AppData/Local` on Windows).
    pub data: PathBuf,
    /// Temporary directory.
    pub temp: PathBuf,
}

impl PlatformPaths {
    /// Resolve standard paths for the current platform.
    pub fn resolve() -> Option<Self> {
        let home = home_dir()?;
        let os = Os::current();

        let config = match os {
            Os::Windows => std::env::var("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| home.join("AppData").join("Roaming")),
            Os::MacOs => home.join("Library").join("Application Support"),
            Os::Linux => std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| home.join(".config")),
        };

        let data = match os {
            Os::Windows => std::env::var("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| home.join("AppData").join("Local")),
            Os::MacOs => home.join("Library").join("Application Support"),
            Os::Linux => std::env::var("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| home.join(".local").join("share")),
        };

        let temp = std::env::temp_dir();

        Some(Self {
            home,
            config,
            data,
            temp,
        })
    }
}

/// Portable home directory detection.
fn home_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").ok().map(PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}
