//! Visual specification types.
//!
//! [`ViewSpec`] is the top-level type. It contains everything the renderer
//! needs to draw one screen of the architecture graph.

use serde::{Deserialize, Serialize};

// ── Top-level spec ──────────────────────────────────────────────────

/// Complete visual specification for one graph view.
///
/// Produced by the Architecture Agent, consumed by the UI renderer.
/// The renderer draws this verbatim — no layout, no filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewSpec {
    /// View title (e.g. "atrium — crate dependencies").
    pub title: String,

    /// Navigation breadcrumb segments (e.g. ["atrium", "atrium-agent"]).
    #[serde(default)]
    pub breadcrumb: Vec<String>,

    /// Status bar text (e.g. "12 crates · click to explore").
    #[serde(default)]
    pub status: String,

    /// Background groups — drawn first, behind everything.
    #[serde(default)]
    pub groups: Vec<Group>,

    /// Edges — drawn second, behind nodes.
    #[serde(default)]
    pub edges: Vec<Edge>,

    /// Nodes — drawn last, on top.
    pub nodes: Vec<Node>,
}

// ── Node ────────────────────────────────────────────────────────────

/// A visual node — one box on the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier within this view.
    pub id: String,

    // ── Position & size ──

    /// X position (left edge) in logical pixels.
    pub x: f32,
    /// Y position (top edge) in logical pixels.
    pub y: f32,
    /// Box width in logical pixels.
    pub width: f32,
    /// Box height in logical pixels.
    pub height: f32,

    // ── Shape ──

    /// Shape of this node. Defaults to `RoundedRect`.
    #[serde(default)]
    pub shape: NodeShape,

    // ── Content ──

    /// Primary label text.
    pub label: String,
    /// Secondary text below the label (e.g. "42 items").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sublabel: Option<String>,
    /// Icon character (e.g. "◆" for crate, "○" for module).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    // ── Style ──

    /// Visual style. Uses defaults if not specified.
    #[serde(default)]
    pub style: NodeStyle,

    // ── Source location ──

    /// Source file path (relative to project root).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Line number in the source file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,

    // ── Interaction ──

    /// What happens when the user clicks this node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_click: Option<Action>,
    /// What happens when the user double-clicks this node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_double_click: Option<Action>,
}

/// Visual style for a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStyle {
    /// Fill color (CSS hex, e.g. "#F5F5F5").
    #[serde(default = "default_node_fill")]
    pub fill: String,
    /// Border color.
    #[serde(default = "default_node_border")]
    pub border: String,
    /// Label text color.
    #[serde(default = "default_node_text")]
    pub text_color: String,
    /// Sublabel text color.
    #[serde(default = "default_node_subtext")]
    pub subtext_color: String,
    /// Border corner radius.
    #[serde(default = "default_border_radius")]
    pub border_radius: f32,
    /// Border width.
    #[serde(default = "default_border_width")]
    pub border_width: f32,
    /// Label font size.
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    /// Sublabel font size.
    #[serde(default = "default_subfont_size")]
    pub subfont_size: f32,
}

/// Shape of a node — the agent picks the shape for each node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NodeShape {
    /// Rounded rectangle (default for crates, modules).
    #[default]
    RoundedRect,
    /// Rectangle with sharp corners.
    Rect,
    /// Circle (diameter = min(width, height)).
    Circle,
    /// Diamond / rhombus (for traits, interfaces).
    Diamond,
    /// Ellipse.
    Ellipse,
    /// Pill / stadium shape (fully rounded sides).
    Pill,
}

impl Default for NodeStyle {
    fn default() -> Self {
        Self {
            fill: default_node_fill(),
            border: default_node_border(),
            text_color: default_node_text(),
            subtext_color: default_node_subtext(),
            border_radius: default_border_radius(),
            border_width: default_border_width(),
            font_size: default_font_size(),
            subfont_size: default_subfont_size(),
        }
    }
}

fn default_font_size() -> f32 { 13.0 }
fn default_subfont_size() -> f32 { 11.0 }

