//! Top bar — titlebar with navigation, repo/branch label, action icons.

use gpui::{div, prelude::*, px, rgb, Div};

use atrium_core::theme::ThemePalette;

const TITLEBAR_HEIGHT: f32 = 34.0;

/// Top bar rendering component.
pub struct TopBar;

impl TopBar {
    // ┌──────────────────────────────────────────────────────────────┐
    // │ ◀ ▶          Atrium                                    ⚙   │
    // │ [nav]     [center title]                          [actions] │
    // └──────────────────────────────────────────────────────────────┘
    /// Render the top bar.
    pub fn render(palette: &ThemePalette, _can_go_back: bool, _can_go_forward: bool) -> Div {
        div()
            .h(px(TITLEBAR_HEIGHT))
            .w_full()
            .bg(rgb(palette.chrome_bg))
            .border_b_1()
            .border_color(rgb(palette.border))
            .flex()
            .items_center()
            .justify_between()
            .px(px(12.0))
            .child(
                // Left section: nav buttons
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_color(rgb(palette.text_muted))
                            .text_size(px(13.0))
                            .child("\u{25C0}")
                    )
                    .child(
                        div()
                            .text_color(rgb(palette.text_muted))
                            .text_size(px(13.0))
                            .child("\u{25B6}")
                    ),
            )
            .child(
                // Center: app title
                div()
                    .text_color(rgb(palette.text_primary))
                    .text_size(px(13.0))
                    .child("Atrium"),
            )
            .child(
                // Right section: placeholder for action icons
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_color(rgb(palette.text_muted))
                            .text_size(px(12.0))
                            .child("\u{2699}")
                    ),
            )
    }
}
