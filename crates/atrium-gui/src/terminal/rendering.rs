//! Terminal rendering — GPUI elements from `atrium-terminal` styled output.
//!
//! Pure functions that take terminal state and produce GPUI divs.
//! No mutations, no I/O — just reads styled lines and builds UI.

use gpui::{div, prelude::*, px, rgb, Div};

use atrium_core::theme::ThemePalette;
use atrium_terminal::{TerminalSession, TerminalStyledLine};
use atrium_terminal::styled::DEFAULT_FG;

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
        .p(px(4.0))
        .flex()
        .flex_col();

    for (row_idx, line) in styled_lines.iter().enumerate() {
        let is_cursor_row = row_idx == cursor.line;
        let line_div = render_styled_line(line, is_cursor_row, palette);
        container = container.child(line_div);
    }

    container
}

/// Render a single styled line.
///
/// Each line is a flex-row with colored text spans.
/// Backgrounds are ignored — text renders on the terminal background.
fn render_styled_line(
    line: &TerminalStyledLine,
    is_cursor_row: bool,
    palette: &ThemePalette,
) -> Div {
    let mut row = div()
        .h(px(LINE_HEIGHT))
        .w_full()
        .flex()
        .flex_row();

    if is_cursor_row {
        row = row.bg(rgb(palette.border));
    }

    for run in &line.runs {
        let fg = remap_fg(run.fg, palette);
        row = row.child(
            div()
                .text_size(px(FONT_SIZE))
                .text_color(rgb(fg))
                .child(run.text.clone()),
        );
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

// ── Color remapping ─────────────────────────────────────────────────

/// Map terminal foreground color to theme-appropriate color.
///
/// The emulator outputs raw ANSI colors (0xRRGGBB). We remap the
/// "default" terminal colors to the theme palette so text is readable.
fn remap_fg(color: u32, palette: &ThemePalette) -> u32 {
    match color {
        // Default fg → use theme terminal fg
        c if c == DEFAULT_FG => palette.terminal_fg,
        // Black text on dark theme is invisible → use muted text
        0x000000 => palette.text_muted,
        // Everything else: keep the terminal's color
        c => c,
    }
}