fn default_node_fill() -> String { "#F5F5F5".to_owned() }
fn default_node_border() -> String { "#BDBDBD".to_owned() }
fn default_node_text() -> String { "#333333".to_owned() }
fn default_node_subtext() -> String { "#999999".to_owned() }
fn default_border_radius() -> f32 { 8.0 }
fn default_border_width() -> f32 { 1.5 }

// ── Edge ────────────────────────────────────────────────────────────

/// A visual edge — a line connecting two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,

    /// Waypoints that define the edge path. The renderer draws line
    /// segments between consecutive points. Must have at least 2 points.
    pub points: Vec<Point>,

    /// Visual style.
    #[serde(default)]
    pub style: EdgeStyle,

    /// Optional text label shown along the edge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Visual style for an edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeStyle {
    /// Line color.
    #[serde(default = "default_edge_color")]
    pub color: String,
    /// Line width.
    #[serde(default = "default_edge_width")]
    pub width: f32,
    /// Dash pattern: null = solid, [4,4] = dashed, [2,2] = dotted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dash: Option<Vec<f32>>,
    /// Whether to draw an arrowhead at the target end.
    #[serde(default = "default_true")]
    pub arrow: bool,
}

impl Default for EdgeStyle {
    fn default() -> Self {
        Self {
            color: default_edge_color(),
            width: default_edge_width(),
            dash: None,
            arrow: true,
        }
    }
}

fn default_edge_color() -> String { "#BDBDBD".to_owned() }
fn default_edge_width() -> f32 { 1.5 }
fn default_true() -> bool { true }

// ── Group ───────────────────────────────────────────────────────────

/// A background group — a labeled region behind nodes for visual clustering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    /// Unique identifier.
    pub id: String,
    /// Display label (e.g. "Core").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    // ── Position & size ──

    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,

    /// Visual style.
    #[serde(default)]
    pub style: GroupStyle,
}

/// Visual style for a group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupStyle {
    /// Fill color (should be subtle/transparent).
    #[serde(default = "default_group_fill")]
    pub fill: String,
    /// Border color.
    #[serde(default = "default_group_border")]
    pub border: String,
    /// Border dash pattern (null = solid).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dash: Option<Vec<f32>>,
    /// Corner radius.
    #[serde(default = "default_group_radius")]
    pub border_radius: f32,
}

impl Default for GroupStyle {
    fn default() -> Self {
        Self {
            fill: default_group_fill(),
            border: default_group_border(),
            dash: Some(vec![4.0, 4.0]),
            border_radius: default_group_radius(),
        }
    }
}

fn default_group_fill() -> String { "#FAFAFA".to_owned() }
fn default_group_border() -> String { "#E0E0E0".to_owned() }
fn default_group_radius() -> f32 { 12.0 }

// ── Shared types ────────────────────────────────────────────────────

/// A 2D point in logical pixel coordinates.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// An action triggered by user interaction.
///
/// The renderer doesn't interpret these — it passes them to the host
/// application which decides what to do.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    /// Open a source file at a specific line.
    OpenFile {
        file: String,
        #[serde(default)]
        line: u32,
    },
    /// Drill down into a node (load a deeper view).
    DrillDown {
        /// The node ID to drill into.
        target: String,
    },
    /// Navigate back one level.
    Back,
    /// Show an info panel with details about a node.
    ShowInfo {
        /// The node ID to show info for.
        target: String,
    },
    /// Highlight nodes related to this one.
    Highlight {
        /// The node ID to highlight connections for.
        target: String,
    },
}

// ── Helpers ─────────────────────────────────────────────────────────

