//! Live terminal session — PTY + output buffer + reader thread.
//!
//! A `LiveSession` owns the PTY process and accumulates its output.
//! The reader thread runs in the background, appending raw bytes to
//! a shared buffer. Callers use `snapshot()` to read the current state
//! and `write()` to send keystrokes.

use std::io::Read;
use std::sync::{Arc, Mutex};

use atrium_core::id::{SessionId, WorkspaceId};
use atrium_error::Result;

use crate::pty::PtyHandle;
use crate::types::{SessionRecord, Snapshot, State};
use crate::styled::Modes;

/// A live terminal session backed by a PTY.
pub struct LiveSession {
    // ── Identity ────────────────────────────────────────────────────
    session_id: SessionId,
    workspace_id: WorkspaceId,
    cwd: std::path::PathBuf,
    shell: String,
    title: String,
    cols: u16,
    rows: u16,

    // ── Runtime ─────────────────────────────────────────────────────
    pty: PtyHandle,
    raw_output: Arc<Mutex<String>>,
    state: Arc<Mutex<State>>,
    exit_code: Arc<Mutex<Option<i32>>>,
    _reader_thread: std::thread::JoinHandle<()>,
}

impl LiveSession {
    /// Spawn a new live session.
    pub fn spawn(
        session_id: SessionId,
        workspace_id: WorkspaceId,
        cwd: std::path::PathBuf,
        shell: &str,
        title: impl Into<String>,
        cols: u16,
        rows: u16,
    ) -> Result<Self> {
        let (pty, reader) = PtyHandle::spawn(shell, &cwd, cols, rows)?;
        let raw_output: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let state: Arc<Mutex<State>> = Arc::new(Mutex::new(State::Running));
        let exit_code: Arc<Mutex<Option<i32>>> = Arc::new(Mutex::new(None));

        let out_ref = Arc::clone(&raw_output);
        let state_ref = Arc::clone(&state);
        let reader_thread = std::thread::spawn(move || {
            reader_loop(reader, out_ref, state_ref);
        });

        Ok(Self {
            session_id,
            workspace_id,
            cwd,
            shell: shell.to_owned(),
            title: title.into(),
            cols,
            rows,
            pty,
            raw_output,
            state,
            exit_code,
            _reader_thread: reader_thread,
        })
    }

    // ── Accessors ───────────────────────────────────────────────────

    pub fn session_id(&self) -> &SessionId { &self.session_id }
    pub fn title(&self) -> &str { &self.title }
    pub fn shell(&self) -> &str { &self.shell }
    pub fn cols(&self) -> u16 { self.cols }
    pub fn rows(&self) -> u16 { self.rows }

    pub fn state(&self) -> State {
        self.state.lock().map(|s| *s).unwrap_or(State::Failed)
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code.lock().ok().and_then(|c| *c)
    }

    /// Whether the PTY has produced any output yet.
    pub fn has_output(&self) -> bool {
        self.raw_output
            .lock()
            .ok()
            .is_some_and(|s| !s.is_empty())
    }

    // ── I/O ─────────────────────────────────────────────────────────

    /// Send bytes to the PTY's stdin (keystrokes).
    pub fn write(&self, data: &[u8]) -> Result<()> {
        self.pty.write(data)
    }

    // ── Snapshots ───────────────────────────────────────────────────

    /// Get the raw output as display lines.
    ///
    /// Splits on newlines, strips ANSI escape codes and carriage returns.
    /// Includes partial lines (e.g. the shell prompt).
    pub fn output_lines(&self) -> Vec<String> {
        let raw = self.raw_output.lock().ok();
        let Some(raw) = raw else { return Vec::new() };
        if raw.is_empty() {
            return Vec::new();
        }
        raw.split('\n').map(|l| strip_ansi_and_cr(l)).collect()
    }

    /// Build a full snapshot of the current state.
    pub fn snapshot(&self) -> Snapshot {
        let lines = self.output_lines();
        let output = lines.join("\n");

        Snapshot {
            session_id: self.session_id.clone(),
            output,
            styled_lines: Vec::new(), // TODO: populate from emulator
            cursor: None,             // TODO: populate from emulator
            modes: Modes::default(),
            exit_code: self.exit_code(),
            state: self.state(),
            updated_at_unix_ms: None,
        }
    }

    /// Build a persistence record.
    pub fn record(&self) -> SessionRecord {
        let output_tail = self.raw_output.lock().ok().map(|s| {
            let max = 8192;
            if s.len() > max { s[s.len() - max..].to_owned() } else { s.clone() }
        });

        SessionRecord {
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
            exit_code: self.exit_code(),
            state: Some(self.state()),
            updated_at_unix_ms: None,
        }
    }
}

// ── Reader thread ───────────────────────────────────────────────────

fn reader_loop(
    mut reader: Box<dyn Read + Send>,
    output: Arc<Mutex<String>>,
    state: Arc<Mutex<State>>,
) {
    let mut buf = [0u8; 4096];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let text = String::from_utf8_lossy(&buf[..n]);
                if let Ok(mut out) = output.lock() {
                    out.push_str(&text);
                }
            }
            Err(_) => break,
        }
    }
    // Process exited
    if let Ok(mut s) = state.lock() {
        *s = State::Completed;
    }
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
