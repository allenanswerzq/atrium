//! # atrium-executor
//!
//! Task executor with graceful shutdown support.
//!
//! Provides [`TaskManager`] for managing task lifecycles and [`TaskExecutor`]
//! for spawning regular and critical tasks. Critical tasks report panics so the
//! application can react to unexpected failures.
//!
//! Ported from pilot-executor / reth task management.

pub mod shutdown;

use crate::shutdown::{GracefulShutdown, GracefulShutdownGuard, Shutdown, Signal, signal};
use futures_util::{
    Future, FutureExt, TryFutureExt,
    future::{BoxFuture, select},
};
use std::{
    any::Any,
    fmt::{Display, Formatter},
    pin::pin,
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    task::{Context, Poll, ready},
};
use tokio::{
    runtime::Handle,
    sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    task::JoinHandle,
};
use tracing::{Instrument, error};

// ── Task kind ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
enum TaskKind {
    Default,
    Blocking,
}

// ── Metrics (plain atomics, no external crate) ───────────────────────

/// Lightweight counters for task lifecycle tracking.
#[derive(Debug, Clone, Default)]
pub struct TaskMetrics {
    pub(crate) critical_spawned: Arc<AtomicU64>,
    pub(crate) critical_running: Arc<AtomicU64>,
    pub(crate) regular_spawned: Arc<AtomicU64>,
    pub(crate) regular_running: Arc<AtomicU64>,
}

impl TaskMetrics {
    /// Currently running critical tasks.
    pub fn critical_running(&self) -> u64 {
        self.critical_running.load(Ordering::Relaxed)
    }

    /// Currently running regular tasks.
    pub fn regular_running(&self) -> u64 {
        self.regular_running.load(Ordering::Relaxed)
    }

    /// Total tasks currently running.
    pub fn tasks_running(&self) -> u64 {
        self.critical_running() + self.regular_running()
    }
}

/// RAII guard that decrements the running counter on drop.
struct TaskGuard(Arc<AtomicU64>);

impl Drop for TaskGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

// ── TaskSpawner trait ────────────────────────────────────────────────

/// A type that can spawn tasks onto the tokio runtime.
pub trait TaskSpawner: Send + Sync + std::fmt::Debug {
    /// Spawns a regular task.
    fn spawn(&self, fut: BoxFuture<'static, ()>) -> JoinHandle<()>;

    /// Spawns a critical task that reports panics.
    fn spawn_critical(&self, name: &'static str, fut: BoxFuture<'static, ()>) -> JoinHandle<()>;

    /// Spawns a blocking task.
    fn spawn_blocking(&self, fut: BoxFuture<'static, ()>) -> JoinHandle<()>;
}

// ── Panicked task error ──────────────────────────────────────────────

/// Error emitted when a critical task panics.
#[derive(Debug, thiserror::Error)]
pub struct PanickedTaskError {
    task_name: &'static str,
    error: Option<String>,
}

impl Display for PanickedTaskError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(error) = &self.error {
            write!(f, "Critical task `{}` panicked: `{error}`", self.task_name)
        } else {
            write!(f, "Critical task `{}` panicked", self.task_name)
        }
    }
}

impl PanickedTaskError {
    fn new(task_name: &'static str, error: Box<dyn Any>) -> Self {
        let error = match error.downcast::<String>() {
            Ok(value) => Some(*value),
            Err(error) => match error.downcast::<&str>() {
                Ok(value) => Some(value.to_string()),
                Err(_) => None,
            },
        };
        Self { task_name, error }
    }

    fn internal(task_name: &'static str, message: &'static str) -> Self {
        Self {
            task_name,
            error: Some(message.to_owned()),
        }
    }
}

// ── TaskManager ──────────────────────────────────────────────────────

/// Manages task lifecycles and monitors critical task panics.
///
/// Implements `Future` — poll it to receive [`PanickedTaskError`] notifications.
#[derive(Debug)]
#[must_use = "TaskManager must be polled to monitor critical tasks"]
pub struct TaskManager {
    handle: Handle,
    panicked_tasks_tx: UnboundedSender<PanickedTaskError>,
    panicked_tasks_rx: UnboundedReceiver<PanickedTaskError>,
    signal: Option<Signal>,
    on_shutdown: Shutdown,
    graceful_tasks: Arc<AtomicUsize>,
}

