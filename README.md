# Atrium

**Architecture-native development. Let agents handle the code.**

Atrium is a development app where architecture is the primary interface. Instead of reading and writing code line by line, you work at the architecture level — understanding structure, making design decisions, directing AI agents — while agents handle the implementation details.

## The Problem

Today's development tools are built around code text. You read files, edit lines, and manage diffs. AI coding agents operate the same way — they grep, they RAG, they read files one at a time with no structural understanding. The result: agents that waste tokens on orientation, miss cross-module impacts, and make changes that break things they never looked at.

As AI gets better at writing code, the developer's job shifts from typing code to designing systems. But every tool still forces you into the code-text level. There's nothing that lets you stay at the architecture level and delegate downward.

## The Approach

Atrium puts architecture at the center:

1. **You see architecture, not files.** Open a project and see how crates, modules, types, and functions connect — at whatever depth you need.
2. **You think in architecture.** Plan changes by understanding structure, boundaries, and impact — before any code is touched.
3. **Agents work from architecture.** When you direct an agent to make a change, it already knows the full architectural context — what depends on what, what will break, where to look.
4. **You review architecture.** See what changed structurally, not just textually. Did the agent add a dependency? Change a public API boundary? Break an abstraction layer?

The developer becomes the architect. The agent becomes the builder.

## Key Ideas

- **Architecture as the primary view** — code is a detail you zoom into, not the default
- **Architecture-aware agents** — agents that understand structure before they touch code
- **Architectural review** — see structural diffs, not just text diffs
- **Multi-depth navigation** — project → crate → module → symbol, like a map

## Built On

Powered by [llmcc](https://github.com/allenanswerzq/llmcc) for fast, accurate architecture graph generation via tree-sitter parsing, symbol resolution, and dependency analysis.

## Status

Early development. Starting with Rust as the first supported language.

## License

Apache-2.0
