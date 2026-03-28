//! Terminal daemon — manages multiple live sessions.
//!
//! `TerminalDaemon` is the trait that frontends code against.
//! `LocalDaemon` is the concrete impl that spawns local PTY sessions.

use std::collections::HashMap;

use atrium_core::id::SessionId;
use atrium_error::{Error, ErrorKind, Result};

use crate::session::LiveSession;
use crate::types::{
    CreateRequest, KillRequest, SessionRecord, Snapshot, WriteRequest,
};

/// The terminal daemon contract.
///
/// Implemented by `LocalDaemon` (in-process PTY) and potentially
/// by an HTTP client for remote daemons.
pub trait TerminalDaemon {
    fn create(&mut self, request: CreateRequest) -> Result<SessionRecord>;
    fn write(&self, request: WriteRequest) -> Result<()>;
    fn kill(&mut self, request: KillRequest) -> Result<()>;
    fn snapshot(&self, session_id: &SessionId) -> Result<Snapshot>;
    fn list_sessions(&self) -> Vec<SessionRecord>;
}

/// Local daemon that manages PTY sessions in-process.
pub struct LocalDaemon {
    sessions: HashMap<SessionId, LiveSession>,
}

impl LocalDaemon {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Get a reference to a live session.
    pub fn session(&self, id: &SessionId) -> Option<&LiveSession> {
        self.sessions.get(id)
    }

    /// Number of active sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for LocalDaemon {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalDaemon for LocalDaemon {
    fn create(&mut self, req: CreateRequest) -> Result<SessionRecord> {
        let session = LiveSession::spawn(
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

    fn write(&self, req: WriteRequest) -> Result<()> {
        let session = self.sessions.get(&req.session_id).ok_or_else(|| {
            Error::new(ErrorKind::NotFound, "session not found")
                .with_context("session_id", req.session_id.to_string())
        })?;
        session.write(&req.bytes)
    }

    fn kill(&mut self, req: KillRequest) -> Result<()> {
        if self.sessions.remove(&req.session_id).is_some() {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::NotFound, "session not found")
                .with_context("session_id", req.session_id.to_string()))
        }
    }

    fn snapshot(&self, session_id: &SessionId) -> Result<Snapshot> {
        let session = self.sessions.get(session_id).ok_or_else(|| {
            Error::new(ErrorKind::NotFound, "session not found")
                .with_context("session_id", session_id.to_string())
        })?;
        Ok(session.snapshot())
    }

    fn list_sessions(&self) -> Vec<SessionRecord> {
        self.sessions.values().map(|s| s.record()).collect()
    }
}
