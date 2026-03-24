//! Terminal session state and lifecycle.

use atrium_core::id::SessionId;
use atrium_core::terminal::TerminalSessionState;

/// A single terminal session.
#[derive(Debug)]
pub struct TerminalSession {
    /// Unique session identifier.
    id: SessionId,
    /// Human-readable title for the tab.
    title: String,
    /// Current session state.
    state: TerminalSessionState,
    /// Accumulated terminal output (styled lines will go here).
    output_lines: Vec<String>,
    /// Current cursor column.
    cursor_col: u16,
    /// Current cursor row.
    cursor_row: u16,
}

impl TerminalSession {
    /// Create a new terminal session.
    pub fn new(id: SessionId, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            state: TerminalSessionState::Running,
            output_lines: Vec::new(),
            cursor_col: 0,
            cursor_row: 0,
        }
    }

    pub fn id(&self) -> &SessionId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    pub fn state(&self) -> TerminalSessionState {
        self.state
    }

    pub fn set_state(&mut self, state: TerminalSessionState) {
        self.state = state;
    }

    pub fn output_lines(&self) -> &[String] {
        &self.output_lines
    }

    pub fn cursor(&self) -> (u16, u16) {
        (self.cursor_col, self.cursor_row)
    }
}
