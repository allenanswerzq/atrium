//! Worktree lifecycle — create, delete, refresh.
//!
//! Delegates to `atrium_core::worktree` for git operations.

use std::path::Path;

use atrium_core::worktree::Worktree;
use atrium_error::Result;

/// Worktree operations namespace.
pub struct WorktreeOps;

impl WorktreeOps {
    /// List worktrees for a repository.
    pub fn list(repo_root: &Path) -> Result<Vec<Worktree>> {
        atrium_core::worktree::list(repo_root)
    }
}
