//! Protocol types shared between frontend and backend.
//!
//! These are the wire types for terminal operations: requests,
//! responses, signals, lifecycle states, and snapshots.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use strum::{Display, IntoStaticStr};

use atrium_core::id::{SessionId, WorkspaceId};

use crate::styled::{Cursor, Modes, StyledLine};

// ── Requests ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequest {
    pub session_id: SessionId,
    pub workspace_id: WorkspaceId,
    pub cwd: PathBuf,
    pub shell: String,
    pub cols: u16,
    pub rows: u16,
    pub title: Option<String>,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteRequest {
    pub session_id: SessionId,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeRequest {
    pub session_id: SessionId,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillRequest {
    pub session_id: SessionId,
}

// ── Signal ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Signal {
    Interrupt,
    Terminate,
    Kill,
}

// ── State ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Display, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum State {
    #[default]
    Running,
    Completed,
    Failed,
}

// ── Snapshot ────────────────────────────────────────────────────────

/// A point-in-time snapshot of terminal state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub session_id: SessionId,
    pub output: String,
    pub styled_lines: Vec<StyledLine>,
    pub cursor: Option<Cursor>,
    pub modes: Modes,
    pub exit_code: Option<i32>,
    pub state: State,
    pub updated_at_unix_ms: Option<u64>,
}

// ── Session record (for persistence) ────────────────────────────────

/// Persisted session metadata (survives restarts).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: SessionId,
    pub workspace_id: WorkspaceId,
    pub cwd: PathBuf,
    pub shell: String,
    pub root_pid: Option<u32>,
    pub cols: u16,
    pub rows: u16,
    pub title: Option<String>,
    pub last_command: Option<String>,
    pub output_tail: Option<String>,
    pub exit_code: Option<i32>,
    pub state: Option<State>,
    pub updated_at_unix_ms: Option<u64>,
}

// ── Utilities ───────────────────────────────────────────────────────

/// Returns the default shell for the current platform.
pub fn default_shell() -> String {
    if let Ok(shell) = std::env::var("SHELL") {
        if !shell.trim().is_empty() {
            return shell;
        }
    }
    if cfg!(windows) {
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_owned())
    } else {
        "/bin/zsh".to_owned()
    }
}
