//! # atrium-agent-sdk
//!
//! Agent chat sessions and transport backends.
//!
//! ## Modules
//!
//! - `types`     — IDs, enums, messages, events, persistence
//! - `kind`      — AgentKind enum (supported agents)
//! - `discovery` — Discover available agents on PATH and models on providers
//! - `transport` — Transport trait + ACP / Terminal / OpenAI / Anthropic / Responses implementations
//! - `session`   — AgentChatSession + AgentChatManager

pub mod discovery;
pub mod kind;
pub mod session;
pub mod transport;
pub mod types;

pub use atrium_error::{Error, ErrorKind, Result};
pub use discovery::{
    DiscoveredAgent, DiscoveredModel, discover_agents, discover_models, is_installed,
};
pub use kind::AgentKind;
pub use session::{AgentChatManager, AgentChatSession};
pub use transport::{EventReceiver, EventSender, Transport, TransportConfig};
pub use types::{AgentChatEvent, AgentChatStatus, AgentSessionId, AgentState, ChatMessage};
