//! Shutdown signal helpers.
//!
//! Provides a [`Signal`] / [`Shutdown`] pair for cooperative shutdown, plus
//! [`GracefulShutdown`] and its RAII [`GracefulShutdownGuard`] for tracking
//! in-flight work.

use futures_util::{
    FutureExt,
    future::{FusedFuture, Shared},
};
use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, ready},
};
use tokio::sync::oneshot;

/// A future that resolves when shutdown fires, yielding its [`GracefulShutdownGuard`].
///
/// The guard keeps the parent [`TaskManager`](crate::TaskManager) alive until dropped.
#[derive(Debug)]
pub struct GracefulShutdown {
    shutdown: Shutdown,
    guard: Option<GracefulShutdownGuard>,
}

impl GracefulShutdown {
    pub(crate) const fn new(shutdown: Shutdown, guard: GracefulShutdownGuard) -> Self {
        Self {
            shutdown,
            guard: Some(guard),
        }
    }

    /// Returns a future that ignores the returned guard (doesn't drop it early).
    pub fn ignore_guard(self) -> impl Future<Output = ()> + Send + Sync + Unpin + 'static {
        self.map(drop)
    }
}

impl Future for GracefulShutdown {
    type Output = GracefulShutdownGuard;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        ready!(self.shutdown.poll_unpin(cx));
        if let Some(guard) = self.get_mut().guard.take() {
            Poll::Ready(guard)
        } else {
            Poll::Pending
        }
    }
}

impl Clone for GracefulShutdown {
    fn clone(&self) -> Self {
        Self {
            shutdown: self.shutdown.clone(),
            guard: self
                .guard
                .as_ref()
                .map(|g| GracefulShutdownGuard::new(Arc::clone(&g.0))),
        }
    }
}

/// RAII guard — increments counter on creation, decrements on drop.
#[derive(Debug)]
#[must_use = "dropping this guard signals the task manager that graceful work finished"]
pub struct GracefulShutdownGuard(pub(crate) Arc<AtomicUsize>);

impl GracefulShutdownGuard {
    pub(crate) fn new(counter: Arc<AtomicUsize>) -> Self {
        counter.fetch_add(1, Ordering::SeqCst);
        Self(counter)
    }
}

impl Drop for GracefulShutdownGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

/// A cloneable future that resolves when the shutdown event fires.
#[derive(Debug, Clone)]
pub struct Shutdown(Shared<oneshot::Receiver<()>>);

impl Future for Shutdown {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let pin = self.get_mut();
        if pin.0.is_terminated() || pin.0.poll_unpin(cx).is_ready() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

/// Sends the shutdown signal — fires on `fire()` or on drop.
#[derive(Debug)]
pub struct Signal(oneshot::Sender<()>);

impl Signal {
    /// Fire the signal manually.
    pub fn fire(self) {
        let _ = self.0.send(());
    }
}

/// Create a (`Signal`, `Shutdown`) pair for cooperative shutdown.
pub fn signal() -> (Signal, Shutdown) {
    let (sender, receiver) = oneshot::channel();
    (Signal(sender), Shutdown(receiver.shared()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use futures_util::future::join_all;
    use std::time::Duration;

    #[tokio::test(flavor = "multi_thread")]
    async fn drop_signal_completes_shutdown() {
        let (signal, shutdown) = signal();
        tokio::task::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            drop(signal);
        });
        shutdown.await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn multi_shutdown_listeners() {
        let (signal, shutdown) = signal();
        let mut tasks = Vec::with_capacity(100);
        for _ in 0..100 {
            let shutdown = shutdown.clone();
            tasks.push(tokio::task::spawn(shutdown));
        }
        drop(signal);
        join_all(tasks).await;
    }
}
