//! Terminal emulator — wraps `vt100` to parse ANSI into styled cells.
//!
//! ```text
//!  PTY bytes ──→ TerminalEmulator.process(bytes)
//!                     │
//!                     ├─→ styled_lines()  → Vec<TerminalStyledLine>
//!                     ├─→ cursor()        → TerminalCursor
//!                     ├─→ modes()         → TerminalModes
//!                     └─→ plain_output()  → String
//! ```

use crate::styled::{
    TerminalCursor, TerminalModes, TerminalStyledCell, TerminalStyledLine, TerminalStyledRun,
    DEFAULT_BG, DEFAULT_FG,
};

/// Terminal emulator backed by `vt100`.
///
/// Maintains a grid of styled cells. Feed it raw PTY bytes via `process()`,
/// then read the current state via `styled_lines()`, `cursor()`, etc.
pub struct TerminalEmulator {
    parser: vt100::Parser,
    /// Lines that scrolled off the top of the visible grid.
    scrollback: Vec<TerminalStyledLine>,
    /// Max scrollback lines to keep.
    max_scrollback: usize,
}

impl TerminalEmulator {
    /// Create an emulator with the given grid size.
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            parser: vt100::Parser::new(rows, cols, 0),
            scrollback: Vec::new(),
            max_scrollback: 10_000,
        }
    }

    /// Feed raw PTY bytes into the emulator.
    pub fn process(&mut self, bytes: &[u8]) {
        // Capture lines before processing to detect scrolloff
        let old_scrollback = self.parser.screen().scrollback();

        self.parser.process(bytes);

        // Check if new lines scrolled off — vt100 doesn't keep scrollback,
        // so we'd need to detect and save them. For now, scrollback is
        // populated from the visible grid when requested.
        let _ = old_scrollback;
    }

    /// Resize the terminal grid.
    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.parser.set_size(rows, cols);
    }

    /// Get the current cursor position.
    pub fn cursor(&self) -> TerminalCursor {
        let (row, col) = self.parser.screen().cursor_position();
        TerminalCursor {
            line: row as usize,
            column: col as usize,
        }
    }

    /// Get the current terminal modes.
    pub fn modes(&self) -> TerminalModes {
        let screen = self.parser.screen();
        TerminalModes {
            app_cursor: screen.application_cursor(),
            alt_screen: screen.alternate_screen(),
        }
    }

    /// Get the visible grid as styled lines.
    pub fn styled_lines(&self) -> Vec<TerminalStyledLine> {
        let screen = self.parser.screen();
        let (rows, cols) = screen.size();
        let mut lines = Vec::with_capacity(rows as usize);

        for row in 0..rows {
            let mut cells = Vec::with_capacity(cols as usize);
            for col in 0..cols {
                let cell = screen.cell(row, col);
                let (fg, bg) = match cell {
                    Some(c) => (color_to_u32(c.fgcolor()), color_to_u32(c.bgcolor())),
                    None => (DEFAULT_FG, DEFAULT_BG),
                };
                let text = cell
                    .map(|c| c.contents())
                    .unwrap_or_default();
                // Skip trailing empty cells
                if text.is_empty() && col > 0 {
                    continue;
                }
                cells.push(TerminalStyledCell {
                    column: col as usize,
                    text: if text.is_empty() { " ".to_owned() } else { text },
                    fg,
                    bg,
                });
            }

            // Build runs by merging adjacent same-styled cells
            let runs = runs_from_cells(&cells);
            lines.push(TerminalStyledLine { cells, runs });
        }

        lines
    }

    /// Get the visible grid as plain text.
    pub fn plain_output(&self) -> String {
        let screen = self.parser.screen();
        let (rows, _cols) = screen.size();
        let mut output = String::new();
        for row in 0..rows {
            if row > 0 {
                output.push('\n');
            }
            output.push_str(&screen.contents_between(row, 0, row + 1, 0));
        }
        output
    }

    /// Grid dimensions.
    pub fn size(&self) -> (u16, u16) {
        self.parser.screen().size()
    }
}

