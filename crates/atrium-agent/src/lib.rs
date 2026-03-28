//! # atrium-agent
//!
//! Agent chat sessions, presets, and transport backends.
//!
//! GUI-agnostic — provides the agent backend that any UI can consume.
//!
//! ## Architecture
//!
//! ```text
//!  types.rs        — AgentState, AgentProvider, ChatMessage, ChatStatus
//!  event.rs        — AgentChatEvent (streamed to UI via broadcast)
//!  preset.rs       — AgentKind (supported agents + default commands)
//!  session.rs      — AgentChatSession (messages, tokens, status)
//!  manager.rs      — AgentChatManager (create/send/cancel/kill/list)
//!  transport/      — Transport trait + ACP and OpenAI implementations
//!  activity.rs     — Agent activity monitoring (Working/Waiting)
//!  store.rs        — Persist sessions to JSON
//! ```

pub mod activity;
pub mod event;
pub mod manager;
pub mod preset;
pub mod session;
pub mod store;
pub mod transport;
pub mod types;

pub use event::AgentChatEvent;
pub use manager::AgentChatManager;
pub use preset::AgentKind;
pub use session::AgentChatSession;
pub use types::{AgentChatStatus, AgentProvider, AgentSessionId, AgentState, ChatMessage};
