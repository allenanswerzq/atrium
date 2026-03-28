//! Agent transport backends.
//!
//! A transport defines how messages are sent to and received from an agent.

pub mod acp;
pub mod openai;

use tokio::sync::{broadcast, watch};

use crate::event::AgentChatEvent;

/// Trait for agent chat transport backends.
///
/// Each transport runs a "turn" — sends a prompt and streams events back.
#[async_trait::async_trait]
pub trait AgentTransport: Send + Sync {
    /// Execute a turn: send the prompt and stream events until completion.
    async fn run_turn(
        &self,
        prompt: &str,
        event_tx: &broadcast::Sender<AgentChatEvent>,
        cancel_rx: watch::Receiver<bool>,
    ) -> Result<(), String>;
}
