//! OpenAI-compatible HTTP transport — SSE streaming.

use tokio::sync::{broadcast, watch};

use crate::event::AgentChatEvent;
use crate::types::ChatMessage;
use super::AgentTransport;

/// OpenAI-compatible HTTP transport.
pub struct OpenAiTransport {
    pub base_url: String,
    pub api_key: Option<String>,
    pub model_id: String,
    pub messages: Vec<ChatMessage>,
}

#[async_trait::async_trait]
impl AgentTransport for OpenAiTransport {
    async fn run_turn(
        &self,
        _prompt: &str,
        event_tx: &broadcast::Sender<AgentChatEvent>,
        _cancel_rx: watch::Receiver<bool>,
    ) -> Result<(), String> {
        // TODO: Implement HTTP SSE streaming to OpenAI-compatible endpoint
        let _ = event_tx.send(AgentChatEvent::MessageChunk {
            content: format!("[OpenAI turn not yet implemented: {}]", self.base_url),
        });
        Ok(())
    }
}
