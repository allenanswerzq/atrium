//! Error kind definitions for categorizing errors.

use strum::{Display, EnumString, IntoStaticStr};

/// Categorizes errors into actionable types.
///
/// Users can match on `ErrorKind` to decide how to handle specific error cases.
/// Designed as a small, focused set of error categories that users
/// can actually act upon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
#[non_exhaustive]
pub enum ErrorKind {
    /// An unexpected error occurred.
    #[default]
    Unexpected,

    /// The operation is not supported.
    Unsupported,

    /// Configuration is invalid.
    ConfigInvalid,

    /// The requested resource was not found.
    NotFound,

    /// Permission was denied for the operation.
    PermissionDenied,

    /// The resource already exists.
    AlreadyExists,

    /// The operation timed out.
    Timeout,

    /// Network-related error occurred.
    Network,

    /// I/O error occurred.
    Io,

    /// Data format or parsing error.
    DataInvalid,

    /// Rate limit exceeded.
    RateLimited,

    /// Service is unavailable.
    ServiceUnavailable,

    /// Request was cancelled.
    Cancelled,

    /// The input provided was invalid.
    InvalidInput,

    /// Data integrity check failed.
    DataIntegrity,

    /// Git operation failed.
    Git,
}

impl ErrorKind {
    /// Returns a human-readable description of the error kind.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        (*self).into()
    }
}
