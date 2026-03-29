//! Fuzzy-searchable command palette.

use gpui::{Div, div, prelude::*, px, rgb};

use atrium_core::theme::ThemePalette;

/// Command palette component.
pub struct CommandPalette;

impl CommandPalette {
    // ┌─── modal backdrop ────────────────────────────┐
    // │  ┌──────────────────────────────┐             │
    // │  │ [Type a command...        ]  │  ← input   │
    // │  ├──────────────────────────────┤             │
    // │  │  New Terminal                │             │
    // │  │  Open Settings               │  ← items   │
    // │  │  Toggle Sidebar              │  (filtered) │
    // │  └──────────────────────────────┘             │
    // └───────────────────────────────────────────────┘
    /// Render the command palette overlay.
    pub fn render(palette: &ThemePalette, query: &str, items: &[&str]) -> Div {
        let mut list = div().flex().flex_col().max_h(px(300.0));

        for &item in items {
            let matches_query =
                query.is_empty() || item.to_lowercase().contains(&query.to_lowercase());
            if matches_query {
                list = list.child(
                    div()
                        .px(px(8.0))
                        .py(px(4.0))
                        .text_size(px(13.0))
                        .text_color(rgb(palette.text_primary))
                        .hover(|s| s.bg(rgb(palette.border)))
                        .rounded(px(4.0))
                        .cursor_pointer()
                        .child(item.to_owned()),
                );
            }
        }

        super::modal::Modal::render(
            palette,
            460.0,
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(super::input::TextInput::render(
                    query,
                    "Type a command...",
                    palette,
                ))
                .child(list),
        )
    }
}
