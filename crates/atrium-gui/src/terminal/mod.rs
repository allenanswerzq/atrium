//! Terminal module — thin GUI adapter over `atrium-terminal`.
//!
//! All terminal logic (PTY, emulator, sessions, keys) lives in the
//! `atrium-terminal` crate. This module provides:
//! - GPUI rendering of styled terminal output
//! - GPUI keystroke → terminal bytes conversion

pub mod rendering;

// Re-export atrium-terminal types used by the window and rendering code.
pub use atrium_terminal::{
    TerminalCursor, TerminalModes, TerminalRuntime, TerminalSession, TerminalSnapshot,
    TerminalState, TerminalStyledCell, TerminalStyledLine, TerminalStyledRun,
    terminal_escape_bytes,
};
