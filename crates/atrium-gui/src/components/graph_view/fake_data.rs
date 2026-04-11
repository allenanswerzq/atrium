//! Fake `ViewSpec` data for testing the renderer.

use atrium_graph_view::*;

// ── Helpers ─────────────────────────────────────────────────────────

fn crate_node(id: &str, x: f32, y: f32, w: f32, count: u32, rank: f64, important: bool) -> Node {
    let (fill, border, text) = if important {
        ("#E3F2FD", "#1E88E5", "#1565C0")
    } else {
        ("#F5F5F5", "#BDBDBD", "#333333")
    };
    Node {
        id: id.to_owned(),
        x, y, width: w, height: 56.0,
        shape: NodeShape::RoundedRect,
        label: id.to_owned(),
        sublabel: Some(format!("{count} items")),
        icon: Some("◆".to_owned()),
        style: NodeStyle {
            fill: fill.to_owned(),
            border: border.to_owned(),
            text_color: text.to_owned(),
            ..NodeStyle::default()
        },
        file: Some(format!("crates/{id}/src/lib.rs")),
        line: Some(1),
        on_click: Some(Action::OpenFile {
            file: format!("crates/{id}/src/lib.rs"),
            line: 1,
        }),
        on_double_click: Some(Action::DrillDown { target: id.to_owned() }),
    }
}

fn module_node(id: &str, label: &str, x: f32, y: f32, w: f32, count: u32, kind_icon: &str) -> Node {
    Node {
        id: id.to_owned(),
        x, y, width: w, height: 50.0,
        shape: NodeShape::RoundedRect,
        label: label.to_owned(),
        sublabel: Some(format!("{count} symbols")),
        icon: Some(kind_icon.to_owned()),
        style: NodeStyle::default(),
        file: Some(format!("crates/atrium-agent-sdk/src/{id}.rs")),
        line: Some(1),
        on_click: Some(Action::OpenFile {
            file: format!("crates/atrium-agent-sdk/src/{id}.rs"),
            line: 1,
        }),
        on_double_click: Some(Action::DrillDown { target: id.to_owned() }),
    }
}

fn symbol_node(id: &str, label: &str, x: f32, y: f32, kind: &str, shape: NodeShape) -> Node {
    let (icon, fill, border) = match kind {
        "struct"   => ("□", "#E8F5E9", "#66BB6A"),
        "trait"    => ("◇", "#F3E5F5", "#AB47BC"),
        "enum"     => ("△", "#FFF3E0", "#FFA726"),
        "function" => ("ƒ", "#E3F2FD", "#42A5F5"),
        _          => ("·", "#F5F5F5", "#BDBDBD"),
    };
    Node {
        id: id.to_owned(),
        x, y, width: 160.0, height: 44.0,
        shape,
        label: label.to_owned(),
        sublabel: None,
        icon: Some(icon.to_owned()),
        style: NodeStyle {
            fill: fill.to_owned(),
            border: border.to_owned(),
            ..NodeStyle::default()
        },
        file: Some(format!("crates/atrium-agent-sdk/src/transport/{id}.rs")),
        line: Some(1),
        on_click: Some(Action::OpenFile {
            file: format!("crates/atrium-agent-sdk/src/transport/{id}.rs"),
            line: 1,
        }),
        on_double_click: Some(Action::ShowInfo { target: id.to_owned() }),
    }
}

fn edge(from: &str, to: &str, points: Vec<(f32, f32)>) -> Edge {
    Edge {
        from: from.to_owned(),
        to: to.to_owned(),
        points: points.into_iter().map(|(x, y)| Point::new(x, y)).collect(),
        style: EdgeStyle::default(),
        label: None,
    }
}

fn edge_dashed(from: &str, to: &str, points: Vec<(f32, f32)>) -> Edge {
    Edge {
        from: from.to_owned(),
        to: to.to_owned(),
        points: points.into_iter().map(|(x, y)| Point::new(x, y)).collect(),
        style: EdgeStyle {
            color: "#90CAF9".to_owned(),
            dash: Some(vec![6.0, 3.0]),
            ..EdgeStyle::default()
        },
        label: Some("implements".to_owned()),
    }
}

