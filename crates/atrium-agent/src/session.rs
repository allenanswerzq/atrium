//! Agent chat session and manager.

use std::collections::HashMap;
use std::path::PathBuf;

use tokio::sync::broadcast;

use crate::preset::AgentKind;
use crate::types::{
    AgentChatEvent, AgentChatStatus, AgentChatTransport, AgentSessionId,
    AgentSessionSummary, ChatMessage,
};

// ── Session ─────────────────────────────────────────────────────────

/// A single agent chat session.
pub struct AgentChatSession {
    pub id: AgentSessionId,
    pub agent_kind: AgentKind,
    pub workspace_path: PathBuf,
    pub session_name: String,
    pub model_id: Option<String>,
    pub transport: AgentChatTransport,
    pub event_tx: broadcast::Sender<AgentChatEvent>,
    pub messages: Vec<ChatMessage>,
    pub pending_text: String,
    pub pending_tool_calls: Vec<String>,
    pub status: AgentChatStatus,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub turn_start_input_tokens: u64,
    pub turn_start_output_tokens: u64,
    pub turn_cancel: Option<tokio::sync::watch::Sender<bool>>,
}

impl AgentChatSession {
    /// Human-readable transport label.
    pub fn transport_label(&self) -> String {
        match &self.transport {
            AgentChatTransport::Acp => format!("acp:{}", self.agent_kind.key()),
            AgentChatTransport::OpenAiChat { base_url, .. } => format!("openai:{base_url}"),
        }
    }

    /// Finalize the current turn — move pending text into messages.
    pub fn finalize_turn(&mut self) {
        if self.pending_text.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.pending_text);
        let tools = std::mem::take(&mut self.pending_tool_calls);
        let turn_input = self.input_tokens.saturating_sub(self.turn_start_input_tokens);
        let mut turn_output = self.output_tokens.saturating_sub(self.turn_start_output_tokens);
        if turn_output == 0 && !text.is_empty() {
            turn_output = (text.len() as u64).div_ceil(4);
        }
        self.messages.push(ChatMessage {
            role: "assistant".to_owned(),
            content: text,
            tool_calls: tools,
            input_tokens: turn_input,
            output_tokens: turn_output,
            model_id: self.model_id.clone(),
            transport_label: Some(self.transport_label()),
        });
        self.status = AgentChatStatus::Idle;
    }

    /// Get conversation history including any in-progress text.
    pub fn history(&self) -> Vec<ChatMessage> {
        let mut messages = self.messages.clone();
        if !self.pending_text.is_empty() {
            messages.push(ChatMessage {
                role: "assistant".to_owned(),
                content: self.pending_text.clone(),
                tool_calls: self.pending_tool_calls.clone(),
                input_tokens: 0,
                output_tokens: 0,
                model_id: self.model_id.clone(),
                transport_label: Some(self.transport_label()),
            });
        }
        messages
    }
}

// ── Manager ─────────────────────────────────────────────────────────

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

        tracing::info!(session_id = %session_id, transport = %session.transport_label(), "created agent chat session");
        self.sessions.insert(session_id.clone(), session);
        (session_id, event_rx)
    }

    pub fn send_message(&mut self, session_id: &AgentSessionId, message: String) -> Result<(), String> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| format!("session not found: {session_id}"))?;
        if session.status == AgentChatStatus::Working {
            return Err("agent is already processing".to_owned());
        }
        session.messages.push(ChatMessage {
            role: "user".to_owned(), content: message.clone(),
            tool_calls: Vec::new(), input_tokens: 0, output_tokens: 0,
            model_id: None, transport_label: None,
        });
        let _ = session.event_tx.send(AgentChatEvent::UserMessage { content: message });
        session.status = AgentChatStatus::Working;
        let _ = session.event_tx.send(AgentChatEvent::TurnStarted);
        Ok(())
    }

    pub fn cancel(&mut self, session_id: &AgentSessionId) -> Result<(), String> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| format!("session not found: {session_id}"))?;
        if let Some(cancel_tx) = session.turn_cancel.take() {
            let _ = cancel_tx.send(true);
        }
        Ok(())
    }

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

    pub fn remove(&mut self, session_id: &AgentSessionId) {
        if let Some(mut session) = self.sessions.remove(session_id) {
            if let Some(cancel_tx) = session.turn_cancel.take() {
                let _ = cancel_tx.send(true);
            }
        }
    }

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

    pub fn session(&self, id: &AgentSessionId) -> Option<&AgentChatSession> {
        self.sessions.get(id)
    }

    pub fn session_mut(&mut self, id: &AgentSessionId) -> Option<&mut AgentChatSession> {
        self.sessions.get_mut(id)
    }
}

impl Default for AgentChatManager {
    fn default() -> Self {
        Self::new()
    }
}
