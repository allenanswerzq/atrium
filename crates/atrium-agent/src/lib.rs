//! # atrium-agent
//!
//! Agent chat sessions and transport backends.
//!
//! ## Modules
//!
//! - `types`     — IDs, enums, messages, events, persistence
//! - `kind`      — AgentKind enum (supported agents)
//! - `transport` — Transport trait + ACP / Terminal / OpenAI implementations
//! - `session`   — AgentChatSession + AgentChatManager

pub mod kind;
pub mod session;
pub mod transport;
pub mod types;

pub use atrium_error::{Error, ErrorKind, Result};
pub use kind::AgentKind;
pub use session::{AgentChatManager, AgentChatSession};
pub use transport::{Transport, TransportConfig};
pub use types::{AgentChatEvent, AgentChatStatus, AgentSessionId, AgentState, ChatMessage};
