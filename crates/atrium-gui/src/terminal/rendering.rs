//! Terminal canvas rendering — styled text runs, cursor, grid size.

use gpui::{div, prelude::*, px, rgb, Div};

use atrium_core::theme::ThemePalette;

use super::session::TerminalSession;

pub const CELL_WIDTH: f32 = 9.0;
pub const CELL_HEIGHT: f32 = 19.0;
pub const FONT_SIZE: f32 = 15.0;

/// Terminal rendering helpers.
pub struct TerminalRenderer;

impl TerminalRenderer {
    // ┌──────────────────────────────────────┐
    // │ Session: zsh — running               │  ← header (if empty)
    // │ $ ls -la                             │
    // │ drwxr-xr-x  5 user ...              │  ← output lines
    // │ -rw-r--r--  1 user ...              │
    // │                                      │
    // │                          cursor: 0:3 │  ← cursor indicator
    // └──────────────────────────────────────┘
    /// Render a terminal session's output area.
    pub fn render(session: &TerminalSession, palette: &ThemePalette) -> Div {
        let mut container = div()
            .size_full()
            .bg(rgb(palette.terminal_bg))
            .p(px(4.0))
            .flex()
            .flex_col();

        let lines = session.output_lines();
        if lines.is_empty() {
            container = container.child(
                div()
                    .text_size(px(FONT_SIZE))
                    .text_color(rgb(palette.text_muted))
                    .child(format!("Session: {} — {}", session.title(), session.state())),
            );
        } else {
            for line in lines {
                container = container.child(
                    div()
                        .text_size(px(FONT_SIZE))
                        .text_color(rgb(palette.text_primary))
                        .child(line.clone()),
                );
            }
        }

        // Cursor indicator
        let (col, row) = session.cursor();
        container = container.child(
            div()
                .text_size(px(11.0))
                .text_color(rgb(palette.text_muted))
                .mt_auto()
                .child(format!("cursor: {}:{}", row, col)),
        );

        container
    }
}
