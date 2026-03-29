//! Terminal transport — spawns the agent CLI per turn.
//!
//! Single-turn: each prompt spawns a fresh subprocess with `-p`. Useful for
//! quick tests or agents without ACP support.

use std::path::PathBuf;

use atrium_error::{Error, ErrorKind, Result};
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;

use super::{PromptRequest, Transport};
use crate::types::AgentChatEvent;

/// Per-turn subprocess transport.
pub struct TerminalTransport {
    program: String,
    base_args: Vec<String>,
    workspace_path: PathBuf,
}

impl TerminalTransport {
    pub fn new(program: String, base_args: Vec<String>, workspace_path: PathBuf) -> Self {
        Self {
            program,
            base_args,
            workspace_path,
        }
    }
}

#[async_trait::async_trait]
impl Transport for TerminalTransport {
    async fn prompt(&self, req: PromptRequest<'_>) -> Result<()> {
        let mut args = self.base_args.clone();
        args.push("-p".to_owned());
        args.push(req.prompt.to_owned());

        tracing::info!(
            program = %self.program,
            args = ?args,
            cwd = %self.workspace_path.display(),
            "terminal: spawning turn"
        );

        let mut child = Command::new(&self.program)
            .args(&args)
            .current_dir(&self.workspace_path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                Error::new(
                    ErrorKind::Io,
                    format!("failed to spawn `{}`: {e}", self.program),
                )
                .set_source(e)
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::new(ErrorKind::Io, "no stdout"))?;
        let mut lines = tokio::io::BufReader::new(stdout).lines();
        let mut cancel_rx = req.cancel_rx;

        loop {
            tokio::select! {
                biased;
                _ = cancel_rx.changed() => {
                    if *cancel_rx.borrow() {
                        let _ = child.kill().await;
                        return Err(Error::new(ErrorKind::Cancelled, "turn cancelled"));
                    }
                }
                line_result = lines.next_line() => {
                    match line_result {
                        Ok(Some(line)) if !line.trim().is_empty() => {
                            let _ = req.event_tx.send(AgentChatEvent::MessageChunk {
                                content: format!("{line}\n"),
                            });
                        }
                        Ok(Some(_)) => {}
                        Ok(None) => break,
                        Err(e) => {
                            let _ = req.event_tx.send(AgentChatEvent::Error {
                                message: format!("stdout read error: {e}"),
                            });
                            break;
                        }
                    }
                }
            }
        }

        let stderr_text = read_stderr(child.stderr.take()).await;
        let status = child
            .wait()
            .await
            .map_err(|e| Error::new(ErrorKind::Io, format!("wait error: {e}")).set_source(e))?;
        if !status.success() {
            let code = status.code().unwrap_or(-1);
            let detail = if stderr_text.is_empty() {
                format!("`{}` exited with code {code}", self.program)
            } else {
                format!(
                    "`{}` exited with code {code}: {}",
                    self.program,
                    stderr_text.trim()
                )
            };
            return Err(Error::new(ErrorKind::Unexpected, detail)
                .with_context("program", &self.program)
                .with_context("exit_code", code.to_string()));
        }
        Ok(())
    }

    async fn shutdown(&self) {
        // Nothing to do — no persistent process.
    }

    fn label(&self) -> String {
        format!("terminal:{}", self.program)
    }
}

/// Read up to 4KB from an optional stderr handle.
async fn read_stderr(stderr: Option<tokio::process::ChildStderr>) -> String {
    use tokio::io::{AsyncBufReadExt, BufReader};
    let Some(stderr) = stderr else {
        return String::new();
    };
    let mut buf = String::new();
    let mut reader = BufReader::new(stderr);
    let mut line_buf = String::new();
    while buf.len() < 4096 {
        match reader.read_line(&mut line_buf).await {
            Ok(0) => break,
            Ok(_) => {
                buf.push_str(&line_buf);
                line_buf.clear();
            }
            Err(_) => break,
        }
    }
    buf
}
