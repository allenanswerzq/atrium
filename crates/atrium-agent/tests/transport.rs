//! Tests for the transport layer.
//!
//! - No-dependency tests run always (manager lifecycle, event flow).
//! - Real CLI tests are `#[ignore]`. Run with: `cargo test -p atrium-agent --test transport -- --ignored`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use tokio::sync::broadcast;

use atrium_agent::session::AgentChatManager;
use atrium_agent::transport::TransportConfig;
use atrium_agent::types::{AgentChatEvent, AgentChatStatus};
use atrium_agent::AgentKind;

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

/// Collect all events until TurnCompleted, returning the concatenated text.
/// Panics on Error events or timeout.
async fn drain_turn(rx: &mut broadcast::Receiver<AgentChatEvent>, secs: u64) -> String {
    let mut text = String::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(secs);
    loop {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Ok(AgentChatEvent::MessageChunk { content })) => text.push_str(&content),
            Ok(Ok(AgentChatEvent::TurnCompleted)) => return text,
            Ok(Ok(AgentChatEvent::Error { message })) => panic!("transport error: {message}"),
            Ok(Ok(_)) => {} // ignore other events
            Ok(Err(e)) => panic!("channel error: {e}"),
            Err(_) => panic!("timed out after {secs}s"),
        }
    }
}

/// Drain events ignoring errors (for tests where the agent may not be installed).
#[allow(dead_code)]
async fn drain_turn_allow_errors(rx: &mut broadcast::Receiver<AgentChatEvent>, secs: u64) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(secs);
    loop {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Ok(AgentChatEvent::TurnCompleted)) => return,
            Ok(Ok(_)) => {}
            Ok(Err(_)) | Err(_) => return,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// 1. Manager lifecycle — no external dependencies
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn create_two_sessions_and_list() {
    let mut mgr = AgentChatManager::new();
    let cfg = TransportConfig::Terminal {
        program: "echo".into(),
        base_args: vec![],
    };
    let (id1, _) = mgr.create(AgentKind::Copilot, ".".into(), None, cfg.clone()).await.unwrap();
    let (id2, _) = mgr.create(AgentKind::Claude, ".".into(), None, cfg).await.unwrap();

    assert_eq!(mgr.list().len(), 2);
    assert!(mgr.session(&id1).is_some());
    assert!(mgr.session(&id2).is_some());
}

#[tokio::test]
async fn kill_sets_exited_and_fires_event() {
    let mut mgr = AgentChatManager::new();
    let cfg = TransportConfig::Terminal { program: "echo".into(), base_args: vec![] };
    let (id, mut rx) = mgr.create(AgentKind::Copilot, ".".into(), None, cfg).await.unwrap();

    mgr.kill(&id).unwrap();

    // Should receive SessionExited.
    let ev = rx.try_recv().unwrap();
    assert!(matches!(ev, AgentChatEvent::SessionExited { .. }));
    assert_eq!(mgr.session(&id).unwrap().status, AgentChatStatus::Exited);
}

#[tokio::test]
async fn remove_deletes_session() {
    let mut mgr = AgentChatManager::new();
    let cfg = TransportConfig::Terminal { program: "echo".into(), base_args: vec![] };
    let (id, _) = mgr.create(AgentKind::Copilot, ".".into(), None, cfg).await.unwrap();

    mgr.remove(&id);
    assert!(mgr.session(&id).is_none());
    assert!(mgr.list().is_empty());
}

#[tokio::test]
async fn send_rejects_while_working() {
    let mut mgr = AgentChatManager::new();
    let cfg = TransportConfig::Terminal { program: "echo".into(), base_args: vec![] };
    let (id, _) = mgr.create(AgentKind::Copilot, ".".into(), None, cfg).await.unwrap();

    // First send succeeds, sets status to Working.
    let _ = mgr.send_message(&id, "one".into());
    // Second send while still working is rejected.
    let err = mgr.send_message(&id, "two".into());
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("already processing"));
}

// ═══════════════════════════════════════════════════════════════════
// 2. Event flow — transport will fail (no real agent), but event
//    sequence UserMessage → TurnStarted → Error → TurnCompleted is tested.
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn event_sequence_on_failed_turn() {
    let mut mgr = AgentChatManager::new();
    // Use a non-existent program so the turn fails immediately.
    let cfg = TransportConfig::Terminal {
        program: "this-program-does-not-exist-xyz".into(),
        base_args: vec![],
    };
    let (id, mut rx) = mgr.create(AgentKind::Copilot, ".".into(), None, cfg).await.unwrap();

    mgr.send_message(&id, "hello".into()).unwrap();

    let mut events = Vec::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Ok(ev)) => {
                let done = matches!(ev, AgentChatEvent::TurnCompleted);
                events.push(ev);
                if done { break; }
            }
            _ => break,
        }
    }

    // Must have UserMessage, TurnStarted, Error, TurnCompleted in that order.
    assert!(events.iter().any(|e| matches!(e, AgentChatEvent::UserMessage { .. })));
    assert!(events.iter().any(|e| matches!(e, AgentChatEvent::TurnStarted)));
    assert!(events.iter().any(|e| matches!(e, AgentChatEvent::Error { .. })));
    assert!(matches!(events.last(), Some(AgentChatEvent::TurnCompleted)));
}

