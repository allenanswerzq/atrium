//! Shell detection and command wrapping.

use std::path::PathBuf;

use crate::Os;

/// Known shell types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shell {
    Bash(PathBuf),
    Zsh(PathBuf),
    Fish(PathBuf),
    PowerShell(PathBuf),
    Cmd(PathBuf),
}

impl Shell {
    /// Detect the user's default shell on the current platform.
    pub fn detect() -> Option<Self> {
        match Os::current() {
            Os::Windows => Some(Self::PowerShell(PathBuf::from("powershell.exe"))),
            Os::MacOs | Os::Linux => detect_unix_shell(),
        }
    }

    /// Returns the shell executable path.
    pub fn path(&self) -> &PathBuf {
        match self {
            Self::Bash(p) | Self::Zsh(p) | Self::Fish(p) | Self::PowerShell(p) | Self::Cmd(p) => p,
        }
    }

    /// Returns the flag used to pass an inline command string.
    pub fn command_flag(&self) -> &str {
        match self {
            Self::Bash(_) | Self::Zsh(_) | Self::Fish(_) => "-c",
            Self::PowerShell(_) => "-Command",
            Self::Cmd(_) => "/C",
        }
    }
}

fn detect_unix_shell() -> Option<Shell> {
    let shell_env = std::env::var("SHELL").ok()?;
    let path = PathBuf::from(&shell_env);
    let name = path.file_name()?.to_str()?;
    match name {
        "bash" => Some(Shell::Bash(path)),
        "zsh" => Some(Shell::Zsh(path)),
        "fish" => Some(Shell::Fish(path)),
        _ => Some(Shell::Bash(path)),
    }
}
