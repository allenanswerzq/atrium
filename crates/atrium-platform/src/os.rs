//! OS and architecture detection.

use strum::{Display, IntoStaticStr};

/// Detected operating system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Os {
    Windows,
    #[strum(serialize = "macos")]
    MacOs,
    Linux,
}

impl Os {
    /// Detect the current OS at compile time.
    pub const fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else {
            Self::Linux
        }
    }

    /// Returns the conventional line ending for this OS.
    pub const fn line_ending(self) -> &'static str {
        match self {
            Self::Windows => "\r\n",
            _ => "\n",
        }
    }
}

/// Detected CPU architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Arch {
    X86_64,
    Aarch64,
    Other,
}

impl Arch {
    /// Detect the current architecture at compile time.
    pub const fn current() -> Self {
        if cfg!(target_arch = "x86_64") {
            Self::X86_64
        } else if cfg!(target_arch = "aarch64") {
            Self::Aarch64
        } else {
            Self::Other
        }
    }
}

/// Combined platform descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
}

impl Platform {
    /// Detect the current platform.
    pub const fn current() -> Self {
        Self {
            os: Os::current(),
            arch: Arch::current(),
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.os, self.arch)
    }
}
