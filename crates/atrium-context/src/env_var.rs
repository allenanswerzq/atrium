//! Environment variable helpers — single source of truth.
//!
//! Ported from pilot-context/env_var.rs.

use std::env;

// ── Atrium-specific env var names ────────────────────────────────────

/// Daemon base URL override.
pub const DAEMON_URL: &str = "ATRIUM_DAEMON_URL";

/// Daemon auth token.
pub const DAEMON_AUTH_TOKEN: &str = "ATRIUM_DAEMON_AUTH_TOKEN";

/// Override the config directory.
pub const CONFIG_DIR: &str = "ATRIUM_CONFIG_DIR";

/// Override the data directory.
pub const DATA_DIR: &str = "ATRIUM_DATA_DIR";

// ── Helpers ──────────────────────────────────────────────────────────

/// Get the value of an environment variable, returning `None` if not set.
pub fn get(name: &str) -> Option<String> {
    env::var(name).ok()
}

/// Get the value of an environment variable, returning the default if not set.
pub fn get_or(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

/// Check if an environment variable is set (non-empty).
pub fn is_set(name: &str) -> bool {
    env::var(name).is_ok()
}