// ── View 1: Crate map (14 crates) ──────────────────────────────────

/// Full atrium workspace — 14 crates, 4 layers, 2 groups.
pub fn atrium_crate_view() -> ViewSpec {
    ViewSpec {
        title: "atrium — crate dependencies".to_owned(),
        breadcrumb: vec!["atrium".to_owned()],
        status: "14 crates · double-click to explore".to_owned(),
        groups: vec![
            Group {
                id: "app-layer".to_owned(),
                label: Some("Application".to_owned()),
                x: 100.0, y: 20.0, width: 700.0, height: 80.0,
                style: GroupStyle {
                    fill: "#F0F7FF".to_owned(),
                    border: "#BBDEFB".to_owned(),
                    ..GroupStyle::default()
                },
            },
            Group {
                id: "agent-layer".to_owned(),
                label: Some("Agent".to_owned()),
                x: 100.0, y: 130.0, width: 700.0, height: 80.0,
                style: GroupStyle {
                    fill: "#F1F8E9".to_owned(),
                    border: "#C5E1A5".to_owned(),
                    ..GroupStyle::default()
                },
            },
            Group {
                id: "infra-layer".to_owned(),
                label: Some("Infrastructure".to_owned()),
                x: 100.0, y: 240.0, width: 700.0, height: 80.0,
                style: GroupStyle::default(),
            },
            Group {
                id: "core-layer".to_owned(),
                label: Some("Core".to_owned()),
                x: 100.0, y: 350.0, width: 700.0, height: 80.0,
                style: GroupStyle::default(),
            },
        ],
        nodes: vec![
            // Layer 0: Application
            crate_node("atrium-gui",   280.0, 30.0, 170.0, 15, 0.95, true),
            crate_node("atrium-init",  500.0, 30.0, 150.0, 4, 0.5, false),
            // Layer 1: Agent
            crate_node("atrium-agents",     120.0, 140.0, 170.0, 6, 0.7, true),
            crate_node("atrium-agent-sdk",  340.0, 140.0, 185.0, 42, 0.9, true),
            crate_node("atrium-graph-view", 580.0, 140.0, 180.0, 8, 0.4, false),
            // Layer 2: Infrastructure
            crate_node("atrium-terminal",  120.0, 250.0, 165.0, 8, 0.6, false),
            crate_node("atrium-watcher",   320.0, 250.0, 160.0, 3, 0.3, false),
            crate_node("atrium-context",   520.0, 250.0, 160.0, 5, 0.35, false),
            crate_node("atrium-platform",  710.0, 250.0, 160.0, 2, 0.2, false),
            // Layer 3: Core
            crate_node("atrium-executor", 120.0, 360.0, 165.0, 6, 0.55, false),
            crate_node("atrium-error",    320.0, 360.0, 150.0, 8, 0.45, false),
            crate_node("atrium-core",     500.0, 360.0, 150.0, 12, 0.5, false),
            crate_node("atrium-fs",       680.0, 360.0, 120.0, 4, 0.25, false),
            crate_node("atrium-io",       830.0, 360.0, 120.0, 5, 0.3, false),
        ],
        edges: vec![
            // Layer 0 → 1: gui dependencies
            // gui → agent-sdk (straight down, slight jog right)
            edge("atrium-gui", "atrium-agent-sdk", vec![
                (365.0, 86.0), (365.0, 113.0), (432.0, 113.0), (432.0, 140.0)
            ]),
            // gui → init (horizontal on same layer)
            edge("atrium-gui", "atrium-init", vec![
                (450.0, 58.0), (500.0, 58.0)
            ]),
            // gui → terminal (down-left, two layers)
            edge("atrium-gui", "atrium-terminal", vec![
                (300.0, 86.0), (300.0, 113.0), (202.0, 113.0), (202.0, 250.0)
            ]),

            // Layer 0 → 2: init dependencies
            // init → context
            edge("atrium-init", "atrium-context", vec![
                (575.0, 86.0), (575.0, 220.0), (600.0, 220.0), (600.0, 250.0)
            ]),
            // init → executor (long route down-left)
            edge("atrium-init", "atrium-executor", vec![
                (540.0, 86.0), (540.0, 113.0), (100.0, 113.0), (100.0, 388.0), (120.0, 388.0)
            ]),

            // Layer 1 → 1: agent layer internal
            // agents → agent-sdk (horizontal)
            edge("atrium-agents", "atrium-agent-sdk", vec![
                (290.0, 168.0), (340.0, 168.0)
            ]),
            // agents → graph-view (horizontal)
            edge("atrium-agents", "atrium-graph-view", vec![
                (290.0, 160.0), (310.0, 160.0), (560.0, 160.0), (580.0, 160.0)
            ]),

            // Layer 1 → 3: agent-sdk dependencies (skip layer 2)
            // agent-sdk → executor
            edge("atrium-agent-sdk", "atrium-executor", vec![
                (380.0, 196.0), (380.0, 340.0), (202.0, 340.0), (202.0, 360.0)
            ]),
            // agent-sdk → error
            edge("atrium-agent-sdk", "atrium-error", vec![
                (460.0, 196.0), (460.0, 340.0), (395.0, 340.0), (395.0, 360.0)
            ]),

            // Layer 2 → 3: infra dependencies
            // terminal → core
            edge("atrium-terminal", "atrium-core", vec![
                (202.0, 306.0), (202.0, 340.0), (575.0, 340.0), (575.0, 360.0)
            ]),
            // terminal → error
            edge("atrium-terminal", "atrium-error", vec![
                (250.0, 306.0), (250.0, 340.0), (395.0, 340.0), (395.0, 360.0)
            ]),

            // Layer 3 → 3: core layer internal
            // core → error (horizontal)
            edge("atrium-core", "atrium-error", vec![
                (500.0, 388.0), (470.0, 388.0)
            ]),
        ],
    }
}

