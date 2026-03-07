# Atrium

**See the architecture, not just the code.**

Atrium is an interactive architecture visualization app for codebases. Instead of reading code file by file, Atrium lets you see the entire structure of a project — crates, modules, types, and how they connect — then zoom into the details that matter.

## The Problem

Developers spend most of their time understanding code, not writing it. Today's tools show code as flat text. You open files, grep for symbols, scroll through thousands of lines, and slowly build a mental model of how things fit together. This doesn't scale.

## The Approach

Atrium generates a live, multi-depth architecture graph from real source code and lets you navigate it interactively:

- **Depth 1** — Project: crates/packages and their boundaries
- **Depth 2** — Module: subsystem structure and relationships
- **Depth 3** — Symbol: functions, types, traits, and their connections

Zoom like a map. See the whole forest, then descend into the trees.

## Key Features (Planned)

- **Semantic zoom** — scroll to transition between architecture depths automatically
- **Live updates** — graph rebuilds incrementally as you edit code
- **Architecture diff** — see what changed structurally between branches
- **PageRank filtering** — surface the most important nodes in large codebases
- **Search & filter** — find symbols in architectural context
- **Click-through** — jump to source code from any node

## Built On

Powered by [llmcc](https://github.com/allenanswerzq/llmcc) for fast, accurate architecture graph generation via tree-sitter parsing, symbol resolution, and dependency analysis.

## Status

Early development. Starting with Rust as the first supported language.

## License

Apache-2.0
