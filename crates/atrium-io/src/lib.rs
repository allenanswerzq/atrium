//! # atrium-io
//!
//! Streaming I/O primitives and concurrency pool for Atrium.
//!
//! ## Modules
//!
//! - [`pool`] — Global I/O concurrency pool (semaphore-based)
//! - [`stats`] — Operation statistics tracking
//! - [`stream`] — Core streaming traits (`FromStream`, `IntoStream`)
//!
//! Ported from pilot-io — focused skeleton for Atrium's needs.

pub mod pool;
pub mod stats;
pub mod stream;

pub use pool::IoPool;
pub use stats::{AtomicIoStats, IoStats};
pub use stream::{BytesStream, FromStream, IntoStream};