// ── View 2: Module map (inside atrium-agent-sdk) ────────────────────

/// Drill-down into atrium-agent-sdk — 7 modules.
pub fn agent_sdk_module_view() -> ViewSpec {
    ViewSpec {
        title: "atrium-agent-sdk — modules".to_owned(),
        breadcrumb: vec!["atrium".to_owned(), "atrium-agent-sdk".to_owned()],
        status: "7 modules · double-click to explore".to_owned(),
        groups: vec![
            Group {
                id: "transport-group".to_owned(),
                label: Some("Transport Layer".to_owned()),
                x: 250.0, y: 180.0, width: 520.0, height: 180.0,
                style: GroupStyle {
                    fill: "#F3E5F5".to_owned(),
                    border: "#CE93D8".to_owned(),
                    ..GroupStyle::default()
                },
            },
        ],
        nodes: vec![
            module_node("session",   "session",   100.0, 40.0, 150.0, 12, "○"),
            module_node("kind",      "kind",      300.0, 40.0, 120.0, 3,  "○"),
            module_node("types",     "types",     470.0, 40.0, 120.0, 8,  "○"),
            module_node("discovery", "discovery", 640.0, 40.0, 140.0, 5,  "○"),
            // Inside transport group
            module_node("transport", "transport (mod)", 270.0, 200.0, 170.0, 6, "○"),
            module_node("acp",       "acp",            480.0, 200.0, 120.0, 15, "○"),
            module_node("openai",    "openai",         270.0, 280.0, 130.0, 10, "○"),
            module_node("anthropic", "anthropic",      430.0, 280.0, 140.0, 8, "○"),
            module_node("responses", "responses",      600.0, 280.0, 140.0, 8, "○"),
            module_node("terminal",  "terminal",       640.0, 200.0, 130.0, 7, "○"),
        ],
        edges: vec![
            edge("session", "transport", vec![(175.0, 90.0), (175.0, 140.0), (355.0, 140.0), (355.0, 200.0)]),
            edge("session", "types",     vec![(250.0, 65.0), (470.0, 65.0)]),
            edge("session", "kind",      vec![(250.0, 55.0), (300.0, 55.0)]),
            edge("transport", "acp",     vec![(440.0, 225.0), (480.0, 225.0)]),
            edge("transport", "openai",  vec![(355.0, 250.0), (355.0, 265.0), (335.0, 265.0), (335.0, 280.0)]),
            edge("transport", "anthropic", vec![(440.0, 240.0), (500.0, 240.0), (500.0, 280.0)]),
            edge("transport", "responses", vec![(440.0, 235.0), (560.0, 235.0), (560.0, 260.0), (670.0, 260.0), (670.0, 280.0)]),
            edge("transport", "terminal",  vec![(440.0, 215.0), (640.0, 215.0)]),
            // implements edges (dashed)
            edge_dashed("acp", "transport",       vec![(480.0, 225.0), (440.0, 225.0)]),
            edge_dashed("openai", "transport",    vec![(335.0, 280.0), (335.0, 250.0)]),
            edge_dashed("anthropic", "transport", vec![(500.0, 280.0), (500.0, 250.0), (440.0, 250.0)]),
        ],
    }
}

