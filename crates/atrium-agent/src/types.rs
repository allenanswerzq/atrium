//! Core agent types — IDs, status, messages, events, persistence.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use strum::{Display, IntoStaticStr};

// ── IDs ─────────────────────────────────────────────────────────────

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

// ── Enums ───────────────────────────────────────────────────────────

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

/// Transport used by an agent chat session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentChatTransport {
    Acp,
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

// ── Messages ────────────────────────────────────────────────────────

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

// ── Events ──────────────────────────────────────────────────────────

/// A structured event emitted by an agent session, streamed to the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentChatEvent {
    MessageChunk { content: String },
    ThoughtChunk { content: String },
    ToolCall { name: String, status: String },
    TurnStarted,
    TurnCompleted,
    UsageUpdate { input_tokens: u64, output_tokens: u64 },
    Error { message: String },
    SessionExited { exit_code: Option<i32> },
    Snapshot {
        messages: Vec<ChatMessage>,
        status: AgentChatStatus,
        input_tokens: u64,
        output_tokens: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transport_label: Option<String>,
    },
    UserMessage { content: String },
    StatusUpdate { message: String },
}

// ── Activity ────────────────────────────────────────────────────────

/// A record of agent activity for a worktree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentActivityRecord {
    pub session_id: String,
    pub cwd: String,
    pub state: AgentState,
    pub updated_at_unix_ms: Option<u64>,
}

// ── Persistence ─────────────────────────────────────────────────────

/// A serializable snapshot of an agent chat session (for JSON storage).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChatRecord {
    pub id: String,
    pub agent_kind: String,
    pub workspace_path: PathBuf,
    pub session_name: String,
    #[serde(default)]
    pub model_id: Option<String>,
    #[serde(default)]
    pub transport: AgentChatTransport,
    pub messages: Vec<ChatMessage>,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Load persisted sessions from a JSON file.
pub fn load_agent_sessions(path: &std::path::Path) -> Vec<AgentChatRecord> {
    let data = match std::fs::read_to_string(path) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    serde_json::from_str(&data).unwrap_or_default()
}

/// Save sessions to a JSON file.
pub fn save_agent_sessions(path: &std::path::Path, records: &[AgentChatRecord]) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(records) {
        let _ = std::fs::write(path, format!("{json}\n"));
    }
}

// ── Summary DTO ─────────────────────────────────────────────────────

/// Summary DTO for listing sessions.
#[derive(Debug, Clone, Serialize)]
pub struct AgentSessionSummary {
    pub id: String,
    pub agent_kind: String,
    pub workspace_path: String,
    pub status: AgentChatStatus,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub transport_label: String,
}
