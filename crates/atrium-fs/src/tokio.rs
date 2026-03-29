//! Async wrapper for `tokio::fs` methods with unified error handling.
//!
//! Mirrors the sync API in the parent module but uses `tokio::fs` underneath.

use std::path::Path;

use atrium_error::{Error, ErrorKind, Result};
use serde::de::DeserializeOwned;

pub use tokio::fs::{File, OpenOptions, ReadDir};
pub use tokio::io::AsyncWriteExt;

use crate::io_error;

/// Async wrapper for [`tokio::fs::File::open`].
pub async fn open(path: impl AsRef<Path> + Send + Sync) -> Result<File> {
    let path = path.as_ref();
    File::open(path)
        .await
        .map_err(|e| io_error(e, "fs_open", path))
}

/// Async wrapper for `tokio::fs::read_to_string`.
pub async fn read_to_string(path: impl AsRef<Path> + Send + Sync) -> Result<String> {
    let path = path.as_ref();
    tokio::fs::read_to_string(path)
        .await
        .map_err(|e| io_error(e, "fs_read", path))
}

/// Async read entire file into bytes.
pub async fn read(path: impl AsRef<Path> + Send + Sync) -> Result<Vec<u8>> {
    let path = path.as_ref();
    tokio::fs::read(path)
        .await
        .map_err(|e| io_error(e, "fs_read", path))
}

/// Async wrapper for `tokio::fs::write`.
pub async fn write(
    path: impl AsRef<Path> + Send + Sync,
    contents: impl AsRef<[u8]> + Send + Sync,
) -> Result<()> {
    let path = path.as_ref();
    tokio::fs::write(path, contents.as_ref())
        .await
        .map_err(|e| io_error(e, "fs_write", path))
}

/// Async wrapper for `tokio::fs::remove_dir_all`.
pub async fn remove_dir_all(path: impl AsRef<Path> + Send + Sync) -> Result<()> {
    let path = path.as_ref();
    tokio::fs::remove_dir_all(path)
        .await
        .map_err(|e| io_error(e, "fs_remove_dir", path))
}

/// Async wrapper for `tokio::fs::File::create`.
pub async fn create_file(path: impl AsRef<Path> + Send + Sync) -> Result<File> {
    let path = path.as_ref();
    File::create(path)
        .await
        .map_err(|e| io_error(e, "fs_create_file", path))
}

/// Opens a file in append mode, creating it if it doesn't exist.
pub async fn open_append(path: impl AsRef<Path> + Send + Sync) -> Result<File> {
    let path = path.as_ref();
    OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(path)
        .await
        .map_err(|e| io_error(e, "fs_open_append", path))
}

/// Async wrapper for `tokio::fs::remove_file`.
pub async fn remove_file(path: impl AsRef<Path> + Send + Sync) -> Result<()> {
    let path = path.as_ref();
    tokio::fs::remove_file(path)
        .await
        .map_err(|e| io_error(e, "fs_remove_file", path))
}

/// Async wrapper for `tokio::fs::create_dir_all`.
pub async fn create_dir_all(path: impl AsRef<Path> + Send + Sync) -> Result<()> {
    let path = path.as_ref();
    tokio::fs::create_dir_all(path)
        .await
        .map_err(|e| io_error(e, "fs_create_dir", path))
}

/// Async wrapper for `tokio::fs::read_dir`.
pub async fn read_dir(path: impl AsRef<Path> + Send + Sync) -> Result<ReadDir> {
    let path = path.as_ref();
    tokio::fs::read_dir(path)
        .await
        .map_err(|e| io_error(e, "fs_read_dir", path))
}

/// Async wrapper for `tokio::fs::rename`.
pub async fn rename(
    from: impl AsRef<Path> + Send + Sync,
    to: impl AsRef<Path> + Send + Sync,
) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    tokio::fs::rename(from, to)
        .await
        .map_err(|e| crate::rename_error(e, from, to))
}

