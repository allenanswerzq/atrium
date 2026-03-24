//! Text input widget.

use gpui::{div, prelude::*, px, rgb, Div};

use atrium_core::theme::ThemePalette;

/// A single-line text input field.
pub struct TextInput;

impl TextInput {
    /// Render a text input with placeholder.
    pub fn render(value: &str, placeholder: &str, palette: &ThemePalette) -> Div {
        let display = if value.is_empty() { placeholder } else { value };
        let color = if value.is_empty() {
            palette.text_muted
        } else {
            palette.text_primary
        };

        div()
            .h(px(28.0))
            .w_full()
            .px(px(8.0))
            .bg(rgb(palette.app_bg))
            .border_1()
            .border_color(rgb(palette.border))
            .rounded(px(4.0))
            .flex()
            .items_center()
            .text_size(px(13.0))
            .text_color(rgb(color))
            .child(display.to_owned())
    }
}
