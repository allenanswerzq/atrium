//! Terminal PTY — spawn a shell and provide read/write access.
//!
//! Wraps `portable-pty` to provide a clean API for the session layer.
//! This module only deals with the OS-level PTY; it knows nothing about
//! emulation, sessions, or rendering.

use std::io::{Read, Write};

use atrium_error::Result;
use parking_lot::Mutex;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};

//  Our GUI App                          Shell process (cmd.exe)
//  ┌─────────┐                          ┌──────────┐
//  │         │──── write("ls\r") ──────→│          │
//  │  MASTER │                          │  SLAVE   │
//  │  side   │←── read("file1 file2") ──│  side    │
//  │         │                          │          │
//  └─────────┘                          └──────────┘
//       ↑                                    ↑
//    We control this                    Shell thinks this
//    (send keystrokes,                  is a real terminal
//     read output)                      (keyboard + screen)

/// Handle to a spawned PTY process.
///
/// Owns the master PTY, child process, and writer. The reader is
/// returned separately from [`TerminalPty::spawn`] for the caller to
/// consume in a background thread.
pub struct TerminalPty {
    writer: Mutex<Box<dyn Write + Send>>,
    _master: Box<dyn MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
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
        drop(pair.slave);

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        Ok((
            Self {
                writer: Mutex::new(writer),
                _master: pair.master,
                child,
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

    /// Kill the child process.
    pub fn kill(&mut self) {
        let _ = self.child.kill();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::time::{Duration, Instant};

    fn test_shell() -> String {
        if cfg!(windows) {
            std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_owned())
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_owned())
        }
    }

    fn test_cwd() -> std::path::PathBuf {
        std::env::current_dir().unwrap()
    }

    #[test]
    fn spawn_creates_pty_and_reader() {
        let (mut pty, reader) = TerminalPty::spawn(&test_shell(), &test_cwd(), 80, 24).unwrap();
        drop(reader);
        pty.kill();
    }

    #[test]
    fn spawn_with_different_sizes() {
        let (mut p1, r1) = TerminalPty::spawn(&test_shell(), &test_cwd(), 120, 40).unwrap();
        let (mut p2, r2) = TerminalPty::spawn(&test_shell(), &test_cwd(), 40, 10).unwrap();
        drop(r1);
        drop(r2);
        p1.kill();
        p2.kill();
    }

    #[test]
    fn spawn_invalid_shell_returns_error() {
        let result = TerminalPty::spawn(
            "nonexistent_shell_binary_xyz",
            &test_cwd(),
            80,
            24,
        );
        assert!(result.is_err());
    }

    #[test]
    fn reader_produces_output() {
        let (mut pty, mut reader) = TerminalPty::spawn(&test_shell(), &test_cwd(), 80, 24).unwrap();

        let mut buf = [0u8; 4096];
        let mut total = 0;
        let deadline = Instant::now() + Duration::from_secs(3);

        while Instant::now() < deadline {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    total += n;
                    break;
                }
                Err(_) => break,
            }
        }

        assert!(total > 0, "expected shell to produce output (prompt)");
        drop(reader);
        pty.kill();
    }

    #[test]
    fn write_sends_to_shell() {
        let (mut pty, mut reader) = TerminalPty::spawn(&test_shell(), &test_cwd(), 80, 24).unwrap();

        std::thread::sleep(Duration::from_millis(500));
        pty.write(b"echo PTY_TEST_OUTPUT\r\n").unwrap();

        let mut all_output = String::new();
        let mut buf = [0u8; 4096];
        let deadline = Instant::now() + Duration::from_secs(5);

        while Instant::now() < deadline {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    all_output.push_str(&String::from_utf8_lossy(&buf[..n]));
                    if all_output.contains("PTY_TEST_OUTPUT") {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        assert!(
            all_output.contains("PTY_TEST_OUTPUT"),
            "expected to find PTY_TEST_OUTPUT in output, got: {all_output}"
        );
        drop(reader);
        pty.kill();
    }

    #[test]
    fn write_multiple_times() {
        let (mut pty, reader) = TerminalPty::spawn(&test_shell(), &test_cwd(), 80, 24).unwrap();
        pty.write(b"a").unwrap();
        pty.write(b"b").unwrap();
        pty.write(b"c\r\n").unwrap();
        drop(reader);
        pty.kill();
    }
}
