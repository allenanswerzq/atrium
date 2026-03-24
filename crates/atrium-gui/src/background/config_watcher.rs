//! Auto-reload config file on changes.

use std::time::Duration;

/// Interval between config file checks.
pub const CONFIG_CHECK_INTERVAL: Duration = Duration::from_secs(5);

/// Watches the config file for modifications and triggers reload.
pub struct ConfigWatcher;

impl ConfigWatcher {
    /// Start watching. Returns a handle that keeps the watcher alive.
    pub fn start() -> Self {
        // TODO: spawn background task that stats the config file periodically
        tracing::debug!("config watcher initialized");
        Self
    }
}
