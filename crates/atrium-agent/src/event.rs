//! Agent chat events — streamed to the UI via broadcast channels.

use serde::{Deserialize, Serialize};

use crate::types::{AgentChatStatus, ChatMessage};

/// A structured event emitted by an agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentChatEvent {
    /// A chunk of the assistant's text response (streamed).
    MessageChunk { content: String },
    /// A chunk of the agent's internal reasoning.
    ThoughtChunk { content: String },
    /// A tool invocation by the agent.
    ToolCall { name: String, status: String },
    /// Agent started processing a turn.
    TurnStarted,
    /// Agent finished processing a turn.
    TurnCompleted,
    /// Token usage update (cumulative).
    UsageUpdate {
        input_tokens: u64,
        output_tokens: u64,
    },
    /// Error from the agent.
    Error { message: String },
    /// The agent process exited.
    SessionExited { exit_code: Option<i32> },
    /// Full conversation snapshot (sent on connect).
    Snapshot {
        messages: Vec<ChatMessage>,
        status: AgentChatStatus,
        input_tokens: u64,
        output_tokens: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transport_label: Option<String>,
    },
    /// A user message (for history reconstruction).
    UserMessage { content: String },
    /// Status update (mode changes, config updates).
    StatusUpdate { message: String },
}
