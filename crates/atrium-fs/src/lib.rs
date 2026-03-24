//! Filesystem wrappers with unified error handling.
//!
//! Every operation automatically includes path context in error messages.
//!
//! - Sync operations: top-level functions (`read_to_string`, `write`, etc.)
//! - Async operations: [`tokio`] submodule (`tokio::read_to_string`, `tokio::write`, etc.)

/// Async (tokio) filesystem wrappers.
pub mod tokio;

use std::{
    fs::{self, File, ReadDir},
    io,
    path::Path,
};

use atrium_error::{Error, ErrorKind, Result};
use serde::{de::DeserializeOwned, Serialize};

/// Creates an error from an I/O error with path context.
pub fn io_error(source: io::Error, operation: &'static str, path: &Path) -> Error {
    Error::from(source)
        .with_operation(operation)
        .with_context("path", path.display().to_string())
}

/// Creates an error for rename operations with both paths.
pub fn rename_error(source: io::Error, from: &Path, to: &Path) -> Error {
    Error::new(ErrorKind::Io, source.to_string())
        .with_operation("fs_rename")
        .with_context("from", from.display().to_string())
        .with_context("to", to.display().to_string())
        .set_source(source)
}

/// Wrapper for [`File::open`].
pub fn open(path: impl AsRef<Path>) -> Result<File> {
    let path = path.as_ref();
    File::open(path).map_err(|e| io_error(e, "fs_open", path))
}

/// Wrapper for `std::fs::read_to_string`.
pub fn read_to_string(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    fs::read_to_string(path).map_err(|e| io_error(e, "fs_read", path))
}

/// Read the entire contents of a file into a bytes vector.
pub fn read(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let path = path.as_ref();
    fs::read(path).map_err(|e| io_error(e, "fs_read", path))
}

/// Wrapper for `std::fs::write`.
pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
    let path = path.as_ref();
    fs::write(path, contents).map_err(|e| io_error(e, "fs_write", path))
}

/// Write contents to a file, creating parent directories if they don't exist.
pub fn write_and_create_parents(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    write(path, contents)
}

/// Wrapper for `File::create`.
pub fn create_file(path: impl AsRef<Path>) -> Result<File> {
    let path = path.as_ref();
    File::create(path).map_err(|e| io_error(e, "fs_create_file", path))
}

/// Wrapper for `std::fs::remove_file`.
pub fn remove_file(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    fs::remove_file(path).map_err(|e| io_error(e, "fs_remove_file", path))
}

/// Wrapper for `std::fs::remove_dir_all`.
pub fn remove_dir_all(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    fs::remove_dir_all(path).map_err(|e| io_error(e, "fs_remove_dir", path))
}

/// Wrapper for `std::fs::create_dir_all`.
pub fn create_dir_all(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    fs::create_dir_all(path).map_err(|e| io_error(e, "fs_create_dir", path))
}

/// Wrapper for `std::fs::read_dir`.
pub fn read_dir(path: impl AsRef<Path>) -> Result<ReadDir> {
    let path = path.as_ref();
    fs::read_dir(path).map_err(|e| io_error(e, "fs_read_dir", path))
}

/// Wrapper for `std::fs::rename`.
pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    fs::rename(from, to).map_err(|e| rename_error(e, from, to))
}

/// Wrapper for `std::fs::metadata`.
pub fn metadata(path: impl AsRef<Path>) -> Result<fs::Metadata> {
    let path = path.as_ref();
    fs::metadata(path).map_err(|e| io_error(e, "fs_metadata", path))
}

/// Check if a path exists.
pub fn exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

/// Read and deserialize a JSON file.
pub fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let path = path.as_ref();
    let content = read_to_string(path)?;
    serde_json::from_str(&content).map_err(|e| {
        Error::new(ErrorKind::DataInvalid, "failed to parse json")
            .with_operation("fs_read_json")
            .with_context("path", path.display().to_string())
            .set_source(e)
    })
}

/// Serialize and write a JSON file.
pub fn write_json<T: Serialize>(path: impl AsRef<Path>, value: &T) -> Result<()> {
    let path = path.as_ref();
    let content = serde_json::to_string_pretty(value).map_err(|e| {
        Error::new(ErrorKind::DataInvalid, "failed to serialize json")
            .with_operation("fs_write_json")
            .with_context("path", path.display().to_string())
            .set_source(e)
    })?;
    write_and_create_parents(path, content.as_bytes())
}

/// Cross-platform home directory resolution.
///
/// Checks `HOME` (Unix, Git Bash), `USERPROFILE` (Windows),
/// then `HOMEDRIVE`+`HOMEPATH` (legacy Windows).
pub fn home_dir() -> Option<std::path::PathBuf> {
    use std::{env, path::PathBuf};

    if let Some(home) = env::var_os("HOME") {
        return Some(PathBuf::from(home));
    }
    if let Some(home) = env::var_os("USERPROFILE") {
        return Some(PathBuf::from(home));
    }
    if let (Some(drive), Some(path)) = (env::var_os("HOMEDRIVE"), env::var_os("HOMEPATH")) {
        let mut home = PathBuf::from(drive);
        home.push(path);
        return Some(home);
    }
    None
}

/// Resolve a path relative to the user's home directory.
pub fn home_path(relative: impl AsRef<Path>) -> Option<std::path::PathBuf> {
    home_dir().map(|home| home.join(relative))
}
