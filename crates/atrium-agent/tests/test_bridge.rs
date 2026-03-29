//! Bridge server integration tests.
//!
//! Tests the OpenAI-compatible bridge on localhost:5168 that proxies
//! to Claude, GPT, Gemini etc. via VS Code's Copilot auth.
//!
//! Covers all three HTTP API formats:
//! - OpenAI Chat Completions (`/v1/chat/completions`)
//! - Anthropic Messages (`/v1/messages`)
//! - OpenAI Responses (`/v1/responses`)

#![allow(clippy::unwrap_used, clippy::expect_used)]

use atrium_agent::transport::{self, PromptRequest, TransportConfig};
use atrium_agent::types::{AgentChatEvent, ChatMessage};
use atrium_executor::TaskManager;
use tokio::sync::mpsc;

const BRIDGE_URL: &str = "http://localhost:5168/v1";

/// Helper: check if bridge is reachable, return false to skip.
async fn bridge_available() -> bool {
    reqwest::get("http://localhost:5168").await.is_ok()
}

/// Helper: build a single user ChatMessage.
fn user_message(content: &str) -> ChatMessage {
    ChatMessage {
        role: "user".to_owned(),
        content: content.to_owned(),
        tool_calls: Vec::new(),
        input_tokens: 0,
        output_tokens: 0,
        model_id: None,
        transport_label: None,
    }
}

/// Helper: run a prompt through a transport and collect response text.
async fn prompt_and_collect(
    t: &dyn atrium_agent::Transport,
    messages: &[ChatMessage],
    event_rx: &mut mpsc::UnboundedReceiver<AgentChatEvent>,
) -> String {
    let (_cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);

    let req = PromptRequest {
        messages,
        cancel_rx,
    };

    t.prompt(req).await.unwrap();

    let mut text = String::new();
    while let Ok(ev) = event_rx.try_recv() {
        if let AgentChatEvent::MessageChunk { content } = ev {
            text.push_str(&content);
        }
    }
    text
}

// ── OpenAI Chat Completions ─────────────────────────────────────────

#[tokio::test]
async fn bridge_openai_chat_completions() {
    if !bridge_available().await {
        println!("bridge not running — skipping");
        return;
    }

    let tm = TaskManager::current();
    let executor = tm.executor();
    let cwd = std::env::current_dir().unwrap();

    let cfg = TransportConfig::OpenAi {
        base_url: BRIDGE_URL.to_owned(),
        api_key: None,
        model: Some("claude-sonnet-4".to_owned()),
    };
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AgentChatEvent>();
    let t = transport::create(cfg, cwd, &executor, event_tx).await.unwrap();

    let messages = [user_message("Reply with exactly: CHAT_OK")];
    let text = prompt_and_collect(t.as_ref(), &messages, &mut event_rx).await;

    println!("[chat/completions] claude-sonnet-4: {text}");
    assert!(text.contains("CHAT_OK"), "got: {text}");
}

// ── Anthropic Messages ──────────────────────────────────────────────

#[tokio::test]
async fn bridge_anthropic_messages() {
    if !bridge_available().await {
        println!("bridge not running — skipping");
        return;
    }

    let tm = TaskManager::current();
    let executor = tm.executor();
    let cwd = std::env::current_dir().unwrap();

    let cfg = TransportConfig::Anthropic {
        base_url: BRIDGE_URL.to_owned(),
        api_key: None,
        model: Some("claude-sonnet-4".to_owned()),
    };
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AgentChatEvent>();
    let t = transport::create(cfg, cwd, &executor, event_tx).await.unwrap();

    let messages = [user_message("Say hello in one word")];
    let text = prompt_and_collect(t.as_ref(), &messages, &mut event_rx).await;

    println!("[messages] claude-sonnet-4: {text}");
    assert!(!text.is_empty(), "expected non-empty response, got nothing");
}

// ── OpenAI Responses ────────────────────────────────────────────────

