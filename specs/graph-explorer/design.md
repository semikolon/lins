# Design: Lins — FalkorDB Graph Explorer

> **Note (Mar 17, 2026):** This design reflects the SvelteKit + Sigma.js architecture from the initial build. The rendering technology is now an **open decision** — egui + wgpu (Rust-native, compiles to WASM for browser) is under evaluation. The visual design direction has also evolved: mid-to-dark theme (not pure dark, not light), Söhne-style font, Bloom-inspired UX with great defaults. See `docs/founding_session_2026_03_16.md` §8-9 for current state.

## Tech Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| **Backend language** | Rust | Performance, stability, `redis-rs` maturity. User: "programmer happiness and stellar performance and stability" |
| **Web framework** | Axum | Async, tower-based, ergonomic. Natural fit for Rust web services |
| **DB connection** | `redis-rs` | FalkorDB speaks RESP (Redis protocol), not Bolt. `redis-rs` is the mature Rust Redis client |
| **Frontend** | SvelteKit + TypeScript | Shared stack with Fyr (Tauri 2 + SvelteKit), compiles away framework overhead, excellent TS support |
| **Graph rendering** | Sigma.js + graphology (front-runner) | WebGL-native, MIT, focused API. See "Rendering Library Decision" below |
| **Desktop app** | Tauri 2 (v1.5) | ~5-10MB vs Electron's 200MB+. Shared frontend. Native feel |
| **Config format** | TOML | Rust-native (serde), human-readable, version-controllable |
| **Template engine** | Tera (v3, Living Docs) | Rust-native Jinja2 syntax, zero-cost, well-maintained |

### Rendering Library Decision

**Front-runner: Sigma.js + graphology**

Based on research (Neo4j NVL architecture, Bloom UX patterns, and the full rendering library landscape):

| Criterion | Sigma.js | G6/AntV |
|-----------|----------|---------|
| Renderer | WebGL-native (designed for it) | Multi-renderer (Canvas/SVG/WebGL) |
| Scale | 100K+ nodes | Medium-large |
| API surface | Small, focused — you build on top | Large, opinionated framework |
| Layout | External via graphology-layout | 15+ built-in |
| Custom UX | Easy — thin rendering layer | Harder — must work within G6's abstractions |
| License | MIT | MIT |
| npm downloads | ~50K/wk | ~137K/wk |

**Why Sigma.js**: Lins needs a custom UX layer (Perspectives, search-driven exploration, scene building). Sigma.js is a thin, fast WebGL renderer — it draws nodes and edges and gets out of the way. The Bloom-inspired UX (search bar, property panels, scene actions) is all custom code on top. A framework like G6 would fight this customization.

**graphology** provides the graph data model, algorithms (centrality, shortest path, community detection), and layout algorithms (ForceAtlas2, force-directed, circular) as a composable library.

**Alternative to evaluate**: G6 v5 deserves a quick prototype if Sigma.js's external layout story proves insufficient. Decision should be finalized during T-3 (prototype evaluation).

**Cosmos.gl rejected**: CC-BY-NC license on the wrapper is incompatible with MIT open-source release. Also massive overkill for typical FalkorDB graph sizes.

### Frontend Framework: SvelteKit

**Decision**: SvelteKit, matching Fyr's stack (Tauri 2 + SvelteKit). Rationale:
- **Component reuse with Fyr** — graph rendering components, property panels, search UI can be shared as a Svelte component library
- **Compiles away** — no runtime framework overhead, just DOM operations
- **Tauri 2 integration** — proven in Fyr, `@tauri-apps/api` works natively with SvelteKit
- **Lighter than React** — smaller bundle, faster hydration, less boilerplate
- **Excellent TypeScript support** — native TS in `<script lang="ts">` blocks

---

## Architecture Overview

### Crate Structure