impl TaskManager {
    /// Create over the currently running tokio runtime.
    ///
    /// # Panics
    /// Panics if called outside a tokio runtime.
    pub fn current() -> Self {
        Self::new(Handle::current())
    }

    /// Create for a specific runtime handle.
    pub fn new(handle: Handle) -> Self {
        let (panicked_tasks_tx, panicked_tasks_rx) = unbounded_channel();
        let (sig, on_shutdown) = signal();
        Self {
            handle,
            panicked_tasks_tx,
            panicked_tasks_rx,
            signal: Some(sig),
            on_shutdown,
            graceful_tasks: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Returns a [`TaskExecutor`] that can spawn tasks.
    pub fn executor(&self) -> TaskExecutor {
        TaskExecutor {
            handle: self.handle.clone(),
            on_shutdown: self.on_shutdown.clone(),
            panicked_tasks_tx: self.panicked_tasks_tx.clone(),
            metrics: TaskMetrics::default(),
        }
    }

    /// Fire shutdown and wait for all graceful tasks to complete.
    pub async fn shutdown(self) {
        self.do_graceful_shutdown(None).await;
    }

    /// Fire shutdown with a timeout. Returns `true` if all tasks finished in time.
    pub async fn shutdown_timeout(self, timeout: std::time::Duration) -> bool {
        self.do_graceful_shutdown(Some(timeout)).await
    }

    async fn do_graceful_shutdown(self, timeout: Option<std::time::Duration>) -> bool {
        drop(self.signal);
        let check_interval = std::time::Duration::from_millis(1);
        let deadline = timeout.map(|t| tokio::time::Instant::now() + t);

        loop {
            if self.graceful_tasks.load(Ordering::Relaxed) == 0 {
                return true;
            }
            if let Some(deadline) = deadline {
                if tokio::time::Instant::now() >= deadline {
                    return false;
                }
            }
            tokio::time::sleep(check_interval).await;
        }
    }

    /// Returns a [`GracefulShutdown`] future.
    pub fn graceful_shutdown(&self) -> GracefulShutdown {
        GracefulShutdown::new(
            self.on_shutdown.clone(),
            GracefulShutdownGuard::new(Arc::clone(&self.graceful_tasks)),
        )
    }
}

impl Future for TaskManager {
    type Output = PanickedTaskError;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let err = ready!(self.get_mut().panicked_tasks_rx.poll_recv(cx));
        match err {
            Some(err) => Poll::Ready(err),
            None => Poll::Ready(PanickedTaskError::internal(
                "task-manager",
                "panicked task channel closed unexpectedly",
            )),
        }
    }
}

// ── TaskExecutor ─────────────────────────────────────────────────────

/// Spawns tasks onto the tokio runtime with shutdown awareness and panic tracking.
#[derive(Debug, Clone)]
pub struct TaskExecutor {
    handle: Handle,
    on_shutdown: Shutdown,
    panicked_tasks_tx: UnboundedSender<PanickedTaskError>,
    metrics: TaskMetrics,
}

impl TaskExecutor {
    /// Returns the tokio runtime handle.
    pub const fn handle(&self) -> &Handle {
        &self.handle
    }

    /// Wait for the shutdown signal.
    pub async fn wait_for_shutdown(&self) {
        self.on_shutdown.clone().await;
    }

    /// Returns the shutdown receiver.
    pub fn on_shutdown(&self) -> Shutdown {
        self.on_shutdown.clone()
    }

    /// Task metrics snapshot.
    pub fn metrics(&self) -> &TaskMetrics {
        &self.metrics
    }

    fn spawn_on_rt<F: Future<Output = ()> + Send + 'static>(
        &self,
        fut: F,
        task_kind: TaskKind,
    ) -> JoinHandle<()> {
        match task_kind {
            TaskKind::Default => self.handle.spawn(fut),
            TaskKind::Blocking => {
                let handle = self.handle.clone();
                self.handle.spawn_blocking(move || handle.block_on(fut))
            }
        }
    }

