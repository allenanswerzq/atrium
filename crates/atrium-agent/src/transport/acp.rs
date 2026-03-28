//! ACP transport — spawns agent CLI via `acpx` subprocess.

use tokio::sync::{broadcast, watch};

use crate::event::AgentChatEvent;
use super::AgentTransport;

/// ACP transport configuration.
pub struct AcpTransport {
    pub agent_kind: String,
    pub workspace_path: std::path::PathBuf,
    pub session_name: String,
    pub model_id: Option<String>,
}

#[async_trait::async_trait]
impl AgentTransport for AcpTransport {
    async fn run_turn(
        &self,
        _prompt: &str,
        event_tx: &broadcast::Sender<AgentChatEvent>,
        _cancel_rx: watch::Receiver<bool>,
    ) -> Result<(), String> {
        // TODO: Implement acpx subprocess spawn + JSONL parsing
        // For now, emit a placeholder message
        let _ = event_tx.send(AgentChatEvent::MessageChunk {
            content: format!("[ACP {} turn not yet implemented]", self.agent_kind),
        });
        Ok(())
    }
}
