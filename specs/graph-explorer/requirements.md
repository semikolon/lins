# Requirements: Lins — FalkorDB Graph Explorer

> **Note (Mar 17, 2026):** This spec was written during the initial /spec session and reflects the SvelteKit + Sigma.js architecture. The rendering technology and UI/UX design are now **open decisions** — egui + wgpu (Rust-native with WASM) is under active evaluation as an alternative. See `docs/founding_session_2026_03_16.md` §8 for the full rendering pivot discussion and §9 for all open decisions.

## Overview

- **Type**: New feature (greenfield project)
- **Problem**: FalkorDB has no good visualization or exploration tool. Neo4j Bloom/Desktop require Bolt protocol (FalkorDB uses Redis/RESP). FalkorDB Browser has 47 GitHub stars, is Docker-only, with minimal UX. The competitive landscape is wide open.
- **Pain Point**: Knowledge graphs are invisible. Data goes in via Graphiti/MCP but there's no way to *see* the graph, explore relationships, or generate human-readable views of graph contents. This affects both the developer (debugging, understanding) and downstream consumers (documentation, knowledge sharing).
- **Success Unlocks**: Visual graph exploration enables understanding what Graphiti has been building. Living Docs turns graph data into auto-updating markdown. Plugin system enables ecosystem growth. Open-source fills a genuine market gap in the FalkorDB community.
- **Name**: Lins (Swedish: "lens" — a lens on knowledge)

---

## Core Philosophy

> "I feel like it should be quite generic and I don't mind the extra upfront work on the schema introspection layer." — User

> "User configurable but intelligently auto derived for styling and templates." — User

> "Programmer happiness and stellar performance and stability." — User

Lins is a **generic FalkorDB graph explorer** — not a Graphiti-specific tool. Graphiti is the first plugin/use case, but the core knows nothing about Entity/Episodic/Community node types. Schema is auto-discovered via FalkorDB introspection (`db.labels()`, `db.relationshipTypes()`, `db.propertyKeys()`).

The intelligence model is **config-as-contract with plugin overlays**: auto-discovered defaults → plugin overlays → user config overrides. The config file (`lins.toml`) is a first-class, version-controllable, shareable artifact.

> "There should be the possibility for plugins to contribute in a way that's kind of like an overlay. A plugin can specify styling rules, template generation, semantic search, specific output document rendering, file names even for the documents, whatever we want." — User

> "There could be plugins or something like that, that I could bundle with the personal assistant app that I'm building or whatever uses this and integrates with it." — User

---

## User Stories

### US-1: Visual Graph Exploration

**As a** developer working with FalkorDB knowledge graphs
**I want** to connect to my FalkorDB instance and visually explore the graph
**So that** I can understand the shape, contents, and relationships in my data

**Acceptance Criteria:**
- [ ] AC-1.1: Connect to FalkorDB via Redis protocol (host, port, password)
- [ ] AC-1.2: Auto-discover graph schema (labels, relationship types, property keys) via `CALL db.labels()`, `CALL db.relationshipTypes()`, `CALL db.propertyKeys()`
- [ ] AC-1.3: List available named graphs via `GRAPH.LIST`
- [ ] AC-1.4: Render graph nodes and edges with WebGL-accelerated library
- [ ] AC-1.5: Force-directed layout with smooth physics animation
- [ ] AC-1.6: Click node → property panel showing all properties
- [ ] AC-1.7: Click edge → property panel showing relationship properties
- [ ] AC-1.8: Zoom, pan, drag nodes with responsive interaction
- [ ] AC-1.9: Auto-assign colors by label (intelligently derived, user-overridable)
- [ ] AC-1.10: Node size encoding based on configurable property

**EARS Constraints:**
- **When** a user connects to a FalkorDB instance, **the system shall** auto-discover all available graphs and their schemas
- **While** the graph is being rendered, **the system shall** maintain interactive frame rates (>30 FPS for graphs up to 1,000 nodes)
- **If** a graph has more than 10,000 nodes, **then the system shall** warn the user and offer to load a subset or apply filters

### US-2: Search-Driven Exploration

