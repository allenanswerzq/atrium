//! # atrium-trace
//!
//! Tracing and observability for Atrium.
//!
//! Provides structured logging initialization, event categories for metrics
//! routing, and file-based sink infrastructure.
//!
//! Ported from pilot-trace patterns — simplified for Atrium's needs,
//! without Linux /proc monitoring.

mod event;
mod sink;

pub use event::{MetricCategory, MetricEvent};
pub use sink::{FileSink, NullSink, Sink, SinkRouter};

pub use tracing;

/// Initialize a basic tracing subscriber with env filter.
///
/// Reads `RUST_LOG` for filter directives (default: `info`).
pub fn init() {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
}

/// Initialize with a specific default filter.
pub fn init_with_default(default_filter: &str) {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter)))
        .init();
}
