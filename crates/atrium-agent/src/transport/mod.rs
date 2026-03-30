//! Agent transport trait and implementations.
//!
//! A [`Transport`] handles communication with an AI agent across turns.
//! The session creates one transport at startup and reuses it for every
//! prompt. Events are streamed back through an [`EventSender`] (`mpsc`)
//! provided at creation time.
//!
//! | Transport | Process model | Multi-turn |
//! |-----------|--------------|------------|
//! | [`AcpTransport`](acp::AcpTransport) | Long-lived process via ACP | Server-managed sessions |
//! | [`TerminalTransport`](terminal::TerminalTransport) | New subprocess per turn | Single-turn only |
//! | [`OpenAiTransport`](openai::OpenAiTransport) | HTTP `/v1/chat/completions` | Client sends full history |
//! | [`AnthropicTransport`](anthropic::AnthropicTransport) | HTTP `/v1/messages` | Client sends full history |
//! | [`ResponsesTransport`](responses::ResponsesTransport) | HTTP `/v1/responses` | Client sends full history |

pub mod acp;
pub mod anthropic;
pub mod openai;
pub mod responses;
pub mod terminal;

use std::path::PathBuf;

use atrium_error::Result;
use atrium_executor::TaskExecutor;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::types::{AgentChatEvent, ChatMessage};

// ── Trait ────────────────────────────────────────────────────────────

/// Per-turn request passed to [`Transport::prompt`].
pub struct PromptRequest<'a> {
    /// Full conversation history (including the current user message as last entry).
    pub messages: &'a [ChatMessage],
    /// Cancellation token — cancelled to abort the current turn.
    pub cancel: CancellationToken,
}

impl<'a> PromptRequest<'a> {
    /// Extract the latest user message text from the conversation history.
    pub fn last_user_message(&self) -> &str {
        self.messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .unwrap_or_default()
    }
}

/// Sender type for streaming events from transports.
pub type EventSender = mpsc::UnboundedSender<AgentChatEvent>;
/// Receiver type for consuming events from transports.
pub type EventReceiver = mpsc::UnboundedReceiver<AgentChatEvent>;

/// A transport handles communication with an AI agent.
///
/// Created once per session, reused across turns. Must be `Send + Sync`
/// so it can be held across `.await` points and shared with spawned tasks.
///
/// Events are sent through the [`EventSender`] provided at creation time.
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Send a prompt and stream response events via the transport's event sender.
    ///
    /// Blocks until the turn completes or is cancelled. The caller sends
    /// `TurnCompleted` / `Error` events after this returns.
    async fn prompt(&self, req: PromptRequest<'_>) -> Result<()>;

    /// Shut down the transport, killing any background processes.
    async fn shutdown(&self);

    /// Human-readable label for logging / UI.
    fn label(&self) -> String;
}

// ── Factory ─────────────────────────────────────────────────────────

/// Configuration for creating a transport — serializable for persistence.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransportConfig {
    /// ACP: long-lived agent process communicating via JSON-RPC over stdio.
    Acp {
        /// Command to start the ACP server (e.g. `copilot`, `codex-acp`).
        program: String,
        /// Arguments (e.g. `["--acp"]`).
        #[serde(default)]
        args: Vec<String>,
    },
    /// Terminal: per-turn subprocess with `-p <prompt>`.
    Terminal {
        /// Command to run (e.g. `copilot`).
        program: String,
        /// Base arguments appended before `-p <prompt>`.
        #[serde(default)]
        base_args: Vec<String>,
    },
    /// OpenAI-compatible HTTP API with SSE streaming (`/v1/chat/completions`).
    OpenAi {
        /// Base URL (e.g. `https://api.openai.com/v1`).
        base_url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        api_key: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
    },
    /// Anthropic Messages API with SSE streaming (`/v1/messages`).
    Anthropic {
        /// Base URL (e.g. `https://api.anthropic.com/v1`).
        base_url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        api_key: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
    },
    /// OpenAI Responses API with SSE streaming (`/v1/responses`).
    Responses {
        /// Base URL (e.g. `https://api.openai.com/v1`).
        base_url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        api_key: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
    },
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self::Terminal {
            program: "copilot".to_owned(),
            base_args: vec!["--allow-all".to_owned(), "-s".to_owned()],
        }
    }
}

/// Create a [`Transport`] from a [`TransportConfig`].
///
/// The `event_tx` sender is cloned into the transport and used for all
/// subsequent event streaming. For ACP, the agent process is spawned
/// immediately and the ACP handshake (initialize + new_session) runs
/// before this returns.
pub async fn create(
    config: TransportConfig,
    workspace_path: PathBuf,
    executor: &TaskExecutor,
    event_tx: EventSender,
) -> Result<Box<dyn Transport>> {
    match config {
        TransportConfig::Acp { program, args } => {
            let t =
                acp::AcpTransport::spawn(program, args, workspace_path, executor, event_tx).await?;
            Ok(Box::new(t))
        }
        TransportConfig::Terminal { program, base_args } => {
            let t = terminal::TerminalTransport::new(program, base_args, workspace_path, event_tx);
            Ok(Box::new(t))
        }
        TransportConfig::OpenAi {
            base_url,
            api_key,
            model,
        } => {
            let t = openai::OpenAiTransport::new(base_url, api_key, model, event_tx);
            Ok(Box::new(t))
        }
        TransportConfig::Anthropic {
            base_url,
            api_key,
            model,
        } => {
            let t = anthropic::AnthropicTransport::new(base_url, api_key, model, event_tx);
            Ok(Box::new(t))
        }
        TransportConfig::Responses {
            base_url,
            api_key,
            model,
        } => {
            let t = responses::ResponsesTransport::new(base_url, api_key, model, event_tx);
            Ok(Box::new(t))
        }
    }
}
