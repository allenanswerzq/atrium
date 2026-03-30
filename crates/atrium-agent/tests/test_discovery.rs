//! Discovery and managed session integration tests.
//!
//! Tests agent process discovery, model discovery, and full lifecycle
//! management through AgentChatManager.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use tokio::sync::mpsc;

use atrium_agent::AgentKind;
use atrium_agent::discovery::{discover_agents, discover_models, is_installed};
use atrium_agent::session::AgentChatManager;
use atrium_agent::types::{AgentChatEvent, AgentChatStatus};
use atrium_executor::TaskManager;

async fn drain_turn(rx: &mut mpsc::UnboundedReceiver<AgentChatEvent>, secs: u64) -> String {
    let mut text = String::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(secs);
    loop {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Some(AgentChatEvent::MessageChunk { content })) => text.push_str(&content),
            Ok(Some(AgentChatEvent::TurnCompleted)) => return text,
            Ok(Some(AgentChatEvent::Error { message })) => panic!("error: {message}"),
            Ok(Some(_)) => {}
            Ok(None) => panic!("channel closed"),
            Err(_) => panic!("timed out after {secs}s"),
        }
    }
}

// ── Agent process discovery ─────────────────────────────────────────

#[tokio::test]
async fn discover_running_agents() {
    let agents = discover_agents();

    println!("found {} running agent processes:", agents.len());
    for a in &agents {
        println!(
            "  [{}] pid={} name={} acp={} cmd={:?}",
            a.kind, a.pid, a.name, a.is_acp, a.cmd,
        );
    }
}

#[tokio::test]
async fn is_installed_finds_copilot() {
    let path = is_installed(AgentKind::Copilot);
    if let Some(p) = &path {
        println!("copilot installed at: {}", p.display());
    } else {
        println!("copilot not on PATH — skipping");
    }
}

// ── Model discovery ─────────────────────────────────────────────────

#[tokio::test]
async fn discover_bridge_models_from_discovery_module() {
    if reqwest::get("http://localhost:5168").await.is_err() {
        println!("bridge not running — skipping");
        return;
    }

    let models = discover_models("http://localhost:5168/v1", None)
        .await
        .unwrap();

    assert!(!models.is_empty());
    println!("discovered {} models", models.len());
}

// ── Full lifecycle: discover → create → send → receive → kill ───────

#[tokio::test]
async fn managed_copilot_lifecycle() {
    // 1. Check copilot is installed
    if is_installed(AgentKind::Copilot).is_none() {
        println!("copilot not on PATH — skipping");
        return;
    }

    // 2. Create session through manager
    let cwd = std::env::current_dir().unwrap();
    let tm = TaskManager::current();
    let mut mgr = AgentChatManager::new(tm.executor());

    let (id, mut rx) = mgr.create(AgentKind::Copilot, cwd, None).await.unwrap();

    assert_eq!(mgr.list().len(), 1);
    let session = mgr.session(&id).unwrap();
    assert_eq!(session.status, AgentChatStatus::Idle);
    println!(
        "session created: {id} (transport: {})",
        session.transport_label()
    );

    // 3. Verify copilot is now running
    let running = discover_agents();
    let copilot_procs: Vec<_> = running
        .iter()
        .filter(|a| a.kind == AgentKind::Copilot)
        .collect();
    println!("copilot processes running: {}", copilot_procs.len());
    assert!(
        !copilot_procs.is_empty(),
        "expected at least one copilot process after creating session"
    );

    // 4. Send a message and receive response
    mgr.prompt(&id, "Reply with exactly: LIFECYCLE_OK".into())
        .await
        .unwrap();

    let text = drain_turn(&mut rx, 120).await;
    println!("response: {text}");
    assert!(text.contains("LIFECYCLE_OK"), "got: {text}");

    // 5. Kill the session
    mgr.kill(&id).unwrap();
    assert_eq!(mgr.session(&id).unwrap().status, AgentChatStatus::Exited);
    println!("session killed");

    // 6. Remove
    mgr.remove(&id);
    assert!(mgr.session(&id).is_none());
    assert!(mgr.list().is_empty());
    println!("session removed — lifecycle complete");
}
