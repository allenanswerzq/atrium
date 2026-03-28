//! # atrium-terminal
//!
//! Terminal emulation, PTY management, and session lifecycle.
//!
//! This crate is **GUI-agnostic** — it provides the terminal backend
//! that any UI (GPUI, web, TUI) can consume.
//!

pub mod daemon;
pub mod emulator;
pub mod keys;
pub mod pty;
pub mod session;
pub mod store;
pub mod styled;
pub mod types;

// Re-export the most commonly used types at crate root.
pub use daemon::{LocalTerminalService, TerminalService};
pub use emulator::TerminalEmulator;
pub use keys::terminal_escape_bytes;
pub use pty::TerminalPty;
pub use session::{TerminalRuntime, TerminalSession};
pub use store::{JsonTerminalSessionStore, TerminalSessionStore};
pub use styled::{TerminalCursor, TerminalModes, TerminalStyledCell, TerminalStyledLine, TerminalStyledRun};
pub use types::{
    TerminalCreateRequest, TerminalKillRequest, TerminalResizeRequest,
    TerminalSessionRecord, TerminalSignal, TerminalSnapshot, TerminalState,
    TerminalWriteRequest,
};
