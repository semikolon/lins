# Lins

A lightweight FalkorDB graph explorer with search-driven exploration and living docs.

**Lins** (Swedish: *lens*) fills a genuine gap in the FalkorDB ecosystem — there is no good visualization tool for FalkorDB graphs. Neo4j Bloom and Desktop require Bolt protocol; FalkorDB speaks Redis/RESP. Lins connects natively.

## Features (v1)

- **Connect to FalkorDB** via Redis protocol (RESP)
- **Auto-discover schema** — labels, relationship types, property keys
- **WebGL graph rendering** — Sigma.js + graphology with ForceAtlas2 layout
- **Property panel** — click any node or edge to inspect properties
- **Vocabulary search** — instant autocomplete from graph schema
- **Config-driven styling** — `lins.toml` for colors, sizes, display rules
- **Multiple graphs** — switch between named FalkorDB graphs

## Quickstart

### Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- [Node.js](https://nodejs.org/) (20+)
- FalkorDB running on Redis protocol (default: `127.0.0.1:6380`)

### FalkorDB recommended settings

```
# In your redis.conf — prevents query hangs and improves cache performance
loadmodule /path/to/falkordb.so TIMEOUT_DEFAULT 120000 TIMEOUT_MAX 300000 CACHE_SIZE 200
```

### Run

```bash
# Clone and build
git clone https://github.com/semikolon/lins.git
cd lins

# Configure connection
cp lins.toml.example lins.toml
# Edit lins.toml with your FalkorDB host/port/password

# Start backend
cargo run --bin lins-server -- serve --config lins.toml

# In another terminal — start frontend dev server
cd lins-web
npm install
npm run dev

# Open http://localhost:5173
```

## Architecture

```
lins-core/        Rust library — FalkorDB connection, schema discovery,
                  query builder, config parsing, vocabulary index

lins-server/      Rust binary — Axum web server, REST API, serves frontend

lins-web/         SvelteKit + TypeScript — Sigma.js graph renderer,
                  search bar, property panel, graph selector
```

### Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust, Axum, redis-rs |
| Frontend | SvelteKit, TypeScript, Sigma.js, graphology |
| DB Protocol | Redis/RESP (FalkorDB native) |
| Graph Rendering | WebGL via Sigma.js |
| Layout | ForceAtlas2 (web worker) |
| Config | TOML |

## Configuration

See [`lins.toml.example`](lins.toml.example) for all options.

```toml
[connection]
host = "127.0.0.1"
port = 6380
password = "falkordb"

[server]
port = 3000

[display.properties]
hide = ["name_embedding", "fact_embedding", "group_id", "uuid"]
```

## Roadmap

| Version | Theme | Key Features |
|---------|-------|--------------|
| **v1** | Explorer Core | FalkorDB connection, schema discovery, WebGL rendering, vocabulary search |
| **v1.5** | Search & UX | LLM-enhanced NL→Cypher search, workspaces, Tauri desktop app, plugin system |
| **v2** | Intelligence | Graphiti plugin, cross-graph search, embedding-powered semantic search |
| **v3** | Living Docs | Auto-generated markdown from graph queries, File Provider, Fyr integration |

## Plugin System (v1.5)

Lins uses a **config-as-contract** model: auto-discovered defaults → plugin overlays → user config overrides.

```toml
# lins-graphiti.toml — plugin overlay for Graphiti knowledge graphs
[plugin]
name = "graphiti"

[styling.labels.Entity]
color = "#2d6a4f"
caption = "name"

[styling.labels.Episodic]
color = "#b45309"
shape = "diamond"
```

## License

MIT
