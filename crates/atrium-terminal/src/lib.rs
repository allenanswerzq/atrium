//! # atrium-terminal
//!
//! Terminal emulation, PTY management, and session lifecycle.
//! GUI-agnostic — any UI (GPUI, web, TUI) can consume this.
//!
//! ## Modules
//!
//! - `types`    — Styled output types, protocol types, requests, snapshots
//! - `emulator` — TerminalEmulator (vt100-backed ANSI parser)
//! - `pty`      — TerminalPty (portable-pty wrapper)
//! - `session`  — TerminalSession, TerminalRuntime, service, store, keys

pub mod emulator;
pub mod pty;
pub mod session;
pub mod types;

pub use emulator::TerminalEmulator;
pub use pty::TerminalPty;
pub use session::{
    JsonTerminalSessionStore, LocalTerminalService, TerminalRuntime,
    TerminalService, TerminalSession, TerminalSessionStore, terminal_escape_bytes,
};
pub use types::*;
