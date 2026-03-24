//! Check for new releases on GitHub.

/// Version check state.
pub struct VersionCheck;

impl VersionCheck {
    /// Start a background check for updates.
    pub fn start() -> Self {
        // TODO: query GitHub releases API in background
        tracing::debug!("version check initialized");
        Self
    }
}
