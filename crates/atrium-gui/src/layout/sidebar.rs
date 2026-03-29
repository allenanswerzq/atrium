//! Sidebar — left pane with repository groups and worktree rows.

use gpui::{Div, div, prelude::*, px, rgb};

use atrium_core::theme::ThemePalette;

/// Sidebar rendering component.
pub struct Sidebar;

impl Sidebar {
    // ┌────────────┐
    // │REPOSITORIES│
    // │            │
    // │ ▸ repo-a   │
    // │ ▸ repo-b   │
    // │ ▸ repo-c   │
    // │            │
    // │ (or empty  │
    // │  message)  │
    // └────────────┘
    /// Render the sidebar.
    pub fn render(palette: &ThemePalette, width: f32, repos: &[std::path::PathBuf]) -> Div {
        let mut sidebar = div()
            .w(px(width))
            .h_full()
            .bg(rgb(palette.chrome_bg))
            .border_r_1()
            .border_color(rgb(palette.border))
            .flex()
            .flex_col();

        // Header
        sidebar = sidebar.child(
            div()
                .px(px(12.0))
                .py(px(8.0))
                .text_size(px(11.0))
                .text_color(rgb(palette.text_muted))
                .child("REPOSITORIES"),
        );

        // Repo list
        if repos.is_empty() {
            sidebar = sidebar.child(
                div()
                    .px(px(12.0))
                    .py(px(16.0))
                    .text_size(px(12.0))
                    .text_color(rgb(palette.text_muted))
                    .child("No repositories added."),
            );
        } else {
            for repo in repos {
                let label = repo
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_owned();
                sidebar = sidebar.child(
                    div()
                        .px(px(12.0))
                        .py(px(6.0))
                        .text_size(px(13.0))
                        .text_color(rgb(palette.text_primary))
                        .hover(|s| s.bg(rgb(palette.border)))
                        .rounded(px(4.0))
                        .mx(px(4.0))
                        .child(label),
                );
            }
        }

        sidebar
    }
}
