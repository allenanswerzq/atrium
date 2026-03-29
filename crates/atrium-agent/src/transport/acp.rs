//! ACP transport — thin JSON-RPC 2.0 client over newline-delimited stdio.
//!
//! Fully `Send + Sync` — no dedicated thread, no `LocalSet`, no `Rc`.
//! Spawns the agent binary once (e.g. `copilot --acp`), runs an I/O reader
//! via `tokio::spawn`, and communicates using plain JSON-RPC messages.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use atrium_error::{Error, ErrorKind, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin};
use tokio::sync::{Mutex, oneshot};

use super::{EventSender, PromptRequest, Transport};
use crate::types::AgentChatEvent;

// ── JSON-RPC wire types ─────────────────────────────────────────────

/// Monotonic JSON-RPC request identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
struct RequestId(i64);

#[derive(serde::Serialize)]
struct RpcRequest<P: serde::Serialize> {
    jsonrpc: &'static str,
    id: RequestId,
    method: &'static str,
    params: P,
}

#[derive(serde::Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    id: serde_json::Value,
    result: serde_json::Value,
}

#[derive(serde::Serialize)]
struct RpcError {
    jsonrpc: &'static str,
    id: serde_json::Value,
    error: RpcErrorBody,
}

#[derive(serde::Serialize)]
struct RpcErrorBody {
    code: i32,
    message: &'static str,
}

// ── ACP param types (just what we need) ─────────────────────────────

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InitParams {
    protocol_version: u16,
    client_info: ClientInfo,
}

#[derive(serde::Serialize)]
struct ClientInfo {
    name: &'static str,
    version: &'static str,
    title: &'static str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct NewSessionParams {
    cwd: String,
    mcp_servers: Vec<()>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PromptParams {
    session_id: String,
    prompt: Vec<serde_json::Value>,
}

// ── Shared connection state ─────────────────────────────────────────

/// Max time to wait for a single JSON-RPC response (handshake or prompt).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

type PendingMap = HashMap<RequestId, oneshot::Sender<serde_json::Value>>;

struct AcpConn {
    writer: Mutex<ChildStdin>,
    pending: Mutex<PendingMap>,
    next_id: AtomicI64,
}

impl AcpConn {
    async fn request<P: serde::Serialize>(
        &self,
        method: &'static str,
        params: P,
    ) -> Result<serde_json::Value> {
        let id = RequestId(self.next_id.fetch_add(1, Ordering::Relaxed));
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);

        let msg = RpcRequest {
            jsonrpc: "2.0",
            id,
            method,
            params,
        };
        let mut line = serde_json::to_string(&msg)?;
        line.push('\n');

        self.writer.lock().await.write_all(line.as_bytes()).await?;

        match tokio::time::timeout(REQUEST_TIMEOUT, rx).await {
            Ok(Ok(val)) => Ok(val),
            Ok(Err(_)) => Err(Error::new(
                ErrorKind::Unexpected,
                format!("ACP request `{method}` channel closed"),
            )),
            Err(_) => {
                // Clean up the pending entry on timeout.
                self.pending.lock().await.remove(&id);
                Err(Error::new(
                    ErrorKind::Network,
                    format!("ACP request `{method}` timed out after {REQUEST_TIMEOUT:?}"),
                ))
            }
        }
    }

    async fn respond(&self, id: serde_json::Value, result: serde_json::Value) {
        let msg = RpcResponse {
            jsonrpc: "2.0",
            id,
            result,
        };
        if let Ok(mut line) = serde_json::to_string(&msg) {
            line.push('\n');
            let _ = self.writer.lock().await.write_all(line.as_bytes()).await;
        }
    }

    async fn respond_error(&self, id: serde_json::Value, code: i32, message: &'static str) {
        let msg = RpcError {
            jsonrpc: "2.0",
            id,
            error: RpcErrorBody { code, message },
        };
        if let Ok(mut line) = serde_json::to_string(&msg) {
            line.push('\n');
            let _ = self.writer.lock().await.write_all(line.as_bytes()).await;
        }
    }
}

// ── Transport ───────────────────────────────────────────────────────

/// Long-lived ACP transport. Fully `Send + Sync` — no background thread.
pub struct AcpTransport {
    conn: Arc<AcpConn>,
    session_id: String,
    label: String,
}

