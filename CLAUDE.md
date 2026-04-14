# CLAUDE.md — Lins

## What Is Lins

**Lins** (Swedish: *lens* — "a lens on knowledge") is a lightweight FalkorDB graph explorer with search-driven exploration, plugin-based styling, and a planned Living Docs feature.

**Market gap**: FalkorDB has NO good visualization tool. Neo4j Bloom/Desktop require Bolt protocol; FalkorDB uses Redis/RESP. FalkorDB Browser has 47 GitHub stars, Docker-only, minimal UX.

**GitHub**: https://github.com/semikolon/lins

## Current State (Mar 17, 2026)

**Working v1 prototype**: Backend connects to live FalkorDB (16 graphs discovered), parses nodes/edges, serves REST API. SvelteKit frontend renders graphs with Sigma.js. Force-directed layout, click-to-select, property panel, graph selector, search bar, status bar.

**CRITICAL: Rendering library NOT committed.** User is actively evaluating:
- Sigma.js (current, working) — fast, focused, but limited visual control for Bloom-quality UX
- egui + wgpu (proposed) — Rust-native GPU rendering, compiles to WASM for browser, full pixel control
- The SvelteKit frontend (`lins-web/`) may be replaced entirely by egui if the Rust-native path is chosen

**egui limitations discovered (Mar 21, 2026 — dotfiles fleet offloading session, Beeper client research):**
- **No color emoji** — monochrome glyphs only (issue #2551, open since Jan 2023, unresolved). Affects node labels with emoji.
- **Image memory** — loading 10 images (~1 MB each) = 1.1 GB (issue #5439). Graph with many node thumbnails/avatars would need aggressive cache management.
- **IME broken** — Tab intercepted, can't select IME candidates. Affects international text input in search/labels.
- **RAM 150-300 MB** for moderate apps (not the ~30 MB expected). Regression tracked in issue #5245.
- **No native macOS menu bar** (issue #3411). Users expect top-of-screen menu.
- **For Lins specifically**: These may be acceptable — graph visualization is less text/emoji-heavy than chat. The image memory issue is the biggest concern for graph nodes with visual content. Evaluate whether Sigma.js + SvelteKit gives better UX/effort ratio for the UI layer while keeping `lins-core` in Rust.

**UI/UX design NOT decided.** User explicitly stated: "I'm not at all decided yet on the UI/UX design, will need some deep thought, I want to make super conscious decisions about it."

## Architecture

### Current (working prototype)
```
lins-core/     (Rust lib)  — FalkorDB connection, schema discovery, config, query builder
lins-server/   (Rust bin)  — Axum REST API, serves frontend
lins-web/      (SvelteKit) — Sigma.js graph renderer + UI components
```

### Proposed (under evaluation)
```
lins-core/     (Rust lib)  — FalkorDB connection, schema, config (KEEP)
lins-server/   (Rust bin)  — Axum API + serves WASM build (KEEP)
lins-render/   (Rust lib)  — Graph renderer + all UI (egui, NEW)
lins-app/      (Rust bin)  — Native desktop via eframe/wgpu (NEW)
lins-wasm/     (Rust)      — Same renderer → WASM for browser (NEW)
```

### Key: Same renderer, two targets
egui compiles to both native (wgpu) and WASM (browser). One codebase, same visual quality everywhere. The `lins-web` SvelteKit frontend becomes unnecessary.

## FalkorDB Connection

- **Protocol**: Redis/RESP via `redis-rs` (NOT Bolt)
- **Default**: `127.0.0.1:6380`, password `falkordb`
- **Multi-graph**: `GRAPH.QUERY <name> "<cypher>"`, `GRAPH.LIST`
- **Schema introspection**: `CALL db.labels()`, `CALL db.relationshipTypes()`, `CALL db.propertyKeys()`
- **Verbose mode**: queries run WITHOUT `--compact` flag for human-readable labels/property names
- **Read-only**: all Cypher validated — CREATE/MERGE/DELETE/SET blocked

### FalkorDB Gotchas
- **Reserved words crash fulltext**: `MAP`, `REDUCE`, `FILTER`, `APPLY`, `LIMIT`, `LOAD`, `AS`, `GROUPBY`, `SORTBY` — must sanitize
- **`TIMEOUT_DEFAULT 120000`** in redis.conf prevents indefinite hangs
- **`CACHE_SIZE 200`** prevents query plan recompilation (default 25 too low)
- **`vecf32()` wrapper** needed for vector fields (differs from Neo4j)
- **No HNSW vector indexes by default** — Graphiti never creates them for FalkorDB

## Core Design Decisions

### Generic FalkorDB (not Graphiti-specific)
> "I feel like it should be quite generic and I don't mind the extra upfront work on the schema introspection layer." — User

Graphiti is the FIRST PLUGIN, not the core. Schema auto-discovered via `db.labels()` etc.

### Config-as-Contract with Plugin Overlays
> "There should be the possibility for plugins to contribute in a way that's kind of like an overlay. A plugin can specify styling rules, template generation, semantic search, specific output document rendering, file names even for the documents, whatever we want." — User

Three layers: auto-discovered defaults → plugin overlays → user config overrides. The `lins.toml` config file is a first-class, version-controllable, shareable artifact.

### Layered Search (Vocabulary + LLM)
> "The search should be LLM enhanced. So that you could give it natural language queries." — User
> "It could also autocomplete or suggest autocompletions as I'm typing a natural language sentence, right?" — User

Two-tier typeahead: instant vocabulary matches (~0ms) + async LLM semantic completions (300ms debounce). Works fully without LLM configured.

### Cross-Graph Workspaces
> "I would like to enable cross-graph search and queries and visualizations." — User
> "Allow the user to define different workspaces." — User

FalkorDB has no cross-graph queries. Lins queries each graph, merges results client-side.

### Visual Design Philosophy
> "The user just wants to make sense of their life instead of spending half an hour customizing styling, aint nobody got time for that." — User
> "We should have great UX and great defaults for styling and auto-adjustments of node spacing, font sizes, padding, distances etc to maximize readability and make it really smooth on the eyes." — User
> "The default color scheme should have subtle contrast and feel super intuitive instead of like a terminal from the Matrix or something a sysadmin would endure looking at." — User
> "We should also make maximal smooth use of AI where applicable." — User

Bloom-inspired but better. **Mid-to-dark theme** (not pure dark, not light — "somewhere in the middle," dark grey or dark blue, white/bright text, accent color). Söhne-style font (what Claude's UI uses). Large nodes, warm colors, generous spacing. Zero-config beauty. NOT "a terminal from the Matrix."

## Version Tiers

| Version | Theme | Key Features |
|---------|-------|--------------|
| **v1** | Explorer Core | FalkorDB connection, schema discovery, graph rendering, vocabulary search, `lins.toml` |
| **v1.5** | Search & UX | LLM search, workspaces, desktop app, plugin system, animated scene building |
| **v2** | Intelligence | Graphiti plugin, cross-graph search, embedding-powered semantic search |
| **v3** | Living Docs | Auto-generated markdown from graph queries, File Provider, Fyr integration |

## Related Projects

| Project | Relevance |
|---------|-----------|
| **Fyr** (`~/Projects/fyr/`) | Personal assistant, Tauri 2 + SvelteKit. Primary early user. Shares Rust stack — `lins-render` would be a shared crate. Custom entity types (Task, Deadline, Routine). |
| **Graphiti Fork** (`~/Projects/graphiti-official/`) | Knowledge graph that produces the data Lins visualizes. Fork with reliability fixes. |
| **dotfiles** (`~/dotfiles/`) | Where Graphiti MCP config, FalkorDB setup, and knowledge management lives. |

## Specs

Full specifications at `specs/graph-explorer/`:
- `requirements.md` — 11 user stories, acceptance criteria, version tiers
- `design.md` — architecture, data models, API, config schema
- `tasks.md` — 40 tasks across v1-v3

## Running

```bash
# Backend
cargo run --bin lins-server -- serve --port 3334 --config lins.toml

# Frontend (current SvelteKit prototype)
cd lins-web && npm run dev -- --port 5173

# Open http://localhost:5173
```

Note: port 3000 is taken by brf-auto Rails app. Use 3334 or configure in lins.toml.
