//! # atrium-core
//!
//! Core domain types, git operations, and architecture graph primitives for Atrium.
//!
//! ## Module Organization
//!
//! - [`id`] — Type-safe identifier newtypes (`SessionId`, `WorkspaceId`)
//! - [`worktree`] — Git worktree CRUD operations via gix/git2
//! - [`changes`] — File change detection with line-level diff stats
//! - [`config`] — Repository-local configuration (`atrium.toml`)
//! - [`process`] — Managed process types and lifecycle
//! - [`task`] — Scheduled task types and execution history
//! - [`agent`] — AI agent state and session detection
//! - [`theme`] — Built-in color theme definitions
//! - [`terminal`] — Terminal daemon protocol types and traits

pub mod agent;
pub mod changes;
pub mod config;
pub mod id;
pub mod process;
pub mod task;
pub mod terminal;
pub mod theme;
pub mod worktree;

pub use id::{SessionId, WorkspaceId};
