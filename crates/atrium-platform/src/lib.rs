//! # atrium-platform
//!
//! Platform-specific abstractions and detection for Atrium.
//!
//! Provides unified access to OS-dependent functionality such as:
//! - OS and architecture detection
//! - Platform-specific paths (home, config, data, temp)
//! - Shell detection and command wrapping
//! - Line ending conventions

mod os;
mod paths;
mod shell;

pub use os::{Arch, Os, Platform};
pub use paths::PlatformPaths;
pub use shell::Shell;
