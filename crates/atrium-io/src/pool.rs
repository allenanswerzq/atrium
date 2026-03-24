//! Global I/O concurrency pool.
//!
//! [`IoPool`] provides a shared semaphore that bounds the total number of
//! concurrent I/O operations across the entire application.
//!
//! # Why a single pool?
//!
//! Nested parallelism can explode resource usage if each level has its own
//! concurrency limit. A single shared pool avoids this — every I/O operation
//! competes for the same permits. No deadlocks, bounded memory, automatic fairness.
//!
//! # Usage
//!
//! ```rust
//! use atrium_io::IoPool;
//!
//! # async fn example() {
//! let pool = IoPool::new(8);
//! let _permit = pool.acquire().await;
//! // ... do I/O work ...
//! // permit drops automatically, releasing the slot
//! # }
//! ```

use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// A shared I/O concurrency pool.
///
/// Cheap to clone (backed by `Arc`). All clones share the same permit pool.
#[derive(Clone, Debug)]
pub struct IoPool {
    semaphore: Arc<Semaphore>,
    max_permits: usize,
}

impl IoPool {
    /// Create a new pool with `max_concurrent` permits.
    ///
    /// # Panics
    /// Panics if `max_concurrent` is 0.
    pub fn new(max_concurrent: usize) -> Self {
        assert!(max_concurrent > 0, "IoPool requires at least 1 permit");
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_permits: max_concurrent,
        }
    }

    /// Acquire a permit, waiting if all are in use.
    pub async fn acquire(&self) -> OwnedSemaphorePermit {
        self.semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("IoPool semaphore closed")
    }

    /// Try to acquire a permit without waiting.
    pub fn try_acquire(&self) -> Option<OwnedSemaphorePermit> {
        self.semaphore.clone().try_acquire_owned().ok()
    }

    /// Number of permits currently available.
    pub fn available(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Total number of permits in the pool.
    pub fn max_permits(&self) -> usize {
        self.max_permits
    }

    /// Number of permits currently in use.
    pub fn in_use(&self) -> usize {
        self.max_permits - self.available()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn acquire_and_release() {
        let pool = IoPool::new(2);
        assert_eq!(pool.available(), 2);

        let p1 = pool.acquire().await;
        assert_eq!(pool.available(), 1);
        assert_eq!(pool.in_use(), 1);

        let p2 = pool.acquire().await;
        assert_eq!(pool.available(), 0);

        drop(p1);
        assert_eq!(pool.available(), 1);

        drop(p2);
        assert_eq!(pool.available(), 2);
    }

    #[tokio::test]
    async fn try_acquire_returns_none_when_full() {
        let pool = IoPool::new(1);
        let _p = pool.acquire().await;
        assert!(pool.try_acquire().is_none());
    }

    #[tokio::test]
    async fn clone_shares_permits() {
        let pool = IoPool::new(2);
        let pool2 = pool.clone();

        let _p = pool.acquire().await;
        assert_eq!(pool2.available(), 1);
    }
}
