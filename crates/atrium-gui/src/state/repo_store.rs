//! Tracked repository store.
//!
//! Persists the list of repositories the user has added to Atrium.

use std::path::PathBuf;

use atrium_error::Result;
use serde::{Deserialize, Serialize};

/// A tracked repository entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoEntry {
    /// Absolute path to the repository root.
    pub path: PathBuf,
    /// Optional display label.
    pub label: Option<String>,
}

/// Manages the persisted list of tracked repositories.
#[derive(Debug, Clone)]
pub struct RepoStore {
    path: PathBuf,
    entries: Vec<RepoEntry>,
}

impl RepoStore {
    /// Create a store backed by the given file path.
    pub fn new(path: PathBuf) -> Self {
        let entries = Self::load_from(&path).unwrap_or_default();
        Self { path, entries }
    }

    /// Current tracked repos.
    pub fn entries(&self) -> &[RepoEntry] {
        &self.entries
    }

    /// Add a repository and persist.
    pub fn add(&mut self, entry: RepoEntry) -> Result<()> {
        self.entries.push(entry);
        self.save()
    }

    /// Remove a repository by index and persist.
    pub fn remove(&mut self, index: usize) -> Result<()> {
        if index < self.entries.len() {
            self.entries.remove(index);
        }
        self.save()
    }

    fn save(&self) -> Result<()> {
        atrium_fs::write_json(&self.path, &self.entries)
    }

    fn load_from(path: &PathBuf) -> Result<Vec<RepoEntry>> {
        atrium_fs::read_json(path)
    }
}
