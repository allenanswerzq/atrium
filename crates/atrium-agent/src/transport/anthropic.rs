//! Anthropic Messages API transport — `/v1/messages` with SSE.
//!
//! Multi-turn: the full conversation history is sent with every request.
//! Works with the Anthropic API directly or via bridge servers that
//! expose the Anthropic Messages format.

use std::time::Duration;

use atrium_error::{Error, ErrorKind, Result};

use super::{EventSender, PromptRequest, Transport};
use crate::types::AgentChatEvent;
use tokio_util::sync::CancellationToken;

/// Connect timeout for the HTTP client.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
/// Max time to wait between SSE chunks before treating the stream as dead.
const CHUNK_IDLE_TIMEOUT: Duration = Duration::from_secs(120);
/// Default max tokens for Anthropic API.
const DEFAULT_MAX_TOKENS: u32 = 8192;

/// HTTP-based transport talking to an Anthropic Messages API endpoint.
pub struct AnthropicTransport {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    model: Option<String>,
    event_tx: EventSender,
}

impl AnthropicTransport {
    pub fn new(
        base_url: String,
        api_key: Option<String>,
        model: Option<String>,
        event_tx: EventSender,
    ) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .unwrap_or_default();
        Self {
            client,
            base_url,
            api_key,
            model,
            event_tx,
        }
    }
}

#[async_trait::async_trait]
impl Transport for AnthropicTransport {
    async fn prompt(&self, req: PromptRequest<'_>) -> Result<()> {
        let url = format!("{}/messages", self.base_url.trim_end_matches('/'));
        let model = self.model.as_deref().unwrap_or("claude-sonnet-4");

        // Convert history to Anthropic message format.
        let messages: Vec<serde_json::Value> = req
            .messages
            .iter()
            .filter(|m| m.role == "user" || m.role == "assistant")
            .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
            .collect();

        let body = serde_json::json!({
            "model": model,
            "max_tokens": DEFAULT_MAX_TOKENS,
            "messages": messages,
            "stream": true,
        });

        tracing::info!(url = %url, model = %model, messages = messages.len(), "anthropic request");

        let mut request = self
            .client
            .post(&url)
            .header("content-type", "application/json");
        if let Some(key) = &self.api_key {
            request = request.header("x-api-key", key);
            request = request.header("anthropic-version", "2023-06-01");
        }
        request = request.body(serde_json::to_string(&body).unwrap_or_default());

        let response = request.send().await.map_err(|e| {
            Error::new(ErrorKind::Network, format!("request failed: {e}")).set_source(e)
        })?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(
                Error::new(ErrorKind::Network, format!("HTTP {status}: {body}"))
                    .with_context("status", status.to_string()),
            );
        }

        let event_tx = self.event_tx.clone();
        let cancel = req.cancel;

        consume_anthropic_sse(response, &cancel, &event_tx).await
    }

    async fn shutdown(&self) {
        // Stateless — nothing to shut down.
    }

    fn label(&self) -> String {
        format!("anthropic:{}", self.base_url)
    }
}

/// Consume an Anthropic SSE stream.
///
/// Anthropic SSE uses `event:` lines to distinguish event types:
/// - `content_block_delta` with `delta.text` for text chunks
/// - `message_delta` with `usage` for token counts
/// - `message_stop` signals completion
async fn consume_anthropic_sse(
    response: reqwest::Response,
    cancel: &CancellationToken,
    event_tx: &EventSender,
) -> Result<()> {
    use futures::StreamExt;
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut current_event = String::new();

    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => {
                return Err(Error::new(ErrorKind::Cancelled, "turn cancelled"));
            }
            chunk = tokio::time::timeout(CHUNK_IDLE_TIMEOUT, stream.next()) => {
                let Ok(chunk_opt) = chunk else {
                    return Err(Error::new(ErrorKind::Network, "SSE stream idle timeout"));
                };
                let Some(chunk_result) = chunk_opt else { break };
                let bytes = chunk_result.map_err(|e| {
                    Error::new(ErrorKind::Network, format!("stream error: {e}")).set_source(e)
                })?;
                buffer.push_str(&String::from_utf8_lossy(&bytes));

                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].trim().to_owned();
                    buffer = buffer[pos + 1..].to_owned();

                    if line.is_empty() {
                        current_event.clear();
                        continue;
                    }
                    if let Some(event_type) = line.strip_prefix("event:") {
                        current_event = event_type.trim().to_owned();
                        continue;
                    }
                    if line.starts_with(':') { continue; }
                    if !line.starts_with("data:") { continue; }

                    let data = line.trim_start_matches("data:").trim();
                    let Ok(value) = serde_json::from_str::<serde_json::Value>(data) else {
                        continue;
                    };

                    match current_event.as_str() {
                        "content_block_delta" => {
                            if let Some(text) = value
                                .pointer("/delta/text")
                                .and_then(|v| v.as_str())
                            {
                                if !text.is_empty() {
                                    let _ = event_tx.send(AgentChatEvent::MessageChunk {
                                        content: text.to_owned(),
                                    });
                                }
                            }
                        }
                        "message_delta" => {
                            if let Some(usage) = value.get("usage") {
                                let output = usage
                                    .get("output_tokens")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0);
                                if output > 0 {
                                    let _ = event_tx.send(AgentChatEvent::UsageUpdate {
                                        input_tokens: 0,
                                        output_tokens: output,
                                    });
                                }
                            }
                        }
                        "message_start" => {
                            if let Some(usage) = value.pointer("/message/usage") {
                                let input = usage
                                    .get("input_tokens")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0);
                                if input > 0 {
                                    let _ = event_tx.send(AgentChatEvent::UsageUpdate {
                                        input_tokens: input,
                                        output_tokens: 0,
                                    });
                                }
                            }
                        }
                        "message_stop" => {
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}
