//! Sink infrastructure for routing metric events.
//!
//! Sinks receive serialized metric events and write them to destinations
//! (files, console, network, etc.).

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::event::{MetricCategory, MetricEvent};

/// Trait for metric event sinks.
pub trait Sink: Send + Sync {
    /// Write a serialized event line.
    fn write(&self, line: &str);

    /// Flush pending writes (if buffered).
    fn flush(&self) {}
}

/// A sink that discards all events.
#[derive(Debug, Default)]
pub struct NullSink;

impl Sink for NullSink {
    fn write(&self, _line: &str) {}
}

/// A sink that writes events to a file via a background channel.
pub struct FileSink {
    tx: tokio::sync::mpsc::UnboundedSender<String>,
}

impl FileSink {
    /// Create a new file sink that writes to the given path.
    ///
    /// Spawns a background task (on the current tokio runtime) to handle writes.
    pub fn new(path: PathBuf) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;

            let file = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .await;

            let mut file = match file {
                Ok(f) => f,
                Err(e) => {
                    tracing::error!("FileSink: failed to open {}: {e}", path.display());
                    return;
                }
            };

            while let Some(line) = rx.recv().await {
                if let Err(e) = file.write_all(line.as_bytes()).await {
                    tracing::warn!("FileSink: write error: {e}");
                    break;
                }
                if let Err(e) = file.write_all(b"\n").await {
                    tracing::warn!("FileSink: write error: {e}");
                    break;
                }
            }
        });

        Self { tx }
    }
}

impl Sink for FileSink {
    fn write(&self, line: &str) {
        let _ = self.tx.send(line.to_owned());
    }
}

/// Routes metric events to category-specific sinks.
pub struct SinkRouter {
    routes: HashMap<MetricCategory, Vec<Arc<dyn Sink>>>,
    catch_all: Vec<Arc<dyn Sink>>,
}

impl SinkRouter {
    /// Create a new empty router.
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            catch_all: Vec::new(),
        }
    }

    /// Add a sink for a specific category.
    pub fn route(mut self, category: MetricCategory, sink: Arc<dyn Sink>) -> Self {
        self.routes.entry(category).or_default().push(sink);
        self
    }

    /// Add a sink that receives all events.
    pub fn route_all(mut self, sink: Arc<dyn Sink>) -> Self {
        self.catch_all.push(sink);
        self
    }

    /// Emit an event to all matching sinks.
    pub fn emit(&self, event: &MetricEvent) {
        let line = match serde_json::to_string(event) {
            Ok(line) => line,
            Err(e) => {
                tracing::warn!("SinkRouter: failed to serialize event: {e}");
                return;
            }
        };

        if let Some(sinks) = self.routes.get(&event.category) {
            for sink in sinks {
                sink.write(&line);
            }
        }

        for sink in &self.catch_all {
            sink.write(&line);
        }
    }
}

impl Default for SinkRouter {
    fn default() -> Self {
        Self::new()
    }
}
