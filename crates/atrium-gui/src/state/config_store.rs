//! Application configuration store.
//!
//! Loads and persists app-level config (daemon bind mode, notification
//! preferences, etc.) via `atrium-fs` JSON helpers.

use std::path::PathBuf;

use atrium_error::Result;

/// Application-level configuration (not per-repo).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    /// Last-used daemon base URL.
    pub daemon_url: Option<String>,
    /// Whether desktop notifications are enabled.
    pub notifications_enabled: bool,
}

/// Manages loading and saving of [`AppConfig`].
#[derive(Debug, Clone)]
pub struct ConfigStore {
    path: PathBuf,
    config: AppConfig,
}

impl ConfigStore {
    /// Create a store backed by the given file path.
    pub fn new(path: PathBuf) -> Self {
        let config = Self::load_from(&path).unwrap_or_default();
        Self { path, config }
    }

    /// Current config snapshot.
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Update the config and persist to disk.
    pub fn update(&mut self, config: AppConfig) -> Result<()> {
        self.config = config;
        atrium_fs::write_json(&self.path, &self.config)
    }

    fn load_from(path: &PathBuf) -> Result<AppConfig> {
        atrium_fs::read_json(path)
    }
}
