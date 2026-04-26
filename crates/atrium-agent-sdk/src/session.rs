//! Agent chat session and manager.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use atrium_error::{Error, ErrorKind, Result};
use atrium_executor::TaskExecutor;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::kind::AgentKind;
use crate::transport::{
    self, EventReceiver, EventSender, PromptRequest, Transport, TransportConfig,
};
use crate::types::{
    AgentChatEvent, AgentChatStatus, AgentSessionId, AgentSessionSummary, ChatMessage,
};

// ── Session ─────────────────────────────────────────────────────────

/// A single agent chat session.
///
/// Holds a [`Transport`] that is created once and reused for every turn.
pub struct AgentChatSession {
    pub id: AgentSessionId,
    pub agent_kind: AgentKind,
    pub workspace_path: PathBuf,
    pub model_id: Option<String>,
    pub event_tx: EventSender,
    // TODO(zhang): disk based message history
    pub messages: Vec<ChatMessage>,
    pub pending_text: String,
    pub pending_tool_calls: Vec<String>,
    pub status: AgentChatStatus,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub turn_start_input_tokens: u64,
    pub turn_start_output_tokens: u64,
    pub turn_cancel: Option<CancellationToken>,
    /// The transport used for this session — created once, reused across turns.
    transport: Arc<dyn Transport>,
}

impl AgentChatSession {
    /// Human-readable transport label.
    pub fn transport_label(&self) -> String {
        self.transport.label()
    }

    /// Finalize the current turn — move pending text into messages.
    pub fn finalize_turn(&mut self) {
        if self.pending_text.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.pending_text);
        let tools = std::mem::take(&mut self.pending_tool_calls);
        let turn_input = self
            .input_tokens
            .saturating_sub(self.turn_start_input_tokens);
        let mut turn_output = self
            .output_tokens
            .saturating_sub(self.turn_start_output_tokens);
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
    executor: TaskExecutor,
    next_id: u64,
}

impl AgentChatManager {
    pub fn new(executor: TaskExecutor) -> Self {
        Self {
            sessions: HashMap::new(),
            executor,
            next_id: 0,
        }
    }

    fn get_session(&mut self, id: &AgentSessionId) -> Result<&mut AgentChatSession> {
        self.sessions
            .get_mut(id)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, format!("session not found: {id}")))
    }

    fn next_session_id(&mut self) -> AgentSessionId {
        let id = self.next_id;
        self.next_id += 1;
        AgentSessionId::new(format!("agent-chat-{id}"))
    }

    /// Create a new session using the agent's default transport config.
    pub async fn create(
        &mut self,
        agent_kind: AgentKind,
        workspace_path: PathBuf,
        model_id: Option<String>,
    ) -> Result<(AgentSessionId, EventReceiver)> {
        self.create_with_config(
            agent_kind,
            workspace_path,
            model_id,
            agent_kind.default_transport_config(),
        )
        .await
    }

    /// Create a new session with an explicit transport config override.
    pub async fn create_with_config(
        &mut self,
        agent_kind: AgentKind,
        workspace_path: PathBuf,
        model_id: Option<String>,
        config: TransportConfig,
    ) -> Result<(AgentSessionId, EventReceiver)> {
        let session_id = self.next_session_id();
        let (event_tx, event_rx) = mpsc::unbounded_channel::<AgentChatEvent>();
        let transport = transport::create(
            config,
            workspace_path.clone(),
            &self.executor,
            event_tx.clone(),
        )
        .await?;

        let session = AgentChatSession {
            id: session_id.clone(),
            agent_kind,
            workspace_path,
            model_id,
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
            transport: Arc::from(transport),
        };

        tracing::info!(
            session_id = %session_id,
            transport = %session.transport_label(),
            "created agent chat session"
        );
        self.sessions.insert(session_id.clone(), session);
        Ok((session_id, event_rx))
    }

    fn shutdown_transport(&self, transport: Arc<dyn Transport>) {
        self.executor.spawn(async move {
            transport.shutdown().await;
        });
    }

    /// Send a prompt and wait for the turn to complete.
    ///
    /// Cancellation: call [`cancel()`](AgentChatManager::cancel) from another
    /// task to abort this turn via the shared `CancellationToken`.
    pub async fn prompt(&mut self, session_id: &AgentSessionId, message: String) -> Result<()> {
        let session = self.get_session(session_id)?;
        if session.status == AgentChatStatus::Working {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "agent is already processing",
            ));
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
            content: message.clone(),
        });
        session.status = AgentChatStatus::Working;
        session.turn_start_input_tokens = session.input_tokens;
        session.turn_start_output_tokens = session.output_tokens;
        let _ = session.event_tx.send(AgentChatEvent::TurnStarted);

        let cancel = CancellationToken::new();
        session.turn_cancel = Some(cancel.clone());

        let transport = Arc::clone(&session.transport);
        let messages = session.messages.clone();
        let event_tx = session.event_tx.clone();

        let req = PromptRequest {
            messages: &messages,
            cancel,
        };

        match transport.prompt(req).await {
            Ok(()) => {
                let _ = event_tx.send(AgentChatEvent::TurnCompleted);
            }
            Err(e) => {
                let _ = event_tx.send(AgentChatEvent::Error {
                    message: e.to_string(),
                });
                let _ = event_tx.send(AgentChatEvent::TurnCompleted);
            }
        }

        Ok(())
    }

    pub fn cancel(&mut self, session_id: &AgentSessionId) -> Result<()> {
        let session = self.get_session(session_id)?;
        if let Some(cancel) = session.turn_cancel.take() {
            cancel.cancel();
        }
        Ok(())
    }

    pub fn kill(&mut self, session_id: &AgentSessionId) -> Result<()> {
        let session = self.get_session(session_id)?;
        if let Some(cancel) = session.turn_cancel.take() {
            cancel.cancel();
        }
        let transport = Arc::clone(&session.transport);
        session.status = AgentChatStatus::Exited;
        let _ = session
            .event_tx
            .send(AgentChatEvent::SessionExited { exit_code: None });
        self.shutdown_transport(transport);
        Ok(())
    }

    pub fn remove(&mut self, session_id: &AgentSessionId) {
        if let Some(session) = self.sessions.remove(session_id) {
            if let Some(cancel) = &session.turn_cancel {
                cancel.cancel();
            }
            self.shutdown_transport(Arc::clone(&session.transport));
        }
    }

    pub fn list(&self) -> Vec<AgentSessionSummary> {
        self.sessions
            .values()
            .map(|s| AgentSessionSummary {
                id: s.id.to_string(),
                agent_kind: s.agent_kind.key().to_owned(),
                workspace_path: s.workspace_path.display().to_string(),
                status: s.status,
                input_tokens: s.input_tokens,
                output_tokens: s.output_tokens,
                transport_label: s.transport_label(),
            })
            .collect()
    }

    pub fn session(&self, id: &AgentSessionId) -> Option<&AgentChatSession> {
        self.sessions.get(id)
    }

    pub fn session_mut(&mut self, id: &AgentSessionId) -> Option<&mut AgentChatSession> {
        self.sessions.get_mut(id)
    }
}
