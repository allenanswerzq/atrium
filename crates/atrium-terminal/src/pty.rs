//! Terminal PTY — spawn a shell and provide read/write access.
//!
//! Wraps `portable-pty` to provide a clean API for the session layer.
//! This module only deals with the OS-level PTY; it knows nothing about
//! emulation, sessions, or rendering.

use std::io::{Read, Write};
use std::sync::Arc;

use atrium_error::Result;
use parking_lot::Mutex;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};

/// Handle to a spawned PTY process.
///
/// Owns the master PTY, child process, and writer. The reader is
/// returned separately from [`TerminalPty::spawn`] for the caller to
/// consume in a background thread.
pub struct TerminalPty {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    _master: Box<dyn MasterPty + Send>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl TerminalPty {
    /// Spawn a shell in a new PTY.
    ///
    /// Returns `(handle, reader)`. The reader blocks until output is
    /// available — it must be read from a dedicated thread.
    pub fn spawn(
        shell: &str,
        cwd: &std::path::Path,
        cols: u16,
        rows: u16,
    ) -> Result<(Self, Box<dyn Read + Send>)> {
        let pty_system = native_pty_system();

        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })?;

        let mut cmd = CommandBuilder::new(shell);
        cmd.cwd(cwd);

        let child = pair.slave.spawn_command(cmd)?;

        // Must drop the slave after spawn on Windows (ConPTY requirement).
        drop(pair.slave);

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        Ok((
            Self {
                writer: Arc::new(Mutex::new(writer)),
                _master: pair.master,
                _child: child,
            },
            reader,
        ))
    }

    /// Send bytes to the PTY's stdin.
    pub fn write(&self, data: &[u8]) -> Result<()> {
        let mut w = self.writer.lock();
        w.write_all(data)?;
        w.flush()?;
        Ok(())
    }
}
