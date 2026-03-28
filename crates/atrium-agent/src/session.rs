//! Agent chat session — conversation state and token tracking.

use std::path::PathBuf;

use tokio::sync::broadcast;

use crate::event::AgentChatEvent;
use crate::preset::AgentKind;
use crate::types::{AgentChatStatus, AgentChatTransport, AgentSessionId, ChatMessage};

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
    /// Cumulative token counts.
    pub input_tokens: u64,
    pub output_tokens: u64,
    /// Token counts at start of current turn (for per-turn delta).
    pub turn_start_input_tokens: u64,
    pub turn_start_output_tokens: u64,
    /// Cancel handle for the running turn.
    pub turn_cancel: Option<tokio::sync::watch::Sender<bool>>,
}

impl AgentChatSession {
    /// Human-readable transport label for debugging.
    pub fn transport_label(&self) -> String {
        match &self.transport {
            AgentChatTransport::Acp => format!("acp:{}", self.agent_kind.key()),
            AgentChatTransport::OpenAiChat { base_url, .. } => {
                format!("openai:{base_url}")
            }
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