/// Merge adjacent cells with the same fg/bg into runs.
fn runs_from_cells(cells: &[TerminalStyledCell]) -> Vec<TerminalStyledRun> {
    let mut runs = Vec::new();
    let mut current_text = String::new();
    let mut current_fg = DEFAULT_FG;
    let mut current_bg = DEFAULT_BG;

    for cell in cells {
        if cell.fg == current_fg && cell.bg == current_bg {
            current_text.push_str(&cell.text);
        } else {
            if !current_text.is_empty() {
                runs.push(TerminalStyledRun {
                    text: std::mem::take(&mut current_text),
                    fg: current_fg,
                    bg: current_bg,
                });
            }
            current_fg = cell.fg;
            current_bg = cell.bg;
            current_text.push_str(&cell.text);
        }
    }

    if !current_text.is_empty() {
        runs.push(TerminalStyledRun {
            text: current_text,
            fg: current_fg,
            bg: current_bg,
        });
    }

    runs
}

/// Convert a vt100 color to 0xRRGGBB u32.
fn color_to_u32(color: vt100::Color) -> u32 {
    match color {
        vt100::Color::Default => DEFAULT_FG,
        vt100::Color::Idx(idx) => ansi_index_to_rgb(idx),
        vt100::Color::Rgb(r, g, b) => ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
    }
}

/// Map ANSI 256-color index to RGB.
fn ansi_index_to_rgb(idx: u8) -> u32 {
    // Standard 16 ANSI colors
    const ANSI_16: [u32; 16] = [
        0x000000, 0xcd0000, 0x00cd00, 0xcdcd00, 0x0000ee, 0xcd00cd, 0x00cdcd, 0xe5e5e5,
        0x7f7f7f, 0xff0000, 0x00ff00, 0xffff00, 0x5c5cff, 0xff00ff, 0x00ffff, 0xffffff,
    ];

    match idx {
        0..=15 => ANSI_16[idx as usize],
        // 216-color cube: indices 16-231
        16..=231 => {
            let idx = idx - 16;
            let r = (idx / 36) % 6;
            let g = (idx / 6) % 6;
            let b = idx % 6;
            let r = if r == 0 { 0 } else { 55 + 40 * r as u32 };
            let g = if g == 0 { 0 } else { 55 + 40 * g as u32 };
            let b = if b == 0 { 0 } else { 55 + 40 * b as u32 };
            (r << 16) | (g << 8) | b
        }
        // Grayscale: indices 232-255
        232..=255 => {
            let v = 8 + 10 * (idx - 232) as u32;
            (v << 16) | (v << 8) | v
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn process_plain_text() {
        let mut emu = TerminalEmulator::new(24, 80);
        emu.process(b"hello world");

        let lines = emu.styled_lines();
        assert!(!lines.is_empty());
        // First line should contain "hello world"
        let first_run = &lines[0].runs[0];
        assert!(first_run.text.starts_with("hello world"));
    }

    #[test]
    fn cursor_position() {
        let mut emu = TerminalEmulator::new(24, 80);
        emu.process(b"abc");
        let cursor = emu.cursor();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.column, 3);
    }

    #[test]
    fn ansi_color() {
        let mut emu = TerminalEmulator::new(24, 80);
        // ESC[31m = red foreground, then "red", then ESC[0m = reset
        emu.process(b"\x1b[31mred\x1b[0m");
        let lines = emu.styled_lines();
        // Should have a run with red fg
        let red_run = lines[0].runs.iter().find(|r| r.text.starts_with("red"));
        assert!(red_run.is_some());
        let red_run = red_run.unwrap();
        assert_eq!(red_run.fg, 0xcd0000); // ANSI red
    }

    #[test]
    fn resize() {
        let mut emu = TerminalEmulator::new(24, 80);
        emu.resize(40, 120);
        assert_eq!(emu.size(), (40, 120));
    }

    #[test]
    fn alt_screen_mode() {
        let mut emu = TerminalEmulator::new(24, 80);
        assert!(!emu.modes().alt_screen);
        // Enable alternate screen buffer
        emu.process(b"\x1b[?1049h");
        assert!(emu.modes().alt_screen);
        // Disable
        emu.process(b"\x1b[?1049l");
        assert!(!emu.modes().alt_screen);
    }

    #[test]
    fn color_conversion() {
        assert_eq!(ansi_index_to_rgb(0), 0x000000);  // black
        assert_eq!(ansi_index_to_rgb(1), 0xcd0000);  // red
        assert_eq!(ansi_index_to_rgb(15), 0xffffff); // white
        // 256-color index 232 = dark gray
        assert_eq!(ansi_index_to_rgb(232), 0x080808);
    }
}
