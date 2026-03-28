//! Terminal session and runtime.
//!
//! `TerminalRuntime` owns the live PTY, reader task, and shared output buffer.
//! `TerminalSession` holds identity metadata + an optional runtime.
//! A session without a runtime is "detached" (restored from disk, no live PTY).
//!
//! The reader runs as a blocking task on the centralized `TaskExecutor`,
//! so it participates in graceful shutdown and metric tracking.

use std::io::Read;
use std::sync::Arc;

use atrium_core::id::{SessionId, WorkspaceId};
use atrium_error::Result;
use atrium_executor::TaskExecutor;
use parking_lot::Mutex;

use crate::pty::TerminalPty;
use crate::types::{TerminalSessionRecord, TerminalSnapshot, TerminalState};
use crate::styled::TerminalModes;

// ── Runtime ─────────────────────────────────────────────────────────

/// The live PTY backend — owns the process, reader task, and output buffer.
///
/// Separated from `TerminalSession` so that a session can exist without
/// a live PTY (e.g. restored from disk after restart).
///
/// The reader runs on `TaskExecutor::spawn_blocking` so it is tracked
/// by the central task manager and cancelled on shutdown.
pub struct TerminalRuntime {
    pty: TerminalPty,
    /// Raw PTY output bytes — shared with the reader task.
    raw_output: Arc<Mutex<String>>,
    /// Lifecycle state — set to `Completed` by the reader task on EOF.
    state: Arc<Mutex<TerminalState>>,
}

impl TerminalRuntime {
    /// Spawn a new PTY and start the reader as a managed blocking task.
    pub fn spawn(
        executor: &TaskExecutor,
        shell: &str,
        cwd: &std::path::Path,
        cols: u16,
        rows: u16,
    ) -> Result<Self> {
        let (pty, reader) = TerminalPty::spawn(shell, cwd, cols, rows)?;
        let raw_output: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let state: Arc<Mutex<TerminalState>> = Arc::new(Mutex::new(TerminalState::Running));

        let out_ref = Arc::clone(&raw_output);
        let state_ref = Arc::clone(&state);

        // Run the blocking PTY reader on the executor's thread pool.
        // This is automatically cancelled on shutdown and tracked in metrics.
        executor.spawn_blocking(async move {
            reader_loop(reader, out_ref, state_ref);
        });

        Ok(Self {
            pty,
            raw_output,
            state,
        })
    }

    /// Send bytes to the PTY's stdin.
    pub fn write(&self, data: &[u8]) -> Result<()> {
        self.pty.write(data)
    }

    /// Current lifecycle state.
    pub fn state(&self) -> TerminalState {
        *self.state.lock()
    }

    /// Whether the PTY has produced any output.
    pub fn has_output(&self) -> bool {
        !self.raw_output.lock().is_empty()
    }

    /// Get current output as display lines (ANSI stripped).
    pub fn output_lines(&self) -> Vec<String> {
        let raw = self.raw_output.lock();
        if raw.is_empty() {
            return Vec::new();
        }
        raw.split('\n').map(|l| strip_ansi_and_cr(l)).collect()
    }

    /// Get a tail of the raw output for persistence.
    pub fn output_tail(&self, max_bytes: usize) -> Option<String> {
        let raw = self.raw_output.lock();
        if raw.is_empty() {
            None
        } else if raw.len() > max_bytes {
            Some(raw[raw.len() - max_bytes..].to_owned())
        } else {
            Some(raw.clone())
        }
    }
}

// ── Session ─────────────────────────────────────────────────────────

/// A terminal session — identity metadata + optional live runtime.
pub struct TerminalSession {
    session_id: SessionId,
    workspace_id: WorkspaceId,
    cwd: std::path::PathBuf,
    shell: String,
    title: String,
    cols: u16,
    rows: u16,
    /// Live PTY runtime. `None` if session is detached/restored.
    runtime: Option<TerminalRuntime>,
}