    /// Spawn a regular task. Cancelled on shutdown.
    pub fn spawn<F: Future<Output = ()> + Send + 'static>(&self, fut: F) -> JoinHandle<()> {
        self.spawn_task_as(fut, TaskKind::Default)
    }

    /// Spawn a regular blocking task. Cancelled on shutdown.
    pub fn spawn_blocking<F: Future<Output = ()> + Send + 'static>(
        &self,
        fut: F,
    ) -> JoinHandle<()> {
        self.spawn_task_as(fut, TaskKind::Blocking)
    }

    fn spawn_task_as<F: Future<Output = ()> + Send + 'static>(
        &self,
        fut: F,
        task_kind: TaskKind,
    ) -> JoinHandle<()> {
        let on_shutdown = self.on_shutdown.clone();
        let running = Arc::clone(&self.metrics.regular_running);
        running.fetch_add(1, Ordering::Relaxed);
        self.metrics.regular_spawned.fetch_add(1, Ordering::Relaxed);

        let task = async move {
            let _guard = TaskGuard(running);
            let fut = pin!(fut);
            let _ = select(on_shutdown, fut).await;
        }
        .in_current_span();

        self.spawn_on_rt(task, task_kind)
    }

    /// Spawn a critical task. Panics are reported to the [`TaskManager`].
    pub fn spawn_critical<F: Future<Output = ()> + Send + 'static>(
        &self,
        name: &'static str,
        fut: F,
    ) -> JoinHandle<()> {
        self.spawn_critical_as(name, fut, TaskKind::Default)
    }

    /// Spawn a critical blocking task.
    pub fn spawn_critical_blocking<F: Future<Output = ()> + Send + 'static>(
        &self,
        name: &'static str,
        fut: F,
    ) -> JoinHandle<()> {
        self.spawn_critical_as(name, fut, TaskKind::Blocking)
    }

    fn spawn_critical_as<F: Future<Output = ()> + Send + 'static>(
        &self,
        name: &'static str,
        fut: F,
        task_kind: TaskKind,
    ) -> JoinHandle<()> {
        let panicked_tasks_tx = self.panicked_tasks_tx.clone();
        let on_shutdown = self.on_shutdown.clone();
        let running = Arc::clone(&self.metrics.critical_running);
        running.fetch_add(1, Ordering::Relaxed);
        self.metrics
            .critical_spawned
            .fetch_add(1, Ordering::Relaxed);

        let task = std::panic::AssertUnwindSafe(fut)
            .catch_unwind()
            .map_err(move |err| {
                let task_error = PanickedTaskError::new(name, err);
                error!("{task_error}");
                let _ = panicked_tasks_tx.send(task_error);
            })
            .in_current_span();

        let task = async move {
            let _guard = TaskGuard(running);
            let task = pin!(task);
            let _ = select(on_shutdown, task).await;
        };

        self.spawn_on_rt(task, task_kind)
    }

    /// Spawn a task that receives the shutdown signal.
    pub fn spawn_with_signal<F, Fut>(&self, f: F) -> JoinHandle<()>
    where
        F: FnOnce(Shutdown) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let fut = f(self.on_shutdown.clone());
        self.handle.spawn(fut.in_current_span())
    }
}

impl TaskSpawner for TaskExecutor {
    fn spawn(&self, fut: BoxFuture<'static, ()>) -> JoinHandle<()> {
        self.spawn(fut)
    }

    fn spawn_critical(&self, name: &'static str, fut: BoxFuture<'static, ()>) -> JoinHandle<()> {
        self.spawn_critical(name, fut)
    }

    fn spawn_blocking(&self, fut: BoxFuture<'static, ()>) -> JoinHandle<()> {
        TaskExecutor::spawn_blocking(self, fut)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test(flavor = "multi_thread")]
    async fn spawn_and_shutdown() {
        let manager = TaskManager::current();
        let executor = manager.executor();

        let (tx, rx) = tokio::sync::oneshot::channel();
        executor.spawn(async move {
            let _ = tx.send(42);
        });

        let val = rx.await.unwrap();
        assert_eq!(val, 42);
        manager.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn critical_panic_reported() {
        let mut manager = TaskManager::current();
        let executor = manager.executor();

        executor.spawn_critical("panicker", async { panic!("boom") });

        let err = (&mut manager).await;
        assert!(err.to_string().contains("panicker"));
        assert!(err.to_string().contains("boom"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn shutdown_timeout_succeeds() {
        let manager = TaskManager::current();
        let _executor = manager.executor();
        let completed = manager.shutdown_timeout(Duration::from_secs(1)).await;
        assert!(completed);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn metrics_track_tasks() {
        let manager = TaskManager::current();
        let executor = manager.executor();

        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        executor.spawn(async move {
            let _ = rx.await;
        });

        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(executor.metrics().regular_running(), 1);

        let _ = tx.send(());
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(executor.metrics().regular_running(), 0);

        manager.shutdown().await;
    }
}
