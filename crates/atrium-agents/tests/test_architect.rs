//! Integration tests for the Architecture Agent.
//!
//! Uses the bridge server (localhost:5168) to call a real LLM and verify
//! it produces valid ViewSpec JSON.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use atrium_agents::architect::{ArchitectAgent, ArchitectRequest, GraphData, GraphEdge, GraphNode};
use atrium_graph_view::ViewSpec;
use atrium_executor::TaskManager;

/// Fake graph data representing the atrium crate map.
fn atrium_crate_graph() -> GraphData {
    GraphData {
        project: "atrium".to_owned(),
        nodes: vec![
            GraphNode {
                id: "atrium-gui".to_owned(),
                label: "atrium-gui".to_owned(),
                kind: "crate".to_owned(),
                child_count: 15,
                rank: 0.9,
                file: Some("crates/atrium-gui/src/lib.rs".to_owned()),
                line: Some(1),
            },
            GraphNode {
                id: "atrium-agent-sdk".to_owned(),
                label: "atrium-agent-sdk".to_owned(),
                kind: "crate".to_owned(),
                child_count: 42,
                rank: 0.85,
                file: Some("crates/atrium-agent-sdk/src/lib.rs".to_owned()),
                line: Some(1),
            },
            GraphNode {
                id: "atrium-terminal".to_owned(),
                label: "atrium-terminal".to_owned(),
                kind: "crate".to_owned(),
                child_count: 8,
                rank: 0.6,
                file: Some("crates/atrium-terminal/src/lib.rs".to_owned()),
                line: Some(1),
            },
            GraphNode {
                id: "atrium-executor".to_owned(),
                label: "atrium-executor".to_owned(),
                kind: "crate".to_owned(),
                child_count: 6,
                rank: 0.5,
                file: Some("crates/atrium-executor/src/lib.rs".to_owned()),
                line: Some(1),
            },
            GraphNode {
                id: "atrium-error".to_owned(),
                label: "atrium-error".to_owned(),
                kind: "crate".to_owned(),
                child_count: 8,
                rank: 0.4,
                file: Some("crates/atrium-error/src/lib.rs".to_owned()),
                line: Some(1),
            },
            GraphNode {
                id: "atrium-core".to_owned(),
                label: "atrium-core".to_owned(),
                kind: "crate".to_owned(),
                child_count: 12,
                rank: 0.55,
                file: Some("crates/atrium-core/src/lib.rs".to_owned()),
                line: Some(1),
            },
        ],
        edges: vec![
            GraphEdge { from: "atrium-gui".to_owned(), to: "atrium-agent-sdk".to_owned(), relation: "depends_on".to_owned() },
            GraphEdge { from: "atrium-gui".to_owned(), to: "atrium-terminal".to_owned(), relation: "depends_on".to_owned() },
            GraphEdge { from: "atrium-gui".to_owned(), to: "atrium-core".to_owned(), relation: "depends_on".to_owned() },
            GraphEdge { from: "atrium-agent-sdk".to_owned(), to: "atrium-executor".to_owned(), relation: "depends_on".to_owned() },
            GraphEdge { from: "atrium-agent-sdk".to_owned(), to: "atrium-error".to_owned(), relation: "depends_on".to_owned() },
            GraphEdge { from: "atrium-terminal".to_owned(), to: "atrium-core".to_owned(), relation: "depends_on".to_owned() },
            GraphEdge { from: "atrium-terminal".to_owned(), to: "atrium-error".to_owned(), relation: "depends_on".to_owned() },
            GraphEdge { from: "atrium-core".to_owned(), to: "atrium-error".to_owned(), relation: "depends_on".to_owned() },
        ],
    }
}

