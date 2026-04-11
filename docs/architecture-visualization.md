# Architecture Visualization — Design

## Goal

Open a Rust project in Atrium and see its architecture — how crates, modules,
types, and functions connect — in an interactive, navigable view.

## Hard Constraint: Speed

The default view must appear in **< 10 second** after opening a project. Users
will not wait for an agent call to see the basic architecture. This means:

- **Default views are deterministic** — computed from llmcc data by code, not by
  an agent. No network call, no LLM latency.
- **Agent is for intelligence** — descriptions, impact analysis, natural language
  queries. These are on-demand, async, non-blocking.

## Two Layers

### Layer 1: Graph Engine (deterministic, fast)

`atrium-graph` crate. Pure Rust. No AI. Responsibilities:

1. **Build** — Run llmcc on a project directory → `ProjectGraph` in memory (~1-2s)
2. **Extract** — Produce `GraphView` at any depth (crate / module / symbol)
3. **Filter** — Scope to a specific crate or module
4. **Focus** — "What depends on X?" / "What does X depend on?"
5. **Diff** — Compare two snapshots, report structural changes

Every operation returns a `GraphView` — a flat list of nodes + edges with source
locations. This is the data the UI renders directly.

### Layer 2: Architecture Agent (intelligent, async)

An AI agent session managed by `atrium-agent`. Not on the critical render path.
Adds value for things code can't do:

- **Describe** — "What does this module do?" → agent reads the code and explains
- **Advise** — "What would break if I change this?" → agent reasons about impact
- **Plan** — "How should I refactor this?" → agent proposes a structural change
- **Annotate** — Enrich a `GraphView` with human-readable descriptions

The agent receives `GraphView` JSON as context and the user's question. Its
response is displayed as text alongside the graph, not as the graph itself.

## Data Model

### `GraphView` — the universal output

Every graph operation produces this. Every UI renderer consumes this. One format,
one schema.

```rust
/// A renderable graph view at any granularity.
struct GraphView {
    /// What this view shows.
    title: String,
    /// Depth level: "crate", "module", or "symbol".
    depth: Depth,
    /// Scope: which part of the project this view covers (None = whole project).
    scope: Option<String>,
    /// The nodes to display.
    nodes: Vec<GraphNode>,
    /// The edges to display.
    edges: Vec<GraphEdge>,
    /// Grouping containers (crate or module boundaries).
    groups: Vec<GraphGroup>,
}

struct GraphNode {
    /// Unique identifier within this view.
    id: String,
    /// Display label.
    label: String,
    /// What kind of thing: crate, module, struct, trait, enum, function.
    kind: NodeKind,
    /// Source file path (relative to project root).
    file: Option<String>,
    /// Line number in the source file.
    line: Option<u32>,
    /// Importance score from PageRank (0.0–1.0). Drives visual weight.
    rank: f64,
    /// How many child nodes are collapsed inside (for aggregated nodes).
    child_count: u32,
    /// Which group this node belongs to.
    group: Option<String>,
}

enum NodeKind {
    Crate, Module, Struct, Enum, Trait, Function, Impl, Const, TypeAlias,
}

struct GraphEdge {
    from: String,
    to: String,
    /// Relationship: depends_on, uses, implements, calls, has_field.
    relation: EdgeRelation,
}

enum EdgeRelation {
    DependsOn, Uses, Implements, Calls, HasField, HasMethod, Extends,
}

struct GraphGroup {
    id: String,
    label: String,
    /// Parent group (for nested grouping: crate > module).
    parent: Option<String>,
}
```

### Serialized as JSON for the UI:

```json
{
  "title": "atrium — crate dependencies",
  "depth": "crate",
  "scope": null,
  "nodes": [
    {
      "id": "atrium-agent",
      "label": "atrium-agent",
      "kind": "crate",
      "file": "crates/atrium-agent/src/lib.rs",
      "line": 1,
      "rank": 0.85,
      "child_count": 42,
      "group": null
    }
  ],
  "edges": [
    { "from": "atrium-agent", "to": "atrium-error", "relation": "depends_on" }
  ],
  "groups": []
}
```

## Navigation Model

Navigation is **instant** — no agent calls. All driven by pre-computed graph
data at different depths.

