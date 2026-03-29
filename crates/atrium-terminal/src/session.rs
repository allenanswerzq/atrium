//! Terminal session, runtime, service, store, and keystroke mapping.

use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;

use atrium_core::id::{TerminalSessionId, WorkspaceId};
use atrium_error::{Error, ErrorKind, Result};
use atrium_executor::TaskExecutor;
use parking_lot::Mutex;

use crate::emulator::TerminalEmulator;
use crate::pty::TerminalPty;
use crate::types::*;

// ── Runtime ─────────────────────────────────────────────────────────

/// The live PTY backend — owns the process, emulator, and reader task.
///
// PTY stdout bytes
//      │
//      ▼
// reader_loop() on TaskExecutor::spawn_blocking
//      │
//      ▼
// TerminalEmulator.process(bytes)   ← vt100 parses ANSI
//      │
//      ├─→ styled_lines()  → colored cells with fg/bg
//      ├─→ cursor()        → row, column position
//      ├─→ modes()         → app_cursor, alt_screen
//      └─→ plain_output()  → clean text for persistence
pub struct TerminalRuntime {
    pty: TerminalPty,
    /// The emulator parses ANSI into styled cells. Shared with reader task.
    emulator: Arc<Mutex<TerminalEmulator>>,
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
        let emulator = Arc::new(Mutex::new(TerminalEmulator::new(rows, cols)));
        let state: Arc<Mutex<TerminalState>> = Arc::new(Mutex::new(TerminalState::Running));

        let emu_ref = Arc::clone(&emulator);
        let state_ref = Arc::clone(&state);

        executor.spawn_blocking(async move {
            reader_loop(reader, emu_ref, state_ref);
        });

