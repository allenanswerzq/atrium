//! Modal backdrop and common modal patterns.

use gpui::{Div, div, prelude::*, px, rgb, rgba};

use atrium_core::theme::ThemePalette;

/// A modal overlay container.
pub struct Modal;

impl Modal {
    // ┌─── full screen backdrop (semi-transparent) ───┐
    // │                                               │
    // │   ┌─────────────────────────────┐             │
    // │   │  modal content (rounded,    │             │
    // │   │  bordered, chrome_bg)       │             │
    // │   └─────────────────────────────┘             │
    // │                                               │
    // └───────────────────────────────────────────────┘
    /// Render a centered modal with a backdrop.
    pub fn render(palette: &ThemePalette, width: f32, content: Div) -> Div {
        div()
            .size_full()
            .absolute()
            .top_0()
            .left_0()
            .bg(rgba(0x00000088))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(width))
                    .bg(rgb(palette.chrome_bg))
                    .border_1()
                    .border_color(rgb(palette.border))
                    .rounded(px(8.0))
                    .p(px(16.0))
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
                    .child(content),
            )
    }
}
