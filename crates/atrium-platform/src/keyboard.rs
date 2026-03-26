//! Platform-aware keyboard modifier helpers.
//!
//! On macOS the primary modifier is `cmd`, on Windows/Linux it's `ctrl`.
//! This module provides helpers to build keystroke strings that work
//! correctly on the current platform.

use crate::Os;

/// The primary modifier key name for the current platform.
///
/// - macOS: `"cmd"`
/// - Windows/Linux: `"ctrl"`
pub const fn primary_modifier() -> &'static str {
    match Os::current() {
        Os::MacOs => "cmd",
        Os::Windows | Os::Linux => "ctrl",
    }
}

/// Build a keystroke string with the primary modifier.
///
/// # Examples
///
/// ```ignore
/// // On macOS:  "cmd-t"
/// // On Linux:  "ctrl-t"
/// let ks = primary_keystroke("t");
/// ```
pub fn primary_keystroke(key: &str) -> String {
    format!("{}-{}", primary_modifier(), key)
}

/// Build a keystroke string with primary modifier + shift.
///
/// # Examples
///
/// ```ignore
/// // On macOS:  "cmd-shift-n"
/// // On Linux:  "ctrl-shift-n"
/// let ks = primary_shift_keystroke("n");
/// ```
pub fn primary_shift_keystroke(key: &str) -> String {
    format!("{}-shift-{}", primary_modifier(), key)
}
