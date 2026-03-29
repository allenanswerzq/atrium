//! Agent kind — supported agent types and their default commands.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};

/// Supported agent types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, EnumIter, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AgentKind {
    Claude,
    Codex,
    Copilot,
    Cursor,
    Gemini,
    #[strum(serialize = "opencode")]
    #[serde(rename = "opencode")]
    OpenCode,
    Pi,
}

impl AgentKind {
    /// The identifier key. Same as `Display` output.
    pub fn key(self) -> &'static str {
        self.into()
    }

    /// Default shell command to launch this agent interactively.
    pub fn default_command(self) -> &'static str {
        match self {
            Self::Claude => "claude --dangerously-skip-permissions",
            Self::Codex => "codex --dangerously-bypass-approvals-and-sandbox",
            Self::Copilot => "copilot --allow-all",
            Self::Cursor => "cursor",
            Self::Gemini => "gemini",
            Self::OpenCode => "opencode",
            Self::Pi => "pi",
        }
    }
}
