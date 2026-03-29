//! Transport integration tests.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use tokio::sync::broadcast;

use atrium_agent::session::AgentChatManager;
use atrium_agent::transport::TransportConfig;
use atrium_agent::types::{AgentChatEvent, AgentChatStatus};
use atrium_agent::{AgentKind, ErrorKind};

async fn drain_turn(rx: &mut broadcast::Receiver<AgentChatEvent>, secs: u64) -> String {
    let mut text = String::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(secs);
    loop {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Ok(AgentChatEvent::MessageChunk { content })) => text.push_str(&content),
            Ok(Ok(AgentChatEvent::TurnCompleted)) => return text,
            Ok(Ok(AgentChatEvent::Error { message })) => panic!("transport error: {message}"),
            Ok(Ok(_)) => {}
            Ok(Err(e)) => panic!("channel error: {e}"),
            Err(_) => panic!("timed out after {secs}s"),
        }
    }
}

#[tokio::test]
async fn create_two_sessions_and_list() {
    let mut mgr = AgentChatManager::new();
    let cfg = AgentKind::Copilot.default_terminal_config();
    let (id1, _) = mgr
        .create_with_config(AgentKind::Copilot, ".".into(), None, cfg.clone())
        .await
        .unwrap();
    let (id2, _) = mgr
        .create_with_config(AgentKind::Claude, ".".into(), None, cfg)
        .await
        .unwrap();

    assert_eq!(mgr.list().len(), 2);
    assert!(mgr.session(&id1).is_some());
    assert!(mgr.session(&id2).is_some());
}

#[tokio::test]
async fn kill_sets_exited_and_fires_event() {
    let mut mgr = AgentChatManager::new();
    let cfg = AgentKind::Copilot.default_terminal_config();
    let (id, mut rx) = mgr
        .create_with_config(AgentKind::Copilot, ".".into(), None, cfg)
        .await
        .unwrap();

    mgr.kill(&id).unwrap();
    let ev = rx.try_recv().unwrap();
    assert!(matches!(ev, AgentChatEvent::SessionExited { .. }));
    assert_eq!(mgr.session(&id).unwrap().status, AgentChatStatus::Exited);
}

#[tokio::test]
async fn remove_deletes_session() {
    let mut mgr = AgentChatManager::new();
    let cfg = AgentKind::Copilot.default_terminal_config();
    let (id, _) = mgr
        .create_with_config(AgentKind::Copilot, ".".into(), None, cfg)
        .await
        .unwrap();
    mgr.remove(&id);
    assert!(mgr.session(&id).is_none());
    assert!(mgr.list().is_empty());
}

#[tokio::test]
async fn send_rejects_while_working() {
    let mut mgr = AgentChatManager::new();
    let cfg = AgentKind::Copilot.default_terminal_config();
    let (id, _) = mgr
        .create_with_config(AgentKind::Copilot, ".".into(), None, cfg)
        .await
        .unwrap();
    let _ = mgr.send_message(&id, "one".into());
    let err = mgr.send_message(&id, "two".into());
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().kind(), ErrorKind::InvalidInput);
}

#[tokio::test]
async fn event_sequence_on_failed_turn() {
    let mut mgr = AgentChatManager::new();
    let cfg = TransportConfig::Terminal {
        program: "no-such-program-xyz".into(),
        base_args: vec![],
    };
    let (id, mut rx) = mgr
        .create_with_config(AgentKind::Copilot, ".".into(), None, cfg)
        .await
        .unwrap();
    mgr.send_message(&id, "hello".into()).unwrap();

    let mut events = Vec::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Ok(ev)) => {
                let done = matches!(ev, AgentChatEvent::TurnCompleted);
                events.push(ev);
                if done {
                    break;
                }
            }
            _ => break,
        }
    }
    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentChatEvent::UserMessage { .. }))
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentChatEvent::TurnStarted))
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentChatEvent::Error { .. }))
    );
    assert!(matches!(events.last(), Some(AgentChatEvent::TurnCompleted)));
}

#[tokio::test]
async fn transport_label_terminal() {
    let mut mgr = AgentChatManager::new();
    let cfg = AgentKind::Copilot.default_terminal_config();
    let (id, _) = mgr
        .create_with_config(AgentKind::Copilot, ".".into(), None, cfg)
        .await
        .unwrap();
    assert_eq!(
        mgr.session(&id).unwrap().transport_label(),
        "terminal:copilot"
    );
}