impl AcpTransport {
    /// Spawn the agent process and perform the ACP handshake.
    pub async fn spawn(
        program: String,
        args: Vec<String>,
        workspace_path: PathBuf,
        executor: &atrium_executor::TaskExecutor,
        event_tx: EventSender,
    ) -> Result<Self> {
        let label = format!("acp:{program}");

        let mut child = tokio::process::Command::new(&program)
            .args(&args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                Error::new(ErrorKind::Io, format!("failed to spawn `{program}`")).set_source(e)
            })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::new(ErrorKind::Io, "no stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::new(ErrorKind::Io, "no stdout"))?;

        let conn = Arc::new(AcpConn {
            writer: Mutex::new(stdin),
            pending: Mutex::new(HashMap::new()),
            next_id: AtomicI64::new(0),
        });

        // Spawn the reader as a regular tokio task (Send!).
        Self::spawn_reader(executor, Arc::clone(&conn), stdout, event_tx, child);

        // Initialize.
        conn.request(
            "initialize",
            InitParams {
                protocol_version: 1,
                client_info: ClientInfo {
                    name: "atrium",
                    version: "0.1.0",
                    title: "Atrium",
                },
            },
        )
        .await
        .map_err(|e| e.with_operation("acp::initialize"))?;

        // Create session.
        let cwd = workspace_path
            .canonicalize()
            .unwrap_or(workspace_path)
            .display()
            .to_string();

        let resp = conn
            .request(
                "session/new",
                NewSessionParams {
                    cwd,
                    mcp_servers: vec![],
                },
            )
            .await
            .map_err(|e| e.with_operation("acp::new_session"))?;

        let session_id = resp
            .get("sessionId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::new(ErrorKind::DataInvalid, "missing sessionId in response"))?
            .to_owned();

        tracing::info!(session_id = %session_id, label = %label, "ACP transport ready");

        Ok(Self {
            conn,
            session_id,
            label,
        })
    }

    /// Spawn on the executor a read loop that dispatches responses and notifications.
    fn spawn_reader(
        executor: &atrium_executor::TaskExecutor,
        conn: Arc<AcpConn>,
        stdout: tokio::process::ChildStdout,
        event_tx: EventSender,
        child: Child,
    ) {
        executor.spawn(async move {
            let _child = child; // keep alive; kill_on_drop handles cleanup
            let mut lines = BufReader::new(stdout).lines();

            while let Ok(Some(line)) = lines.next_line().await {
                let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) else {
                    continue;
                };

                // Response to our request (has "id", no "method").
                if msg.get("id").is_some() && msg.get("method").is_none() {
                    let id_num: RequestId =
                        serde_json::from_value(msg["id"].clone()).unwrap_or(RequestId(-1));
                    if let Some(tx) = conn.pending.lock().await.remove(&id_num) {
                        if let Some(result) = msg.get("result") {
                            let _ = tx.send(result.clone());
                        } else if let Some(err) = msg.get("error") {
                            let _ = tx.send(err.clone());
                        }
                    }
                    continue;
                }

                let method = msg
                    .get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                match method {
                    // Session notification (no id).
                    "session/update" => {
                        if let Some(params) = msg.get("params") {
                            Self::handle_notification(params, &event_tx);
                        }
                    }
                    // Permission request — auto-approve.
                    "session/requestPermission" => {
                        if let Some(id) = msg.get("id") {
                            let option_id = msg
                                .pointer("/params/options/0/optionId")
                                .and_then(|v| v.as_str())
                                .unwrap_or("allow");
                            conn.respond(
                                id.clone(),
                                serde_json::json!({
                                    "outcome": {
                                        "selected": { "optionId": option_id }
                                    }
                                }),
                            )
                            .await;
                        }
                    }
                    // Unknown request → method_not_found.
                    _ if msg.get("id").is_some() => {
                        if let Some(id) = msg.get("id") {
                            conn.respond_error(id.clone(), -32601, "method not found")
                                .await;
                        }
                    }
                    _ => {}
                }
            }

            // Agent exited — fail any pending requests.
            for (_, tx) in conn.pending.lock().await.drain() {
                let _ = tx.send(serde_json::json!(null));
            }
        });
    }

    fn handle_notification(
        params: &serde_json::Value,
        event_tx: &EventSender,
    ) {
        let Some(update) = params.get("update") else {
            return;
        };
        let kind = update
            .get("sessionUpdate")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        match kind {
            "agent_message_chunk" => {
                if let Some(text) = update.pointer("/content/text").and_then(|v| v.as_str()) {
                    let _ = event_tx.send(AgentChatEvent::MessageChunk {
                        content: text.to_owned(),
                    });
                }
            }
            "agent_thought_chunk" => {
                if let Some(text) = update.pointer("/content/text").and_then(|v| v.as_str()) {
                    let _ = event_tx.send(AgentChatEvent::ThoughtChunk {
                        content: text.to_owned(),
                    });
                }
            }
            "tool_call" => {
                let name = update
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let status = update
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let _ = event_tx.send(AgentChatEvent::ToolCall {
                    name: name.to_owned(),
                    status: status.to_owned(),
                });
            }
            "tool_call_update" => {
                let name = update
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let status = update
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let _ = event_tx.send(AgentChatEvent::ToolCall {
                    name: name.to_owned(),
                    status: status.to_owned(),
                });
            }
            _ => {
                tracing::trace!(kind, "unhandled ACP session update");
            }
        }
    }
}

#[async_trait::async_trait]
impl Transport for AcpTransport {
    async fn prompt(&self, req: PromptRequest<'_>) -> Result<()> {
        let conn = Arc::clone(&self.conn);
        let session_id = self.session_id.clone();
        let text = req.last_user_message().to_owned();
        let mut cancel_rx = req.cancel_rx;

        let prompt_fut = async move {
            conn.request(
                "session/prompt",
                PromptParams {
                    session_id,
                    prompt: vec![serde_json::json!({"type": "text", "text": text})],
                },
            )
            .await
        };

        tokio::select! {
            result = prompt_fut => {
                result.map_err(|e| e.with_operation("acp::prompt"))?;
                Ok(())
            }
            _ = cancel_rx.changed() => {
                if *cancel_rx.borrow() {
                    Err(Error::new(ErrorKind::Cancelled, "turn cancelled"))
                } else {
                    Err(Error::new(ErrorKind::Unexpected, "cancel channel closed"))
                }
            }
        }
    }

    async fn shutdown(&self) {
        // Dropping the writer closes stdin → agent exits via kill_on_drop.
    }

    fn label(&self) -> String {
        self.label.clone()
    }
}
