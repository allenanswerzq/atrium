//! Terminal rendering — GPUI elements from `atrium-terminal` styled output.
//!
//! Pure functions that take terminal state and produce GPUI divs.
//! No mutations, no I/O — just reads styled lines and builds UI.

use gpui::{div, prelude::*, px, rgb, Div};

use atrium_core::theme::ThemePalette;
use atrium_terminal::{TerminalSession, TerminalStyledLine};
use atrium_terminal::styled::DEFAULT_BG;

pub const CELL_WIDTH: f32 = 9.0;
pub const LINE_HEIGHT: f32 = 19.0;
pub const FONT_SIZE: f32 = 15.0;

// ┌──────────────────────────────────────────┐
// │ C:\Users\zhangqiang>                     │  ← styled with colors
// │ $ dir                                    │
// │  Directory of C:\Users\...              │
// │                                          │
// │ █                                        │  ← cursor
// └──────────────────────────────────────────┘

/// Render a terminal session's output area with styled text.
///
/// Uses the emulator's styled lines (colored cells) instead of plain text.
pub fn render_terminal(session: &TerminalSession, palette: &ThemePalette) -> Div {
    let runtime = match session.runtime() {
        Some(rt) => rt,
        None => return render_detached(palette),
    };

    if !runtime.has_output() {
        return render_waiting(palette);
    }

    let styled_lines = runtime.styled_lines();
    let cursor = runtime.cursor();

    let mut container = div()
        .size_full()
        .bg(rgb(palette.terminal_bg))
        .flex()
        .flex_col();

    for (row_idx, line) in styled_lines.iter().enumerate() {
        let line_div = render_styled_line(line, row_idx, &cursor, palette);
        container = container.child(line_div);
    }

    container
}

/// Render a single styled line as a row of colored text runs.
fn render_styled_line(
    line: &TerminalStyledLine,
    row_idx: usize,
    cursor: &atrium_terminal::TerminalCursor,
    palette: &ThemePalette,
) -> Div {
    let mut row = div()
        .h(px(LINE_HEIGHT))
        .w_full()
        .flex()
        .flex_row();

    for run in &line.runs {
        let _is_cursor_in_run = row_idx == cursor.line;
        let fg = if run.fg == DEFAULT_BG { palette.text_primary } else { run.fg };
        let bg = run.bg;

        let mut run_div = div()
            .text_size(px(FONT_SIZE))
            .text_color(rgb(fg));

        // Only paint background if it differs from terminal background
        if bg != palette.terminal_bg && bg != DEFAULT_BG {
            run_div = run_div.bg(rgb(bg));
        }

        run_div = run_div.child(run.text.clone());
        row = row.child(run_div);
    }

    // Highlight the cursor row subtly
    if row_idx == cursor.line {
        row = row.bg(rgb(palette.border));
    }

    row
}

/// Render "waiting for output" state.
fn render_waiting(palette: &ThemePalette) -> Div {
    div()
        .size_full()
        .bg(rgb(palette.terminal_bg))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_size(px(14.0))
                .text_color(rgb(palette.text_muted))
                .child("Waiting for shell output..."),
        )
}

/// Render "detached session" state (no runtime).
fn render_detached(palette: &ThemePalette) -> Div {
    div()
        .size_full()
        .bg(rgb(palette.terminal_bg))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_size(px(14.0))
                .text_color(rgb(palette.text_muted))
                .child("Session detached"),
        )
}
