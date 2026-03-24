//! Managed process types and lifecycle.

use serde::{Deserialize, Serialize};

/// Runtime status of a managed process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessStatus {
    Running,
    Restarting,
    Crashed,
    Stopped,
}

/// Where a process definition came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessSource {
    AtriumToml,
    Procfile,
}

/// Runtime information about a managed process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub id: String,
    pub name: String,
    pub command: String,
    pub repo_root: String,
    pub workspace_id: String,
    pub source: ProcessSource,
    pub status: ProcessStatus,
    pub exit_code: Option<i32>,
    pub restart_count: u32,
    pub memory_bytes: Option<u64>,
    pub session_id: Option<String>,
}