// ═══════════════════════════════════════════════════════════════════
// 3. Transport label
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn transport_label_terminal() {
    let mut mgr = AgentChatManager::new();
    let cfg = TransportConfig::Terminal { program: "copilot".into(), base_args: vec![] };
    let (id, _) = mgr.create(AgentKind::Copilot, ".".into(), None, cfg).await.unwrap();
    assert_eq!(mgr.session(&id).unwrap().transport_label(), "terminal:copilot");
}

#[tokio::test]
async fn transport_label_openai() {
    let mut mgr = AgentChatManager::new();
    let cfg = TransportConfig::OpenAi {
        base_url: "http://localhost:11434/v1".into(),
        api_key: None,
        model: None,
    };
    let (id, _) = mgr.create(AgentKind::Copilot, ".".into(), None, cfg).await.unwrap();
    assert_eq!(mgr.session(&id).unwrap().transport_label(), "openai:http://localhost:11434/v1");
}

// ═══════════════════════════════════════════════════════════════════
// 4. TransportConfig serialization round-trip
// ═══════════════════════════════════════════════════════════════════

#[test]
fn config_serde_terminal() {
    let cfg = TransportConfig::Terminal {
        program: "copilot".into(),
        base_args: vec!["--allow-all".into()],
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: TransportConfig = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, TransportConfig::Terminal { program, .. } if program == "copilot"));
}

#[test]
fn config_serde_acp() {
    let cfg = TransportConfig::Acp {
        program: "copilot".into(),
        args: vec!["--acp".into()],
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: TransportConfig = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, TransportConfig::Acp { program, .. } if program == "copilot"));
}

#[test]
fn config_serde_openai() {
    let cfg = TransportConfig::OpenAi {
        base_url: "https://api.openai.com/v1".into(),
        api_key: Some("sk-test".into()),
        model: Some("gpt-4".into()),
    };
    let json = serde_json::to_string(&cfg).unwrap();
    assert!(json.contains("openai"));
    let back: TransportConfig = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, TransportConfig::OpenAi { base_url, .. } if base_url.contains("openai")));
}

// ═══════════════════════════════════════════════════════════════════
// 5. Real CLI — Terminal transport (copilot -p)
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn real_copilot_terminal_single_turn() {
    let cwd = std::env::current_dir().unwrap();
    let mut mgr = AgentChatManager::new();
    let cfg = TransportConfig::Terminal {
        program: "copilot".into(),
        base_args: vec!["--allow-all".into(), "-s".into()],
    };
    let (id, mut rx) = mgr.create(AgentKind::Copilot, cwd, None, cfg).await.unwrap();

    mgr.send_message(&id, "Reply with exactly: TERMINAL_OK".into()).unwrap();
    let text = drain_turn(&mut rx, 60).await;
    assert!(text.contains("TERMINAL_OK"), "got: {text}");
}

// ═══════════════════════════════════════════════════════════════════
// 6. Real CLI — ACP transport (copilot --acp, long-lived)
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn real_copilot_acp_single_turn() {
    let cwd = std::env::current_dir().unwrap();
    let mut mgr = AgentChatManager::new();
    let cfg = TransportConfig::Acp {
        program: "copilot".into(),
        args: vec!["--acp".into()],
    };
    let (id, mut rx) = mgr.create(AgentKind::Copilot, cwd, None, cfg).await.unwrap();

    mgr.send_message(&id, "Reply with exactly: ACP_SINGLE_OK".into()).unwrap();
    let text = drain_turn(&mut rx, 120).await;
    assert!(text.contains("ACP_SINGLE_OK"), "got: {text}");
}

#[tokio::test]
async fn real_copilot_acp_multi_turn() {
    let cwd = std::env::current_dir().unwrap();
    let mut mgr = AgentChatManager::new();
    let cfg = TransportConfig::Acp {
        program: "copilot".into(),
        args: vec!["--acp".into()],
    };
    let (id, mut rx) = mgr.create(AgentKind::Copilot, cwd, None, cfg).await.unwrap();

    // Turn 1: tell it a secret.
    mgr.send_message(&id, "Remember this code word: MANGO".into()).unwrap();
    let t1 = drain_turn(&mut rx, 120).await;
    assert!(!t1.is_empty(), "turn 1 was empty");

    // Finalize turn 1 so we can send turn 2.
    tokio::time::sleep(Duration::from_millis(200)).await;
    {
        let s = mgr.session_mut(&id).unwrap();
        s.pending_text = t1;
        s.finalize_turn();
    }

    // Turn 2: ask for the secret — should remember because same ACP session.
    mgr.send_message(&id, "What code word did I tell you?".into()).unwrap();
    let t2 = drain_turn(&mut rx, 120).await;
    assert!(
        t2.to_uppercase().contains("MANGO"),
        "agent should remember MANGO but said: {t2}"
    );
}