        Ok(Self {
            pty,
            emulator,
            state,
        })
    }

    /// Spawn without an executor — uses a plain OS thread for the reader.
    /// Useful when no tokio runtime is available (e.g. GUI-only mode).
    pub fn spawn_standalone(
        shell: &str,
        cwd: &std::path::Path,
        cols: u16,
        rows: u16,
    ) -> Result<Self> {
        let (pty, reader) = TerminalPty::spawn(shell, cwd, cols, rows)?;
        let emulator = Arc::new(Mutex::new(TerminalEmulator::new(rows, cols)));
        let state: Arc<Mutex<TerminalState>> = Arc::new(Mutex::new(TerminalState::Running));

        let emu_ref = Arc::clone(&emulator);
        let state_ref = Arc::clone(&state);

        std::thread::Builder::new()
            .name("terminal-reader".to_owned())
            .spawn(move || {
                reader_loop(reader, emu_ref, state_ref);
            })
            .map_err(|e| Error::from(e))?;

        Ok(Self {
            pty,
            emulator,
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

    /// Current cursor position.
    pub fn cursor(&self) -> TerminalCursor {
        self.emulator.lock().cursor()
    }

    /// Current terminal modes.
    pub fn modes(&self) -> TerminalModes {
        self.emulator.lock().modes()
    }

    /// Get the visible grid as styled lines.
    pub fn styled_lines(&self) -> Vec<TerminalStyledLine> {
        self.emulator.lock().styled_lines()
    }

    /// Get plain text output from the emulator.
    pub fn plain_output(&self) -> String {
        self.emulator.lock().plain_output()
    }

    /// Whether the emulator has any content.
    pub fn has_output(&self) -> bool {
        // Check if any row has non-empty content
        let emu = self.emulator.lock();
        let lines = emu.styled_lines();
        lines
            .iter()
            .any(|l| l.runs.iter().any(|r| r.text.trim() != ""))
    }

    /// Get output as simple display lines (for compatibility).
    pub fn output_lines(&self) -> Vec<String> {
        let emu = self.emulator.lock();
        let plain = emu.plain_output();
        plain.split('\n').map(|s| s.to_owned()).collect()
    }
}

// ── Session ─────────────────────────────────────────────────────────

/// A terminal session — identity metadata + optional live runtime.
pub struct TerminalSession {
    session_id: TerminalSessionId,
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
    /// Spawn a new session with a live PTY (managed by executor).
    pub fn spawn(
        executor: &TaskExecutor,
        session_id: TerminalSessionId,
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

    /// Spawn a new session with a standalone reader thread (no executor needed).
    pub fn spawn_standalone(
        session_id: TerminalSessionId,
        workspace_id: WorkspaceId,
        cwd: std::path::PathBuf,
        shell: &str,
        title: impl Into<String>,
        cols: u16,
        rows: u16,
    ) -> Result<Self> {
        let runtime = TerminalRuntime::spawn_standalone(shell, &cwd, cols, rows)?;
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

    pub fn session_id(&self) -> &TerminalSessionId {
        &self.session_id
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn shell(&self) -> &str {
        &self.shell
    }
    pub fn cols(&self) -> u16 {
        self.cols
    }
    pub fn rows(&self) -> u16 {
        self.rows
    }
    pub fn runtime(&self) -> Option<&TerminalRuntime> {
        self.runtime.as_ref()
    }

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
            .ok_or_else(|| Error::new(ErrorKind::Unsupported, "no runtime attached"))?
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
        if let Some(rt) = &self.runtime {
            let emu = rt.emulator.lock();
            TerminalSnapshot {
                session_id: self.session_id.clone(),
                output: emu.plain_output(),
                styled_lines: emu.styled_lines(),
                cursor: Some(emu.cursor()),
                modes: emu.modes(),
                exit_code: None,
                state: rt.state(),
                updated_at_unix_ms: None,
            }
        } else {
            TerminalSnapshot {
                session_id: self.session_id.clone(),
                output: String::new(),
                styled_lines: Vec::new(),
                cursor: None,
                modes: TerminalModes::default(),
                exit_code: None,
                state: TerminalState::Completed,
                updated_at_unix_ms: None,
            }
        }
    }

    pub fn record(&self) -> TerminalSessionRecord {
        let output_tail = self.runtime.as_ref().map(|rt| {
            let plain = rt.emulator.lock().plain_output();
            let max = 8192;
            if plain.len() > max {
                plain[plain.len() - max..].to_owned()
            } else {
                plain
            }
        });
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

// ── Reader task ─────────────────────────────────────────────────────

fn reader_loop(
    mut reader: Box<dyn Read + Send>,
    emulator: Arc<Mutex<TerminalEmulator>>,
    state: Arc<Mutex<TerminalState>>,
) {
    let mut buf = [0u8; 4096];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                emulator.lock().process(&buf[..n]);
            }
            Err(_) => break,
        }
    }
    *state.lock() = TerminalState::Completed;
}

// ── Service ─────────────────────────────────────────────────────────

/// The terminal service contract.
pub trait TerminalService {
    fn create(&mut self, request: TerminalCreateRequest) -> Result<TerminalSessionRecord>;
    fn write(&self, request: TerminalWriteRequest) -> Result<()>;
    fn kill(&mut self, request: TerminalKillRequest) -> Result<()>;
    fn snapshot(&self, session_id: &TerminalSessionId) -> Result<TerminalSnapshot>;
    fn list_sessions(&self) -> Vec<TerminalSessionRecord>;
}

/// Local terminal service — manages PTY sessions in-process.
pub struct LocalTerminalService {
    executor: TaskExecutor,
    sessions: HashMap<TerminalSessionId, TerminalSession>,
}

impl LocalTerminalService {
    pub fn new(executor: TaskExecutor) -> Self {
        Self {
            executor,
            sessions: HashMap::new(),
        }
    }

    pub fn session(&self, id: &TerminalSessionId) -> Option<&TerminalSession> {
        self.sessions.get(id)
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl TerminalService for LocalTerminalService {
    fn create(&mut self, req: TerminalCreateRequest) -> Result<TerminalSessionRecord> {
        let session = TerminalSession::spawn(
            &self.executor,
            req.session_id.clone(),
            req.workspace_id,
            req.cwd,
            &req.shell,
            req.title.as_deref().unwrap_or("Terminal"),
            req.cols,
            req.rows,
        )?;
        let record = session.record();
        self.sessions.insert(req.session_id, session);
        Ok(record)
    }

    fn write(&self, req: TerminalWriteRequest) -> Result<()> {
        self.sessions
            .get(&req.session_id)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "session not found"))?
            .write(&req.bytes)
    }

    fn kill(&mut self, req: TerminalKillRequest) -> Result<()> {
        self.sessions
            .remove(&req.session_id)
            .map(|_| ())
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "session not found"))
    }

    fn snapshot(&self, session_id: &TerminalSessionId) -> Result<TerminalSnapshot> {
        Ok(self
            .sessions
            .get(session_id)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "session not found"))?
            .snapshot())
    }

    fn list_sessions(&self) -> Vec<TerminalSessionRecord> {
        self.sessions.values().map(|s| s.record()).collect()
    }
}

