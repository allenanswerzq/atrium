//! Scheduled task types and execution history.

use serde::{Deserialize, Serialize};

/// Runtime status of a scheduled task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Idle,
    Running,
    Disabled,
}

/// A record of one task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub task_name: String,
    pub started_at_unix_ms: u64,
    pub finished_at_unix_ms: Option<u64>,
    pub exit_code: Option<i32>,
    pub stdout_tail: Option<String>,
    pub agent_spawned: bool,
}

/// Runtime information about a scheduled task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub name: String,
    pub schedule: Option<String>,
    pub command: String,
    pub status: TaskStatus,
    pub has_trigger: bool,
    pub last_run_unix_ms: Option<u64>,
    pub last_exit_code: Option<i32>,
    pub next_run_unix_ms: Option<u64>,
    pub run_count: u32,
}
