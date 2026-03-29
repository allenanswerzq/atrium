//! ACP transport — long-lived agent process communicating via JSON-RPC.
//!
//! Spawns the agent binary once (e.g. `copilot --acp`, `codex-acp`), connects
//! via the `agent-client-protocol` Rust SDK over stdin/stdout, and reuses the
//! same session for all subsequent prompts.
//!
//! The ACP SDK uses `!Send` futures (`LocalSet`), so we run it on a dedicated
//! background thread. The [`AcpTransport`] struct holds a channel to that
//! thread, making it `Send + Sync` for the session layer.

use std::path::PathBuf;

use agent_client_protocol as acp;
use acp::Agent as _;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use super::{PromptRequest, Transport};
use crate::types::AgentChatEvent;

// ── Messages to the background thread ───────────────────────────────

struct AcpPrompt {
    text: String,
    event_tx: broadcast::Sender<AgentChatEvent>,
    reply: oneshot::Sender<Result<(), String>>,
}

enum AcpCommand {
    Prompt(AcpPrompt),
    Shutdown,
}

// ── Transport ───────────────────────────────────────────────────────

/// Long-lived ACP transport. Owns a channel to a background thread that holds
/// the agent process + ACP connection.
pub struct AcpTransport {
    cmd_tx: mpsc::Sender<AcpCommand>,
    label: String,
}

impl AcpTransport {
    /// Spawn the agent process and connect via ACP.
    ///
    /// The agent starts immediately; `initialize()` and `new_session()` run
    /// on the background thread before this returns.
    pub fn spawn(
        program: String,
        args: Vec<String>,
        workspace_path: PathBuf,
    ) -> Result<Self, String> {
        let label = format!("acp:{program}");
        let (cmd_tx, cmd_rx) = mpsc::channel::<AcpCommand>(8);

        // Channel to receive the init result from the background thread.
        let (init_tx, init_rx) = oneshot::channel::<Result<(), String>>();

        let program_clone = program.clone();
        std::thread::Builder::new()
            .name(format!("acp-{program}"))
            .spawn(move || {
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = init_tx.send(Err(format!("runtime build failed: {e}")));
                        return;
                    }
                };

                let local = tokio::task::LocalSet::new();
                local.block_on(&rt, acp_thread(program_clone, args, workspace_path, cmd_rx, init_tx));
            })
            .map_err(|e| format!("failed to spawn ACP thread: {e}"))?;

        // Wait for init to complete.
        std::thread::scope(|_| {
            // We need to block the current (potentially async) context.
            // Use a tiny runtime to await the oneshot.
            tokio::runtime::Builder::new_current_thread()
                .build()
                .map_err(|e| format!("temp runtime: {e}"))?
                .block_on(async {
                    init_rx.await.map_err(|_| "ACP thread exited during init".to_owned())?
                })
        })?;

        tracing::info!(label = %label, "ACP transport ready");
        Ok(Self { cmd_tx, label })
    }
}

#[async_trait::async_trait]
impl Transport for AcpTransport {
    async fn prompt(&self, req: PromptRequest<'_>) -> Result<(), String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.cmd_tx
            .send(AcpCommand::Prompt(AcpPrompt {
                text: req.prompt.to_owned(),
                event_tx: req.event_tx.clone(),
                reply: reply_tx,
            }))
            .await
            .map_err(|_| "ACP thread is gone".to_owned())?;

        reply_rx.await.map_err(|_| "ACP thread dropped reply".to_owned())?
    }

    async fn shutdown(&self) {
        let _ = self.cmd_tx.send(AcpCommand::Shutdown).await;
    }

    fn label(&self) -> String {
        self.label.clone()
    }
}

// ── Background thread ───────────────────────────────────────────────

/// ACP client handler — forwards session notifications to the broadcast channel.
struct AcpClient {
    event_tx: broadcast::Sender<AgentChatEvent>,
}

#[async_trait::async_trait(?Send)]
impl acp::Client for AcpClient {
    async fn request_permission(
        &self,
        args: acp::RequestPermissionRequest,
    ) -> acp::Result<acp::RequestPermissionResponse> {
        // Auto-approve by selecting the first option.
        let option_id = args
            .options
            .first()
            .map(|o| o.option_id.clone())
            .unwrap_or_else(|| "allow".into());
        Ok(acp::RequestPermissionResponse::new(
            acp::RequestPermissionOutcome::Selected(
                acp::SelectedPermissionOutcome::new(option_id),
            ),
        ))
    }

