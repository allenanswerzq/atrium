//! Terminal types — styled output, protocol types, requests, snapshots.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use strum::{Display, IntoStaticStr};

use atrium_core::id::{TerminalSessionId, WorkspaceId};

// ── Styled output types ─────────────────────────────────────────────

pub const DEFAULT_FG: u32 = 0xabb2bf;
pub const DEFAULT_BG: u32 = 0x282c34;
pub const DEFAULT_CURSOR: u32 = 0x74ade8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalStyledCell {
    pub column: usize,
    pub text: String,
    pub fg: u32,
    pub bg: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalStyledRun {
    pub text: String,
    pub fg: u32,
    pub bg: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalStyledLine {
    pub cells: Vec<TerminalStyledCell>,
    pub runs: Vec<TerminalStyledRun>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TerminalCursor {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TerminalModes {
    pub app_cursor: bool,
    pub alt_screen: bool,
}

// ── Requests ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCreateRequest {
    pub session_id: TerminalSessionId,
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
    pub session_id: TerminalSessionId,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalResizeRequest {
    pub session_id: TerminalSessionId,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalKillRequest {
    pub session_id: TerminalSessionId,
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
    pub session_id: TerminalSessionId,
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
    pub session_id: TerminalSessionId,
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
