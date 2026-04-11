//! Graph view — renders a [`ViewSpec`] as an interactive architecture diagram.
//!
//! The renderer is intentionally dumb: it draws nodes at the exact positions,
//! edges along the exact waypoints, and groups as background regions — all as
//! specified in the [`ViewSpec`]. No layout logic, no filtering, no intelligence.

mod fake_data;
mod render;

pub use render::GraphViewPanel;
