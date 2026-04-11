//! Agent kind — supported agent types and their default transport configs.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};

use crate::transport::TransportConfig;

/// Supported agent types.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    EnumIter,
    IntoStaticStr,
)]
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

    /// Default transport config for this agent.
    pub fn default_transport_config(self) -> TransportConfig {
        self.default_acp_config()
    }

    /// Default ACP transport config for this agent.
    pub fn default_acp_config(self) -> TransportConfig {
        match self {
            Self::Copilot => TransportConfig::Acp {
                program: "copilot".into(),
                args: vec!["--acp".into()],
            },
            Self::Claude => TransportConfig::Acp {
                program: "claude-agent-acp".into(),
                args: vec![],
            },
            Self::Codex => TransportConfig::Acp {
                program: "codex-acp".into(),
                args: vec![],
            },
            // Agents without ACP fall back to terminal.
            other => other.default_terminal_config(),
        }
    }

    /// Default terminal (per-turn subprocess) transport config.
    pub fn default_terminal_config(self) -> TransportConfig {
        match self {
            Self::Copilot => TransportConfig::Terminal {
                program: "copilot".into(),
                base_args: vec!["--allow-all".into(), "-s".into()],
            },
            Self::Claude => TransportConfig::Terminal {
                program: "claude".into(),
                base_args: vec!["--dangerously-skip-permissions".into()],
            },
            Self::Codex => TransportConfig::Terminal {
                program: "codex".into(),
                base_args: vec!["-q".into()],
            },
            Self::Cursor => TransportConfig::Terminal {
                program: "cursor".into(),
                base_args: vec![],
            },
            Self::Gemini => TransportConfig::Terminal {
                program: "gemini".into(),
                base_args: vec![],
            },
            Self::OpenCode => TransportConfig::Terminal {
                program: "opencode".into(),
                base_args: vec![],
            },
            Self::Pi => TransportConfig::Terminal {
                program: "pi".into(),
                base_args: vec![],
            },
        }
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
