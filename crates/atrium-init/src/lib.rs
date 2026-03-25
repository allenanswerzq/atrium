//! # atrium-init
//!
//! Two-phase initialization for Atrium applications.
//!
//! Follows the typestate pattern from pilot-init:
//!
//! 1. **Early initialization** ([`EarlyInitialized`]): minimal bootstrap
//!    - Task executor
//!    - Global context
//!    - I/O pool
//!    - Tracing
//!
//! 2. **Full initialization** ([`Initialized`]): complete runtime
//!    - Event manager
//!    - Additional services
//!
//! # Example
//!
//! ```rust,ignore
//! let early = atrium_init::early_init().await?;
//! // use early.executor(), early.ctxt(), early.io_pool()
//!
//! let initialized = early.finalize().await?;
//! // full runtime available
//! ```

use std::sync::Arc;

use atrium_context::GlobalCtxt;
use atrium_error::{Error, ErrorKind, Result};
use atrium_executor::{TaskExecutor, TaskManager};
use atrium_io::IoPool;
use atrium_trace::SinkRouter;

// ── Early Initialized (Phase 1) ────────────────────────────────────────

struct EarlyInner {
    task_manager: TaskManager,
    ctxt: GlobalCtxt,
    io_pool: IoPool,
    application: Option<gpui::Application>,
}

/// Handle to the early-initialized Atrium runtime.
///
/// Provides minimal resources for the bootstrap phase.
/// Cheap to clone (backed by `Arc`).
#[derive(Clone)]
pub struct EarlyInitialized {
    inner: Arc<EarlyInner>,
}

impl EarlyInitialized {
    /// Get the task manager.
    pub fn task_manager(&self) -> &TaskManager {
        &self.inner.task_manager
    }

    /// Get a task executor for spawning background tasks.
    pub fn executor(&self) -> TaskExecutor {
        self.inner.task_manager.executor()
    }

    /// Get the global context.
    pub fn ctxt(&self) -> &GlobalCtxt {
        &self.inner.ctxt
    }

    /// Get the I/O concurrency pool.
    pub fn io_pool(&self) -> &IoPool {
        &self.inner.io_pool
    }

    /// Take the GPUI Application out of the early-init handle.
    ///
    /// Can only be called once. Returns `None` if already taken.
    pub fn take_application(&mut self) -> Option<gpui::Application> {
        Arc::get_mut(&mut self.inner)
            .and_then(|inner| inner.application.take())
    }

    /// Finalize into a fully initialized runtime.
    ///
    /// Consumes the early handle. All clones must be dropped first.
    pub async fn finalize(self) -> Result<Initialized> {
        let inner = Arc::try_unwrap(self.inner).map_err(|_| {
            Error::new(
                ErrorKind::InvalidInput,
                "Cannot finalize: other references to EarlyInitialized exist",
            )
        })?;

        let sink_router = SinkRouter::new();
        // Future: route events to file sinks based on config

        tracing::info!("Atrium initialization complete");

        Ok(Initialized {
            inner: Arc::new(InitializedInner {
                ctxt: inner.ctxt,
                task_manager: inner.task_manager,
                io_pool: inner.io_pool,
                sink_router,
            }),
        })
    }
}

// ── Fully Initialized (Phase 2) ─────────────────────────────────────

struct InitializedInner {
    ctxt: GlobalCtxt,
    task_manager: TaskManager,
    io_pool: IoPool,
    sink_router: SinkRouter,
}

/// Handle to the fully initialized Atrium runtime.
///
/// Cheap to clone (backed by `Arc`).
#[derive(Clone)]
pub struct Initialized {
    inner: Arc<InitializedInner>,
}

impl Initialized {
    pub fn ctxt(&self) -> &GlobalCtxt {
        &self.inner.ctxt
    }

    pub fn task_manager(&self) -> &TaskManager {
        &self.inner.task_manager
    }

    pub fn executor(&self) -> TaskExecutor {
        self.inner.task_manager.executor()
    }

    pub fn io_pool(&self) -> &IoPool {
        &self.inner.io_pool
    }

    pub fn sink_router(&self) -> &SinkRouter {
        &self.inner.sink_router
    }

    /// Gracefully shut down all tasks.
    pub async fn shutdown(self) {
        let inner = match Arc::try_unwrap(self.inner) {
            Ok(inner) => inner,
            Err(_) => {
                tracing::warn!("shutdown called with outstanding references");
                return;
            }
        };
        inner.task_manager.shutdown().await;
    }
}

// ── Factory ──────────────────────────────────────────────────────────

/// Default number of I/O concurrency permits.
const DEFAULT_IO_PERMITS: usize = 16;

/// Perform early initialization.
///
/// Sets up tracing, creates the task manager, global context, and I/O pool.
pub async fn early_init() -> Result<EarlyInitialized> {
    atrium_trace::init_with_default("info");

    let ctxt = GlobalCtxt::builder().build()?;

    let task_manager = TaskManager::current();
    let io_pool = IoPool::new(DEFAULT_IO_PERMITS);

    tracing::info!(
        hostname = %ctxt.hostname(),
        config_dir = %ctxt.config_dir().display(),
        data_dir = %ctxt.data_dir().display(),
        "Atrium early initialization complete"
    );

    Ok(EarlyInitialized {
        inner: Arc::new(EarlyInner {
            task_manager,
            ctxt,
            io_pool,
            application: Some(gpui::Application::new()),
        }),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn early_init_and_finalize() {
        let early = early_init().await.unwrap();
        assert!(!early.ctxt().hostname().is_empty());
        assert_eq!(early.io_pool().max_permits(), DEFAULT_IO_PERMITS);

        let initialized = early.finalize().await.unwrap();
        assert!(!initialized.ctxt().hostname().is_empty());
        initialized.shutdown().await;
    }
}
