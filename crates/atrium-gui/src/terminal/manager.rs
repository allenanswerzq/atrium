//! Multi-session terminal manager.

use atrium_core::id::SessionId;

use super::session::TerminalSession;

/// Manages multiple terminal sessions and tracks the active one.
#[derive(Debug, Default)]
pub struct TerminalManager {
    sessions: Vec<TerminalSession>,
    active_id: Option<SessionId>,
}

impl TerminalManager {
    /// All sessions.
    pub fn sessions(&self) -> &[TerminalSession] {
        &self.sessions
    }

    /// The currently active session, if any.
    pub fn active(&self) -> Option<&TerminalSession> {
        let id = self.active_id.as_ref()?;
        self.sessions.iter().find(|s| s.id() == id)
    }

    /// The currently active session (mutable), if any.
    pub fn active_mut(&mut self) -> Option<&mut TerminalSession> {
        let id = self.active_id.as_ref()?;
        self.sessions.iter_mut().find(|s| s.id() == id)
    }

    /// Add a session and make it active.
    pub fn add(&mut self, session: TerminalSession) {
        let id = session.id().clone();
        self.sessions.push(session);
        self.active_id = Some(id);
    }

    /// Switch to a session by ID.
    pub fn activate(&mut self, id: &SessionId) {
        if self.sessions.iter().any(|s| s.id() == id) {
            self.active_id = Some(id.clone());
        }
    }

    /// Remove a session by ID. Returns the removed session if found.
    pub fn remove(&mut self, id: &SessionId) -> Option<TerminalSession> {
        let pos = self.sessions.iter().position(|s| s.id() == id)?;
        let removed = self.sessions.remove(pos);

        // If we removed the active session, activate an adjacent one.
        if self.active_id.as_ref() == Some(id) {
            self.active_id = if self.sessions.is_empty() {
                None
            } else {
                let new_idx = pos.min(self.sessions.len() - 1);
                Some(self.sessions[new_idx].id().clone())
            };
        }

        Some(removed)
    }

    /// Number of sessions.
    pub fn count(&self) -> usize {
        self.sessions.len()
    }
}
