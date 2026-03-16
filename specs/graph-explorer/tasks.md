# Tasks: Lins — FalkorDB Graph Explorer

## Overview

- **Estimated scope**: Large (multi-version project)
- **v1 scope**: Medium (~2-4 focused sessions for core explorer)
- **Dependencies**: FalkorDB running on localhost:6380, Node.js for frontend build
- **Cut first if needed**: Property panel polish, config editor UI, theme customization

---

## v1: Explorer Core

### Phase 1: Project Foundation

- [ ] **T-1**: Initialize Rust workspace with crate structure
  - `lins-core` (library), `lins-server` (binary)
  - Workspace Cargo.toml with shared dependencies
  - `.gitignore` for `target/`, `node_modules/`, `.env`

- [ ] **T-2**: Set up frontend project (`lins-web/`)
  - SvelteKit + TypeScript (matching Fyr's stack for component reuse)
  - Install Sigma.js + graphology + graphology-layout-forceatlas2
  - Basic `+page.svelte` with graph container div
  - Manrope font, dark theme CSS variables

- [ ] **T-3**: Rendering library prototype evaluation
  - Build minimal Sigma.js prototype: 50 nodes, force-directed layout, click handler
  - Evaluate: interaction feel, label rendering, zoom behavior, WebGL performance
  - If unsatisfactory: quick G6 prototype for comparison
  - **Decision gate**: Commit to rendering library before proceeding
  - Acceptance: AC-1.4, AC-1.5, AC-1.8

### Phase 2: FalkorDB Connection + Schema Discovery

- [ ] **T-4**: Implement FalkorDB connection in `lins-core`
  - `redis-rs` client with connection config
  - `GRAPH.LIST` to enumerate available graphs
  - `GRAPH.QUERY <name> "CALL db.labels()"` for labels
  - `GRAPH.QUERY <name> "CALL db.relationshipTypes()"` for relationship types
  - `GRAPH.QUERY <name> "CALL db.propertyKeys()"` for property keys
  - Parse FalkorDB's compact result format (header + result rows)
  - Acceptance: AC-1.1, AC-1.2, AC-1.3

- [ ] **T-5**: Schema sampling — discover per-label properties
  - For each label: `MATCH (n:<label>) RETURN n LIMIT 10`
  - Extract property keys, infer types, collect sample values
  - Detect vector properties (arrays of floats with consistent dimensions)
  - Cache schema in memory with timestamp
  - Depends on: T-4

- [ ] **T-6**: Graph data loading with smart curation
  - Count first: `MATCH (n) RETURN count(n)` — warn if >1,000 nodes
  - Small graphs (<1,000): load all nodes and edges
  - Large graphs: paginated loading (`SKIP $offset LIMIT $limit`) or label-first (one label at a time)
  - Transform FalkorDB result format → `GraphNode` / `GraphEdge` structs
  - Sanitize entity names for RediSearch reserved words (`MAP`, `REDUCE`, `FILTER`, `APPLY`, `LIMIT`, `LOAD`, `AS`, `GROUPBY`, `SORTBY`)
  - Depends on: T-4

### Phase 3: Web Server + API

- [ ] **T-7**: Set up Axum web server in `lins-server`
  - Static file serving for `lins-web` build output
  - CORS configuration (localhost + configurable LAN)
  - CLI: `lins serve --port 3000 --config lins.toml`
  - Depends on: T-4

- [ ] **T-8**: Implement REST API endpoints
  - `GET /api/graphs` — list available FalkorDB graphs
  - `GET /api/graphs/:name/schema` — schema for a specific graph
  - `GET /api/graphs/:name/data` — full graph data (nodes + edges)
  - `POST /api/graphs/:name/query` — execute read-only Cypher query
  - Query safety validator (block write operations)
  - Depends on: T-4, T-5, T-6, T-7

### Phase 4: Graph Rendering + Interaction

- [ ] **T-9**: Connect frontend to API
  - Fetch graph list → populate graph selector dropdown
  - Fetch graph data → build graphology Graph model
  - Fetch schema → apply auto-derived styling (colors per label)
  - Depends on: T-2, T-3, T-8

- [ ] **T-10**: Implement graph rendering
  - Sigma.js renderer with WebGL
  - ForceAtlas2 layout in web worker
  - Auto-assign distinct colors per label (desaturated palette)
  - Node labels from configurable property (default: "name" or first string property)
  - Edge labels from relationship type
  - Acceptance: AC-1.4, AC-1.5, AC-1.9
  - Depends on: T-3, T-9

- [ ] **T-11**: Implement property panel
  - Click node → show all properties in side panel
  - Click edge → show relationship properties
  - Hide vector/embedding properties by default
  - Show related nodes/edges count with clickable list
  - Acceptance: AC-1.6, AC-1.7
  - Depends on: T-10

- [ ] **T-12**: Implement zoom, pan, drag
  - Mouse wheel zoom with smooth transitions
  - Click-drag to pan canvas
  - Click-drag on node to reposition (manual pinning)
  - Double-click to zoom to node
  - Acceptance: AC-1.8
  - Depends on: T-10

### Phase 5: Config System

- [ ] **T-13**: Implement `lins.toml` parsing
  - Define TOML schema with serde
  - Connection config, styling rules, display preferences
  - `lins init` command to auto-generate config from discovered schema
  - Depends on: T-5

- [ ] **T-14**: Apply config to rendering
  - Read styling rules → apply colors, sizes, captions per label
  - Read display preferences → show/hide properties in panel
  - Hot-reload on file change (notify/fswatch)
  - Acceptance: AC-3.1, AC-3.2, AC-3.4, AC-3.5
  - Depends on: T-10, T-13

### Phase 6: Vocabulary Search

- [ ] **T-15**: Build vocabulary index from schema
  - Index all labels, relationship types, property keys, sample values
  - Trie or prefix-hashmap for instant lookup
  - Refresh on schema re-discovery
  - Depends on: T-5

- [ ] **T-16**: Search bar UI with autocomplete
  - Input field with dropdown suggestions
  - Categorized results: labels, relationship types, property values
  - Keyboard navigation (arrow keys, Enter to select)
  - Acceptance: AC-2.1, AC-2.2
  - Depends on: T-2, T-15

- [ ] **T-17**: Search → Cypher → render
  - Selected vocabulary item → construct Cypher query
  - Example: select label "Entity" → `MATCH (n:Entity) RETURN n`
  - Example: select property value "TTS" → `MATCH (n) WHERE n.name CONTAINS 'TTS' RETURN n`
  - Execute query → add results to graph (additive, not replace)
  - Acceptance: AC-2.1, AC-2.5
  - Depends on: T-8, T-10, T-16

### Phase 7: Polish & Ship

- [ ] **T-18**: Graph selector dropdown
  - List all graphs from `GRAPH.LIST`
  - Switch graph → reload schema + data
  - Show node/edge counts per graph

- [ ] **T-19**: Error handling and edge cases
  - Connection failure → clear error message with retry
  - Empty graph → helpful empty state
  - Large graph warning → suggest filtering
  - Query timeout → graceful handling

- [ ] **T-20**: Documented example config
  - `lins.toml.example` with inline comments explaining every option
  - README.md with quickstart: install → connect → explore

- [ ] **T-21**: `cargo install` readiness
  - Ensure `lins serve` works from `cargo install lins`
  - Frontend build embedded in binary (include_dir or rust-embed)
  - Test on clean machine (no dev dependencies required)

---

## v1.5: Search & UX

### LLM-Enhanced Search

- [ ] **T-22**: LLM integration in `lins-core`
  - OpenAI-compatible API client (reqwest)
  - System prompt with graph schema context → NL→Cypher translation
  - Read-only Cypher validation on LLM output
  - Config: endpoint URL, model, API key env var
  - Graceful fallback when LLM unavailable

- [ ] **T-23**: Two-tier typeahead UI
  - Tier 1: instant vocabulary matches (existing T-16)
  - Tier 2: async LLM suggestions (300ms debounce)
  - Visual separation between tiers
  - Enter on NL sentence → full LLM search
  - Acceptance: AC-2.3, AC-2.4

### Multi-Graph Workspaces

- [ ] **T-24**: Workspace config in `lins.toml`
  - Named workspace with list of graphs
  - Per-workspace styling overrides
  - Workspace selector in UI

- [ ] **T-25**: Workspace switching
  - Load workspace → show all graphs in sidebar
  - Quick-switch between graphs within workspace
  - Acceptance: AC-4.1, AC-4.2

### Tauri Desktop App

- [ ] **T-26**: Set up `lins-app` Tauri crate
  - Tauri 2 config
  - Embed same `lins-web` frontend
  - Tauri commands → `lins-core` calls

- [ ] **T-27**: Desktop app features
  - Dock icon, native window management
  - System tray icon for daemon status (future)
  - Platform builds: `.dmg` (macOS), `.msi` (Windows), `.AppImage` (Linux)
  - Acceptance: AC-7.2, AC-7.3, AC-7.4

### Plugin System

- [ ] **T-28**: Plugin loading and overlay composition
  - Parse plugin TOML files
  - Merge: base config → plugin overlays (in order) → user overrides
  - Plugin discovery from config-specified paths
  - Acceptance: AC-6.1, AC-6.2, AC-6.3

- [ ] **T-29**: Graphiti plugin (first plugin)
  - `lins-graphiti.toml` with Entity/Episodic/Community styling
  - Schema hints for expected labels and relationships
  - Property display rules (hide embeddings, format dates)
  - Acceptance: AC-6.4

### Scene Building

- [ ] **T-30**: Additive scene animation
  - New search results animate into existing graph (not replace)
  - Smooth node/edge appearance with physics settling
  - Clear scene button to reset
  - Acceptance: AC-2.5

- [ ] **T-31**: Scene Actions (Bloom-inspired)
  - Right-click node → context menu with query options
  - "Expand neighbors" → fetch connected nodes
  - "Find similar" → nodes with same label
  - Custom scene actions definable in config

---

## v2: Intelligence

### Cross-Graph Search

- [ ] **T-32**: Multi-graph query engine
  - Query N graphs in parallel (tokio tasks)
  - Merge results with graph-name namespacing
  - Visual distinction: color-code nodes by source graph
  - Acceptance: AC-4.3, AC-4.4

### Embedding-Powered Semantic Search

- [ ] **T-33**: Vector similarity search
  - Detect vector properties on nodes
  - Embed search query via configured embedding endpoint
  - Cosine similarity against node embeddings
  - Rank and display results

### Graphiti Auto-Discovery Plugin

- [ ] **T-34**: Graphiti-aware plugin features
  - Auto-discover all Graphiti-managed graphs from FalkorDB
  - Detect Graphiti schema (Entity/Episodic/Community + edge types)
  - Map Graphiti custom entity types from ontology config
  - Time-based exploration (filter by valid_at, created_at)

---

## v3: Living Docs & Integration

### Living Docs Daemon

- [ ] **T-35**: View definition parsing
  - TOML config: name, graph, Cypher query, template path, output path
  - Template loading (Tera engine)

- [ ] **T-36**: Template rendering pipeline
  - Execute Cypher query → pass results to Tera template → write markdown file
  - Metadata header in generated files (source graph, timestamp, episode count)
  - Staleness check: only re-render when graph data changed

- [ ] **T-37**: Change detection daemon
  - Poll-based: check `MAX(e.created_at)` or node/edge counts periodically
  - Configurable poll interval (default: 60s)
  - `lins daemon` CLI command for headless operation
  - LaunchAgent/systemd service file for background operation
  - Acceptance: AC-5.1, AC-5.2, AC-5.3, AC-5.4, AC-5.8

### Web API for Live Docs

- [ ] **T-38**: `GET /api/docs/:view` endpoint
  - Execute view query and render template on-demand
  - Return markdown string
  - Cache with configurable TTL
  - Acceptance: AC-5.5, AC-5.6

### File Provider (v3 stretch)

- [ ] **T-39**: Research File Provider API feasibility
  - macOS File Provider API for virtual files
  - Cross-platform alternatives (FUSE-T for macOS, FUSE for Linux)
  - Decision: implement or defer based on complexity assessment
  - Acceptance: AC-5.7

### Fyr Integration

- [ ] **T-40**: Define Fyr integration API
  - HTTP API for Fyr to query graph data
  - Shared `lins-core` crate as library dependency (if Fyr is Rust)
  - Plugin interface for Fyr-specific views and templates

---

## Verification Checklist

Before marking each version complete:

### v1
- [ ] `cargo install lins` works on clean machine
- [ ] `lins serve` → browser shows graph from FalkorDB
- [ ] Click nodes/edges → properties display correctly
- [ ] Search bar autocomplete works with FalkorDB schema
- [ ] `lins.toml` hot-reload changes styling
- [ ] Tested with at least 2 different FalkorDB graphs (Graphiti + non-Graphiti)
- [ ] README with quickstart guide
- [ ] MIT LICENSE file

### v1.5
- [ ] LLM search generates valid Cypher and renders results
- [ ] LLM search degrades gracefully without LLM config
- [ ] Tauri app launches and has feature parity with web mode
- [ ] Plugin overlay merging works (base → plugin → user)
- [ ] Graphiti plugin applies correct styling to Graphiti graphs

### v2
- [ ] Cross-graph search returns merged results from multiple graphs
- [ ] Semantic search finds relevant nodes via embeddings
- [ ] Graphiti plugin auto-discovers all Graphiti-managed graphs

### v3
- [ ] Living Docs daemon generates markdown files
- [ ] Files auto-update when graph changes
- [ ] Web API serves live doc content

---

## Notes

- **Build order rationale**: Visual exploration (A) first because "seeing the graph for the first time" is the core value proposition. Search (B) second because it transforms the explorer from a viewer into a discovery tool. Living Docs (C) third because it's a separate daemon that can be built independently. User: "A and B first."
- **Rendering library**: Sigma.js is the front-runner but T-3 includes a prototype evaluation gate. Don't over-invest before confirming the interaction feel.
- **Config-as-contract**: The `lins.toml` design is central to the plugin architecture. Get the schema right in v1 even though plugins aren't loaded until v1.5 — the config format is the contract.
- **Conscious debt (v1)**: No plugin system, no LLM search, no multi-graph, no desktop app. These are architectural extensions, not refactors — the v1 code structure supports them without rewriting.
- **FalkorDB Cypher subset**: Test all generated Cypher against FalkorDB specifically. Neo4j Cypher docs may show features FalkorDB doesn't support (label disjunction `|`, some APOC, certain SET behaviors).
- **FalkorDB prerequisites**: Document that `TIMEOUT_DEFAULT 120000`, `TIMEOUT_MAX 300000`, and `CACHE_SIZE 200` are recommended redis.conf settings. Without these, queries can hang indefinitely and performance degrades from query plan recompilation.
- **SvelteKit committed**: Matches Fyr's stack (Tauri 2 + SvelteKit). Component reuse between projects is a strategic goal.
- **Custom entity types**: Graphiti plugin (v2) should detect custom entity types via label introspection — labels like `['Entity', 'Task', 'Entity_fyr-fredrik']` indicate custom types with domain-specific attributes.
- **Living Docs reference**: The `knowledge_graph_as_documentation_source_of_truth.md` already defines view definitions, episode types, and the rendering pipeline. Lins Living Docs (v3) should build on this design, not re-invent it.