```
Open project
  → atrium-graph builds ProjectGraph (~1-2s, cached)
  → default view: crate-level GraphView (10-60 nodes)
  → rendered in < 100ms

Click "atrium-agent" crate
  → atrium-graph extracts module-level GraphView for atrium-agent
  → instant (data already in memory)
  → shows: session, transport, discovery, types, kind modules

Click "transport" module
  → atrium-graph extracts symbol-level GraphView for transport
  → shows: Transport trait, PromptRequest, AcpTransport, OpenAiTransport...

Click "AcpTransport" struct
  → opens crates/atrium-agent/src/transport/acp.rs at the struct definition

Right-click "AcpTransport" → "What depends on this?"
  → atrium-graph computes focus view (incoming edges)
  → shows only nodes that reference AcpTransport

Right-click "AcpTransport" → "Explain this"
  → Architecture Agent call (async, shows loading indicator)
  → agent reads the code + graph context, returns text description
  → displayed in a panel next to the graph
```

### Navigation stack

The UI maintains a breadcrumb / back-stack:

```
atrium (crate map) → atrium-agent (module map) → transport (symbol map)
  [← Back]
```

Each level is a `GraphView`. Going back is instant (cached).

## UI Interaction

| Action              | What happens                                         | Speed    |
|---------------------|------------------------------------------------------|----------|
| Click node          | Open source file at `file:line` in editor            | Instant  |
| Double-click node   | Drill down to next depth level                       | Instant  |
| Back button         | Return to previous depth level                       | Instant  |
| Right-click → Focus | Show "what depends on this" / "what this depends on" | Instant  |
| Right-click → Ask   | Ask Architecture Agent a question about this node    | 2-10s    |
| Search bar          | Find a node by name across all depths                | Instant  |
| After agent edits   | Rebuild graph, diff with previous, highlight changes | ~2s      |

The key insight: **all graph navigation is instant**. The only thing that takes
time is asking the agent a question, and that's explicitly async with a loading
state.

## Rendering

### Layout

The UI renderer receives `GraphView` JSON and computes layout.

| Depth  | Layout style   | Why                                  |
|--------|----------------|--------------------------------------|
| Crate  | Hierarchical   | Dependencies flow top→down or L→R    |
| Module | Hierarchical   | Same — module deps are mostly a DAG  |
| Symbol | Force-directed | More cross-references, organic layout |

### Visual encoding

| Property       | Visual                                                |
|----------------|-------------------------------------------------------|
| `kind`         | Shape: crate=box, module=rounded-box, struct=rect, trait=diamond, function=ellipse |
| `rank`         | Size: higher rank = larger node                       |
| `child_count`  | Opacity/badge: shows how much is collapsed inside     |
| `relation`     | Edge style: depends_on=solid, implements=dashed, calls=dotted |
| `group`        | Background color region                               |
| Selected node  | Highlighted border, connected edges highlighted       |

### Platform

| Platform | Technology               | Notes                              |
|----------|--------------------------|-------------------------------------|
| Web      | Cytoscape.js             | Accepts JSON, built-in layouts, interactive |
| GPUI     | Custom elements + canvas | Nodes as GPUI divs, edges as paths  |

Start with **web** (Cytoscape.js) — faster to iterate. Port to GPUI later.

## What We Build

### Phase 1: `atrium-graph` crate

Small crate (~500 lines). Links llmcc as library dependency.

```rust
// Public API surface
pub fn build(project_path: &Path) -> Result<ProjectGraph>;
pub fn view(graph: &ProjectGraph, depth: Depth, scope: Option<&str>) -> GraphView;
pub fn focus(graph: &ProjectGraph, node_id: &str, direction: Direction) -> GraphView;
pub fn diff(before: &ProjectGraph, after: &ProjectGraph, depth: Depth) -> GraphDiff;
pub fn search(graph: &ProjectGraph, query: &str) -> Vec<GraphNode>;
```

### Phase 2: Web renderer

Minimal HTML page with Cytoscape.js. Takes `GraphView` JSON, renders interactive
graph. Emits events on click/drill-down. Can run standalone or embedded in the
GPUI app via webview.

### Phase 3: Architecture Agent integration

System prompt + `GraphView` as context. Agent answers questions about the
architecture, displayed as text alongside the graph. Not on the render path.

### Phase 4: Graph diff after agent changes

After an agent modifies code, rebuild the graph and diff. Highlight new/removed
nodes and edges in the graph view. "The agent added a dependency from X to Y."

## Key Principles

1. **Default views are instant** — no agent in the critical path
2. **Agent adds intelligence, not rendering** — descriptions, advice, planning
3. **One data format** — `GraphView` JSON flows everywhere
4. **Every node has a source location** — click always jumps to code
5. **Navigation is a stack** — drill down / pop back, always instant
6. **Small code footprint** — `atrium-graph` is thin extraction, UI uses existing libraries