// ── Store ───────────────────────────────────────────────────────────

/// Trait for persisting terminal session records.
pub trait TerminalSessionStore: Send + Sync {
    fn load(&self) -> Result<Vec<TerminalSessionRecord>>;
    fn save(&self, records: &[TerminalSessionRecord]) -> Result<()>;
}

/// JSON file-backed session store.
pub struct JsonTerminalSessionStore {
    path: std::path::PathBuf,
}

impl JsonTerminalSessionStore {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl TerminalSessionStore for JsonTerminalSessionStore {
    fn load(&self) -> Result<Vec<TerminalSessionRecord>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        atrium_fs::read_json(&self.path)
    }

    fn save(&self, records: &[TerminalSessionRecord]) -> Result<()> {
        atrium_fs::write_json(&self.path, &records)
    }
}

// ── Keys ────────────────────────────────────────────────────────────

/// Convert a key name + modifiers to terminal escape bytes.
pub fn terminal_escape_bytes(
    key: &str,
    ctrl: bool,
    alt: bool,
    _modes: TerminalModes,
) -> Option<Vec<u8>> {
    if ctrl {
        return ctrl_byte(key);
    }
    let seq: Option<&[u8]> = match key {
        "enter" | "return" => Some(b"\r"),
        "tab" => Some(b"\t"),
        "escape" => Some(b"\x1b"),
        "backspace" => Some(b"\x7f"),
        "delete" => Some(b"\x1b[3~"),
        "up" => Some(b"\x1b[A"),
        "down" => Some(b"\x1b[B"),
        "right" => Some(b"\x1b[C"),
        "left" => Some(b"\x1b[D"),
        "home" => Some(b"\x1b[H"),
        "end" => Some(b"\x1b[F"),
        "pageup" => Some(b"\x1b[5~"),
        "pagedown" => Some(b"\x1b[6~"),
        "insert" => Some(b"\x1b[2~"),
        "f1" => Some(b"\x1bOP"),
        "f2" => Some(b"\x1bOQ"),
        "f3" => Some(b"\x1bOR"),
        "f4" => Some(b"\x1bOS"),
        "f5" => Some(b"\x1b[15~"),
        "f6" => Some(b"\x1b[17~"),
        "f7" => Some(b"\x1b[18~"),
        "f8" => Some(b"\x1b[19~"),
        "f9" => Some(b"\x1b[20~"),
        "f10" => Some(b"\x1b[21~"),
        "f11" => Some(b"\x1b[23~"),
        "f12" => Some(b"\x1b[24~"),
        _ => None,
    };
    if let Some(bytes) = seq {
        return Some(bytes.to_vec());
    }
    if alt && key.len() == 1 {
        let mut bytes = vec![0x1b];
        bytes.extend_from_slice(key.as_bytes());
        return Some(bytes);
    }
    None
}

fn ctrl_byte(key: &str) -> Option<Vec<u8>> {
    let ch = key.chars().next()?;
    match ch {
        'a'..='z' => Some(vec![ch as u8 - b'a' + 1]),
        'A'..='Z' => Some(vec![ch as u8 - b'A' + 1]),
        '@' => Some(vec![0x00]),
        '[' => Some(vec![0x1b]),
        '\\' => Some(vec![0x1c]),
        ']' => Some(vec![0x1d]),
        '^' => Some(vec![0x1e]),
        '_' => Some(vec![0x1f]),
        '?' => Some(vec![0x7f]),
        _ => None,
    }
}