**As a** developer exploring a knowledge graph
**I want** to search for nodes, relationships, and patterns using both vocabulary autocomplete and natural language
**So that** I can find relevant subgraphs without writing raw Cypher

> "The search should be LLM enhanced. So that you could give it natural language queries." — User
> "It could also autocomplete or suggest autocompletions as I'm typing a natural language sentence, right?" — User

**Acceptance Criteria:**
- [ ] AC-2.1: Search bar with instant vocabulary autocomplete from auto-discovered schema (~0ms latency)
- [ ] AC-2.2: Autocomplete suggests labels, relationship types, property values, entity names as user types
- [ ] AC-2.3: (v1.5) LLM-enhanced search: natural language sentence → generated Cypher query
- [ ] AC-2.4: (v1.5) Two-tier typeahead: instant vocabulary matches + async LLM semantic completions (debounced ~300ms)
- [ ] AC-2.5: Search results animate additively into the scene (Bloom-style scene building)
- [ ] AC-2.6: LLM-generated Cypher is sandboxed to read-only operations (no MERGE, DELETE, SET)
- [ ] AC-2.7: LLM search degrades gracefully when no LLM is configured (vocabulary search remains fully functional)
- [ ] AC-2.8: Search history with re-executable past queries

**EARS Constraints:**
- **When** a user types in the search bar, **the system shall** show vocabulary autocomplete results within 50ms
- **If** an LLM endpoint is configured, **then the system shall** additionally show semantic completion suggestions after a 300ms debounce
- **If** no LLM endpoint is configured, **then the system shall** provide full vocabulary-based search without degradation

### US-3: Configuration & Styling

**As a** user of Lins
**I want** the graph to be styled automatically based on schema, with the ability to customize via config files and plugins
**So that** different graphs and use cases get appropriate visual treatment without manual setup

**Acceptance Criteria:**
- [ ] AC-3.1: Auto-generate `lins.toml` config from discovered schema on first connection
- [ ] AC-3.2: Config specifies: connection details, styling rules (colors, sizes, icons per label), visible/hidden properties, layout preferences
- [ ] AC-3.3: Plugin configs overlay onto base config (plugin adds styling rules, templates, schema awareness)
- [ ] AC-3.4: User edits to `lins.toml` override both auto-derived and plugin settings
- [ ] AC-3.5: Config changes hot-reload without restart
- [ ] AC-3.6: Config file is version-controllable and shareable (publishable `lins-graphiti.toml` for Graphiti users)

**EARS Constraints:**
- **When** a config file is modified, **the system shall** hot-reload the changes within 1 second
- **While** no user config exists, **the system shall** generate intelligent defaults from schema introspection (distinct colors per label, readable property display)

### US-4: Multi-Graph Workspaces (v1.5)

**As a** developer with multiple FalkorDB graphs
**I want** to define workspaces that combine multiple graphs with shared styling and configuration
**So that** I can explore cross-project knowledge and switch contexts efficiently

> "I would like to enable cross-graph search and queries and visualizations." — User
> "Allow the user to define different workspaces." — User
> "Allow the user to define styling overlays, living docs templates. Plugable and shareable. Either as a combined plugin or as separate plugins. Whatever the user wants." — User

**Acceptance Criteria:**
- [ ] AC-4.1: Workspace definition in config: list of graphs, per-graph or shared styling, layout preferences
- [ ] AC-4.2: Single-graph dropdown for quick switching
- [ ] AC-4.3: (v2) Cross-graph search: query multiple graphs and merge results client-side
- [ ] AC-4.4: (v2) Cross-graph visualization: render nodes from different graphs with visual distinction (namespace/color coding)
- [ ] AC-4.5: Workspace configs shareable as standalone files

**EARS Constraints:**
- **When** a workspace includes multiple graphs, **the system shall** query each graph independently and merge results (FalkorDB has no cross-graph queries)
- **If** a graph in the workspace is unreachable, **the system shall** display available graphs and indicate the unreachable one

### US-5: Living Docs (v3)

**As a** knowledge graph user
**I want** auto-generated markdown documents that stay in sync with graph data
**So that** graph knowledge is accessible as human-readable documentation without manual maintenance