```
lins/
├── Cargo.toml                    (workspace)
├── lins-core/                    (library crate — all business logic)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── connection.rs         # FalkorDB connection via redis-rs
│   │   ├── schema.rs             # Schema introspection (labels, types, properties)
│   │   ├── query.rs              # Query builder (vocabulary → Cypher)
│   │   ├── config.rs             # lins.toml parsing, plugin overlay merging
│   │   ├── plugin.rs             # Plugin loading, overlay composition
│   │   ├── workspace.rs          # Multi-graph workspace management
│   │   ├── search/
│   │   │   ├── mod.rs
│   │   │   ├── vocabulary.rs     # Instant autocomplete from schema cache
│   │   │   └── llm.rs            # LLM-enhanced NL→Cypher (v1.5)
│   │   ├── living_docs/          # (v3)
│   │   │   ├── mod.rs
│   │   │   ├── daemon.rs         # Change detection + regeneration
│   │   │   ├── template.rs       # Tera template rendering
│   │   │   └── watcher.rs        # Graph change polling
│   │   └── graph_types.rs        # Node, Edge, Property type definitions
│   └── Cargo.toml
│
├── lins-server/                  (binary crate — web server)
│   ├── src/
│   │   ├── main.rs
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   ├── graph.rs          # GET /api/graph/:name — nodes, edges, schema
│   │   │   ├── search.rs         # POST /api/search — vocabulary + LLM search
│   │   │   ├── config.rs         # GET/PUT /api/config — runtime config
│   │   │   ├── workspace.rs      # GET /api/workspaces — workspace management
│   │   │   └── docs.rs           # GET /api/docs/:view — live doc rendering (v3)
│   │   └── static_files.rs       # Serve frontend build output
│   └── Cargo.toml
│
├── lins-app/                     (Tauri crate — desktop app, v1.5)
│   ├── src/
│   │   ├── main.rs
│   │   └── commands.rs           # Tauri commands → lins-core calls
│   ├── tauri.conf.json
│   └── Cargo.toml
│
├── lins-web/                     (frontend — shared between server and Tauri)
│   ├── src/
│   │   ├── main.ts
│   │   ├── routes/+page.svelte   # Main explorer view
│   │   ├── components/
│   │   │   ├── GraphCanvas.svelte   # Sigma.js renderer wrapper
│   │   │   ├── SearchBar.svelte     # Two-tier typeahead (vocabulary + LLM)
│   │   │   ├── PropertyPanel.svelte # Node/edge property inspector
│   │   │   ├── GraphSelector.svelte # Graph/workspace switcher
│   │   │   └── ConfigEditor.svelte  # Visual config editor (future)
│   │   ├── lib/
│   │   │   ├── api.ts            # REST API client (works with both server and Tauri)
│   │   │   ├── graph.ts          # graphology graph model management
│   │   │   └── search.ts         # Search state management
│   │   └── styles/
│   │       └── theme.ts          # Forest green palette, Manrope font
│   ├── svelte.config.js
│   ├── vite.config.ts
│   ├── package.json
│   └── tsconfig.json
│
├── plugins/                      (example plugins)
│   ├── graphiti/
│   │   └── lins-graphiti.toml    # Graphiti schema awareness + styling
│   └── README.md
│
├── specs/                        (this directory)
│   └── graph-explorer/
│       ├── requirements.md
│       ├── design.md
│       └── tasks.md
│
├── lins.toml.example             # Documented example config
└── README.md
```

### Component Interaction

```
                     ┌─────────────────────────────────────────────┐
                     │              lins-web (frontend)             │
                     │                                             │
                     │  SearchBar ──→ GraphCanvas ←── PropertyPanel│
                     │      │              ↑                ↑      │
                     │      └──────────────┼────────────────┘      │
                     │                     │ (graphology model)     │
                     └─────────────────────┼───────────────────────┘
                                           │
                              REST API / Tauri commands
                                           │
                     ┌─────────────────────┼───────────────────────┐
                     │           lins-server / lins-app             │
                     │                     │                        │
                     │              ┌──────┴──────┐                │
                     │              │  lins-core  │                │
                     │              │             │                │
                     │  ┌───────┬───┴───┬─────────┴──────┐        │
                     │  │Schema │Query  │Config  │Search  │        │
                     │  │Disco  │Builder│Overlay │Engine  │        │
                     │  └───┬───┴───┬───┴────┬───┴────┬───┘        │
                     │      │       │        │        │            │
                     └──────┼───────┼────────┼────────┼────────────┘
                            │       │        │        │
                     ┌──────┴───────┴──┐  ┌──┴──┐  ┌──┴──────────┐
                     │    FalkorDB     │  │TOML │  │LLM Endpoint │
                     │  (redis:6380)   │  │files│  │  (optional)  │
                     └─────────────────┘  └─────┘  └─────────────┘
```

