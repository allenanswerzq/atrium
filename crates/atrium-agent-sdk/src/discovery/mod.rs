//! Discovery — find available agents and models on the local machine.
//!
//! - [`agents`] — scan PATH for known agent CLIs, probe their version
//! - [`models`] — probe `/v1/models` on OpenAI-compatible endpoints

pub mod agents;
pub mod models;

pub use agents::{DiscoveredAgent, discover_agent, discover_agents, is_installed};
pub use models::{DiscoveredModel, discover_models};
