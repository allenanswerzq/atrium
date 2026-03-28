//! Terminal output styling types.
//!
//! These represent the visual state of terminal output: styled cells,
//! runs of same-styled text, cursor position, and terminal modes.
//! Used by both the emulator and the renderer.

use serde::{Deserialize, Serialize};

/// A single character cell in the terminal grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyledCell {
    /// Grid column (0-based).
    pub column: usize,
    /// The character(s) at this position.
    pub text: String,
    /// Foreground color as 0xRRGGBB.
    pub fg: u32,
    /// Background color as 0xRRGGBB.
    pub bg: u32,
}

/// A run of consecutive cells with the same style.
///
/// Merging adjacent same-styled cells into runs reduces the number of
/// draw calls in the renderer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyledRun {
    pub text: String,
    pub fg: u32,
    pub bg: u32,
}

/// One line of terminal output, in both cell and run form.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyledLine {
    pub cells: Vec<StyledCell>,
    pub runs: Vec<StyledRun>,
}

/// Cursor position in the terminal grid.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

/// Terminal mode flags that affect rendering and input behavior.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Modes {
    /// Application cursor mode — arrow keys emit SS3 sequences.
    pub app_cursor: bool,
    /// Alternate screen buffer is active (for TUIs like vim).
    pub alt_screen: bool,
}

// ── Default colors ──────────────────────────────────────────────────

pub const DEFAULT_FG: u32 = 0xabb2bf;
pub const DEFAULT_BG: u32 = 0x282c34;
pub const DEFAULT_CURSOR: u32 = 0x74ade8;