---

## Data Models

### FalkorDB Connection

```rust
/// Connection configuration parsed from lins.toml
pub struct ConnectionConfig {
    pub host: String,           // default: "127.0.0.1"
    pub port: u16,              // default: 6380
    pub password: Option<String>,
    pub default_graph: Option<String>,
}
```

### Schema (Auto-Discovered)

```rust
/// Auto-discovered graph schema from FalkorDB introspection
pub struct GraphSchema {
    pub graph_name: String,
    pub labels: Vec<String>,                    // from CALL db.labels()
    pub relationship_types: Vec<String>,         // from CALL db.relationshipTypes()
    pub property_keys: Vec<String>,             // from CALL db.propertyKeys()
    pub label_properties: HashMap<String, Vec<PropertyInfo>>,  // sampled per label
    pub node_count: usize,
    pub edge_count: usize,
    pub discovered_at: DateTime<Utc>,
}

pub struct PropertyInfo {
    pub key: String,
    pub value_type: PropertyType,  // String, Number, Boolean, Vector, DateTime, etc.
    pub sample_values: Vec<String>, // for autocomplete
    pub is_vector: bool,           // detected vector embeddings
    pub vector_dimensions: Option<usize>,
}

pub enum PropertyType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Vector(usize),  // dimension count
    Array,
    Map,
    Null,
}
```

### Graph Data (Query Results)

```rust
pub struct GraphNode {
    pub id: i64,                     // FalkorDB internal ID
    pub labels: Vec<String>,
    pub properties: HashMap<String, PropertyValue>,
}

pub struct GraphEdge {
    pub id: i64,
    pub relationship_type: String,
    pub source_id: i64,
    pub target_id: i64,
    pub properties: HashMap<String, PropertyValue>,
}

pub struct QueryResult {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub query_time_ms: f64,
    pub source_graph: String,
}
```

### Config (lins.toml)

```toml
# lins.toml — auto-generated, user-editable, plugin-overlayable

[connection]
host = "127.0.0.1"
port = 6380
password = "falkordb"
default_graph = "sarpetorp"

[server]
port = 3000
host = "0.0.0.0"

# Plugins loaded in order — later overrides earlier
plugins = [
    "plugins/graphiti/lins-graphiti.toml",
]

# Styling rules — auto-generated from schema, customizable
[styling.defaults]
font_family = "Manrope"
background = "#1a1a2e"        # dark theme
edge_color = "#4a5568"

[styling.labels.Entity]
color = "#2d6a4f"             # forest green
size = "property:degree"       # size by connection count
caption = "name"              # property to display as label

[styling.labels.Episodic]
color = "#6b7280"
shape = "diamond"
caption = "source_description"

[styling.labels.Community]
color = "#7c3aed"
shape = "hexagon"
caption = "name"

# Property panel — which properties to show, in what order
[display.properties]
show = ["name", "summary", "created_at", "valid_at"]
hide = ["name_embedding", "fact_embedding", "group_id"]  # hide vectors, internal fields

# Search configuration
[search]
# LLM endpoint for natural language search (optional)
# llm_endpoint = "https://api.openai.com/v1"
# llm_model = "gpt-4o-mini"
# llm_api_key_env = "OPENAI_API_KEY"  # read from environment variable

# Embedding endpoint for semantic search (optional, v2)
# embedding_endpoint = "http://192.168.4.1:8080/v1"
# embedding_dimensions = 1024

# Workspaces (v1.5)
# [[workspaces]]
# name = "development"
# graphs = ["dotfiles", "brf-auto"]
# plugins = ["plugins/graphiti/lins-graphiti.toml"]

# Living Docs (v3)
# [[living_docs.views]]
# name = "DECISIONS.md"
# graph = "sarpetorp"
# query = "MATCH (e:Episodic) WHERE e.source_description CONTAINS 'Decision' RETURN e ORDER BY e.valid_at DESC"
# template = "templates/decisions.md.tera"
# output = "./DECISIONS.md"
```

### Plugin Config Overlay

