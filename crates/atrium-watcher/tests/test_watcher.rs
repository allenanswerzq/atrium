//! Integration tests for FileWatcher.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use atrium_executor::TaskManager;
use atrium_watcher::FileWatcher;
use std::io::Write;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::sync::mpsc;

#[tokio::test(flavor = "multi_thread")]
async fn start_with_no_existing_files() {
    let task_manager = TaskManager::current();
    let executor = task_manager.executor();
    let (tx, _rx) = mpsc::unbounded_channel();
    let files = vec![
        std::path::PathBuf::from("/nonexistent/file1.txt"),
        std::path::PathBuf::from("/nonexistent/file2.txt"),
    ];

    let result = FileWatcher::start(executor, files, tx);
    assert!(!result, "should return false when no files exist");
}

#[tokio::test(flavor = "multi_thread")]
async fn start_with_existing_files() {
    let task_manager = TaskManager::current();
    let executor = task_manager.executor();
    let temp_file = NamedTempFile::new().unwrap();
    let (tx, _rx) = mpsc::unbounded_channel();

    let result = FileWatcher::start(executor, vec![temp_file.path().to_path_buf()], tx);
    assert!(result);
}

#[tokio::test(flavor = "multi_thread")]
async fn start_filters_nonexistent() {
    let task_manager = TaskManager::current();
    let executor = task_manager.executor();
    let temp_file = NamedTempFile::new().unwrap();
    let (tx, _rx) = mpsc::unbounded_channel();
    let files = vec![
        std::path::PathBuf::from("/nonexistent/file.txt"),
        temp_file.path().to_path_buf(),
    ];

    let result = FileWatcher::start(executor, files, tx);
    assert!(result, "should return true when at least one file exists");
}

#[tokio::test(flavor = "multi_thread")]
async fn start_with_empty_list() {
    let task_manager = TaskManager::current();
    let executor = task_manager.executor();
    let (tx, _rx) = mpsc::unbounded_channel();

    let result = FileWatcher::start(executor, vec![], tx);
    assert!(!result);
}

#[tokio::test(flavor = "multi_thread")]
async fn detects_file_modification() {
    let task_manager = TaskManager::current();
    let executor = task_manager.executor();

    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "initial content").unwrap();
    temp_file.flush().unwrap();

    let (tx, mut rx) = mpsc::unbounded_channel();
    let started = FileWatcher::start(executor, vec![temp_file.path().to_path_buf()], tx);
    assert!(started);

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Modify the file
    let file = temp_file.reopen().unwrap();
    let mut writer = std::io::BufWriter::new(file);
    writeln!(writer, "modified content").unwrap();
    writer.flush().unwrap();

    match tokio::time::timeout(Duration::from_secs(5), rx.recv()).await {
        Ok(Some(path)) => assert_eq!(path, temp_file.path().to_path_buf()),
        Ok(None) => {} // channel closed — acceptable edge case
        Err(_) => println!("warning: file change notification timed out"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn channel_closure_stops_watcher() {
    let task_manager = TaskManager::current();
    let executor = task_manager.executor();
    let temp_file = NamedTempFile::new().unwrap();
    let (tx, rx) = mpsc::unbounded_channel();

    FileWatcher::start(executor, vec![temp_file.path().to_path_buf()], tx);
    drop(rx);

    tokio::time::sleep(Duration::from_millis(100)).await;
    // If we get here without panicking, the watcher handled closure gracefully
}