> "It should be able to do both!" — User (re: file writing AND web serving)
> "Could it sense when a markdown file path is being read, and sort of serve the content live from the graph? Would be cool." — User

**Acceptance Criteria:**
- [ ] AC-5.1: View definitions in config: name, Cypher query, template, output path, ordering
- [ ] AC-5.2: Template engine renders query results to markdown files
- [ ] AC-5.3: Daemon watches for graph changes and auto-regenerates affected docs
- [ ] AC-5.4: Generated docs include metadata header (source graph, last rendered, episode count, etc.)
- [ ] AC-5.5: Web API serves live doc content from graph (not just cached files)
- [ ] AC-5.6: Both file output and web serving supported simultaneously
- [ ] AC-5.7: (v3) File Provider / virtual files — files rendered from graph on access
- [ ] AC-5.8: Staleness detection: only re-render when graph data has changed
- [ ] AC-5.9: Plugin-definable templates (Graphiti plugin ships decision doc templates, entity summary templates, etc.)

**EARS Constraints:**
- **When** graph data changes and a Living Doc view is affected, **the system shall** regenerate the document within the configured poll interval
- **While** no graph changes have occurred, **the system shall** serve cached file content without re-querying
- **If** a template render fails, **the system shall** preserve the last successful render and log the error

### US-6: Plugin System (v1.5)