/// Async wrapper for `tokio::fs::metadata`.
pub async fn metadata(path: impl AsRef<Path> + Send + Sync) -> Result<std::fs::Metadata> {
    let path = path.as_ref();
    tokio::fs::metadata(path)
        .await
        .map_err(|e| io_error(e, "fs_metadata", path))
}

/// Async wrapper for `tokio::fs::hard_link`.
pub async fn hard_link(
    src: impl AsRef<Path> + Send + Sync,
    dst: impl AsRef<Path> + Send + Sync,
) -> Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    tokio::fs::hard_link(src, dst).await.map_err(|e| {
        Error::new(ErrorKind::Io, e.to_string())
            .with_operation("fs_hard_link")
            .with_context("src", src.display().to_string())
            .with_context("dst", dst.display().to_string())
            .set_source(e)
    })
}

/// Async read and deserialize a JSON file.
pub async fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = read(path).await?;
    serde_json::from_slice(&bytes).map_err(|e| {
        Error::new(ErrorKind::DataInvalid, "failed to parse json")
            .with_operation("fs_read_json")
            .with_context("path", path.display().to_string())
            .set_source(e)
    })
}

/// Atomic write: temp file → fsync → rename → fsync parent.
pub async fn atomic_write(file_path: &Path, data: impl AsRef<[u8]> + Send + Sync) -> Result<()> {
    let mut tmp_path = file_path.to_path_buf();
    let extension = file_path.extension().map_or_else(
        || std::ffi::OsString::from("tmp"),
        |ext| {
            let mut new_ext = ext.to_os_string();
            new_ext.push(".tmp");
            new_ext
        },
    );
    tmp_path.set_extension(extension);

    if let Some(parent) = tmp_path.parent() {
        if !parent.exists() {
            create_dir_all(parent).await?;
        }
    }

    let mut file = File::create(&tmp_path)
        .await
        .map_err(|e| io_error(e, "fs_create_file", &tmp_path))?;

    file.write_all(data.as_ref())
        .await
        .map_err(|e| io_error(e, "fs_write", &tmp_path))?;

    file.flush()
        .await
        .map_err(|e| io_error(e, "fs_flush", &tmp_path))?;

    file.sync_all()
        .await
        .map_err(|e| io_error(e, "fs_sync", &tmp_path))?;

    drop(file);

    rename(&tmp_path, file_path).await?;

    if let Some(parent) = file_path.parent() {
        fsync_dir(parent).await?;
    }

    Ok(())
}

/// Fsync a directory (ensures rename is durable).
///
/// On Windows, directory fsync is not reliably supported, so this is a no-op.
pub async fn fsync_dir(parent: &Path) -> Result<()> {
    // Windows doesn't support fsyncing directories in the same way as Unix.
    // NTFS provides durability guarantees via the filesystem journal.
    #[cfg(windows)]
    {
        let _ = parent;
        Ok(())
    }

    #[cfg(not(windows))]
    {
        let dir = tokio::fs::OpenOptions::new()
            .read(true)
            .open(parent)
            .await
            .map_err(|e| io_error(e, "fs_open", parent))?;

        dir.sync_all()
            .await
            .map_err(|e| io_error(e, "fs_sync", parent))?;

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn async_write_read_string() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_async.txt");
        let content = "Hello, async world!";

        write(&file_path, content).await.unwrap();

        let read_content = read_to_string(&file_path).await.unwrap();
        assert_eq!(content, read_content);
    }

    #[tokio::test]
    async fn async_atomic_write() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_atomic.txt");
        let content = b"Hello, atomic world!";

        atomic_write(&file_path, content).await.unwrap();

        let read_content = read(&file_path).await.unwrap();
        assert_eq!(read_content, content);

        let mut tmp_path = file_path.clone();
        tmp_path.set_extension("txt.tmp");
        assert!(!tmp_path.exists(), "Temp file should not exist");
    }
}