```toml
# lins-graphiti.toml — Graphiti plugin overlay
# Loaded after base config, before user overrides

[plugin]
name = "graphiti"
version = "0.1.0"
description = "Graphiti knowledge graph schema awareness and styling"

# Schema hints — tell Lins what to expect
[schema.hints]
expected_labels = ["Entity", "Episodic", "Community"]
expected_relationships = ["RELATES_TO", "MENTIONS", "HAS_MEMBER"]

# Styling overlays
[styling.labels.Entity]
color = "#2d6a4f"
icon = "circle"
caption = "name"
size = "property:degree"
tooltip = ["name", "summary"]

[styling.labels.Episodic]
color = "#b45309"
icon = "diamond"
caption = "source_description"
size = 8
tooltip = ["source_description", "created_at", "valid_at"]

[styling.labels.Community]
color = "#7c3aed"
icon = "hexagon"
caption = "name"
size = 12

[styling.relationships.RELATES_TO]
color = "#059669"
label = "fact"

[styling.relationships.MENTIONS]
color = "#6b7280"
style = "dashed"

[styling.relationships.HAS_MEMBER]
color = "#7c3aed"
style = "dotted"

# Property display
[display.properties]
hide = ["name_embedding", "fact_embedding", "group_id", "uuid"]
date_format = "%Y-%m-%d %H:%M"

# Living Docs templates (v3)
# [[living_docs.templates]]
# name = "decisions"
# description = "Architectural decisions from Graphiti episodes"
# file = "templates/decisions.md.tera"
```

---

## API Design

### REST API (lins-server)

| Method | Path | Description | Response |
|--------|------|-------------|----------|
| `GET` | `/api/graphs` | List available FalkorDB graphs | `[{name, node_count, edge_count}]` |
| `GET` | `/api/graphs/:name/schema` | Get auto-discovered schema for a graph | `GraphSchema` |
| `GET` | `/api/graphs/:name/data` | Get graph nodes and edges (with optional filters) | `QueryResult` |
| `POST` | `/api/graphs/:name/query` | Execute a Cypher query (read-only) | `QueryResult` |
| `POST` | `/api/search/vocabulary` | Vocabulary autocomplete | `[{label, type, value}]` |
| `POST` | `/api/search/llm` | LLM-enhanced NL→Cypher search (v1.5) | `{cypher, result: QueryResult}` |
| `GET` | `/api/config` | Get current merged config | `LinsConfig` |
| `PUT` | `/api/config` | Update runtime config (future) | `LinsConfig` |
| `GET` | `/api/workspaces` | List workspace definitions (v1.5) | `[Workspace]` |
| `GET` | `/api/docs/:view` | Render Living Doc view live (v3) | Markdown string |

### Query Safety

All Cypher queries — whether user-composed, vocabulary-built, or LLM-generated — pass through a **read-only validator** before execution:

```rust
pub fn validate_read_only(cypher: &str) -> Result<(), QuerySafetyError> {
    let blocked_keywords = [
        "CREATE", "MERGE", "DELETE", "DETACH", "SET",
        "REMOVE", "DROP", "CALL db.idx", "GRAPH.DELETE",
    ];
    let upper = cypher.to_uppercase();
    for keyword in &blocked_keywords {
        if upper.contains(keyword) {
            return Err(QuerySafetyError::WriteOperation(keyword.to_string()));
        }
    }
    Ok(())
}
```

Note: This is a simple keyword check. A more robust approach (parsing Cypher AST) is a v2 improvement. For v1, keyword blocking is sufficient since all queries are either auto-generated from vocabulary or manually typed by the developer user.

### FalkorDB Reserved Word Sanitization

FalkorDB's RediSearch fulltext engine crashes on reserved words in entity names. The vocabulary search and query builder must sanitize:

```rust
const REDISEARCH_RESERVED: &[&str] = &[
    "MAP", "REDUCE", "FILTER", "APPLY", "LIMIT",
    "LOAD", "AS", "GROUPBY", "SORTBY",
];

pub fn sanitize_fulltext_query(input: &str) -> String {
    let upper = input.to_uppercase();
    for reserved in REDISEARCH_RESERVED {
        if upper.contains(reserved) {
            // Escape with backslash or use property-based search instead
            return input.replace(reserved, &format!("\\{}", reserved));
        }
    }
    input.to_string()
}
```

### FalkorDB Recommended Configuration

