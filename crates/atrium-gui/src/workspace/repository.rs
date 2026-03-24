//! Repository scanning and active repo tracking.

use std::path::PathBuf;

/// Tracks scanned repositories and the active selection.
#[derive(Debug, Default)]
pub struct RepositoryState {
    /// Known repository root paths.
    roots: Vec<PathBuf>,
    /// Index of the currently active repository.
    active_index: Option<usize>,
}

impl RepositoryState {
    /// Add a repository root.
    pub fn add(&mut self, root: PathBuf) {
        if !self.roots.contains(&root) {
            self.roots.push(root);
        }
    }

    /// All known repository roots.
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// The currently active repository root.
    pub fn active_root(&self) -> Option<&PathBuf> {
        self.active_index.and_then(|i| self.roots.get(i))
    }

    /// Set the active repository by index.
    pub fn set_active(&mut self, index: usize) {
        if index < self.roots.len() {
            self.active_index = Some(index);
        }
    }
}