// ── View 3: Symbol map (inside transport) ───────────────────────────

/// Drill-down into the transport module — individual types.
pub fn transport_symbol_view() -> ViewSpec {
    ViewSpec {
        title: "transport — symbols".to_owned(),
        breadcrumb: vec![
            "atrium".to_owned(),
            "atrium-agent-sdk".to_owned(),
            "transport".to_owned(),
        ],
        status: "8 symbols · click to open source".to_owned(),
        groups: vec![],
        nodes: vec![
            symbol_node("Transport",       "Transport",       300.0, 30.0,  "trait",    NodeShape::Diamond),
            symbol_node("PromptRequest",   "PromptRequest",   100.0, 30.0,  "struct",   NodeShape::RoundedRect),
            symbol_node("TransportConfig", "TransportConfig",  520.0, 30.0, "enum",     NodeShape::RoundedRect),
            symbol_node("EventSender",     "EventSender",     100.0, 120.0, "function", NodeShape::Pill),
            symbol_node("AcpTransport",    "AcpTransport",    100.0, 210.0, "struct",   NodeShape::RoundedRect),
            symbol_node("OpenAiTransport", "OpenAiTransport", 300.0, 210.0, "struct",   NodeShape::RoundedRect),
            symbol_node("AnthropicTransport", "AnthropicTransport", 500.0, 210.0, "struct", NodeShape::RoundedRect),
            symbol_node("ResponsesTransport", "ResponsesTransport", 300.0, 290.0, "struct", NodeShape::RoundedRect),
        ],
        edges: vec![
            // All transports implement Transport trait
            edge_dashed("AcpTransport",       "Transport", vec![(180.0, 210.0), (180.0, 140.0), (380.0, 140.0), (380.0, 74.0)]),
            edge_dashed("OpenAiTransport",    "Transport", vec![(380.0, 210.0), (380.0, 74.0)]),
            edge_dashed("AnthropicTransport", "Transport", vec![(580.0, 210.0), (580.0, 140.0), (380.0, 140.0), (380.0, 74.0)]),
            edge_dashed("ResponsesTransport", "Transport", vec![(380.0, 290.0), (380.0, 74.0)]),
            // Transport uses PromptRequest
            edge("Transport", "PromptRequest", vec![(300.0, 52.0), (260.0, 52.0)]),
            // create() uses TransportConfig
            edge("Transport", "TransportConfig", vec![(460.0, 52.0), (520.0, 52.0)]),
        ],
    }
}
