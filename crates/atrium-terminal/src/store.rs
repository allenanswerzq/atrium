//! Session persistence — save/load session records to disk.

use atrium_error::Result;

use crate::types::SessionRecord;

/// Trait for persisting terminal session records.
pub trait SessionStore: Send + Sync {
    fn load(&self) -> Result<Vec<SessionRecord>>;
    fn save(&self, records: &[SessionRecord]) -> Result<()>;
}

/// JSON file-backed session store.
pub struct JsonSessionStore {
    path: std::path::PathBuf,
}

impl JsonSessionStore {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl SessionStore for JsonSessionStore {
    fn load(&self) -> Result<Vec<SessionRecord>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        atrium_fs::read_json(&self.path)
    }

    fn save(&self, records: &[SessionRecord]) -> Result<()> {
        atrium_fs::write_json(&self.path, &records)
    }
}
