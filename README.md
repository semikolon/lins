# Lins

A lightweight FalkorDB graph explorer with search-driven exploration and living docs.

**Lins** (Swedish: *lens*) fills a genuine gap in the FalkorDB ecosystem — there is no good visualization tool for FalkorDB graphs. Neo4j Bloom and Desktop require Bolt protocol; FalkorDB speaks Redis/RESP. Lins connects natively.

## Features (v1)

- **Connect to FalkorDB** via Redis protocol (RESP)
- **Auto-discover schema** — labels, relationship types, property keys
- **WebGL graph rendering** — Sigma.js + graphology with ForceAtlas2 layout
- **Property panel** — click any node or edge to inspect properties
- **Vocabulary search** — instant autocomplete from graph schema
- **Search by words** — type words from a node's name, in any order, or its exact id
- **A bounded first view** — a large graph opens on its best-connected nodes; a node opens on its neighbourhood; a double click widens the view in place. (How legible that view is, is the open work: see `TODO.md`.)
- **Config-driven styling** — `lins.toml` for colors, names, sizes, and what a view leaves out
- **Plugin overlays** — a settings file for one kind of graph, applied only to the graphs it names
- **Multiple graphs** — switch between named FalkorDB graphs
- **One binary** — the server carries the built page, so there is one thing to run

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

# Build the page, then the server that carries it
(cd lins-web && npm install && npm run build)
cargo build --release -p lins-server

# Run it and open the browser
target/release/lins-server serve --config lins.toml --host 127.0.0.1 --open
```

The page is embedded when the server is compiled. After changing the page, build it again and
rebuild the server (`touch lins-server/src/main.rs` first, since the compiler does not see that the
embedded files changed).

**While working on the page**, run the two apart: `cargo run --bin lins-server -- serve --port 3334`
and, in `lins-web`, `npm run dev` (which proxies `/api` to port 3334), then open
`http://localhost:5173`.

**The server has no sign-in and shows every graph in the database.** `--host 127.0.0.1` keeps it
to your own machine; the config's default, `0.0.0.0`, opens it to your network.

**Where FalkorDB is can come from the environment** in place of the config file, so that a host
name or a password need not be written into a file you share: `LINS_FALKORDB_HOST`,
`LINS_FALKORDB_PORT`, `LINS_FALKORDB_PASSWORD`, or failing those `FALKORDB_HOST`, `FALKORDB_PORT`,
`FALKORDB_PASSWORD`.

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
hide = ["group_id", "uuid"]          # vectors are left out without being named
```

### What a view shows first

A graph with at most `max_nodes` visible nodes (1,500 unless set) is loaded whole. A larger one
opens on its `overview` best-connected nodes (150) and the links among them. Choosing a search
result opens that node with up to `neighbours` (60) of its neighbours, and a double click on any
node widens the view by that node's neighbours without moving what is already there.

A view can leave kinds of node and link out. They come back with *show hidden* in the
status bar.

```toml
[styling.labels.Claim]
caption = ["short_title", "text"]    # the property that names a node; several are tried in order
color = "#8b7fd6"

[styling.relationships.ATTACKS]
color = "#e0605a"

[[display.nodes.hide]]               # hide the nodes of a label whose key holds one of the values
label = "Claim"
key = "kind"
values = ["scaffolding"]

[[display.nodes.hide]]               # or a whole label
label = "Session"

[display.relationships]
hide = ["SIMILAR_TO"]

[display.nodes]
overview = 150
neighbours = 60
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

**Working today**: `plugins = ["some-view.toml"]` in `lins.toml` (a path beside the config file, or
an absolute one). An overlay holds `[styling]` and `[display]` tables like the config's own, and
may say which graphs it is for: `graphs = ["notes", "notes_blob_*"]` matches a name exactly, or by
prefix where it ends in `*`. With no `graphs` it applies to every graph. Overlays are laid on in
the order listed and the config's own styling and display rules go on top.

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