Lins should document these `redis.conf` settings as prerequisites:

```
# Prevent indefinite query hangs
loadmodule /path/to/falkordb.so TIMEOUT_DEFAULT 120000 TIMEOUT_MAX 300000

# Query plan cache (default 25 causes recompilation overhead)
# CACHE_SIZE 200 recommended for interactive use
loadmodule /path/to/falkordb.so CACHE_SIZE 200
```

### FalkorDB Protocol

FalkorDB uses Redis commands with graph extensions:

```
# List graphs
GRAPH.LIST

# Query a specific graph
GRAPH.QUERY <graph_name> "MATCH (n) RETURN n LIMIT 10"

# Schema introspection
GRAPH.QUERY <graph_name> "CALL db.labels()"
GRAPH.QUERY <graph_name> "CALL db.relationshipTypes()"
GRAPH.QUERY <graph_name> "CALL db.propertyKeys()"

# Node/edge counts
GRAPH.QUERY <graph_name> "MATCH (n) RETURN count(n)"
GRAPH.QUERY <graph_name> "MATCH ()-[r]->() RETURN count(r)"
```

All commands are sent via `redis-rs` using the standard Redis RESP protocol on port 6380.

---

## Frontend Architecture

### Graph Rendering (Sigma.js + graphology)

```typescript
// Core graph management
import Graph from 'graphology';
import Sigma from 'sigma';

// Graph model — single source of truth
const graph = new Graph({ multi: true, type: 'directed' });

// Sigma renderer — WebGL, handles all rendering
const sigma = new Sigma(graph, container, {
  renderLabels: true,
  labelRenderedSizeThreshold: 8,  // hide labels when zoomed out
  defaultNodeColor: '#2d6a4f',
  defaultEdgeColor: '#4a5568',
});

// Layout — ForceAtlas2 in web worker
import FA2Layout from 'graphology-layout-forceatlas2/worker';
const layout = new FA2Layout(graph, {
  settings: { gravity: 1, scalingRatio: 2 },
});
layout.start();
```

### Search Bar (Two-Tier Typeahead)

```
┌──────────────────────────────────────────────────┐
│ 🔍 Search graph...                               │
├──────────────────────────────────────────────────┤
│ ── Schema matches (instant) ──────────────────── │
│   Entity          (label, 142 nodes)             │
│   Episodic        (label, 83 nodes)              │
│   "encryption"    (property value, 3 nodes)      │
│ ── Suggestions (LLM, v1.5) ─────────────────── │
│   "decisions about TTS architecture"             │
│   "decisions about security model"               │
│   "decisions about deployment"                   │
│ ────────────────────────────────────────────────│
│   ⏎ Enter for full search  │  ⌘K for Cypher    │
└──────────────────────────────────────────────────┘
```

### Property Panel

```
┌─────────────────────────────────┐
│ ● Entity: TTS Daemon            │  ← label + name
├─────────────────────────────────┤
│ name         TTS Daemon         │
│ summary      Multi-tier TTS     │
│              narration and      │
│              significance...    │
│ created_at   2026-01-15T10:30Z  │
│ valid_at     2026-03-10T14:00Z  │
├─────────────────────────────────┤
│ Relationships (12)              │
│  → RELATES_TO  ElevenLabs       │
│  → RELATES_TO  Ruby Voice       │
│  ← MENTIONS    Episode #42      │
│  ← MENTIONS    Episode #87      │
│  [Show all 12...]               │
└─────────────────────────────────┘
```

### Visual Design

Following the user's established design sensibility (forest green, Manrope, desaturated):

- **Background**: Dark theme (`#1a1a2e` → `#16213e`)
- **Primary accent**: Forest green (`#2d6a4f`)
- **Secondary**: Muted purple (`#7c3aed` at 60% opacity)
- **Text**: `#e2e8f0` (light gray on dark)
- **Font**: Manrope (headings, UI) + JetBrains Mono (code, Cypher)
- **Node colors**: Auto-derived distinct colors per label, desaturated palette
- **Interactions**: Subtle hover glow, smooth zoom transitions, physics-based layout settling

---

## Integration Points