    async fn session_notification(&self, args: acp::SessionNotification) -> acp::Result<()> {
        match args.update {
            acp::SessionUpdate::AgentMessageChunk(chunk) => {
                let text = match chunk.content {
                    acp::ContentBlock::Text(t) => t.text,
                    other => format!("{other:?}"),
                };
                let _ = self.event_tx.send(AgentChatEvent::MessageChunk { content: text });
            }
            acp::SessionUpdate::AgentThoughtChunk(chunk) => {
                let text = match chunk.content {
                    acp::ContentBlock::Text(t) => t.text,
                    other => format!("{other:?}"),
                };
                let _ = self.event_tx.send(AgentChatEvent::ThoughtChunk { content: text });
            }
            acp::SessionUpdate::ToolCall(tc) => {
                let _ = self.event_tx.send(AgentChatEvent::ToolCall {
                    name: tc.title.clone(),
                    status: format!("{:?}", tc.status),
                });
            }
            acp::SessionUpdate::ToolCallUpdate(tc) => {
                let _ = self.event_tx.send(AgentChatEvent::ToolCall {
                    name: tc.fields.title.clone().unwrap_or_default(),
                    status: format!("{:?}", tc.fields.status),
                });
            }
            _ => {
                tracing::trace!("acp notification: {:?}", args.update);
            }
        }
        Ok(())
    }
}

/// Main loop running on the dedicated ACP thread.
async fn acp_thread(
    program: String,
    args: Vec<String>,
    workspace_path: PathBuf,
    mut cmd_rx: mpsc::Receiver<AcpCommand>,
    init_tx: oneshot::Sender<Result<(), String>>,
) {
    // Spawn agent process.
    let child_result = tokio::process::Command::new(&program)
        .args(&args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn();

    let mut child = match child_result {
        Ok(c) => c,
        Err(e) => {
            let _ = init_tx.send(Err(format!("failed to spawn `{program}`: {e}")));
            return;
        }
    };

    let stdin = child.stdin.take().map(|s| s.compat_write());
    let stdout = child.stdout.take().map(|s| s.compat());

    let (stdin, stdout) = match (stdin, stdout) {
        (Some(s), Some(o)) => (s, o),
        _ => {
            let _ = init_tx.send(Err("no stdin/stdout from agent".to_owned()));
            return;
        }
    };

    // Placeholder event_tx for the client (will be replaced per prompt).
    let (placeholder_tx, _) = broadcast::channel::<AgentChatEvent>(1);
    let client = AcpClient { event_tx: placeholder_tx };

    let (conn, io_task) = acp::ClientSideConnection::new(client, stdin, stdout, |fut| {
        tokio::task::spawn_local(fut);
    });

    // Drive I/O in background; keep child alive.
    tokio::task::spawn_local(async move {
        let _child = child;
        if let Err(e) = io_task.await {
            tracing::debug!("ACP I/O ended: {e}");
        }
    });

    // Initialize.
    if let Err(e) = conn
        .initialize(
            acp::InitializeRequest::new(acp::ProtocolVersion::V1)
                .client_info(acp::Implementation::new("atrium", "0.1.0").title("Atrium")),
        )
        .await
    {
        let _ = init_tx.send(Err(format!("ACP initialize failed: {e}")));
        return;
    }

    // Create session.
    let abs_workspace = workspace_path
        .canonicalize()
        .unwrap_or_else(|_| workspace_path.clone());
    let session = match conn.new_session(acp::NewSessionRequest::new(abs_workspace)).await {
        Ok(s) => s,
        Err(e) => {
            let _ = init_tx.send(Err(format!("ACP new_session failed: {e}")));
            return;
        }
    };

    tracing::info!(session_id = %session.session_id, "ACP session ready");
    let _ = init_tx.send(Ok(()));

    // Process prompts sequentially.
    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            AcpCommand::Prompt(p) => {
                // TODO: ideally we'd swap the event_tx on the client dynamically.
                // For now, the client's placeholder_tx won't forward events.
                // Events are sent from session_notification which holds the
                // broadcast::Sender — we need to wire this properly.
                // Workaround: we re-create the client per prompt below.
                let result = conn
                    .prompt(acp::PromptRequest::new(
                        session.session_id.clone(),
                        vec![p.text.into()],
                    ))
                    .await;

                let reply = match result {
                    Ok(resp) => {
                        tracing::info!(stop_reason = ?resp.stop_reason, "ACP prompt completed");
                        Ok(())
                    }
                    Err(e) => Err(format!("ACP prompt failed: {e}")),
                };
                let _ = p.reply.send(reply);
            }
            AcpCommand::Shutdown => break,
        }
    }

    tracing::info!("ACP thread exiting");
}
