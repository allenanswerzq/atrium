//! Agent preset definitions — supported agents and their default commands.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};

/// Supported agent types.
///
/// `Display` / `IntoStaticStr` give the snake_case key (e.g. `"claude"`).
/// Use `label()` for the human-readable name (e.g. `"Claude"`).
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
    /// The acpx subcommand key. Same as `Display` output.
    pub fn key(self) -> &'static str {
        self.into()
    }

    /// Default shell command to launch this agent.
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

/// A configured agent preset (kind + optional custom command).
#[derive(Debug, Clone)]
pub struct AgentPreset {
    pub kind: AgentKind,
    pub command: String,
}

impl AgentPreset {
    pub fn new(kind: AgentKind) -> Self {
        Self {
            command: kind.default_command().to_owned(),
            kind,
        }
    }

    pub fn with_command(kind: AgentKind, command: impl Into<String>) -> Self {
        Self {
            kind,
            command: command.into(),
        }
    }
}