impl ViewSpec {
    /// Parse a `ViewSpec` from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Find a node by ID.
    pub fn find_node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Get the bounding box of all nodes (for viewport calculation).
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for node in &self.nodes {
            min_x = min_x.min(node.x);
            min_y = min_y.min(node.y);
            max_x = max_x.max(node.x + node.width);
            max_y = max_y.max(node.y + node.height);
        }
        (min_x, min_y, max_x, max_y)
    }
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_view_spec() {
        let spec = ViewSpec {
            title: "test".to_owned(),
            breadcrumb: vec!["root".to_owned()],
            status: "1 node".to_owned(),
            groups: vec![],
            edges: vec![Edge {
                from: "a".to_owned(),
                to: "b".to_owned(),
                points: vec![Point::new(0.0, 0.0), Point::new(100.0, 100.0)],
                style: EdgeStyle::default(),
                label: None,
            }],
            nodes: vec![
                Node {
                    id: "a".to_owned(),
                    x: 100.0,
                    y: 50.0,
                    width: 160.0,
                    height: 60.0,
                    shape: NodeShape::default(),
                    label: "Module A".to_owned(),
                    sublabel: Some("5 items".to_owned()),
                    icon: Some("○".to_owned()),
                    style: NodeStyle::default(),
                    file: Some("src/a.rs".to_owned()),
                    line: Some(1),
                    on_click: Some(Action::OpenFile {
                        file: "src/a.rs".to_owned(),
                        line: 1,
                    }),
                    on_double_click: Some(Action::DrillDown {
                        target: "a".to_owned(),
                    }),
                },
                Node {
                    id: "b".to_owned(),
                    x: 100.0,
                    y: 200.0,
                    width: 160.0,
                    height: 60.0,
                    shape: NodeShape::default(),
                    label: "Module B".to_owned(),
                    sublabel: None,
                    icon: None,
                    style: NodeStyle::default(),
                    file: None,
                    line: None,
                    on_click: None,
                    on_double_click: None,
                },
            ],
        };

        let json = spec.to_json().unwrap();
        let parsed = ViewSpec::from_json(&json).unwrap();

        assert_eq!(parsed.title, "test");
        assert_eq!(parsed.nodes.len(), 2);
        assert_eq!(parsed.edges.len(), 1);
        assert_eq!(parsed.nodes[0].label, "Module A");
        assert_eq!(parsed.nodes[0].x, 100.0);
    }

    #[test]
    fn default_styles_serialize() {
        let node = Node {
            id: "x".to_owned(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            label: "X".to_owned(),
            sublabel: None,
            icon: None,
            style: NodeStyle::default(),
            file: None,
            line: None,
            on_click: None,
            on_double_click: None,
        };

        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("#F5F5F5")); // default fill
        assert!(json.contains("#BDBDBD")); // default border
    }

    #[test]
    fn action_variants_roundtrip() {
        let actions = vec![
            Action::OpenFile { file: "src/lib.rs".to_owned(), line: 42 },
            Action::DrillDown { target: "mod_a".to_owned() },
            Action::Back,
            Action::ShowInfo { target: "struct_x".to_owned() },
            Action::Highlight { target: "trait_y".to_owned() },
        ];

        for action in &actions {
            let json = serde_json::to_string(action).unwrap();
            let parsed: Action = serde_json::from_str(&json).unwrap();
            // Just verify it roundtrips without panicking
            let _ = serde_json::to_string(&parsed).unwrap();
        }
    }

    #[test]
    fn bounds_calculation() {
        let spec = ViewSpec {
            title: String::new(),
            breadcrumb: vec![],
            status: String::new(),
            groups: vec![],
            edges: vec![],
            nodes: vec![
                Node {
                    id: "a".to_owned(),
                    x: 50.0, y: 30.0, width: 100.0, height: 60.0,
                    shape: NodeShape::default(),
                    label: String::new(), sublabel: None, icon: None,
                    style: NodeStyle::default(),
                    file: None, line: None,
                    on_click: None, on_double_click: None,
                },
                Node {
                    id: "b".to_owned(),
                    x: 200.0, y: 150.0, width: 120.0, height: 50.0,
                    shape: NodeShape::default(),
                    label: String::new(), sublabel: None, icon: None,
                    style: NodeStyle::default(),
                    file: None, line: None,
                    on_click: None, on_double_click: None,
                },
            ],
        };

        let (min_x, min_y, max_x, max_y) = spec.bounds();
        assert_eq!(min_x, 50.0);
        assert_eq!(min_y, 30.0);
        assert_eq!(max_x, 320.0);  // 200 + 120
        assert_eq!(max_y, 200.0);  // 150 + 50
    }
}