#[tokio::test]
async fn bridge_openai_responses() {
    if !bridge_available().await {
        println!("bridge not running — skipping");
        return;
    }

    let tm = TaskManager::current();
    let executor = tm.executor();
    let cwd = std::env::current_dir().unwrap();

    let cfg = TransportConfig::Responses {
        base_url: BRIDGE_URL.to_owned(),
        api_key: None,
        model: Some("gpt-4.1".to_owned()),
    };
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AgentChatEvent>();
    let t = transport::create(cfg, cwd, &executor, event_tx).await.unwrap();

    let messages = [user_message("Reply with exactly: RESPONSES_OK")];
    let text = prompt_and_collect(t.as_ref(), &messages, &mut event_rx).await;

    println!("[responses] gpt-4.1: {text}");
    assert!(text.contains("RESPONSES_OK"), "got: {text}");
}

// ── Per-model tests (via chat/completions) ──────────────────────────

/// Helper: test a single model via `/v1/chat/completions`.
async fn test_model(model: &str) {
    if !bridge_available().await {
        println!("bridge not running — skipping {model}");
        return;
    }

    let tm = TaskManager::current();
    let executor = tm.executor();
    let cwd = std::env::current_dir().unwrap();

    let cfg = TransportConfig::OpenAi {
        base_url: BRIDGE_URL.to_owned(),
        api_key: None,
        model: Some(model.to_owned()),
    };
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AgentChatEvent>();
    let t = transport::create(cfg, cwd, &executor, event_tx).await.unwrap();

    let messages = [user_message("Say hello in one word")];
    let text = prompt_and_collect(t.as_ref(), &messages, &mut event_rx).await;

    println!("[{model}] {text}");
    assert!(!text.is_empty(), "{model} returned empty response");
}

// Claude models

#[tokio::test]
async fn model_claude_haiku_4_5() {
    test_model("claude-haiku-4.5").await;
}

#[tokio::test]
async fn model_claude_opus_4_5() {
    test_model("claude-opus-4.5").await;
}

#[tokio::test]
async fn model_claude_opus_4_6() {
    test_model("claude-opus-4.6").await;
}

#[tokio::test]
async fn model_claude_opus_4_6_1m() {
    test_model("claude-opus-4.6-1m").await;
}

#[tokio::test]
async fn model_claude_sonnet_4() {
    test_model("claude-sonnet-4").await;
}

#[tokio::test]
async fn model_claude_sonnet_4_5() {
    test_model("claude-sonnet-4.5").await;
}

#[tokio::test]
async fn model_claude_sonnet_4_6() {
    test_model("claude-sonnet-4.6").await;
}

// Gemini models

#[tokio::test]
async fn model_gemini_2_5_pro() {
    test_model("gemini-2.5-pro").await;
}

#[tokio::test]
async fn model_gemini_3_flash_preview() {
    test_model("gemini-3-flash-preview").await;
}

#[tokio::test]
async fn model_gemini_3_1_pro_preview() {
    test_model("gemini-3.1-pro-preview").await;
}

// GPT models

#[tokio::test]
async fn model_gpt_4_1() {
    test_model("gpt-4.1").await;
}

#[tokio::test]
async fn model_gpt_4o() {
    test_model("gpt-4o").await;
}

#[tokio::test]
async fn model_gpt_5_mini() {
    test_model("gpt-5-mini").await;
}

#[tokio::test]
async fn model_gpt_5_1() {
    test_model("gpt-5.1").await;
}

#[tokio::test]
async fn model_gpt_5_1_codex() {
    test_model("gpt-5.1-codex").await;
}

#[tokio::test]
async fn model_gpt_5_1_codex_max() {
    test_model("gpt-5.1-codex-max").await;
}

#[tokio::test]
async fn model_gpt_5_1_codex_mini() {
    test_model("gpt-5.1-codex-mini").await;
}

#[tokio::test]
async fn model_gpt_5_2() {
    test_model("gpt-5.2").await;
}

#[tokio::test]
async fn model_gpt_5_2_codex() {
    test_model("gpt-5.2-codex").await;
}

#[tokio::test]
async fn model_gpt_5_3_codex() {
    test_model("gpt-5.3-codex").await;
}

#[tokio::test]
async fn model_gpt_5_4() {
    test_model("gpt-5.4").await;
}

#[tokio::test]
async fn model_gpt_5_4_mini() {
    test_model("gpt-5.4-mini").await;
}
