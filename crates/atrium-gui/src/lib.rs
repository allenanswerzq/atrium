//! # Atrium GUI
//!
//! GPUI desktop application — the main user-facing surface.
//!
//! Architecture overview:
//! - `app/`        — Application shell: window, bootstrap, actions, keybindings
//! - `state/`      — Persistent stores (config, UI state, repositories)
//! - `terminal/`   — Terminal sessions, rendering, input, key mapping
//! - `workspace/`  — Repository and worktree management
//! - `git/`        — Change detection, diff engine, git actions
//! - `github/`     — GitHub integration (auth, PRs, service)
//! - `agent/`      — AI agent presets, activity monitoring, chat
//! - `daemon/`     — Daemon connectivity and remote terminal runtime
//! - `layout/`     — UI layout components (sidebar, center, top bar, etc.)
//! - `components/` — Reusable UI primitives (modals, buttons, inputs)
//! - `theme/`      — Theme management and GPUI color conversions
//! - `background/` — Background pollers and watchers

pub mod app;
pub mod background;
pub mod components;
pub mod layout;
pub mod state;
pub mod terminal;
pub mod theme;
pub mod workspace;
