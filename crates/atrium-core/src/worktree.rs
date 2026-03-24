//! Git worktree CRUD operations.
//!
//! Provides enumeration, creation, and removal of git worktrees
//! using gix (Gitoxide) for discovery and git2 for operations.

use atrium_error::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};

/// A git worktree summary.
#[derive(Debug, Clone)]
pub struct Worktree {
    pub path: PathBuf,
    pub head: Option<String>,
    pub branch: Option<String>,
    pub is_bare: bool,
    pub is_detached: bool,
    pub lock_reason: Option<String>,
}

/// Options for creating a new worktree.
#[derive(Debug, Clone, Default)]
pub struct AddWorktreeOptions {
    pub branch: Option<String>,
    pub detach: bool,
    pub force: bool,
}

/// Finds the repository root from any path inside it.
pub fn repo_root(path: &Path) -> Result<PathBuf> {
    let repo = open_git2_repo(path)?;
    main_worktree_path(&repo)
}

/// Lists all worktrees for a repository.
pub fn list(repo_path: &Path) -> Result<Vec<Worktree>> {
    let repo = open_git2_repo(repo_path)?;
    let mut worktrees = Vec::new();

    // Main worktree
    worktrees.push(build_main_worktree(&repo, repo_path));

    // Linked worktrees
    let names = repo.worktrees().map_err(|e| {
        Error::new(ErrorKind::Git, "failed to list worktrees")
            .with_operation("worktree_list")
            .with_context("repo", repo_path.display().to_string())
            .set_source(e)
    })?;

    for name in &names {
        if let Some(name) = name {
            if let Some(wt) = build_linked_worktree(&repo, name) {
                worktrees.push(wt);
            }
        }
    }

    Ok(worktrees)
}

/// Removes a worktree by path.
pub fn remove(repo_path: &Path, worktree_path: &Path) -> Result<()> {
    let repo = open_git2_repo(repo_path)?;
    let names = repo.worktrees().map_err(|e| {
        Error::new(ErrorKind::Git, "failed to list worktrees for removal")
            .with_operation("worktree_remove")
            .set_source(e)
    })?;

    for name in names.iter().flatten() {
        if let Ok(wt) = repo.find_worktree(name) {
            if let Some(wt_path) = wt.path().to_str() {
                if paths_equivalent(Path::new(wt_path), worktree_path) {
                    wt.prune(None).map_err(|e| {
                        Error::new(ErrorKind::Git, "failed to prune worktree")
                            .with_operation("worktree_remove")
                            .with_context("path", worktree_path.display().to_string())
                            .set_source(e)
                    })?;
                    return Ok(());
                }
            }
        }
    }

    Err(Error::new(ErrorKind::NotFound, "worktree not found")
        .with_operation("worktree_remove")
        .with_context("path", worktree_path.display().to_string()))
}

// ── Internal helpers ────────────────────────────────────────────────

fn open_git2_repo(path: &Path) -> Result<git2::Repository> {
    git2::Repository::discover(path).map_err(|e| {
        Error::new(ErrorKind::Git, "failed to open repository")
            .with_operation("open_repository")
            .with_context("path", path.display().to_string())
            .set_source(e)
    })
}

fn main_worktree_path(repo: &git2::Repository) -> Result<PathBuf> {
    repo.workdir()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| {
            Error::new(ErrorKind::Git, "repository has no working directory")
                .with_operation("repo_root")
        })
}

fn build_main_worktree(repo: &git2::Repository, repo_path: &Path) -> Worktree {
    let head = repo.head().ok();
    Worktree {
        path: repo_path.to_path_buf(),
        head: head.as_ref().and_then(|h| h.target()).map(|oid| oid.to_string()),
        branch: head.as_ref().and_then(|h| h.shorthand().map(String::from)),
        is_bare: repo.is_bare(),
        is_detached: repo.head_detached().unwrap_or(false),
        lock_reason: None,
    }
}

fn build_linked_worktree(repo: &git2::Repository, name: &str) -> Option<Worktree> {
    let wt = repo.find_worktree(name).ok()?;
    let wt_repo = git2::Repository::open_from_worktree(&wt).ok()?;
    let head = wt_repo.head().ok();
    let lock_reason = match wt.is_locked() {
        Ok(git2::WorktreeLockStatus::Locked(reason)) => {
            Some(reason.unwrap_or_else(|| "locked".to_owned()))
        }
        _ => None,
    };

    Some(Worktree {
        path: wt.path().to_path_buf(),
        head: head.as_ref().and_then(|h| h.target()).map(|oid| oid.to_string()),
        branch: head.as_ref().and_then(|h| h.shorthand().map(String::from)),
        is_bare: false,
        is_detached: wt_repo.head_detached().unwrap_or(false),
        lock_reason,
    })
}

/// Compare two paths ignoring trailing slashes and case on Windows.
pub fn paths_equivalent(left: &Path, right: &Path) -> bool {
    let left = left.to_string_lossy().trim_end_matches(['/', '\\']).to_owned();
    let right = right.to_string_lossy().trim_end_matches(['/', '\\']).to_owned();
    if cfg!(windows) {
        left.eq_ignore_ascii_case(&right)
    } else {
        left == right
    }
}
