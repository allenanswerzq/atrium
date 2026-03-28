//! Agent chat manager — create, send, cancel, kill sessions.

use std::collections::HashMap;
use std::path::PathBuf;

use tokio::sync::broadcast;

use crate::event::AgentChatEvent;
use crate::preset::AgentKind;
use crate::session::AgentChatSession;
use crate::types::{AgentChatStatus, AgentChatTransport, AgentSessionId, ChatMessage};

/// Manages agent chat sessions.
pub struct AgentChatManager {
    sessions: HashMap<AgentSessionId, AgentChatSession>,
    next_id: u64,
}

impl AgentChatManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            next_id: 0,
        }
    }

    /// Create a new session. Returns (session_id, event_receiver).
    pub fn create(
        &mut self,
        agent_kind: AgentKind,
        workspace_path: PathBuf,
        model_id: Option<String>,
        transport: Option<AgentChatTransport>,
    ) -> (AgentSessionId, broadcast::Receiver<AgentChatEvent>) {
        let id = self.next_id;
        self.next_id += 1;
        let session_id = AgentSessionId::new(format!("agent-chat-{id}"));
        let session_name = format!("atrium-{session_id}");
        let (event_tx, event_rx) = broadcast::channel::<AgentChatEvent>(256);

        let session = AgentChatSession {
            id: session_id.clone(),
            agent_kind,
            workspace_path,
            session_name,
            model_id,
            transport: transport.unwrap_or_default(),
            event_tx,
            messages: Vec::new(),
            pending_text: String::new(),
            pending_tool_calls: Vec::new(),
            status: AgentChatStatus::Idle,
            input_tokens: 0,
            output_tokens: 0,
            turn_start_input_tokens: 0,
            turn_start_output_tokens: 0,
            turn_cancel: None,
        };

        tracing::info!(
            session_id = %session_id,
            transport = %session.transport_label(),
            "created agent chat session"
        );

        self.sessions.insert(session_id.clone(), session);
        (session_id, event_rx)
    }

    /// Send a message to an existing session.
    pub fn send_message(
        &mut self,
        session_id: &AgentSessionId,
        message: String,
    ) -> Result<(), String> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| format!("session not found: {session_id}"))?;

        if session.status == AgentChatStatus::Working {
            return Err("agent is already processing".to_owned());
        }

        session.messages.push(ChatMessage {
            role: "user".to_owned(),
            content: message.clone(),
            tool_calls: Vec::new(),
            input_tokens: 0,
            output_tokens: 0,
            model_id: None,
            transport_label: None,
        });

        let _ = session.event_tx.send(AgentChatEvent::UserMessage {
            content: message,
        });

        session.status = AgentChatStatus::Working;
        let _ = session.event_tx.send(AgentChatEvent::TurnStarted);

        Ok(())
    }

    /// Cancel the running turn.
    pub fn cancel(&mut self, session_id: &AgentSessionId) -> Result<(), String> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| format!("session not found: {session_id}"))?;
        if let Some(cancel_tx) = session.turn_cancel.take() {
            let _ = cancel_tx.send(true);
        }
        Ok(())
    }

    /// Kill a session.
    pub fn kill(&mut self, session_id: &AgentSessionId) -> Result<(), String> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| format!("session not found: {session_id}"))?;
        if let Some(cancel_tx) = session.turn_cancel.take() {
            let _ = cancel_tx.send(true);
        }
        session.status = AgentChatStatus::Exited;
        let _ = session.event_tx.send(AgentChatEvent::SessionExited { exit_code: None });
        Ok(())
    }

    /// Remove a session.
    pub fn remove(&mut self, session_id: &AgentSessionId) {
        if let Some(mut session) = self.sessions.remove(session_id) {
            if let Some(cancel_tx) = session.turn_cancel.take() {
                let _ = cancel_tx.send(true);
            }
        }
    }

    /// List all sessions.
    pub fn list(&self) -> Vec<AgentSessionSummary> {
        self.sessions.values().map(|s| AgentSessionSummary {
            id: s.id.to_string(),
            agent_kind: s.agent_kind.key().to_owned(),
            workspace_path: s.workspace_path.display().to_string(),
            status: s.status,
            input_tokens: s.input_tokens,
            output_tokens: s.output_tokens,
            transport_label: s.transport_label(),
        }).collect()
    }

    /// Get a session by ID.
    pub fn session(&self, id: &AgentSessionId) -> Option<&AgentChatSession> {
        self.sessions.get(id)
    }

    /// Get a mutable session by ID.
    pub fn session_mut(&mut self, id: &AgentSessionId) -> Option<&mut AgentChatSession> {
        self.sessions.get_mut(id)
    }
}

impl Default for AgentChatManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary DTO for listing sessions.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentSessionSummary {
    pub id: String,
    pub agent_kind: String,
    pub workspace_path: String,
    pub status: AgentChatStatus,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub transport_label: String,
}
