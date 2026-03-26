//! Global keybinding registration.

use atrium_platform::{primary_keystroke, primary_shift_keystroke};
use gpui::{App, KeyBinding};

use super::actions::*;

/// Install all global key bindings.
///
/// Uses the platform-aware modifier (cmd on macOS, ctrl on Windows/Linux).
pub fn install(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new(&primary_keystroke("n"), NewWindow, None),
        KeyBinding::new(&primary_keystroke("q"), RequestQuit, None),
        KeyBinding::new(&primary_keystroke("t"), SpawnTerminal, None),
        KeyBinding::new(&primary_keystroke("w"), CloseActiveTerminal, None),
        KeyBinding::new(&primary_keystroke("k"), OpenCommandPalette, None),
        KeyBinding::new(&primary_shift_keystroke("o"), OpenAddRepository, None),
        KeyBinding::new(&primary_shift_keystroke("n"), OpenCreateWorktree, None),
        KeyBinding::new(&primary_shift_keystroke("r"), RefreshWorktrees, None),
        KeyBinding::new(&primary_keystroke("\\"), ToggleSidebar, None),
        KeyBinding::new(&primary_keystroke("["), NavigateBack, None),
        KeyBinding::new(&primary_keystroke("]"), NavigateForward, None),
        KeyBinding::new(&primary_shift_keystroke("l"), ViewLogs, None),
        KeyBinding::new(&primary_keystroke(","), OpenSettings, None),
    ]);
}
