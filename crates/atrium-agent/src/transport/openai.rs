//! OpenAI-compatible HTTP transport — `/v1/chat/completions` with SSE.
//!
//! Multi-turn: the full conversation history is sent with every request.
//! Works with any OpenAI-format endpoint (Ollama, LM Studio, OpenRouter, etc.).

use atrium_error::{Error, ErrorKind, Result};

use super::{PromptRequest, Transport};
use crate::types::AgentChatEvent;

/// HTTP-based transport talking to an OpenAI-compatible API.
pub struct OpenAiTransport {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    model: Option<String>,
}

impl OpenAiTransport {
    pub fn new(base_url: String, api_key: Option<String>, model: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            api_key,
            model,
        }
    }
}

#[async_trait::async_trait]
impl Transport for OpenAiTransport {
    async fn prompt(&self, req: PromptRequest<'_>) -> Result<()> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let model = req.model_id.or(self.model.as_deref()).unwrap_or("gpt-4");

        // Convert full history to OpenAI message format.
        let openai_messages: Vec<serde_json::Value> = req
            .messages
            .iter()
            .filter(|m| m.role == "user" || m.role == "assistant" || m.role == "system")
            .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
            .collect();

        let body = serde_json::json!({
            "model": model,
            "messages": openai_messages,
            "stream": true,
            "stream_options": {"include_usage": true},
        });

        tracing::info!(url = %url, model = %model, messages = openai_messages.len(), "openai request");

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

        let event_tx = req.event_tx.clone();
        let mut cancel_rx = req.cancel_rx;

        consume_sse_stream(response, &mut cancel_rx, |data| {
            let value: serde_json::Value = match serde_json::from_str(data) {
                Ok(v) => v,
                Err(_) => return,
            };

            // Text delta.
            if let Some(delta) = value
                .pointer("/choices/0/delta/content")
                .and_then(|v| v.as_str())
            {
                if !delta.is_empty() {
                    let _ = event_tx.send(AgentChatEvent::MessageChunk {
                        content: delta.to_owned(),
                    });
                }
            }

            // Usage.
            if let Some(usage) = value.get("usage") {
                let input = usage
                    .get("prompt_tokens")
                    .or_else(|| usage.get("input_tokens"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let output = usage
                    .get("completion_tokens")
                    .or_else(|| usage.get("output_tokens"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if input > 0 || output > 0 {
                    let _ = event_tx.send(AgentChatEvent::UsageUpdate {
                        input_tokens: input,
                        output_tokens: output,
                    });
                }
            }
        })
        .await
    }

    async fn shutdown(&self) {
        // Stateless — nothing to shut down.
    }

    fn label(&self) -> String {
        format!("openai:{}", self.base_url)
    }
}

/// Consume an SSE byte stream, calling `on_data` for each `data:` payload.
async fn consume_sse_stream(
    response: reqwest::Response,
    cancel_rx: &mut tokio::sync::watch::Receiver<bool>,
    mut on_data: impl FnMut(&str),
) -> Result<()> {
    use futures::StreamExt;
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    loop {
        tokio::select! {
            biased;
            _ = cancel_rx.changed() => {
                if *cancel_rx.borrow() {
                    return Err(Error::new(ErrorKind::Cancelled, "turn cancelled"));
                }
            }
            chunk = stream.next() => {
                let Some(chunk_result) = chunk else { break };
                let bytes = chunk_result.map_err(|e| {
                    Error::new(ErrorKind::Network, format!("stream error: {e}")).set_source(e)
                })?;
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
