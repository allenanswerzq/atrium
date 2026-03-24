//! Repository and worktree management.

mod navigation;
mod repository;
mod summary;
mod worktree;

pub use navigation::NavigationStack;
pub use repository::RepositoryState;
pub use summary::WorktreeSummary;
pub use worktree::WorktreeOps;

/// Combined workspace state — repos, worktrees, navigation.
#[derive(Debug, Default)]
pub struct WorkspaceState {
    pub repositories: RepositoryState,
    pub navigation: NavigationStack,
}
