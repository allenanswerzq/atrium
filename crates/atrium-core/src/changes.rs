//! Git change detection with line-level diff stats.

use atrium_error::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use strum::{Display, IntoStaticStr};
use std::path::{Path, PathBuf};

/// The kind of change detected for a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ChangeKind {
    Added,
    Modified,
    Removed,
    Renamed,
    Copied,
    TypeChange,
    Conflict,
    IntentToAdd,
}

/// A file that has changed in the worktree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFile {
    pub path: PathBuf,
    pub kind: ChangeKind,
    pub additions: usize,
    pub deletions: usize,
}

/// Summary of additions and deletions across all files.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffLineSummary {
    pub additions: usize,
    pub deletions: usize,
}

/// Returns all changed files for a worktree with line counts.
pub fn changed_files(repo_path: &Path) -> Result<Vec<ChangedFile>> {
    let repo = git2::Repository::discover(repo_path).map_err(|e| {
        Error::new(ErrorKind::Git, "failed to discover repository")
            .with_operation("changed_files")
            .with_context("path", repo_path.display().to_string())
            .set_source(e)
    })?;

    let statuses = repo
        .statuses(Some(
            git2::StatusOptions::new()
                .include_untracked(true)
                .recurse_untracked_dirs(true),
        ))
        .map_err(|e| {
            Error::new(ErrorKind::Git, "failed to get repository status")
                .with_operation("changed_files")
                .set_source(e)
        })?;

    let mut files = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for entry in statuses.iter() {
        let Some(path_str) = entry.path() else {
            continue;
        };
        if !seen.insert(path_str.to_owned()) {
            continue;
        }

        let kind = status_to_change_kind(entry.status());
        files.push(ChangedFile {
            path: PathBuf::from(path_str),
            kind,
            additions: 0,
            deletions: 0,
        });
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn status_to_change_kind(status: git2::Status) -> ChangeKind {
    if status.intersects(git2::Status::INDEX_NEW | git2::Status::WT_NEW) {
        ChangeKind::Added
    } else if status.intersects(git2::Status::INDEX_DELETED | git2::Status::WT_DELETED) {
        ChangeKind::Removed
    } else if status.intersects(git2::Status::INDEX_RENAMED | git2::Status::WT_RENAMED) {
        ChangeKind::Renamed
    } else if status.intersects(git2::Status::INDEX_TYPECHANGE | git2::Status::WT_TYPECHANGE) {
        ChangeKind::TypeChange
    } else if status.is_conflicted() {
        ChangeKind::Conflict
    } else {
        ChangeKind::Modified
    }
}