### FalkorDB (Primary)
- Connection via `redis-rs` (RESP protocol)
- Multi-graph via `GRAPH.QUERY <name> "<cypher>"` and `GRAPH.LIST`
- Schema via `CALL db.labels()`, `CALL db.relationshipTypes()`, `CALL db.propertyKeys()`
- No Bolt, no Neo4j driver — pure Redis protocol

### LLM Endpoint (Optional, v1.5)
- OpenAI-compatible API (works with OpenAI, local Ollama, vLLM, etc.)
- Used for: NL→Cypher translation, semantic search completions
- Configurable in `lins.toml` via endpoint URL, model name, API key env var
- Graceful degradation when unavailable

### Embedding Endpoint (Optional, v2)
- OpenAI-compatible embedding API
- Can use local Darwin server (192.168.4.1:8080) or cloud API
- Detects vector properties on nodes, enables semantic similarity search
- Configurable in `lins.toml`

### Graphiti Plugin (v2)
- Auto-discovers Graphiti-managed graphs (detect Graphiti schema: Entity/Episodic/Community labels + RELATES_TO/MENTIONS/HAS_MEMBER edges)
- Detects **custom entity types** via label introspection — labels like `['Entity', 'Task', 'Entity_fyr-fredrik']` indicate Fyr custom types. Plugin can auto-style by custom type.
- Applies Entity/Episodic/Community styling + custom entity type colors
- Time-based exploration: filter nodes by `valid_at` / `created_at` ranges (Graphiti-specific temporal metadata)
- Provides Living Docs templates for decisions, entity summaries, episode timelines
- Distributed as a `lins-graphiti.toml` config overlay

---

## Trade-off Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Generic FalkorDB vs Graphiti-specific | Generic with Graphiti plugin | Wider open-source value; plugin system handles domain specifics |
| Config-as-contract vs smart-defaults vs dumb-core | Config-as-contract with overlays | Version-controllable, shareable, composable: auto-discovered → plugin → user |
| Tauri vs Web vs Both | Both (shared core) | User wants dock icon + LAN accessibility; ~10-15% extra code |
| Sigma.js vs G6 | Sigma.js (front-runner, prototype first) | Focused API, WebGL-native, easier to build custom UX on top |
| Vocabulary search + LLM vs LLM-only | Layered (vocabulary instant + LLM async) | Works without LLM config; fast baseline with optional intelligence |
| Single-graph vs multi-graph | Multi-graph workspaces | User: "I would like to enable cross-graph search and queries" |
| Read-only vs read-write | Read-only (all versions) | Safer, simpler. Graph writes happen through Graphiti/MCP, not the explorer |
| TOML vs YAML | TOML | Rust-native serde support, cleaner syntax for nested config |
| Tera vs Handlebars | Tera (for Living Docs, v3) | Rust-native Jinja2 syntax, zero-cost, good template inheritance |

---

## Security Considerations

- **Query sandboxing**: All Cypher queries validated as read-only before execution
- **Password handling**: FalkorDB password in config file, never logged or exposed via API
- **LLM API keys**: Read from environment variables, not stored in config files
- **No auth in v1**: Single-user tool. Multi-user auth deferred to Fyr integration (v3)
- **CORS**: Web server allows localhost by default, configurable for LAN access

## Performance Considerations

- **Schema caching**: Auto-discovered schema cached in memory, refreshable on demand
- **Smart query curation**: Never `MATCH (n) RETURN n` for large graphs. Strategies:
  - **Count first**: `MATCH (n) RETURN count(n)` before loading — warn if >1,000 nodes
  - **Paginated loading**: `MATCH (n) RETURN n SKIP $offset LIMIT $limit` with progressive rendering
  - **Label-first**: Load one label at a time, user expands incrementally
  - **Neighborhood expansion**: Start with search result, expand neighbors on demand (Bloom's primary pattern)
- **Layout in Web Workers**: ForceAtlas2 runs off main thread, doesn't block rendering
- **Vocabulary index**: Pre-built from schema cache, instant lookup via trie or hashmap
- **LLM debounce**: 300ms debounce on typeahead to avoid excessive API calls
- **FalkorDB connection pooling**: `redis-rs` connection pool for concurrent queries
- **Performance targets**: Sub-500ms for typical graphs (250 nodes), <2s for large (5K nodes). FalkorDB structure queries are fast (1-50ms) — latency is dominated by rendering, not DB access.
