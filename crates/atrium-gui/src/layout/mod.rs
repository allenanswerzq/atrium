//! UI layout components — sidebar, center panel, top bar, status bar, right pane.

mod center_panel;
mod right_pane;
mod sidebar;
mod status_bar;
mod top_bar;

pub use center_panel::CenterPanel;
pub use right_pane::RightPane;
pub use sidebar::Sidebar;
pub use status_bar::StatusBar;
pub use top_bar::TopBar;

/// Layout-related state owned by the window.
#[derive(Debug, Clone)]
pub struct LayoutState {
    left_pane_width: f32,
    right_pane_width: f32,
    sidebar_visible: bool,
}

impl LayoutState {
    /// Sidebar width in pixels (0 if hidden).
    pub fn sidebar_width(&self) -> f32 {
        if self.sidebar_visible {
            self.left_pane_width
        } else {
            0.0
        }
    }

    /// Right pane width in pixels.
    pub fn right_pane_width(&self) -> f32 {
        self.right_pane_width
    }

    /// Toggle sidebar visibility.
    pub fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
    }

    /// Whether the sidebar is visible.
    pub fn sidebar_visible(&self) -> bool {
        self.sidebar_visible
    }
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            left_pane_width: 240.0,
            right_pane_width: 300.0,
            sidebar_visible: true,
        }
    }
}
