//! Theme selection overlay.

use gpui::{Div, div, prelude::*, px, rgb};

use atrium_core::theme::{ThemeKind, ThemePalette};

/// Theme picker component.
pub struct ThemePicker;

impl ThemePicker {
    // ┌─── modal backdrop ────────────────────────────┐
    // │  ┌──────────────────────────────┐             │
    // │  │  Select Theme                │             │
    // │  ├──────────────────────────────┤             │
    // │  │  ● One Dark                  │  ← active  │
    // │  │    Ayu Dark                  │             │
    // │  │    Gruvbox Dark              │  ← themes   │
    // │  │    Dracula                   │             │
    // │  └──────────────────────────────┘             │
    // └───────────────────────────────────────────────┘
    /// Render the theme picker as a modal.
    pub fn render(palette: &ThemePalette, current: ThemeKind) -> Div {
        let mut grid = div().flex().flex_col().gap(px(4.0));

        for kind in ThemeKind::ALL {
            let is_active = *kind == current;
            let label = kind.label();
            let indicator = if is_active { "\u{25CF} " } else { "  " };

            grid = grid.child(
                div()
                    .px(px(8.0))
                    .py(px(6.0))
                    .text_size(px(13.0))
                    .text_color(rgb(if is_active {
                        palette.accent
                    } else {
                        palette.text_primary
                    }))
                    .hover(|s| s.bg(rgb(palette.border)))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .child(format!("{indicator}{label}")),
            );
        }

        super::modal::Modal::render(
            palette,
            360.0,
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(
                    div()
                        .text_size(px(14.0))
                        .text_color(rgb(palette.text_primary))
                        .child("Select Theme"),
                )
                .child(grid),
        )
    }
}
