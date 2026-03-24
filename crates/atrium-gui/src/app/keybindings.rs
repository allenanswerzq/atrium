//! Global keybinding registration.

use gpui::{App, KeyBinding};

use super::actions::*;

/// Install all global key bindings.
pub fn install(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-n", NewWindow, None),
        KeyBinding::new("cmd-q", RequestQuit, None),
        KeyBinding::new("cmd-t", SpawnTerminal, None),
        KeyBinding::new("cmd-w", CloseActiveTerminal, None),
        KeyBinding::new("cmd-k", OpenCommandPalette, None),
        KeyBinding::new("cmd-shift-o", OpenAddRepository, None),
        KeyBinding::new("cmd-shift-n", OpenCreateWorktree, None),
        KeyBinding::new("cmd-shift-r", RefreshWorktrees, None),
        KeyBinding::new("cmd-\\", ToggleSidebar, None),
        KeyBinding::new("cmd-[", NavigateBack, None),
        KeyBinding::new("cmd-]", NavigateForward, None),
        KeyBinding::new("cmd-shift-l", ViewLogs, None),
        KeyBinding::new("cmd-,", OpenSettings, None),
    ]);
}
