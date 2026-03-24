//! Terminal backend — PTY spawning and process management.
//!
//! This will integrate with `atrium-platform::Shell` for shell detection
//! and eventually support local PTY, SSH, and daemon backends.

use atrium_platform::Shell;

/// Terminal backend abstraction.
pub struct TerminalBackend {
    shell: Shell,
}

impl TerminalBackend {
    /// Create a backend using the detected system shell.
    pub fn detect() -> Option<Self> {
        Shell::detect().map(|shell| Self { shell })
    }

    /// The shell this backend will use.
    pub fn shell(&self) -> &Shell {
        &self.shell
    }
}
