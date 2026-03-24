//! Core streaming traits for byte-level I/O.
//!
//! - [`FromStream`] — consume a byte stream into structured data
//! - [`IntoStream`] — convert structured data into a byte stream
//! - [`BytesStream`] — the standard pinned, boxed, sendable byte stream type

use atrium_error::Result;
use bytes::Bytes;
use futures_util::Stream;

/// A boxed, pinned, sendable stream of bytes.
pub type BytesStream = std::pin::Pin<Box<dyn Stream<Item = Result<Bytes>> + Send + 'static>>;

/// Consumes a byte stream into structured data.
///
/// # Example
///
/// ```rust,ignore
/// struct MyParser;
///
/// #[async_trait::async_trait]
/// impl FromStream for MyParser {
///     type Data = Vec<u8>;
///     async fn from_stream<S>(self, stream: S) -> Result<Self::Data>
///     where S: Stream<Item = Result<Bytes>> + Send + 'static
///     { /* ... */ }
/// }
/// ```
#[allow(clippy::wrong_self_convention)]
#[async_trait::async_trait]
pub trait FromStream<T = Bytes>: Sized {
    /// The type of data produced by consuming the stream.
    type Data: Send + Sync + 'static;

    /// Consumes a stream of bytes into structured data.
    async fn from_stream<S>(self, stream: S) -> Result<Self::Data>
    where
        S: Stream<Item = Result<T>> + Send + 'static;
}

/// Converts structured data into a byte stream.
///
/// # Example
///
/// ```rust,ignore
/// struct MyData(Vec<u8>);
///
/// #[async_trait::async_trait]
/// impl IntoStream for MyData {
///     async fn into_stream(self) -> Result<BytesStream> { /* ... */ }
/// }
/// ```
#[async_trait::async_trait]
pub trait IntoStream<T = Bytes>: Sized
where
    T: Send + 'static,
{
    /// Converts the data into a stream of bytes.
    async fn into_stream(
        self,
    ) -> Result<std::pin::Pin<Box<dyn Stream<Item = Result<T>> + Send + 'static>>>;
}
