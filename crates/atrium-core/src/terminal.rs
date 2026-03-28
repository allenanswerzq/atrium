//! Terminal daemon protocol types and traits.
//!
//! Defines the request/response types for terminal session management
//! and the `TerminalDaemon` trait that backends implement.

use crate::id::{TerminalSessionId, WorkspaceId};
use serde::{Deserialize, Serialize};
use strum::{Display, IntoStaticStr};
use std::path::PathBuf;

// ── Requests ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrAttachRequest {
    pub session_id: TerminalSessionId,
    pub workspace_id: WorkspaceId,
    pub cwd: PathBuf,
    pub shell: String,
    pub cols: u16,
    pub rows: u16,
    pub title: String,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteRequest {
    pub session_id: TerminalSessionId,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeRequest {
    pub session_id: TerminalSessionId,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetachRequest {
    pub session_id: TerminalSessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillRequest {
    pub session_id: TerminalSessionId,
}

// ── Signals ─────────────────────────────────────────────────────────

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
pub enum TerminalSessionState {
    #[default]
    Running,
    Completed,
    Failed,
}

// ── Session record ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSessionRecord {
    pub session_id: TerminalSessionId,
    pub workspace_id: WorkspaceId,
    pub cwd: PathBuf,
    pub shell: String,
    pub root_pid: Option<u32>,
    pub cols: u16,
    pub rows: u16,
    pub title: String,
    pub last_command: Option<String>,
    pub output_tail: Option<String>,
    pub exit_code: Option<i32>,
    pub state: Option<TerminalSessionState>,
    pub updated_at_unix_ms: Option<u64>,
}

// ── Snapshot ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSnapshot {
    pub session_id: TerminalSessionId,
    pub output_tail: String,
    pub exit_code: Option<i32>,
    pub state: TerminalSessionState,
    pub updated_at_unix_ms: Option<u64>,
}

// ── Traits ──────────────────────────────────────────────────────────

/// The core terminal daemon contract.
///
/// Implemented by `LocalTerminalDaemon` in the httpd crate.
pub trait TerminalDaemon {
    fn create_or_attach(
        &mut self,
        request: CreateOrAttachRequest,
    ) -> atrium_error::Result<DaemonSessionRecord>;

    fn write(&mut self, request: WriteRequest) -> atrium_error::Result<()>;
    fn resize(&mut self, request: ResizeRequest) -> atrium_error::Result<()>;
    fn detach(&mut self, request: DetachRequest) -> atrium_error::Result<()>;
    fn kill(&mut self, request: KillRequest) -> atrium_error::Result<()>;
    fn snapshot(&self, session_id: &TerminalSessionId, max_lines: usize) -> atrium_error::Result<TerminalSnapshot>;
    fn list_sessions(&self) -> Vec<DaemonSessionRecord>;
}

/// Persistent storage for session records.
pub trait DaemonSessionStore: Send + Sync {
    fn load(&self) -> atrium_error::Result<Vec<DaemonSessionRecord>>;
    fn save(&self, records: &[DaemonSessionRecord]) -> atrium_error::Result<()>;
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
