//! Architecture Agent — produces [`ViewSpec`] from raw graph data.
//!
//! Takes structured graph input (nodes, edges, ranks from llmcc) and a user
//! request, sends them to an LLM, and parses the response into a renderable
//! [`ViewSpec`].
//!
//! The agent handles all visual decisions: layout positions, grouping,
//! styling, labels, and click actions. The UI renderer is dumb — it just
//! draws what the agent specifies.

mod prompt;

use atrium_agent_sdk::transport::{self, PromptRequest, TransportConfig};
use atrium_agent_sdk::types::{AgentChatEvent, ChatMessage};
use atrium_error::{Error, ErrorKind, Result};
use atrium_graph_view::ViewSpec;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub use prompt::SYSTEM_PROMPT;

/// Input graph data — the raw material the agent works with.
///
/// Produced by `atrium-graph` (or hand-crafted for testing).
/// Contains the nodes, edges, and metadata from llmcc, but no
/// positions or visual styling.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphData {
    /// Project name.
    pub project: String,
    /// Nodes in the graph.
    pub nodes: Vec<GraphNode>,
    /// Edges in the graph.
    pub edges: Vec<GraphEdge>,
}

/// A raw graph node — no position, no style.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    #[serde(default)]
    pub child_count: u32,
    #[serde(default)]
    pub rank: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

/// A raw graph edge — no waypoints, no style.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
}

/// Request to the Architecture Agent.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchitectRequest {
    /// What the user wants to see.
    pub query: String,
    /// Viewport dimensions for layout.
    pub viewport_width: f32,
    pub viewport_height: f32,
    /// The raw graph data to work with.
    pub graph_data: GraphData,
}

/// Architecture Agent — stateless, creates a transport per call.
pub struct ArchitectAgent {
    config: TransportConfig,
}

impl ArchitectAgent {
    /// Create an agent that talks to an OpenAI-compatible endpoint.
    pub fn new(config: TransportConfig) -> Self {
        Self { config }
    }

    /// Create an agent pointing at the local bridge server.
    pub fn bridge(model: &str) -> Self {
        Self {
            config: TransportConfig::OpenAi {
                base_url: "http://localhost:5168/v1".to_owned(),
                api_key: None,
                model: Some(model.to_owned()),
            },
        }
    }

    /// Ask the agent to produce a [`ViewSpec`] for the given request.
    pub async fn render(
        &self,
        request: &ArchitectRequest,
        executor: &atrium_executor::TaskExecutor,
    ) -> Result<ViewSpec> {
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AgentChatEvent>();

        let transport = transport::create(
            self.config.clone(),
            std::env::current_dir().unwrap_or_default(),
            executor,
            event_tx,
        )
        .await?;

        // Build the user message: system prompt context + the actual request.
        let user_content = build_user_message(request)?;

        let messages = [
            ChatMessage {
                role: "system".to_owned(),
                content: SYSTEM_PROMPT.to_owned(),
                tool_calls: Vec::new(),
                input_tokens: 0,
                output_tokens: 0,
                model_id: None,
                transport_label: None,
            },
            ChatMessage {
                role: "user".to_owned(),
                content: user_content,
                tool_calls: Vec::new(),
                input_tokens: 0,
                output_tokens: 0,
                model_id: None,
                transport_label: None,
            },
        ];

        let cancel = CancellationToken::new();
        let req = PromptRequest {
            messages: &messages,
            cancel,
        };

        transport.prompt(req).await?;

        // Collect response text from events.
        let mut text = String::new();
        while let Ok(ev) = event_rx.try_recv() {
            if let AgentChatEvent::MessageChunk { content } = ev {
                text.push_str(&content);
            }
        }

        // Extract JSON from the response (agent may wrap it in markdown code blocks).
        let json = extract_json(&text)?;
        let spec = ViewSpec::from_json(json).map_err(|e| {
            Error::new(
                ErrorKind::DataInvalid,
                format!("failed to parse ViewSpec from agent response: {e}"),
            )
            .with_context("raw_response", text.chars().take(500).collect::<String>())
        })?;

        Ok(spec)
    }
}

/// Build the user message content from the request.
fn build_user_message(request: &ArchitectRequest) -> Result<String> {
    let graph_json = serde_json::to_string_pretty(&request.graph_data)?;

    Ok(format!(
        "Viewport: {}×{} pixels\n\n\
         Graph data:\n```json\n{graph_json}\n```\n\n\
         Request: {}",
        request.viewport_width, request.viewport_height, request.query
    ))
}

/// Extract JSON from the agent response, handling markdown code fences.
fn extract_json(text: &str) -> Result<&str> {
    // Try to find JSON inside ```json ... ``` blocks
    if let Some(start) = text.find("```json") {
        let json_start = start + 7; // skip ```json
        if let Some(end) = text[json_start..].find("```") {
            return Ok(text[json_start..json_start + end].trim());
        }
    }

    // Try to find JSON inside ``` ... ``` blocks
    if let Some(start) = text.find("```") {
        let content_start = start + 3;
        // Skip optional language identifier on the same line
        let line_end = text[content_start..]
            .find('\n')
            .map(|i| content_start + i + 1)
            .unwrap_or(content_start);
        if let Some(end) = text[line_end..].find("```") {
            return Ok(text[line_end..line_end + end].trim());
        }
    }

    // Try to find raw JSON (starts with {)
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return Ok(text[start..=end].trim());
        }
    }

    Err(Error::new(
        ErrorKind::DataInvalid,
        "no JSON found in agent response",
    ))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn extract_json_from_code_block() {
        let text = r#"Here's the view:

```json
{"title": "test", "nodes": [], "edges": []}
```

Done!"#;
        let json = extract_json(text).unwrap();
        assert!(json.starts_with('{'));
        let spec: ViewSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.title, "test");
    }

    #[test]
    fn extract_json_raw() {
        let text = r#"{"title": "raw", "nodes": [], "edges": []}"#;
        let json = extract_json(text).unwrap();
        let spec: ViewSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.title, "raw");
    }

    #[test]
    fn graph_data_roundtrip() {
        let data = GraphData {
            project: "test".to_owned(),
            nodes: vec![GraphNode {
                id: "a".to_owned(),
                label: "A".to_owned(),
                kind: "crate".to_owned(),
                child_count: 5,
                rank: 0.8,
                file: Some("src/lib.rs".to_owned()),
                line: Some(1),
            }],
            edges: vec![GraphEdge {
                from: "a".to_owned(),
                to: "b".to_owned(),
                relation: "depends_on".to_owned(),
            }],
        };

        let json = serde_json::to_string(&data).unwrap();
        let parsed: GraphData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.project, "test");
        assert_eq!(parsed.nodes.len(), 1);
    }
}
