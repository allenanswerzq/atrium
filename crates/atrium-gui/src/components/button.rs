//! Reusable action button widget.

use gpui::{Div, div, prelude::*, px, rgb};

use atrium_core::theme::ThemePalette;

/// An action button with a text label.
pub struct ActionButton;

impl ActionButton {
    // ┌──────────────┐      ┌──────────────┐
    // │  Save  (pri) │      │  Cancel (gh) │
    // └──────────────┘      └──────────────┘
    //  accent bg, white      transparent bg,
    //  text, rounded         muted text, hover

    /// Render a primary action button.
    pub fn primary(label: &str, palette: &ThemePalette) -> Div {
        div()
            .px(px(12.0))
            .py(px(4.0))
            .bg(rgb(palette.accent))
            .text_color(rgb(0xFFFFFF))
            .text_size(px(13.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .hover(|s| s.opacity(0.85))
            .child(label.to_owned())
    }

    /// Render a ghost/muted button.
    pub fn ghost(label: &str, palette: &ThemePalette) -> Div {
        div()
            .px(px(12.0))
            .py(px(4.0))
            .text_color(rgb(palette.text_muted))
            .text_size(px(13.0))
            .rounded(px(4.0))
            .cursor_pointer()
            .hover(|s| s.bg(rgb(palette.border)))
            .child(label.to_owned())
    }
}
