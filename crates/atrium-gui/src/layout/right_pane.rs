//! Right pane — changes list, file tree, notes.

use gpui::{div, prelude::*, px, rgb, Div};

use atrium_core::theme::ThemePalette;

/// Right pane rendering component.
pub struct RightPane;

impl RightPane {
    // ┌───────────────┐
    // │Changes │ Files │  ← tab bar
    // ├───────────────┤
    // │               │
    // │ changed files │  ← content
    // │ or file tree  │
    // │               │
    // └───────────────┘
    /// Render the right pane.
    pub fn render(palette: &ThemePalette, width: f32) -> Div {
        div()
            .w(px(width))
            .h_full()
            .bg(rgb(palette.chrome_bg))
            .border_l_1()
            .border_color(rgb(palette.border))
            .flex()
            .flex_col()
            .child(
                // Tab bar
                div()
                    .h(px(32.0))
                    .w_full()
                    .border_b_1()
                    .border_color(rgb(palette.border))
                    .flex()
                    .items_center()
                    .px(px(8.0))
                    .gap(px(12.0))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(palette.text_primary))
                            .child("Changes"),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(palette.text_muted))
                            .child("Files"),
                    ),
            )
            .child(
                // Content
                div()
                    .flex_1()
                    .w_full()
                    .p(px(12.0))
                    .text_size(px(12.0))
                    .text_color(rgb(palette.text_muted))
                    .child("No changes detected."),
            )
    }
}
