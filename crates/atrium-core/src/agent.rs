//! AI agent state and session detection.

use serde::{Deserialize, Serialize};
use strum::{Display, IntoStaticStr};

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
    Pi,
    OpenCode,
}

/// Summary of a detected agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionSummary {
    pub provider: AgentProvider,
    pub session_id: String,
    pub title: String,
    pub timestamp_unix_ms: u64,
}
