//! Error status definitions for retry behavior.

use std::fmt;

/// Indicates the retry behavior for an error.
///
/// Helps callers decide whether to retry an operation without
/// needing to understand the underlying error details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ErrorStatus {
    /// The error is permanent and will not resolve without external changes.
    #[default]
    Permanent,

    /// The error is temporary and may resolve on retry.
    Temporary,

    /// The error was temporary but persists after multiple retries.
    Persistent,
}

impl ErrorStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Permanent => "permanent",
            Self::Temporary => "temporary",
            Self::Persistent => "persistent",
        }
    }

    #[must_use]
    pub fn is_temporary(&self) -> bool {
        matches!(self, Self::Temporary)
    }

    #[must_use]
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Permanent | Self::Persistent)
    }
}

impl fmt::Display for ErrorStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
