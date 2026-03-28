//! Protocol types shared between frontend and backend.
//!
//! These are the wire types for terminal operations: requests,
//! responses, signals, lifecycle states, and snapshots.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use strum::{Display, IntoStaticStr};

use atrium_core::id::{SessionId, WorkspaceId};

use crate::styled::{TerminalCursor, TerminalModes, TerminalStyledLine};

// ── Requests ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCreateRequest {
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
pub struct TerminalWriteRequest {
    pub session_id: SessionId,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalResizeRequest {
    pub session_id: SessionId,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalKillRequest {
    pub session_id: SessionId,
}

// ── Signal ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TerminalSignal {
    Interrupt,
    Terminate,
    Kill,
}

// ── State ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Display, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TerminalState {
    #[default]
    Running,
    Completed,
    Failed,
}

// ── Snapshot ────────────────────────────────────────────────────────

/// A point-in-time snapshot of terminal state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSnapshot {
    pub session_id: SessionId,
    pub output: String,
    pub styled_lines: Vec<TerminalStyledLine>,
    pub cursor: Option<TerminalCursor>,
    pub modes: TerminalModes,
    pub exit_code: Option<i32>,
    pub state: TerminalState,
    pub updated_at_unix_ms: Option<u64>,
}

// ── Session record (for persistence) ────────────────────────────────

/// Persisted session metadata (survives restarts).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSessionRecord {
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
    pub state: Option<TerminalState>,
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
