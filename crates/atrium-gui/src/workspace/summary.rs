//! Build worktree summaries from repository data.

use std::path::PathBuf;

/// Summary of a worktree for display in the UI.
#[derive(Debug, Clone)]
pub struct WorktreeSummary {
    /// Worktree path.
    pub path: PathBuf,
    /// Branch name, if on a branch.
    pub branch: Option<String>,
    /// Whether this is the main worktree.
    pub is_main: bool,
}
