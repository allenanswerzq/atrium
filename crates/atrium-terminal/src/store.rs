//! Session persistence — save/load session records to disk.

use atrium_error::Result;

use crate::types::TerminalSessionRecord;

/// Trait for persisting terminal session records.
pub trait TerminalSessionStore: Send + Sync {
    fn load(&self) -> Result<Vec<TerminalSessionRecord>>;
    fn save(&self, records: &[TerminalSessionRecord]) -> Result<()>;
}

/// JSON file-backed session store.
pub struct JsonTerminalSessionStore {
    path: std::path::PathBuf,
}

impl JsonTerminalSessionStore {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl TerminalSessionStore for JsonTerminalSessionStore {
    fn load(&self) -> Result<Vec<TerminalSessionRecord>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        atrium_fs::read_json(&self.path)
    }

    fn save(&self, records: &[TerminalSessionRecord]) -> Result<()> {
        atrium_fs::write_json(&self.path, &records)
    }
}
