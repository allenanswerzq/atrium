//! Background pollers — terminal sync, log refresh, memory sampling.

use std::time::Duration;

/// Interval for terminal polling when active.
pub const TERMINAL_POLL_ACTIVE: Duration = Duration::from_millis(16);
/// Interval for terminal polling when idle.
pub const TERMINAL_POLL_IDLE: Duration = Duration::from_millis(200);
/// Interval for log refresh.
pub const LOG_POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Background poller handles.
pub struct Pollers;

impl Pollers {
    /// Start all background pollers. Returns handles that keep them alive.
    pub fn start() -> Self {
        // TODO: spawn GPUI background tasks for terminal sync, log refresh, etc.
        tracing::debug!("background pollers initialized");
        Self
    }
}
