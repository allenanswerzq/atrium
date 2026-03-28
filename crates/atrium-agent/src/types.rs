//! Core agent types — status, providers, messages, IDs.

use serde::{Deserialize, Serialize};
use strum::{Display, IntoStaticStr};

/// Unique identifier for an agent chat session.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentSessionId(String);

impl AgentSessionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AgentSessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for AgentSessionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Runtime state of an AI coding agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AgentState {
    Working,
    Waiting,
}

/// Which AI agent provider a session belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AgentProvider {
    Claude,
    Codex,
    Copilot,
    OpenCode,
}

/// Status of an agent chat session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AgentChatStatus {
    Idle,
    Working,
    Exited,
}

/// A message in the conversation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub input_tokens: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub output_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport_label: Option<String>,
}

fn is_zero(v: &u64) -> bool {
    *v == 0
}

/// Transport used by an agent chat session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentChatTransport {
    /// ACP agent via acpx CLI subprocess.
    Acp,
    /// OpenAI-compatible HTTP API (Ollama, LM Studio, etc.).
    OpenAiChat {
        base_url: String,
        api_key: Option<String>,
    },
}

impl Default for AgentChatTransport {
    fn default() -> Self {
        Self::Acp
    }
}
