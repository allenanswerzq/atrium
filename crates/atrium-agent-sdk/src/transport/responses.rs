//! OpenAI Responses API transport — `/v1/responses` with SSE.
//!
//! Used by Codex CLI. Sends a single `input` string and streams back
//! response events. Multi-turn context is handled by including prior
//! messages in the input array.

use std::time::Duration;

use atrium_error::{Error, ErrorKind, Result};

use super::{EventSender, PromptRequest, Transport};
use crate::types::AgentChatEvent;
use tokio_util::sync::CancellationToken;

/// Connect timeout for the HTTP client.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
/// Max time to wait between SSE chunks before treating the stream as dead.
const CHUNK_IDLE_TIMEOUT: Duration = Duration::from_secs(120);

/// HTTP-based transport talking to the OpenAI Responses API.
pub struct ResponsesTransport {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    model: Option<String>,
    event_tx: EventSender,
}

impl ResponsesTransport {
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
impl Transport for ResponsesTransport {
    async fn prompt(&self, req: PromptRequest<'_>) -> Result<()> {
        let url = format!("{}/responses", self.base_url.trim_end_matches('/'));
        let model = self.model.as_deref().unwrap_or("gpt-4.1");

        // Build the input array: prior messages + current prompt.
        let input: Vec<serde_json::Value> = req
            .messages
            .iter()
            .filter(|m| m.role == "user" || m.role == "assistant" || m.role == "system")
            .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
            .collect();

        let body = serde_json::json!({
            "model": model,
            "input": input,
            "stream": true,
        });

        tracing::info!(url = %url, model = %model, input_len = input.len(), "responses request");

        let mut request = self.client.post(&url).json(&body);
        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }

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

        consume_responses_sse(response, &cancel, &event_tx).await
    }

    async fn shutdown(&self) {
        // Stateless — nothing to shut down.
    }

    fn label(&self) -> String {
        format!("responses:{}", self.base_url)
    }
}

/// Consume an OpenAI Responses API SSE stream.
///
/// The Responses API emits events like:
/// - `response.output_text.delta` with `delta` field for text
/// - `response.function_call_arguments.start` for tool call start
/// - `response.function_call_arguments.done` for tool call completion
/// - `response.completed` signals the end
/// - `response.output_text.done` with full text
async fn consume_responses_sse(
    response: reqwest::Response,
    cancel: &CancellationToken,
    event_tx: &EventSender,
) -> Result<()> {
    use futures::StreamExt;
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut current_event = String::new();
    // Track the active function call name so we can pair start/done.
    let mut active_fn_name: Option<String> = None;

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
                    if data == "[DONE]" { return Ok(()); }

                    let Ok(value) = serde_json::from_str::<serde_json::Value>(data) else {
                        continue;
                    };

                    match current_event.as_str() {
                        "response.output_item.added" => {
                            // When a function_call output item is added,
                            // capture its name and emit a started event.
                            if value.get("type").and_then(|v| v.as_str()) == Some("function_call")
                                || value
                                    .pointer("/item/type")
                                    .and_then(|v| v.as_str())
                                    == Some("function_call")
                            {
                                let item = value.get("item").unwrap_or(&value);
                                let name = item
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("tool")
                                    .to_owned();
                                let _ = event_tx.send(AgentChatEvent::ToolCall {
                                    name: name.clone(),
                                    status: "started".to_owned(),
                                });
                                active_fn_name = Some(name);
                            }
                        }
                        "response.function_call_arguments.done" => {
                            if let Some(name) = active_fn_name.take() {
                                let _ = event_tx.send(AgentChatEvent::ToolCall {
                                    name,
                                    status: "completed".to_owned(),
                                });
                            }
                        }
                        "response.output_text.delta" => {
                            if let Some(text) = value.get("delta").and_then(|v| v.as_str()) {
                                if !text.is_empty() {
                                    let _ = event_tx.send(AgentChatEvent::MessageChunk {
                                        content: text.to_owned(),
                                    });
                                }
                            }
                        }
                        "response.completed" => {
                            // Extract usage from the completed response.
                            if let Some(usage) = value.get("usage") {
                                let input = usage
                                    .get("input_tokens")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0);
                                let output = usage
                                    .get("output_tokens")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0);
                                if input > 0 || output > 0 {
                                    let _ = event_tx.send(AgentChatEvent::UsageUpdate {
                                        input_tokens: input,
                                        output_tokens: output,
                                    });
                                }
                            }
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