**As a** Lins user or ecosystem developer
**I want** to extend Lins with plugins that provide schema-aware styling, templates, and search enhancements
**So that** domain-specific knowledge (like Graphiti's entity types) enhances the experience without coupling the core

**Acceptance Criteria:**
- [ ] AC-6.1: Plugin is a config overlay (TOML/YAML file with styling rules, templates, schema hints)
- [ ] AC-6.2: Plugins can define: node styling per label, edge styling per type, property display rules, Living Docs templates, search enhancements
- [ ] AC-6.3: Multiple plugins compose (later plugins override earlier ones, user config overrides all)
- [ ] AC-6.4: Graphiti plugin: auto-discovers Graphiti-managed graphs, applies Entity/Episodic/Community styling, provides decision doc templates
- [ ] AC-6.5: Plugins distributable as standalone files (publishable, version-controllable)
- [ ] AC-6.6: Plugin loading from local path or (future) registry

**EARS Constraints:**
- **When** multiple plugins define styling for the same label, **the system shall** apply the last-loaded plugin's styling (explicit ordering in config)
- **If** a plugin references a label not present in the current graph, **the system shall** silently ignore those rules

### US-7: Dual Form Factor (v1 web, v1.5 desktop)

**As a** developer
**I want** Lins available as both a web service and a native desktop app
**So that** I can access it from any device on my LAN (web) or have a dedicated app window (desktop)

> "I kind of like having the dock icon and the separation of apps in my main OS. Just having an easier time switching between them. I can quickly open it whenever I want." — User
> "If we need a separate daemon process for the living docs feature, then anyone using this will have to run an installer anyway, right? So why not then also install a native desktop app." — User

**Acceptance Criteria:**
- [ ] AC-7.1: `lins serve` — web server mode, accessible via browser at configurable port
- [ ] AC-7.2: (v1.5) `lins app` — Tauri desktop app with native window, dock icon, window management
- [ ] AC-7.3: 100% shared frontend between web and desktop modes
- [ ] AC-7.4: 100% shared backend logic via `lins-core` Rust library crate
- [ ] AC-7.5: `lins daemon` — headless mode for Living Docs generation (no UI, background process)
- [ ] AC-7.6: Web mode accessible from any device on LAN (Mac Mini, MBP, etc.)

---

## Non-Functional Requirements

### Performance
- Interactive frame rates (>30 FPS) for graphs up to 1,000 nodes
- Usable performance for graphs up to 10,000 nodes
- WebGL-accelerated rendering for scale
- Vocabulary autocomplete: <50ms response
- LLM search: <5s including Cypher generation and execution
- Config hot-reload: <1s

### Reliability
- Graceful degradation when LLM endpoint unavailable (vocabulary search works independently)
- Graceful degradation when embedding endpoint unavailable (property search works independently)
- Living Docs: stale cache beats error display (preserve last successful render on failure)
- FalkorDB connection loss: clear error state, auto-reconnect on availability

### Security
- LLM-generated Cypher sandboxed to read-only (whitelist: MATCH, RETURN, WHERE, ORDER BY, LIMIT, SKIP, UNWIND, WITH, OPTIONAL MATCH, CALL; block: CREATE, MERGE, DELETE, SET, REMOVE, DROP)
- FalkorDB password stored in config, never logged
- No telemetry or external data transmission

### Open-Source Readiness
- MIT license
- Works without LLM configuration (core features: rendering, vocabulary search, config system)
- Works without embedding endpoint (property-based search)
- Documented `lins.toml` schema with examples
- Cross-platform: macOS, Linux (Windows via Tauri)

---

## Embarrassment Criteria

> What would make you embarrassed to ship this, even if it technically works?

- Sluggish rendering that feels amateur (must be WebGL-accelerated, smooth physics)
- A search experience that requires writing raw Cypher (the whole point is abstraction)
- Config that's hard to understand or modify (TOML must be self-documenting with comments)
- A tool that only works with Graphiti (must be generic FalkorDB)

---

## Out of Scope (v1)

- LLM-enhanced search (v1.5)
- Tauri desktop app (v1.5)
- Multi-graph workspaces (v1.5)
- Plugin system (v1.5)
- Additive animated scene building (v1.5)
- Cross-graph search/visualization (v2)
- Embedding-powered semantic search (v2)
- Graphiti auto-discovery plugin (v2)
- Living Docs daemon + templates (v3)
- File Provider / virtual files (v3)
- Fyr integration (v3)
- Mobile support
- Multi-user authentication
- Graph write operations (Lins is read-only for v1-v2)

---

## Risks & Assumptions

### Assumptions
- FalkorDB's `db.labels()`, `db.relationshipTypes()`, `db.propertyKeys()` are sufficient for schema discovery
- Graph sizes in the FalkorDB ecosystem are typically <10K nodes (the tool should handle more, but optimize for this range)
- Users will configure LLM endpoints themselves (no bundled model in v1)
- TOML is the right config format for the Rust ecosystem (over YAML/JSON)

### Risks
- **Rendering library choice**: Sigma.js, G6/AntV, and Cosmos.gl each have trade-offs. Final selection should be based on prototype evaluation. Sigma.js is the current front-runner (WebGL-native, MIT, focused API, good for custom UX layers).
- **LLM Cypher generation quality**: LLMs may generate invalid FalkorDB Cypher (FalkorDB's Cypher subset differs from Neo4j). May need FalkorDB-specific prompt engineering or fine-tuning.
- **FalkorDB Cypher subset**: FalkorDB does not support full Neo4j Cypher. Label disjunction (`|`), some APOC procedures, and certain SET behaviors differ. The query builder must account for this.
- **FalkorDB reserved words crash fulltext queries**: Entity names containing `MAP`, `REDUCE`, `FILTER`, `APPLY`, `LIMIT`, `LOAD`, `AS`, `GROUPBY`, `SORTBY` cause RediSearch syntax errors. The vocabulary search and LLM query builder must sanitize these.
- **FalkorDB query timeouts**: Without `TIMEOUT_DEFAULT` in redis.conf, queries can hang indefinitely. Lins should document that `TIMEOUT_DEFAULT 120000` and `CACHE_SIZE 200` are recommended FalkorDB settings.
- **Vector fields use `vecf32()` wrapper**: FalkorDB vector properties require `vecf32()` in queries, unlike Neo4j's bare arrays. Semantic search must account for this.
- **No HNSW vector indexes by default**: Graphiti never creates vector indexes for FalkorDB. Lins semantic search (v2) would need to create them or recommend creation.
- **Cross-graph performance**: Querying N graphs sequentially and merging adds N* latency. Should use parallel tokio tasks with timeout.
- **Large graph query curation**: Naive `MATCH (n) RETURN n` will be slow for large graphs. All mature tools use server-side query curation (pagination, LOD, cluster summaries). Lins should implement smart loading strategies.

### Conscious Technical Debt (v1)
- Single rendering library chosen upfront (no abstraction layer for swapping)
- No plugin system in v1 — config is monolithic `lins.toml`
- No test suite for LLM-generated Cypher safety (v1 doesn't have LLM search)
- Hardcoded default color palette (auto-derived but not customizable without editing config)

---

## Open Questions

1. **Rendering library**: Sigma.js vs G6/AntV — prototype both before committing? Sigma.js is front-runner for its focused API and WebGL-native rendering, but G6's 15+ layout algorithms and rich behavior system deserve evaluation.
2. **Config schema**: How much of `lins.toml` is auto-generated vs hand-written? Should there be a `lins init` command that generates a starter config?
3. **Cypher query builder**: Should the vocabulary search construct Cypher strings, or use a structured query builder that compiles to Cypher? The latter is safer and more composable.
4. **Scene Actions (Bloom-style)**: Right-click node → run contextual query → results animate in. This is one of Bloom's killer features. Should it be in v1 or v1.5?
5. **Property panel design**: Sidebar panel (always visible) vs modal/popover (on demand)? Sidebar is more Bloom-like but takes screen real estate.

---

## Reference Documents

| Document | Path | Relevance |
|----------|------|-----------|
| Graph viz research (Sep 2025) | `~/dotfiles/docs/research/knowledge-graph/graph-database-visualization-tools-research.md` | Competitive analysis (NOTE: Bolt/FalkorDB claims are incorrect — FalkorDB uses RESP, not Bolt) |
| Graph as docs (Jan 2026) | `~/dotfiles/docs/knowledge_graph_as_documentation_source_of_truth.md` | Living Docs architecture, view definitions, rendering pipeline. **Key**: Section "Converged with Graph Explorer Product Vision, Mar 16, 2026" explicitly merges this research with Lins vision. View definition model (YAML: query + template + output), episode types (`onboarding`, `guidance`, `command`, `architecture`), and `/render-docs` command already designed. |
| Graphiti upstream review | `~/dotfiles/docs/graphiti_upstream_review_2026_03_15.md` | FalkorDB internals, retrieval optimization, fork reliability fixes. **Key gotchas**: reserved word sanitization, `TIMEOUT_DEFAULT`, `CACHE_SIZE`, `vecf32()` wrapper, no HNSW indexes by default. |
| Local embeddings plan | `~/dotfiles/docs/local_embeddings_plan_2026_03_16.md` | Darwin GPU embedding server (192.168.4.1:8080). Qwen3-Embedding-4B Q4_K_M, ~41ms latency, 1024 dims. `FallbackEmbedder` wraps local + OpenAI. Different vector spaces on fallback = degraded vector search, but BM25 fulltext still works. |
| FalkorDB config | `~/.graphiti/redis.conf` | Connection: 127.0.0.1:6380, password `falkordb`. Critical settings: `TIMEOUT_DEFAULT 120000`, `TIMEOUT_MAX 300000`, `CACHE_SIZE 200`. |
| Fyr project | `~/Projects/fyr/` | Personal assistant, Tauri 2 + SvelteKit. Custom entity types (Task, Deadline, Routine) with LLM extraction. Labels: `['Entity', 'Task', 'Entity_fyr-fredrik']`. Primary early user of Lins (memory graph visualization). **Frontend stack consideration**: SvelteKit would maximize code reuse with Fyr. |
| Bloom research (this session) | N/A (captured in this spec) | Bloom UX patterns (Perspectives, Scene Actions, vocabulary search), NVL architecture (Canvas+WebGL+Web Workers), rendering library comparison |

## Version Tier Summary

| Version | Theme | Key Features |
|---------|-------|--------------|
| **v1** | Explorer Core | FalkorDB connection, schema discovery, WebGL rendering, property panels, vocabulary search, `lins.toml`, `lins serve` |
| **v1.5** | Search & UX | LLM search, multi-graph workspaces, Tauri app, plugin overlays, animated scene building |
| **v2** | Intelligence | Graphiti plugin, cross-graph search, embedding-powered semantic search |
| **v3** | Living Docs & Integration | Living Docs daemon + templates, File Provider / virtual files, Fyr integration |