#[tokio::test]
async fn architect_produces_valid_viewspec() {
    // Skip if bridge isn't running.
    if reqwest::get("http://localhost:5168").await.is_err() {
        println!("bridge not running on port 5168 — skipping");
        return;
    }

    let tm = TaskManager::current();
    let executor = tm.executor();

    let agent = ArchitectAgent::bridge("claude-sonnet-4");
    let request = ArchitectRequest {
        query: "Show me the project architecture".to_owned(),
        viewport_width: 1200.0,
        viewport_height: 800.0,
        graph_data: atrium_crate_graph(),
    };

    let spec = agent.render(&request, &executor).await.unwrap();

    // Validate the ViewSpec structure.
    println!("title: {}", spec.title);
    println!("nodes: {}", spec.nodes.len());
    println!("edges: {}", spec.edges.len());
    println!("groups: {}", spec.groups.len());
    println!("breadcrumb: {:?}", spec.breadcrumb);
    println!("status: {}", spec.status);

    // Must have nodes.
    assert!(!spec.nodes.is_empty(), "expected at least one node");

    // Every node must have valid position within viewport.
    for node in &spec.nodes {
        println!(
            "  node: {} at ({}, {}) {}×{}",
            node.label, node.x, node.y, node.width, node.height
        );
        assert!(node.x >= 0.0, "node {} has negative x: {}", node.id, node.x);
        assert!(node.y >= 0.0, "node {} has negative y: {}", node.id, node.y);
        assert!(node.width > 0.0, "node {} has zero width", node.id);
        assert!(node.height > 0.0, "node {} has zero height", node.id);
    }

    // Every edge must reference existing nodes.
    for edge in &spec.edges {
        assert!(
            spec.find_node(&edge.from).is_some(),
            "edge references unknown source node: {}",
            edge.from
        );
        assert!(
            spec.find_node(&edge.to).is_some(),
            "edge references unknown target node: {}",
            edge.to
        );
        assert!(
            edge.points.len() >= 2,
            "edge {}->{} must have at least 2 points",
            edge.from,
            edge.to
        );
    }

    // Print the full JSON for inspection.
    let json = spec.to_json().unwrap();
    println!("\n--- Full ViewSpec JSON ---\n{json}");
}

#[tokio::test]
async fn architect_with_copilot_acp() {
    // Use copilot via ACP transport instead of bridge.
    // Note: Copilot ACP may not follow structured output instructions as
    // reliably as Claude/GPT via the OpenAI API, so this test is best-effort.
    use atrium_agent_sdk::transport::TransportConfig;

    let tm = TaskManager::current();
    let executor = tm.executor();

    let agent = ArchitectAgent::new(TransportConfig::Acp {
        program: "copilot".to_owned(),
        args: vec!["--acp".to_owned()],
    });

    let request = ArchitectRequest {
        query: "Show me the crate dependencies".to_owned(),
        viewport_width: 1000.0,
        viewport_height: 600.0,
        graph_data: GraphData {
            project: "tiny".to_owned(),
            nodes: vec![
                GraphNode {
                    id: "app".to_owned(),
                    label: "app".to_owned(),
                    kind: "crate".to_owned(),
                    child_count: 10,
                    rank: 0.9,
                    file: Some("src/lib.rs".to_owned()),
                    line: Some(1),
                },
                GraphNode {
                    id: "core".to_owned(),
                    label: "core".to_owned(),
                    kind: "crate".to_owned(),
                    child_count: 5,
                    rank: 0.5,
                    file: Some("core/src/lib.rs".to_owned()),
                    line: Some(1),
                },
            ],
            edges: vec![GraphEdge {
                from: "app".to_owned(),
                to: "core".to_owned(),
                relation: "depends_on".to_owned(),
            }],
        },
    };

    match agent.render(&request, &executor).await {
        Ok(spec) => {
            println!("ACP result: {} nodes, {} edges", spec.nodes.len(), spec.edges.len());
            assert!(!spec.nodes.is_empty(), "expected nodes from ACP");
            for node in &spec.nodes {
                println!("  {} at ({}, {})", node.label, node.x, node.y);
            }
        }
        Err(e) => {
            // Copilot ACP may not produce structured JSON — log and skip.
            println!("ACP agent did not produce valid ViewSpec (expected): {e}");
        }
    }
}
