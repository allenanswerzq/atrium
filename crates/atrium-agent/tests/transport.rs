//! Integration tests for the agent transport layer.
//!
//! Tests marked `#[ignore]` require real CLI tools or API endpoints.
//! Run them with:
//!
//!   cargo test -p atrium-agent --test transport -- --ignored

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::time::Duration;

use atrium_agent::session::AgentChatManager;
use atrium_agent::transport::TransportConfig;
use atrium_agent::types::{AgentChatEvent, AgentChatStatus};
use atrium_agent::AgentKind;

use tokio::sync::broadcast;

// ── Helpers ─────────────────────────────────────────────────────────

fn copilot_terminal_config() -> TransportConfig {
    TransportConfig::Terminal {
        program: "copilot".to_owned(),
        base_args: vec!["--allow-all".to_owned(), "-s".to_owned()],
    }
}

fn copilot_acp_config() -> TransportConfig {
    TransportConfig::Acp {
        program: "copilot".to_owned(),
        args: vec!["--acp".to_owned()],
    }
}

async fn collect_turn(rx: &mut broadcast::Receiver<AgentChatEvent>, timeout_secs: u64) -> String {
    let mut response = String::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        let event = tokio::time::timeout_at(deadline, rx.recv())
            .await
            .expect("timed out")
            .expect("channel closed");
        match event {
            AgentChatEvent::MessageChunk { content } => response.push_str(&content),
            AgentChatEvent::TurnCompleted => break,
            AgentChatEvent::Error { message } => panic!("error: {message}"),
            _ => {}
        }
    }
    response
}

// ── Manager lifecycle ───────────────────────────────────────────────

#[tokio::test]
async fn manager_create_and_list() {
    let mut mgr = AgentChatManager::new();
    let (id1, _) = mgr
        .create(AgentKind::Copilot, PathBuf::from("."), None, copilot_terminal_config())
        .await
        .unwrap();
    let (id2, _) = mgr
        .create(AgentKind::Claude, PathBuf::from("."), None, copilot_terminal_config())
        .await
        .unwrap();

    let list = mgr.list();
    assert_eq!(list.len(), 2);
    assert!(list.iter().any(|s| s.id == id1.to_string()));
    assert!(list.iter().any(|s| s.id == id2.to_string()));
}

#[tokio::test]
async fn manager_kill_and_remove() {
    let mut mgr = AgentChatManager::new();
    let (id, mut rx) = mgr
        .create(AgentKind::Copilot, PathBuf::from("."), None, copilot_terminal_config())
        .await
        .unwrap();

    mgr.kill(&id).unwrap();
    let event = rx.try_recv().unwrap();
    assert!(matches!(event, AgentChatEvent::SessionExited { .. }));
    assert_eq!(mgr.session(&id).unwrap().status, AgentChatStatus::Exited);

    mgr.remove(&id);
    assert!(mgr.session(&id).is_none());
}

#[tokio::test]
async fn manager_send_rejects_when_working() {
    let mut mgr = AgentChatManager::new();
    let (id, _) = mgr
        .create(AgentKind::Copilot, PathBuf::from("."), None, copilot_terminal_config())
        .await
        .unwrap();

    assert!(mgr.send_message(&id, "first".to_owned()).is_ok());
    let err = mgr.send_message(&id, "second".to_owned());
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("already processing"));
}

#[tokio::test]
async fn manager_turn_event_sequence() {
    let mut mgr = AgentChatManager::new();
    let (id, mut rx) = mgr
        .create(AgentKind::Copilot, PathBuf::from("."), None, copilot_terminal_config())
        .await
        .unwrap();

    mgr.send_message(&id, "hello".to_owned()).unwrap();

    let mut got_user = false;
    let mut got_started = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let event = tokio::time::timeout_at(deadline, rx.recv())
            .await.expect("timed out").expect("closed");
        match event {
            AgentChatEvent::UserMessage { .. } => got_user = true,
            AgentChatEvent::TurnStarted => got_started = true,
            AgentChatEvent::TurnCompleted => break,
            AgentChatEvent::Error { .. } => {}
            _ => {}
        }
    }
    assert!(got_user);
    assert!(got_started);
}

// ── Real CLI tests ──────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires copilot CLI"]
async fn copilot_terminal_single_turn() {
    let cwd = std::env::current_dir().unwrap();
    let mut mgr = AgentChatManager::new();
    let (id, mut rx) = mgr
        .create(AgentKind::Copilot, cwd, None, copilot_terminal_config())
        .await
        .unwrap();

    mgr.send_message(&id, "Reply with exactly: TERMINAL_OK".to_owned()).unwrap();
    let response = collect_turn(&mut rx, 60).await;
    assert!(response.contains("TERMINAL_OK"), "got: {response}");
}

#[tokio::test]
#[ignore = "requires copilot CLI with ACP support"]
async fn copilot_acp_single_turn() {
    let cwd = std::env::current_dir().unwrap();
    let mut mgr = AgentChatManager::new();
    let (id, mut rx) = mgr
        .create(AgentKind::Copilot, cwd, None, copilot_acp_config())
        .await
        .unwrap();

    mgr.send_message(&id, "Reply with exactly: ACP_OK".to_owned()).unwrap();
    let response = collect_turn(&mut rx, 120).await;
    assert!(response.contains("ACP_OK"), "got: {response}");
}

#[tokio::test]
#[ignore = "requires copilot CLI with ACP support"]
async fn copilot_acp_multi_turn() {
    let cwd = std::env::current_dir().unwrap();
    let mut mgr = AgentChatManager::new();
    let (id, mut rx) = mgr
        .create(AgentKind::Copilot, cwd, None, copilot_acp_config())
        .await
        .unwrap();

    // Turn 1
    mgr.send_message(&id, "Remember the code word: PINEAPPLE".to_owned()).unwrap();
    let t1 = collect_turn(&mut rx, 120).await;
    assert!(!t1.is_empty(), "turn 1 empty");

    // Finalize turn 1
    tokio::time::sleep(Duration::from_millis(200)).await;
    {
        let s = mgr.session_mut(&id).unwrap();
        s.pending_text = t1;
        s.finalize_turn();
    }

    // Turn 2 — agent should remember from turn 1
    mgr.send_message(&id, "What was the code word I told you?".to_owned()).unwrap();
    let t2 = collect_turn(&mut rx, 120).await;
    assert!(
        t2.to_uppercase().contains("PINEAPPLE"),
        "agent forgot: {t2}"
    );
}
