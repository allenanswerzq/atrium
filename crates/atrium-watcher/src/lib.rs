//! File change watcher with mtime polling.
//!
//! Monitors files for changes and sends notifications through a channel.
//! Uses mtime polling for cross-platform compatibility.
//!
//! # Example
//!
//! ```ignore
//! use atrium_watcher::FileWatcher;
//!
//! let files = vec![PathBuf::from("config.toml")];
//! let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
//!
//! FileWatcher::start(executor, files, tx);
//!
//! while let Some(path) = rx.recv().await {
//!     println!("File changed: {}", path.display());
//! }
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use atrium_executor::TaskExecutor;
use tokio::sync::mpsc::UnboundedSender;

/// Sender type for file change notifications.
pub type FileChangeSender = UnboundedSender<PathBuf>;

/// Default polling interval for mtime checks.
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(500);

/// File watcher that monitors files for changes.
///
/// Uses mtime polling for broad cross-platform support (Windows, macOS, Linux).
#[derive(Debug)]
pub struct FileWatcher;

impl FileWatcher {
    /// Start watching the given files for changes.
    ///
    /// When any file changes, sends the path through the sender.
    /// The watcher runs in a background task until the sender is dropped.
    ///
    /// Returns `true` if watching started successfully.
    pub fn start(executor: TaskExecutor, files: Vec<PathBuf>, sender: FileChangeSender) -> bool {
        Self::start_with_interval(executor, files, sender, DEFAULT_POLL_INTERVAL)
    }

    /// Start watching with a custom poll interval.
    ///
    /// Returns `true` if watching started successfully.
    pub fn start_with_interval(
        executor: TaskExecutor,
        files: Vec<PathBuf>,
        sender: FileChangeSender,
        interval: Duration,
    ) -> bool {
        let files: Vec<PathBuf> = files
            .into_iter()
            .filter(|path| {
                if path.exists() {
                    tracing::info!(path = %path.display(), "watching file for changes");
                    true
                } else {
                    tracing::warn!(path = %path.display(), "file does not exist, skipping watch");
                    false
                }
            })
            .collect();

        if files.is_empty() {
            tracing::warn!("no files to watch");
            return false;
        }

        Self::start_mtime_polling(executor, files, sender, interval);
        true
    }

    /// Start watching using mtime polling.
    fn start_mtime_polling(
        executor: TaskExecutor,
        files: Vec<PathBuf>,
        sender: FileChangeSender,
        interval: Duration,
    ) {
        let last_mtimes: HashMap<PathBuf, Option<SystemTime>> = files
            .into_iter()
            .map(|path| {
                let mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
                (path, mtime)
            })
            .collect();

        executor.spawn_with_signal(|shutdown| async move {
            let mut mtimes = last_mtimes;
            tokio::pin!(shutdown);

            loop {
                tokio::select! {
                    _ = &mut shutdown => {
                        tracing::debug!("file watcher received shutdown signal");
                        return;
                    }
                    _ = tokio::time::sleep(interval) => {}
                }

                if sender.is_closed() {
                    tracing::debug!("file watcher channel closed, stopping");
                    return;
                }

                let mut changed_files = Vec::new();
                for (path, last_mtime) in &mut mtimes {
                    if let Ok(metadata) = std::fs::metadata(path.as_path()) {
                        if let Ok(current_mtime) = metadata.modified() {
                            if *last_mtime != Some(current_mtime) {
                                tracing::debug!(path = %path.display(), "file changed (mtime)");
                                *last_mtime = Some(current_mtime);
                                changed_files.push(path.clone());
                            }
                        }
                    }
                }

                for path in changed_files {
                    if sender.send(path).is_err() {
                        tracing::debug!("file watcher channel closed");
                        return;
                    }
                }
            }
        });
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use atrium_executor::TaskManager;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio::sync::mpsc;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_watcher_nonexistent_file() {
        let task_manager = TaskManager::current();
        let executor = task_manager.executor();
        let (tx, _rx) = mpsc::unbounded_channel();
        let result = FileWatcher::start(executor, vec![PathBuf::from("/nonexistent/file.ini")], tx);
        assert!(!result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_watcher_empty_list() {
        let task_manager = TaskManager::current();
        let executor = task_manager.executor();
        let (tx, _rx) = mpsc::unbounded_channel();
        let result = FileWatcher::start(executor, vec![], tx);
        assert!(!result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_watcher_detects_change() {
        let task_manager = TaskManager::current();
        let executor = task_manager.executor();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "initial content").unwrap();
        file.flush().unwrap();

        let (tx, mut rx) = mpsc::unbounded_channel();
        let path = file.path().to_path_buf();

        let result = FileWatcher::start(executor, vec![path.clone()], tx);
        assert!(result);

        tokio::time::sleep(Duration::from_millis(100)).await;

        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&path)
            .unwrap();
        writeln!(f, "modified content").unwrap();
        f.flush().unwrap();
        drop(f);

        let result = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await;
        assert!(result.is_ok(), "should receive change notification");

        let changed_path = result.unwrap().unwrap();
        assert_eq!(changed_path, path);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_channel_closure_stops_watcher() {
        let task_manager = TaskManager::current();
        let executor = task_manager.executor();
        let temp_file = NamedTempFile::new().unwrap();
        let (tx, rx) = mpsc::unbounded_channel();

        FileWatcher::start(executor, vec![temp_file.path().to_path_buf()], tx);
        drop(rx);

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
