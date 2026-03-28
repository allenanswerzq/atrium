//! Agent activity monitoring — track Working/Waiting state per worktree.

use serde::{Deserialize, Serialize};

use crate::types::AgentState;

/// A record of agent activity for a worktree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentActivityRecord {
    pub session_id: String,
    pub cwd: String,
    pub state: AgentState,
    pub updated_at_unix_ms: Option<u64>,
}
