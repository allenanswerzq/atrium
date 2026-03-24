//! # atrium-error
//!
//! Unified error handling for Atrium with rich context and retry support.
//!
//! ## Design Principles
//!
//! - **Know what error occurred**: [`ErrorKind`] categorizes errors into actionable types
//! - **Decide how to handle it**: [`ErrorStatus`] indicates if errors are retryable
//! - **Assist in locating the cause**: Rich context with operation, key-value pairs, and source
//!
//! ## Example
//!
//! ```no_run
//! use atrium_error::{Error, ErrorKind};
//!
//! fn read_config(path: &str) -> Result<String, Error> {
//!     std::fs::read_to_string(path).map_err(|e| {
//!         Error::new(ErrorKind::NotFound, "config file not found")
//!             .with_operation("read_config")
//!             .with_context("path", path)
//!             .set_source(e)
//!     })
//! }
//! ```

mod error;
mod kind;
mod status;

pub use error::Error;
pub use kind::ErrorKind;
pub use status::ErrorStatus;

/// A specialized Result type for Atrium operations.
pub type Result<T> = std::result::Result<T, Error>;
