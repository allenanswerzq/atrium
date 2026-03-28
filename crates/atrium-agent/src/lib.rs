//! # atrium-agent
//!
//! Agent chat sessions, presets, and transport backends.
//!
//! GUI-agnostic — provides the agent backend that any UI can consume.
//!
//! ## Modules
//!
//! - `types`   — IDs, enums, messages, events, persistence
//! - `preset`  — AgentKind (supported agents + default commands)
//! - `session` — AgentChatSession + AgentChatManager

pub mod preset;
pub mod session;
pub mod types;

pub use preset::AgentKind;
pub use session::{AgentChatManager, AgentChatSession};
pub use types::{
    AgentChatEvent, AgentChatStatus, AgentChatTransport, AgentProvider,
    AgentSessionId, AgentState, ChatMessage,
};
