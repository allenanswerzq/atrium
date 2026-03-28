//! Persist agent chat sessions to JSON.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::types::{AgentChatTransport, ChatMessage};

/// A serializable snapshot of an agent chat session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChatRecord {
    pub id: String,
    pub agent_kind: String,
    pub workspace_path: PathBuf,
    pub session_name: String,
    #[serde(default)]
    pub model_id: Option<String>,
    #[serde(default)]
    pub transport: AgentChatTransport,
    pub messages: Vec<ChatMessage>,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Load persisted sessions from a JSON file.
pub fn load_sessions(path: &std::path::Path) -> Vec<AgentChatRecord> {
    let data = match std::fs::read_to_string(path) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    serde_json::from_str(&data).unwrap_or_default()
}

/// Save sessions to a JSON file.
pub fn save_sessions(path: &std::path::Path, records: &[AgentChatRecord]) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(records) {
        let _ = std::fs::write(path, format!("{json}\n"));
    }
}