#[tokio::test]
async fn transport_label_openai() {
    let mut mgr = AgentChatManager::new();
    let cfg = TransportConfig::OpenAi {
        base_url: "http://localhost:11434/v1".into(),
        api_key: None,
        model: None,
    };
    let (id, _) = mgr
        .create_with_config(AgentKind::Copilot, ".".into(), None, cfg)
        .await
        .unwrap();
    assert_eq!(
        mgr.session(&id).unwrap().transport_label(),
        "openai:http://localhost:11434/v1"
    );
}

#[test]
fn config_serde_roundtrip() {
    for cfg in [
        AgentKind::Copilot.default_acp_config(),
        AgentKind::Copilot.default_terminal_config(),
        TransportConfig::OpenAi {
            base_url: "https://api.openai.com/v1".into(),
            api_key: Some("sk-test".into()),
            model: Some("gpt-4".into()),
        },
    ] {
        let json = serde_json::to_string(&cfg).unwrap();
        let _: TransportConfig = serde_json::from_str(&json).unwrap();
    }
}

#[tokio::test]
async fn copilot_terminal_single_turn() {
    let cwd = std::env::current_dir().unwrap();
    let mut mgr = AgentChatManager::new();
    let cfg = AgentKind::Copilot.default_terminal_config();
    let (id, mut rx) = mgr
        .create_with_config(AgentKind::Copilot, cwd, None, cfg)
        .await
        .unwrap();

    mgr.send_message(&id, "Reply with exactly: TERMINAL_OK".into())
        .unwrap();
    let text = drain_turn(&mut rx, 60).await;
    assert!(text.contains("TERMINAL_OK"), "got: {text}");
}

#[tokio::test]
async fn copilot_acp_single_turn() {
    let cwd = std::env::current_dir().unwrap();
    let mut mgr = AgentChatManager::new();
    let (id, mut rx) = mgr.create(AgentKind::Copilot, cwd, None).await.unwrap();

    mgr.send_message(&id, "Reply with exactly: ACP_SINGLE_OK".into())
        .unwrap();
    let text = drain_turn(&mut rx, 120).await;
    assert!(text.contains("ACP_SINGLE_OK"), "got: {text}");
}

#[tokio::test]
async fn copilot_acp_multi_turn() {
    let cwd = std::env::current_dir().unwrap();
    let mut mgr = AgentChatManager::new();
    let (id, mut rx) = mgr.create(AgentKind::Copilot, cwd, None).await.unwrap();

    // Turn 1: tell it a secret.
    mgr.send_message(&id, "Remember this code word: MANGO".into())
        .unwrap();
    let t1 = drain_turn(&mut rx, 120).await;
    assert!(!t1.is_empty(), "turn 1 was empty");

    // Finalize turn 1.
    tokio::time::sleep(Duration::from_millis(200)).await;
    {
        let s = mgr.session_mut(&id).unwrap();
        s.pending_text = t1;
        s.finalize_turn();
    }

    // Turn 2: agent should remember via ACP session.
    mgr.send_message(&id, "What code word did I tell you?".into())
        .unwrap();
    let t2 = drain_turn(&mut rx, 120).await;
    assert!(t2.to_uppercase().contains("MANGO"), "agent forgot: {t2}");
}

#[test]
fn kind_default_terminal_is_structured() {
    let cursor = AgentKind::Cursor.default_terminal_config();
    let gemini = AgentKind::Gemini.default_terminal_config();
    let opencode = AgentKind::OpenCode.default_terminal_config();
    let pi = AgentKind::Pi.default_terminal_config();

    assert!(
        matches!(cursor, TransportConfig::Terminal { program, base_args } if program == "cursor" && base_args.is_empty())
    );
    assert!(
        matches!(gemini, TransportConfig::Terminal { program, base_args } if program == "gemini" && base_args.is_empty())
    );
    assert!(
        matches!(opencode, TransportConfig::Terminal { program, base_args } if program == "opencode" && base_args.is_empty())
    );
    assert!(
        matches!(pi, TransportConfig::Terminal { program, base_args } if program == "pi" && base_args.is_empty())
    );
}

#[test]
fn kind_default_acp_falls_back_when_needed() {
    let cfg = AgentKind::Cursor.default_acp_config();
    assert!(
        matches!(cfg, TransportConfig::Terminal { program, base_args } if program == "cursor" && base_args.is_empty())
    );
}
