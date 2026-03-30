//! Transport integration tests.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use tokio::sync::mpsc;

use atrium_agent::AgentKind;
use atrium_agent::session::AgentChatManager;
use atrium_agent::transport::TransportConfig;
use atrium_agent::types::{AgentChatEvent, AgentChatStatus, ChatMessage};
use atrium_executor::TaskManager;

fn new_test_manager() -> (AgentChatManager, TaskManager) {
    let tm = TaskManager::current();
    let mgr = AgentChatManager::new(tm.executor());
    (mgr, tm)
}

async fn drain_turn(rx: &mut mpsc::UnboundedReceiver<AgentChatEvent>, secs: u64) -> String {
    let mut text = String::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(secs);
    loop {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Some(AgentChatEvent::MessageChunk { content })) => text.push_str(&content),
            Ok(Some(AgentChatEvent::TurnCompleted)) => return text,
            Ok(Some(AgentChatEvent::Error { message })) => panic!("transport error: {message}"),
            Ok(Some(_)) => {}
            Ok(None) => panic!("channel closed"),
            Err(_) => panic!("timed out after {secs}s"),
        }
    }
}

#[tokio::test]
async fn create_two_sessions_and_list() {
    let (mut mgr, _tm) = new_test_manager();
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
    let (mut mgr, _tm) = new_test_manager();
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
    let (mut mgr, _tm) = new_test_manager();
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

async fn event_sequence_on_failed_turn() {
    let (mut mgr, _tm) = new_test_manager();
    let cfg = TransportConfig::Terminal {
        program: "no-such-program-xyz".into(),
        base_args: vec![],
    };
    let (id, mut rx) = mgr
        .create_with_config(AgentKind::Copilot, ".".into(), None, cfg)
        .await
        .unwrap();
    mgr.prompt(&id, "hello".into()).await.unwrap();

    let mut events = Vec::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while let Ok(Some(ev)) = tokio::time::timeout_at(deadline, rx.recv()).await {
        let done = matches!(ev, AgentChatEvent::TurnCompleted);
        events.push(ev);
        if done {
            break;
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
async fn cancel_terminal_turn() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let (mgr, _tm) = new_test_manager();
    let mgr = Arc::new(Mutex::new(mgr));

    // Use a command that runs forever (ping -t on Windows).
    let cfg = TransportConfig::Terminal {
        program: "ping".into(),
        base_args: vec!["-t".into(), "127.0.0.1".into()],
    };
    let (id, mut rx) = mgr
        .lock()
        .await
        .create_with_config(AgentKind::Copilot, ".".into(), None, cfg)
        .await
        .unwrap();

    // Spawn the prompt in a background task.
    let mgr2 = Arc::clone(&mgr);
    let id2 = id.clone();
    let prompt_task =
        tokio::spawn(async move { mgr2.lock().await.prompt(&id2, "ignored".into()).await });

    // Wait for the process to start, then cancel.
    tokio::time::sleep(Duration::from_millis(500)).await;
    let start = std::time::Instant::now();
    mgr.lock().await.cancel(&id).unwrap();
    let _ = prompt_task.await;
    let elapsed = start.elapsed();

    println!("cancel took {elapsed:?}");
    assert!(
        elapsed < Duration::from_secs(5),
        "cancel was too slow: {elapsed:?}"
    );

    // Should have received events.
    let mut got_error = false;
    let mut got_completed = false;
    while let Ok(ev) = rx.try_recv() {
        match ev {
            AgentChatEvent::Error { .. } => got_error = true,
            AgentChatEvent::TurnCompleted => got_completed = true,
            _ => {}
        }
    }
    assert!(
        got_error || got_completed,
        "expected Error or TurnCompleted after cancel"
    );
}

#[tokio::test]
async fn cancel_via_manager() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let (mgr, _tm) = new_test_manager();
    let mgr = Arc::new(Mutex::new(mgr));

    let cfg = TransportConfig::Terminal {
        program: "ping".into(),
        base_args: vec!["-t".into(), "127.0.0.1".into()],
    };
    let (id, mut rx) = mgr
        .lock()
        .await
        .create_with_config(AgentKind::Copilot, ".".into(), None, cfg)
        .await
        .unwrap();

    // Spawn prompt in background, cancel from main task.
    let mgr2 = Arc::clone(&mgr);
    let id2 = id.clone();
    let prompt_task =
        tokio::spawn(async move { mgr2.lock().await.prompt(&id2, "ignored".into()).await });

    tokio::time::sleep(Duration::from_millis(500)).await;
    mgr.lock().await.cancel(&id).unwrap();
    let _ = prompt_task.await;

    // Verify completion events.
    let mut got_completed = false;
    while let Ok(ev) = rx.try_recv() {
        if matches!(ev, AgentChatEvent::TurnCompleted) {
            got_completed = true;
        }
    }
    assert!(got_completed, "expected TurnCompleted after manager cancel");
}

#[tokio::test]
async fn transport_label_terminal() {
    let (mut mgr, _tm) = new_test_manager();
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
    let (mut mgr, _tm) = new_test_manager();
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
        TransportConfig::Anthropic {
            base_url: "https://api.anthropic.com/v1".into(),
            api_key: Some("sk-ant-test".into()),
            model: Some("claude-sonnet-4".into()),
        },
        TransportConfig::Responses {
            base_url: "https://api.openai.com/v1".into(),
            api_key: Some("sk-test".into()),
            model: Some("gpt-4.1".into()),
        },
    ] {
        let json = serde_json::to_string(&cfg).unwrap();
        let _: TransportConfig = serde_json::from_str(&json).unwrap();
    }
}

#[tokio::test]
async fn copilot_terminal_single_turn() {
    let cwd = std::env::current_dir().unwrap();
    let (mut mgr, _tm) = new_test_manager();
    let cfg = AgentKind::Copilot.default_terminal_config();
    let (id, mut rx) = mgr
        .create_with_config(AgentKind::Copilot, cwd, None, cfg)
        .await
        .unwrap();

    mgr.prompt(&id, "Reply with exactly: TERMINAL_OK".into())
        .await
        .unwrap();
    let text = drain_turn(&mut rx, 60).await;
    assert!(text.contains("TERMINAL_OK"), "got: {text}");
}

#[tokio::test]
async fn copilot_acp_single_turn() {
    let cwd = std::env::current_dir().unwrap();
    let (mut mgr, _tm) = new_test_manager();
    let (id, mut rx) = mgr.create(AgentKind::Copilot, cwd, None).await.unwrap();

    mgr.prompt(&id, "Reply with exactly: ACP_SINGLE_OK".into())
        .await
        .unwrap();
    let text = drain_turn(&mut rx, 120).await;
    assert!(text.contains("ACP_SINGLE_OK"), "got: {text}");
}

#[tokio::test]
async fn copilot_acp_multi_turn() {
    let cwd = std::env::current_dir().unwrap();
    let (mut mgr, _tm) = new_test_manager();
    let (id, mut rx) = mgr.create(AgentKind::Copilot, cwd, None).await.unwrap();

    // Turn 1: tell it a secret.
    mgr.prompt(&id, "Remember this code word: MANGO".into())
        .await
        .unwrap();
    let t1 = drain_turn(&mut rx, 120).await;
    println!("turn 1 output:\n{t1}");
    assert!(!t1.is_empty(), "turn 1 was empty");

    // Finalize turn 1.
    tokio::time::sleep(Duration::from_millis(200)).await;
    {
        let s = mgr.session_mut(&id).unwrap();
        s.pending_text = t1;
        s.finalize_turn();
    }

    // Turn 2: agent should remember via ACP session.
    mgr.prompt(&id, "What code word did I tell you?".into())
        .await
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

#[tokio::test(flavor = "multi_thread")]
async fn copilot_acp_100_sessions() {
    use atrium_agent::transport::{self, PromptRequest};
    use tokio_util::sync::CancellationToken;

    const N: usize = 100;
    let cwd = std::env::current_dir().unwrap();
    let tm = TaskManager::current();
    let executor = tm.executor();

    // Create 100 ACP transports directly.
    let mut transports = Vec::with_capacity(N);
    for i in 0..N {
        let cfg = AgentKind::Copilot.default_acp_config();
        let (event_tx, event_rx) = mpsc::unbounded_channel::<AgentChatEvent>();
        let t = transport::create(cfg, cwd.clone(), &executor, event_tx)
            .await
            .unwrap_or_else(|e| panic!("failed to create transport {i}: {e}"));
        transports.push((t, event_rx));
    }
    println!("all {N} transports created");

    // Send prompts to all sessions in parallel.
    let start = std::time::Instant::now();
    let mut handles = Vec::with_capacity(N);
    for (i, (t, mut event_rx)) in transports.into_iter().enumerate() {
        handles.push(tokio::spawn(async move {
            let msg = format!("Reply with exactly: BATCH_{i}");
            let messages = [ChatMessage {
                role: "user".to_owned(),
                content: msg,
                tool_calls: Vec::new(),
                input_tokens: 0,
                output_tokens: 0,
                model_id: None,
                transport_label: None,
            }];
            let req = PromptRequest {
                messages: &messages,
                cancel: CancellationToken::new(),
            };
            t.prompt(req)
                .await
                .unwrap_or_else(|e| panic!("session {i} prompt failed: {e}"));

            // Collect all message chunks from the mpsc receiver.
            let mut text = String::new();
            while let Ok(ev) = event_rx.try_recv() {
                if let AgentChatEvent::MessageChunk { content } = ev {
                    text.push_str(&content);
                }
            }

            let marker = format!("BATCH_{i}");
            assert!(
                text.contains(&marker),
                "session {i} missing marker {marker}: {text}"
            );
            println!("session {i}: {} chars", text.len());
            i
        }));
    }

    let mut success = 0;
    for handle in handles {
        handle.await.unwrap();
        success += 1;
    }
    let elapsed = start.elapsed();
    println!("all {success}/{N} sessions completed in {elapsed:.1?}");
    assert_eq!(success, N);
}
