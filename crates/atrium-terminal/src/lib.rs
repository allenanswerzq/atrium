//! # atrium-terminal
//!
//! Terminal emulation, PTY management, and session lifecycle.
//!
//! This crate is **GUI-agnostic** — it provides the terminal backend
//! that any UI (GPUI, web, TUI) can consume.
//!
//! ## Architecture
//!
//! ```text
//!  types.rs    — Protocol types: requests, signals, states, snapshots
//!  styled.rs   — Output styling: TerminalStyledLine, TerminalStyledCell, TerminalCursor, TerminalModes
//!  pty.rs      — PTY handle: spawn shell, read/write (portable-pty)
//!  session.rs  — TerminalSession: PTY + output buffer + reader thread
//!  daemon.rs   — TerminalService trait + LocalTerminalService (manages sessions)
//!  store.rs    — TerminalSessionStore trait + JSON file impl
//!  keys.rs     — Keystroke → terminal escape sequence mapping
//! ```
//!
//! ## Dependency chain
//!
//! ```text
//!  types ← styled ← pty ← session ← daemon
//!                                  ← store
//! ```

pub mod daemon;
pub mod keys;
pub mod pty;
pub mod session;
pub mod store;
pub mod styled;
pub mod types;

// Re-export the most commonly used types at crate root.
pub use daemon::{LocalTerminalService, TerminalService};
pub use keys::terminal_escape_bytes;
pub use pty::TerminalPty;
pub use session::TerminalSession;
pub use store::{JsonTerminalSessionStore, TerminalSessionStore};
pub use styled::{TerminalCursor, TerminalModes, TerminalStyledCell, TerminalStyledLine, TerminalStyledRun};
pub use types::{
    TerminalCreateRequest, TerminalKillRequest, TerminalResizeRequest,
    TerminalSessionRecord, TerminalSignal, TerminalSnapshot, TerminalState,
    TerminalWriteRequest,
};
