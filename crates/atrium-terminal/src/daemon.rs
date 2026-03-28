//! Terminal service — manages multiple live sessions.
//!
//! `TerminalService` is the trait that frontends code against.
//! `LocalTerminalService` is the concrete impl that spawns local PTY sessions.

use std::collections::HashMap;

use atrium_core::id::SessionId;
use atrium_error::{Error, ErrorKind, Result};
use atrium_executor::TaskExecutor;

use crate::session::TerminalSession;
use crate::types::{
    TerminalCreateRequest, TerminalKillRequest, TerminalSessionRecord,
    TerminalSnapshot, TerminalWriteRequest,
};

/// The terminal service contract.
///
/// Implemented by `LocalTerminalService` (in-process PTY) and potentially
/// by an HTTP client for remote daemons.
pub trait TerminalService {
    fn create(&mut self, request: TerminalCreateRequest) -> Result<TerminalSessionRecord>;
    fn write(&self, request: TerminalWriteRequest) -> Result<()>;
    fn kill(&mut self, request: TerminalKillRequest) -> Result<()>;
    fn snapshot(&self, session_id: &SessionId) -> Result<TerminalSnapshot>;
    fn list_sessions(&self) -> Vec<TerminalSessionRecord>;
}

/// Local terminal service that manages PTY sessions in-process.
///
/// Holds a `TaskExecutor` so new terminal reader tasks are managed centrally.
pub struct LocalTerminalService {
    executor: TaskExecutor,
    sessions: HashMap<SessionId, TerminalSession>,
}

impl LocalTerminalService {
    pub fn new(executor: TaskExecutor) -> Self {
        Self {
            executor,
            sessions: HashMap::new(),
        }
    }

    /// Get a reference to a live session.
    pub fn session(&self, id: &SessionId) -> Option<&TerminalSession> {
        self.sessions.get(id)
    }

    /// Number of active sessions.
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
        let session = self.sessions.get(&req.session_id).ok_or_else(|| {
            Error::new(ErrorKind::NotFound, "session not found")
                .with_context("session_id", req.session_id.to_string())
        })?;
        session.write(&req.bytes)
    }

    fn kill(&mut self, req: TerminalKillRequest) -> Result<()> {
        if self.sessions.remove(&req.session_id).is_some() {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::NotFound, "session not found")
                .with_context("session_id", req.session_id.to_string()))
        }
    }

    fn snapshot(&self, session_id: &SessionId) -> Result<TerminalSnapshot> {
        let session = self.sessions.get(session_id).ok_or_else(|| {
            Error::new(ErrorKind::NotFound, "session not found")
                .with_context("session_id", session_id.to_string())
        })?;
        Ok(session.snapshot())
    }

    fn list_sessions(&self) -> Vec<TerminalSessionRecord> {
        self.sessions.values().map(|s| s.record()).collect()
    }
}
