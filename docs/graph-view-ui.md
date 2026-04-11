# Graph View — UI Implementation Plan

## What We're Building

An interactive architecture graph view inside the Atrium GPUI app. Users open a
project and an **Architecture Agent** produces the complete visual specification —
what nodes to draw, where to place them, how to style them, and what happens
when you click them.

The UI is a **dumb renderer**. It draws exactly what the agent tells it to draw.
No layout algorithms, no filtering logic, no intelligence in the renderer.

## Why Agent-Driven Rendering

If we code layout algorithms ourselves, we get:
- One fixed layout style
- Hard to tune for different graph shapes
- Months of work on edge routing, overlap avoidance, etc.

If the agent produces the full visual spec:
- Layout adapts to the project structure (agent understands the code)
- Style adapts to what's important (agent knows PageRank, relationships)
- Drill-down is intelligent (agent decides what's relevant on click)
- We write ~200 lines of dumb drawing code instead of ~2000 lines of layout

## Architecture

```
User opens project
       │
       ▼
  atrium-graph: run llmcc → raw graph data (nodes, edges, ranks)
       │
       ▼
  Architecture Agent
       │  input:  raw graph data JSON + viewport size + user action
       │  output: complete visual specification JSON
       │          (positioned nodes, routed edges, styles, click targets)
       ▼
  GraphViewPanel (GPUI)
       │  draws exactly what agent specified
       │  reports user actions back (click, hover, drill-down)
       ▼
  User sees and interacts with the graph
```

## Input → Agent

The agent receives a **render request** each time the view needs to update:

```json
{
  "action": "initial_view",
  "viewport": { "width": 1200, "height": 800 },
  "graph_data": {
    "nodes": [
      { "id": "atrium-agent", "kind": "crate", "child_count": 42, "rank": 0.85,
        "file": "crates/atrium-agent/src/lib.rs", "line": 1 },
      { "id": "atrium-error", "kind": "crate", "child_count": 8, "rank": 0.32,
        "file": "crates/atrium-error/src/lib.rs", "line": 1 },
      ...
    ],
    "edges": [
      { "from": "atrium-agent", "to": "atrium-error", "relation": "depends_on" },
      ...
    ]
  }
}
```

For drill-down:

```json
{
  "action": "drill_down",
  "target": "atrium-agent",
  "viewport": { "width": 1200, "height": 800 },
  "graph_data": {
    "nodes": [ ... modules inside atrium-agent ... ],
    "edges": [ ... ]
  }
}
```

For click on a node:

```json
{
  "action": "node_clicked",
  "target": "session",
  "viewport": { "width": 1200, "height": 800 },
  "current_view": { ... the current visual spec ... },
  "graph_data": { ... }
}
```

## Output ← Agent

The agent returns a **complete visual specification**. The renderer draws this
verbatim with zero interpretation.

```json
{
  "title": "atrium — crate dependencies",
  "breadcrumb": ["atrium"],
  "status": "12 crates · click to explore",

  "nodes": [
    {
      "id": "atrium-agent",
      "x": 500, "y": 100,
      "width": 160, "height": 60,
      "label": "atrium-agent",
      "sublabel": "42 items",
      "icon": "◆",
      "style": {
        "fill": "#E3F2FD",
        "border": "#1E88E5",
        "text_color": "#1565C0",
        "border_radius": 8
      },
      "file": "crates/atrium-agent/src/lib.rs",
      "line": 1,
      "on_click": { "action": "open_file" },
      "on_double_click": { "action": "drill_down", "target": "atrium-agent" }
    },
    {
      "id": "atrium-error",
      "x": 300, "y": 300,
      "width": 140, "height": 50,
      "label": "atrium-error",
      "sublabel": "8 items",
      "icon": "◆",
      "style": {
        "fill": "#F5F5F5",
        "border": "#BDBDBD",
        "text_color": "#616161",
        "border_radius": 8
      },
      "file": "crates/atrium-error/src/lib.rs",
      "line": 1,
      "on_click": { "action": "open_file" },
      "on_double_click": { "action": "drill_down", "target": "atrium-error" }
    }
  ],

  "edges": [
    {
      "from": "atrium-agent",
      "to": "atrium-error",
      "points": [
        { "x": 500, "y": 160 },
        { "x": 500, "y": 230 },
        { "x": 370, "y": 230 },
        { "x": 370, "y": 300 }
      ],
      "style": {
        "color": "#BDBDBD",
        "width": 1.5,
        "dash": null,
        "arrow": true
      },
      "label": null
    }
  ],

  "groups": [
    {
      "id": "core-layer",
      "label": "Core",
      "x": 200, "y": 250,
      "width": 500, "height": 150,
      "style": {
        "fill": "#FAFAFA",
        "border": "#E0E0E0",
        "border_dash": [4, 4]
      }
    }
  ]
}
```

### What the agent controls

| Element       | Agent decides                                              |
|---------------|------------------------------------------------------------|
| **Nodes**     | Position (x,y), size, label, sublabel, icon, colors, what happens on click/double-click |
| **Edges**     | Route points, color, width, dash pattern, arrow, optional label |
| **Groups**    | Background regions for visual clustering                   |
| **Breadcrumb**| Navigation path text                                       |
| **Status**    | Bottom bar text                                            |
| **Click behavior** | Per-node: open file, drill down, show info panel, highlight related |

### What the renderer controls

Only mechanical things:
- Text rendering (font, anti-aliasing)
- Actual pixel drawing (rectangles, lines, text)
- Mouse event capture → reports back to agent
- Pan/zoom transform
- Cursor changes on hover

## Visual Design

### Node style

The agent chooses colors, but we suggest a **default palette** in the system prompt:

| State     | Fill      | Border    | Text      |
|-----------|-----------|-----------|-----------|
| Default   | `#F5F5F5` | `#BDBDBD` | `#616161` |
| Important | `#E3F2FD` | `#1E88E5` | `#1565C0` |
| Selected  | `#E8F5E9` | `#43A047` | `#2E7D32` |
| Warning   | `#FFF3E0` | `#FB8C00` | `#E65100` |

### Edge style

| Relation       | Style    | Color     |
|----------------|----------|-----------|
| `depends_on`   | Solid    | `#BDBDBD` |
| `implements`   | Dashed   | `#90CAF9` |
| `calls`        | Dotted   | `#A5D6A7` |
| `uses`         | Solid    | `#BDBDBD` |

### Layout guidelines (in system prompt)

The agent is instructed to:
- Place nodes in layers (dependencies flow top to bottom)
- Keep connected nodes close together
- Avoid edge crossings where possible
- Use groups to visually cluster related nodes
- Size nodes proportional to their rank/importance
- Leave enough whitespace for readability
- Fit within the given viewport dimensions

## What It Looks Like

### Crate-level view (default)

```
┌─────────────────────────────────────────────────────────────────┐
│  [atrium]                                                       │
│─────────────────────────────────────────────────────────────────│
│                                                                 │
│              ┌────────────────┐                                 │
│              │ ◆ atrium-gui   │                                 │
│              │   15 items     │                                 │
│              └───────┬────────┘                                 │
│                      │                                          │
│           ┌──────────┼──────────┐                               │
│           ▼          ▼          ▼                                │
│   ┌────────────┐ ┌──────────┐ ┌────────────┐                   │
│   │◆ atrium-   │ │◆ atrium- │ │◆ atrium-   │                   │
│   │  terminal  │ │  agent   │ │  init       │                   │
│   │  8 items   │ │  42 items│ │  4 items    │                   │
│   └─────┬──────┘ └────┬─────┘ └─────┬──────┘                   │
│         │             │              │                          │
│   ┌─────┼─────────────┼──────────────┼────┐                    │
│   │     ▼             ▼              ▼    │ ── Core ──         │
│   │ ┌────────┐ ┌──────────┐ ┌──────────┐ │                    │
│   │ │◆ core  │ │◆ error   │ │◆ executor│ │                    │
│   │ └────────┘ └──────────┘ └──────────┘ │                    │
│   └───────────────────────────────────────┘                    │
│                                                                 │
│─────────────────────────────────────────────────────────────────│
│  12 crates · click to explore                                   │
└─────────────────────────────────────────────────────────────────┘
```

### After double-clicking "atrium-agent"

```
┌─────────────────────────────────────────────────────────────────┐
│  [atrium] > [atrium-agent]                                      │
│─────────────────────────────────────────────────────────────────│
│                                                                 │
│   ┌─────────────────┐        ┌──────────────────┐              │
│   │ ○ session        │───────▶│ ○ transport       │              │
│   │   5 symbols      │        │   12 symbols      │              │
│   └────────┬─────────┘        └───────┬───────────┘              │
│            │                    ┌─────┼─────┐                   │
│            │                    ▼     ▼     ▼                   │
│            │              ┌──────┐┌──────┐┌──────────┐          │
│            │              │○ acp ││○ http││○ anthro. │          │
│            │              └──────┘└──────┘└──────────┘          │
│            ▼                                                    │
│   ┌─────────────────┐        ┌──────────────────┐              │
│   │ ○ discovery      │        │ ○ types           │              │
│   │   3 symbols      │        │   8 symbols       │              │
│   └─────────────────┘        └──────────────────┘              │
│                                                                 │
│─────────────────────────────────────────────────────────────────│
│  Module view · atrium-agent · 6 modules                         │
└─────────────────────────────────────────────────────────────────┘
```

## Implementation Steps

### Step 1: Dumb renderer + hardcoded visual spec

Write the `GraphViewPanel` GPUI component that:
- Takes a hardcoded visual spec JSON (the exact output format above)
- Draws nodes at specified (x, y) positions with specified styles
- Draws edges along specified point paths
- Draws groups as background rectangles
- Draws breadcrumb and status bar

**No layout code. No intelligence. Just draw what the JSON says.**

The hardcoded JSON represents the atrium crate-level view, hand-positioned
to look good.

**Deliverable:** Screenshot of the crate architecture graph.

### Step 2: Click handling + drill-down

Add interaction:
- Click: read `on_click` from the node spec, emit the action
- Double-click: read `on_double_click`, swap in a second hardcoded visual spec
- Hover: highlight the node (brighten border)
- Breadcrumb click: go back

Still using hardcoded specs — two of them now (crate view + module view).

**Deliverable:** Navigate between crate and module views.

### Step 3: Wire in the Architecture Agent

Replace hardcoded JSON with agent calls:
- On project open: send `initial_view` request to agent with graph data
- On drill-down: send `drill_down` request with the target node
- On click: send `node_clicked` request
- Agent returns the full visual spec, renderer draws it

Loading state while waiting for agent response.

**Deliverable:** Agent-driven architecture visualization of any Rust project.

### Step 4: Pan + zoom

For larger views:
- Drag to pan
- Scroll to zoom
- Agent provides coordinates in a logical space, renderer applies transform

**Deliverable:** Smooth navigation of large architecture views.

## File Structure

```
crates/atrium-gui/src/
  components/
    graph_view/
      mod.rs          — GraphViewPanel GPUI component
      types.rs        — VisualSpec, VisualNode, VisualEdge, VisualGroup
      render.rs       — draw nodes, edges, groups (dumb drawing)
      interaction.rs  — click/hover/drill-down event dispatch
      fake_data.rs    — hardcoded visual specs for testing (Step 1-2)
```

## Non-Goals for Now

- No layout algorithm in our code (agent does it)
- No llmcc integration yet (Step 3 uses agent, Step 4 wires llmcc)
- No web renderer (GPUI first)
- No graph diff
- No search
- No right-click context menus
