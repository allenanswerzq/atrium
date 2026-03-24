//! Metric event types and categories for event routing.

use serde::{Deserialize, Serialize};

/// Categories for routing events to different sinks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricCategory {
    /// Application-level custom events.
    Custom,
    /// Timing measurements.
    Timing,
    /// Monotonic counters.
    Counter,
    /// Point-in-time gauge values.
    Gauge,
}

/// A metric event to be routed to sinks.
#[derive(Debug, Clone, Serialize)]
pub struct MetricEvent {
    /// Category for routing.
    pub category: MetricCategory,
    /// Event name.
    pub name: String,
    /// JSON-compatible value.
    pub value: serde_json::Value,
    /// Epoch milliseconds.
    pub timestamp_ms: u64,
}

impl MetricEvent {
    /// Create a custom event.
    pub fn custom(name: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            category: MetricCategory::Custom,
            name: name.into(),
            value,
            timestamp_ms: epoch_ms(),
        }
    }

    /// Create a timing event.
    pub fn timing(name: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            category: MetricCategory::Timing,
            name: name.into(),
            value: serde_json::json!({ "duration_ms": duration_ms }),
            timestamp_ms: epoch_ms(),
        }
    }

    /// Create a counter event.
    pub fn counter(name: impl Into<String>, increment: u64) -> Self {
        Self {
            category: MetricCategory::Counter,
            name: name.into(),
            value: serde_json::json!({ "increment": increment }),
            timestamp_ms: epoch_ms(),
        }
    }

    /// Create a gauge event.
    pub fn gauge(name: impl Into<String>, value: f64) -> Self {
        Self {
            category: MetricCategory::Gauge,
            name: name.into(),
            value: serde_json::json!({ "value": value }),
            timestamp_ms: epoch_ms(),
        }
    }
}

fn epoch_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
