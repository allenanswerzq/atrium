//! Agent transport trait and implementations.
//!
//! A [`Transport`] handles communication with an AI agent across turns.
//! The session creates one transport at startup and reuses it for every prompt.
//!
//! | Transport | Process model | Multi-turn |
//! |-----------|--------------|------------|
//! | [`AcpTransport`](acp::AcpTransport) | Long-lived process via ACP | Server-managed sessions |
//! | [`TerminalTransport`](terminal::TerminalTransport) | New subprocess per turn | Single-turn only |
//! | [`OpenAiTransport`](openai::OpenAiTransport) | HTTP requests | Client sends full history |

pub mod acp;
pub mod openai;
pub mod terminal;

use std::path::PathBuf;

use tokio::sync::broadcast;

use crate::types::{AgentChatEvent, ChatMessage};

// ── Trait ────────────────────────────────────────────────────────────

/// Request passed to [`Transport::prompt`].
pub struct PromptRequest<'a> {
    /// The new user message for this turn.
    pub prompt: &'a str,
    /// Full conversation history (including the current prompt as last message).
    pub messages: &'a [ChatMessage],
    /// Optional model override.
    pub model_id: Option<&'a str>,
    /// Broadcast channel for streaming events back to the session.
    pub event_tx: &'a broadcast::Sender<AgentChatEvent>,
    /// Cancel signal — send `true` to abort the current turn.
    pub cancel_rx: tokio::sync::watch::Receiver<bool>,
}

/// A transport handles communication with an AI agent.
///
/// Created once per session, reused across turns. Must be `Send + Sync`
/// so it can be held across `.await` points and shared with spawned tasks.
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Send a prompt and stream response events via `req.event_tx`.
    ///
    /// Blocks until the turn completes or is cancelled. The caller sends
    /// `TurnCompleted` / `Error` events after this returns.
    async fn prompt(&self, req: PromptRequest<'_>) -> Result<(), String>;

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
    /// OpenAI-compatible HTTP API with SSE streaming.
    OpenAi {
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
/// For ACP, the agent process is spawned immediately and the ACP handshake
/// (initialize + new_session) runs before this returns.
pub async fn create(
    config: TransportConfig,
    workspace_path: PathBuf,
) -> Result<Box<dyn Transport>, String> {
    match config {
        TransportConfig::Acp { program, args } => {
            let t = acp::AcpTransport::spawn(program, args, workspace_path)?;
            Ok(Box::new(t))
        }
        TransportConfig::Terminal { program, base_args } => {
            let t = terminal::TerminalTransport::new(program, base_args, workspace_path);
            Ok(Box::new(t))
        }
        TransportConfig::OpenAi {
            base_url,
            api_key,
            model,
        } => {
            let t = openai::OpenAiTransport::new(base_url, api_key, model);
            Ok(Box::new(t))
        }
    }
}

// ── Shared helpers ──────────────────────────────────────────────────

/// Read up to 4KB from an optional stderr handle.
pub(crate) async fn read_stderr(stderr: Option<tokio::process::ChildStderr>) -> String {
    use tokio::io::{AsyncBufReadExt, BufReader};
    let Some(stderr) = stderr else {
        return String::new();
    };
    let mut buf = String::new();
    let mut reader = BufReader::new(stderr);
    let mut line_buf = String::new();
    while buf.len() < 4096 {
        match reader.read_line(&mut line_buf).await {
            Ok(0) => break,
            Ok(_) => {
                buf.push_str(&line_buf);
                line_buf.clear();
            }
            Err(_) => break,
        }
    }
    buf
}

/// Consume an SSE byte stream, calling `on_data` for each `data:` payload.
pub(crate) async fn consume_sse_stream(
    response: reqwest::Response,
    cancel_rx: &mut tokio::sync::watch::Receiver<bool>,
    mut on_data: impl FnMut(&str),
) -> Result<(), String> {
    use futures::StreamExt;
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    loop {
        tokio::select! {
            biased;
            _ = cancel_rx.changed() => {
                if *cancel_rx.borrow() {
                    return Err("turn cancelled".to_owned());
                }
            }
            chunk = stream.next() => {
                let Some(chunk_result) = chunk else { break };
                let bytes = chunk_result.map_err(|e| format!("stream error: {e}"))?;
                buffer.push_str(&String::from_utf8_lossy(&bytes));

                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].trim().to_owned();
                    buffer = buffer[pos + 1..].to_owned();
                    if line.is_empty() || line.starts_with(':') { continue; }
                    if !line.starts_with("data:") { continue; }
                    let data = line.trim_start_matches("data:").trim();
                    if data == "[DONE]" { return Ok(()); }
                    on_data(data);
                }
            }
        }
    }
    Ok(())
}