impl TerminalSession {
    /// Spawn a new session with a live PTY.
    pub fn spawn(
        executor: &TaskExecutor,
        session_id: SessionId,
        workspace_id: WorkspaceId,
        cwd: std::path::PathBuf,
        shell: &str,
        title: impl Into<String>,
        cols: u16,
        rows: u16,
    ) -> Result<Self> {
        let runtime = TerminalRuntime::spawn(executor, shell, &cwd, cols, rows)?;
        Ok(Self {
            session_id,
            workspace_id,
            cwd,
            shell: shell.to_owned(),
            title: title.into(),
            cols,
            rows,
            runtime: Some(runtime),
        })
    }

    // ── Accessors ───────────────────────────────────────────────────

    pub fn session_id(&self) -> &SessionId { &self.session_id }
    pub fn title(&self) -> &str { &self.title }
    pub fn shell(&self) -> &str { &self.shell }
    pub fn cols(&self) -> u16 { self.cols }
    pub fn rows(&self) -> u16 { self.rows }
    pub fn runtime(&self) -> Option<&TerminalRuntime> { self.runtime.as_ref() }

    pub fn state(&self) -> TerminalState {
        self.runtime
            .as_ref()
            .map(|r| r.state())
            .unwrap_or(TerminalState::Completed)
    }

    pub fn has_output(&self) -> bool {
        self.runtime.as_ref().is_some_and(|r| r.has_output())
    }

    // ── Delegates to runtime ────────────────────────────────────────

    pub fn write(&self, data: &[u8]) -> Result<()> {
        self.runtime
            .as_ref()
            .ok_or_else(|| atrium_error::Error::new(
                atrium_error::ErrorKind::Unsupported,
                "no runtime attached",
            ))?
            .write(data)
    }

    pub fn output_lines(&self) -> Vec<String> {
        self.runtime
            .as_ref()
            .map(|r| r.output_lines())
            .unwrap_or_default()
    }

    // ── Snapshots ───────────────────────────────────────────────────

    pub fn snapshot(&self) -> TerminalSnapshot {
        let lines = self.output_lines();
        let output = lines.join("\n");
        TerminalSnapshot {
            session_id: self.session_id.clone(),
            output,
            styled_lines: Vec::new(),
            cursor: None,
            modes: TerminalModes::default(),
            exit_code: None,
            state: self.state(),
            updated_at_unix_ms: None,
        }
    }

    pub fn record(&self) -> TerminalSessionRecord {
        let output_tail = self.runtime.as_ref().and_then(|r| r.output_tail(8192));
        TerminalSessionRecord {
            session_id: self.session_id.clone(),
            workspace_id: self.workspace_id.clone(),
            cwd: self.cwd.clone(),
            shell: self.shell.clone(),
            root_pid: None,
            cols: self.cols,
            rows: self.rows,
            title: Some(self.title.clone()),
            last_command: None,
            output_tail,
            exit_code: None,
            state: Some(self.state()),
            updated_at_unix_ms: None,
        }
    }
}

// ── Reader thread ───────────────────────────────────────────────────

fn reader_loop(
    mut reader: Box<dyn Read + Send>,
    output: Arc<Mutex<String>>,
    state: Arc<Mutex<TerminalState>>,
) {
    let mut buf = [0u8; 4096];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let text = String::from_utf8_lossy(&buf[..n]);
                output.lock().push_str(&text);
            }
            Err(_) => break,
        }
    }
    *state.lock() = TerminalState::Completed;
}

// ── ANSI stripping ──────────────────────────────────────────────────

/// Strip carriage returns and ANSI escape sequences for plain display.
fn strip_ansi_and_cr(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {}
            '\x1b' => {
                if chars.peek() == Some(&'[') {
                    chars.next();
                    for c in chars.by_ref() {
                        if c.is_ascii_alphabetic() || c == '~' { break; }
                    }
                } else if chars.peek() == Some(&']') {
                    chars.next();
                    for c in chars.by_ref() {
                        if c == '\x07' { break; }
                        if c == '\x1b' { chars.next(); break; }
                    }
                } else {
                    chars.next();
                }
            }
            _ => result.push(ch),
        }
    }
    result
}
