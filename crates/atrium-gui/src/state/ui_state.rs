//! Persisted UI state — window geometry, sidebar selection, pane widths.

use serde::{Deserialize, Serialize};

/// Persisted UI layout state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiState {
    /// Window geometry (position + size).
    pub window: Option<WindowGeometry>,
    /// Active sidebar selection index.
    pub sidebar_selection: Option<usize>,
    /// Left pane width in pixels.
    pub left_pane_width: Option<f32>,
    /// Right pane width in pixels.
    pub right_pane_width: Option<f32>,
    /// Whether the sidebar is collapsed.
    pub sidebar_collapsed: bool,
    /// Active theme name.
    pub theme: Option<String>,
}

/// Stored window position and size.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
