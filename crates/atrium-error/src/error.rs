//! The main Error type with rich context support.

use std::fmt;

use crate::{ErrorKind, ErrorStatus};

/// A rich error type with context, operation tracking, and source chaining.
///
/// Designed to:
/// - Categorize errors with [`ErrorKind`] for programmatic handling
/// - Indicate retry behavior with [`ErrorStatus`]
/// - Track the operation that caused the error
/// - Accumulate context as the error propagates up the call stack
/// - Preserve the original error source
pub struct Error {
    kind: ErrorKind,
    message: String,
    status: ErrorStatus,
    operation: &'static str,
    context: Vec<(&'static str, String)>,
    source: Option<anyhow::Error>,
}

impl Error {
    /// Creates a new error with the specified kind and message.
    #[must_use]
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            status: ErrorStatus::Permanent,
            operation: "",
            context: Vec::new(),
            source: None,
        }
    }

    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn status(&self) -> ErrorStatus {
        self.status
    }

    #[must_use]
    pub fn operation(&self) -> &'static str {
        self.operation
    }

    #[must_use]
    pub fn context(&self) -> &[(&'static str, String)] {
        &self.context
    }

    #[must_use]
    pub fn is_temporary(&self) -> bool {
        self.status.is_temporary()
    }

    #[must_use]
    pub fn is_final(&self) -> bool {
        self.status.is_final()
    }

    // --- Builder methods ---

    /// Sets the operation that caused the error.
    /// If an operation was already set, it is moved to the context as "called".
    #[must_use]
    pub fn with_operation(mut self, operation: &'static str) -> Self {
        if !self.operation.is_empty() {
            self.context.push(("called", self.operation.to_string()));
        }
        self.operation = operation;
        self
    }

    /// Adds a context key-value pair to the error.
    #[must_use]
    pub fn with_context(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.context.push((key, value.into()));
        self
    }

    #[must_use]
    pub fn set_temporary(mut self) -> Self {
        self.status = ErrorStatus::Temporary;
        self
    }

    #[must_use]
    pub fn set_permanent(mut self) -> Self {
        self.status = ErrorStatus::Permanent;
        self
    }

    #[must_use]
    pub fn set_persistent(mut self) -> Self {
        self.status = ErrorStatus::Persistent;
        self
    }

    /// Sets the source error.
    #[must_use]
    pub fn set_source(mut self, source: impl Into<anyhow::Error>) -> Self {
        debug_assert!(self.source.is_none(), "source error has already been set");
        self.source = Some(source.into());
        self
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}) ", self.kind, self.status)?;
        if !self.operation.is_empty() {
            write!(f, "at {} ", self.operation)?;
        }
        write!(f, "=> {}", self.message)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.operation.is_empty() {
            writeln!(f, "{} ({})", self.kind, self.status)?;
        } else {
            writeln!(f, "{} ({}) at {}", self.kind, self.status, self.operation)?;
        }
        writeln!(f)?;
        writeln!(f, "    {}", self.message)?;

        if !self.context.is_empty() {
            writeln!(f)?;
            writeln!(f, "    Context:")?;
            for (key, value) in &self.context {
                writeln!(f, "        {key}: {value}")?;
            }
        }

        if let Some(source) = &self.source {
            writeln!(f)?;
            writeln!(f, "    Source: {source:?}")?;
        }

        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as _)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        let kind = match err.kind() {
            std::io::ErrorKind::NotFound => ErrorKind::NotFound,
            std::io::ErrorKind::PermissionDenied => ErrorKind::PermissionDenied,
            std::io::ErrorKind::AlreadyExists => ErrorKind::AlreadyExists,
            std::io::ErrorKind::TimedOut => ErrorKind::Timeout,
            std::io::ErrorKind::ConnectionRefused
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::ConnectionAborted => ErrorKind::Network,
            _ => ErrorKind::Io,
        };
        Error::new(kind, err.to_string()).set_source(err)
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::new(ErrorKind::Unexpected, err.to_string()).set_source(err)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Error::new(ErrorKind::Unexpected, err.to_string())
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::new(ErrorKind::Unexpected, msg)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error::new(ErrorKind::Unexpected, msg)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn error_creation() {
        let err = Error::new(ErrorKind::NotFound, "resource not found")
            .with_operation("get_resource")
            .with_context("id", "123")
            .with_context("type", "file");

        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert_eq!(err.message(), "resource not found");
        assert_eq!(err.operation(), "get_resource");
        assert_eq!(err.context().len(), 2);
        assert!(err.is_final());
        assert!(!err.is_temporary());
    }

    #[test]
    fn temporary_error() {
        let err = Error::new(ErrorKind::Network, "connection timeout").set_temporary();
        assert!(err.is_temporary());
        assert!(!err.is_final());
        assert_eq!(err.status(), ErrorStatus::Temporary);
    }

    #[test]
    fn operation_chaining() {
        let err = Error::new(ErrorKind::Io, "read failed")
            .with_operation("read_file")
            .with_operation("process_data");

        assert_eq!(err.operation(), "process_data");
        assert_eq!(err.context().len(), 1);
        assert_eq!(err.context()[0], ("called", "read_file".to_string()));
    }
}
