//! IO statistics for tracking operations.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

/// Statistics for IO operations.
#[derive(Debug, Clone, Default)]
pub struct IoStats {
    pub files: usize,
    pub ops: usize,
    pub bytes: u64,
    pub syncs: usize,
    pub errors: usize,
    pub duration: Duration,
}

impl IoStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge another stats instance into this one.
    pub fn add(&mut self, other: &IoStats) {
        self.files += other.files;
        self.ops += other.ops;
        self.bytes += other.bytes;
        self.syncs += other.syncs;
        self.errors += other.errors;
        self.duration += other.duration;
    }

    /// Throughput in bytes per second.
    pub fn throughput_bps(&self) -> f64 {
        let secs = self.duration.as_secs_f64();
        if secs > 0.0 {
            self.bytes as f64 / secs
        } else {
            0.0
        }
    }

    /// Average operation size in bytes.
    pub fn avg_op_size(&self) -> f64 {
        if self.ops > 0 {
            self.bytes as f64 / self.ops as f64
        } else {
            0.0
        }
    }

    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }
}

impl std::fmt::Display for IoStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ops, {} bytes, {} files, {} syncs, {} errors, {:?}",
            self.ops, self.bytes, self.files, self.syncs, self.errors, self.duration
        )
    }
}

/// Atomic counters for thread-safe statistics accumulation.
#[derive(Debug, Default)]
pub struct AtomicIoStats {
    pub files: AtomicUsize,
    pub ops: AtomicUsize,
    pub bytes: AtomicU64,
    pub syncs: AtomicUsize,
    pub errors: AtomicUsize,
}

impl AtomicIoStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inc_ops(&self) {
        self.ops.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_bytes(&self, n: u64) {
        self.bytes.fetch_add(n, Ordering::Relaxed);
    }

    pub fn inc_syncs(&self) {
        self.syncs.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_errors(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Snapshot the atomic counters into a non-atomic struct.
    pub fn snapshot(&self) -> IoStats {
        IoStats {
            files: self.files.load(Ordering::Relaxed),
            ops: self.ops.load(Ordering::Relaxed),
            bytes: self.bytes.load(Ordering::Relaxed),
            syncs: self.syncs.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            duration: Duration::ZERO,
        }
    }
}
