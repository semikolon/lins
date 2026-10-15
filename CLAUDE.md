# CLAUDE.md — Lins

## What Is Lins

**Lins** (Swedish: *lens* — "a lens on knowledge") is a lightweight FalkorDB graph explorer with search-driven exploration, plugin-based styling, and a planned Living Docs feature.

**Market gap**: FalkorDB has NO good visualization tool. Neo4j Bloom/Desktop require Bolt protocol; FalkorDB uses Redis/RESP. FalkorDB Browser has 47 GitHub stars, Docker-only, minimal UX.

**GitHub**: https://github.com/semikolon/lins

## Current State (updated 2026-10-14)

**The v1 prototype (March 2026)**: Backend connects to live FalkorDB, parses nodes/edges, serves REST API. SvelteKit frontend renders graphs with Sigma.js. Force-directed layout, click-to-select, property panel, graph selector, search bar, status bar. Everything below was added on top of it, and it is all in use against real graphs.

**Added 2026-10-10, measured against a real graph of 7,780 nodes** (a session in another project; the settings for that graph live in that project, not here): plugin overlays load and apply per graph (`plugins = [...]`, `graphs = ["name", "prefix_*"]`, the user's config on top) · a large graph opens on its best-connected nodes (`GET /api/graphs/{name}/data`), a node opens on its neighbourhood (`/around/{id}`), words find nodes (`/find?q=`) · `caption` names a node, one key or several · a view can hide kinds of node and link, with a switch to bring them back · vectors are left out by what they are, not by name · the server carries the built page (`rust-embed`) and takes `--host` and `--open` · the connection can come from the environment (`LINS_FALKORDB_*`, then `FALKORDB_*`). Two faults found by running it, both fixed: FalkorDB returns a vector as TEXT when a whole node is returned, so the check by type never fired; and the schema sampler read a node's reply fields as its properties. None of this is a decision about the rendering library or the UI design: Sigma was kept as it was, and everything visual comes from settings. Open after it: `TODO.md`.

**What draws the graph is decided (2026-10-11): the browser.** The owner, after the trial below: *"So, browser-drawn it is. Just like the Fyr way of doing it."* So: a scene is drawn by the page itself (HTML cards, SVG links, in Svelte), and Rust does everything around it: the database, what a scene holds, the server, and a desktop shell around the same page, the way his assistant application is built (Tauri 2 around a SvelteKit page). **The shell is built since 2026-10-12** (`lins-app`; § Running). **A Rust-drawn view is not the plan; do not re-open it as the default.** Why, beyond the look (which he found identical in both): the browser's text is real text and its boxes can be real links, and the code is the same kind as in his other project that draws maps, so approaches and code pass between the two directly (`docs/display_proposal_2026_10_10.md` § 7b). Not ruled: whether Sigma keeps any part (it still draws the main page today, since the scene is not built; in a scene of cards it has none), and whether the Rust half of the trial is kept (it is in the tree, parked; a far view of a whole graph is the one use it would have).

**A trial of the two renderers exists (2026-10-10): `/trial` draws one scene by the browser and by Rust, side by side.** The Rust half is the crate `lins-render` (egui 0.36 on a canvas, WebAssembly through WebGL): `scene.rs` places a scene, `view.rs` draws and moves it, `web.rs` is what a page calls. The browser half is `lins-web/src/routes/trial/` with its rules in `lins-web/src/lib/trial/scene.ts`. **The placing rules are written twice, once in each language, and must stay the same**: the page asks the Rust rules to place each scene with the browser's text heights and says so when they differ. Build the Rust half with `scripts/build-render.sh` (rustup's toolchain, since egui 0.36 needs Rust 1.95 and Homebrew's cannot build it; the crate is outside `default-members`). What the trial measured and what it left open: `docs/display_proposal_2026_10_10.md` § 8a. The owner found the two halves identical in look, and ruled for the browser on other grounds (above). The rule about two copies holds for as long as both halves are in the tree. Two things learned that any Rust-drawn view must carry: egui stretches set text when a layer is zoomed, so text has to be set again at the new size (the trial does it a moment after the zoom rests); and egui's own line wrapping breaks earlier than a browser's, so the trial breaks lines itself.

**egui limitations discovered (Mar 21, 2026 — dotfiles fleet offloading session, Beeper client research). ⚠ Partly moved since, by the research report of 2026-10-10 (`docs/research/rendering_options_2026_10_10.md` § 3): hinted text since egui 0.34, proper shaping since 0.35, mobile input-method fixes in 0.36, coloured emoji in an unreleased 0.37. Check the report before repeating a line below. The renderer question is ruled (above); these lines are kept for the day a Rust-drawn far view is considered.**
- **No color emoji** — monochrome glyphs only (issue #2551, open since Jan 2023, unresolved). Affects node labels with emoji.
- **Image memory** — loading 10 images (~1 MB each) = 1.1 GB (issue #5439). Graph with many node thumbnails/avatars would need aggressive cache management.
- **IME broken** — Tab intercepted, can't select IME candidates. Affects international text input in search/labels.
- **RAM 150-300 MB** for moderate apps (not the ~30 MB expected). Regression tracked in issue #5245.
- **No native macOS menu bar** (issue #3411). Users expect top-of-screen menu.
- **For Lins specifically**: These may be acceptable — graph visualization is less text/emoji-heavy than chat. The image memory issue is the biggest concern for graph nodes with visual content. Evaluate whether Sigma.js + SvelteKit gives better UX/effort ratio for the UI layer while keeping `lins-core` in Rust.

**UI/UX design NOT decided.** User explicitly stated: "I'm not at all decided yet on the UI/UX design, will need some deep thought, I want to make super conscious decisions about it."

**Built 2026-10-12, for the graphs whose view asks for it: cards in lanes, with a switch between orders (`docs/arrangements.md`).** The owner, on the first picture of a small graph (dots placed by springs): *"it's not helping me in creating mental order by way of the visualization"*; and to a sketch of lanes: *"Yes build it."* A view defines **arrangements** (`[[display.arrangements]]`): a lane is a node (its cards hold a link of the kind `member` to it) or a value of a property, cards are ordered by one number, a card is drawn part full from two numbers (`[display.fill]`), and belonging is the lane, never a line. Pointing at a card draws its links to the other cards on show; a press opens it with its ties in columns around it (the trial's scene, now a view of the main page, 8 of each kind of link with a stub). The switch offers each arrangement, *One thing and its ties*, and *Free* (Sigma's dots, unchanged). **A graph whose view has no arrangement has no switch and opens exactly as before**, so nothing here changes the general explorer. Where it lives: `lins-core/src/lanes.rs` (queries, order, fill; unit-tested), `GET /api/graphs/{name}/lanes/{index}`, and `LanesView`, `SceneView`, `OrderSwitch` in the page. For such a graph it settles parts of the display proposal's decisions 3 to 5; the aim of 40 and stop at 100, forces that know the cards, chips, light or dark cards and the face are still his for every graph. Two things to know before touching it: `overflow: clip` on the page's middle row is what stops focus or a script scrolling the closed panel into view, and a clipped box has no automatic minimum size of zero, so that row needs `minmax(0, 1fr)` and `min-height: 0` beside it; and FalkorDB hands a sum back as text, so numbers from a counting query are read leniently (`lanes::as_number`).

**A proposal for the display exists (2026-10-10): `docs/display_proposal_2026_10_10.md`. Two parts of it are decided: a node is a card that holds its whole name (the owner, 2026-10-10: "I'm with you."), and the browser draws it (2026-10-11, above). Still his, in this order (§ 10): how much one view holds, columns around a node, membership links as chips, light or dark cards, the face.** After the owner judged the first real view barely legible, cramped and overwhelming, three research reports were written and kept word for word in `docs/research/` (how readable node-link diagrams are made; how Neo4j Bloom, Browser and their drawing library work; what each rendering option can do). The proposal: a *scene* of a few dozen nodes in place of a sample of the graph (aim at 40, stop at 100, 8 neighbours per widening, ranked, the rest behind counted stubs), a node as a card holding its whole name, columns around a node with forces for free scenes, membership links as chips, the legend as the control panel, a mid-dark canvas. § 10 lists the owner's decisions in order. **Build none of the undecided parts before he rules, and do not treat any number in it as settled**: every size and count is a starting value. Facts from the research that change what is possible: Sigma 4.0.0 came out 2026-10-08 and draws names on the GPU with no wrapping; Neo4j's drawing library may be used only with Neo4j's databases; no open library avoids collisions between name boxes.

## Architecture

### As built, and the direction decided 2026-10-11
```
lins-core/     (Rust lib)  — FalkorDB connection, schema, config; what an arrangement's lanes hold (`lanes.rs`); to come: what a scene around a node holds (ranking, the budget)
lins-server/   (Rust lib+bin) — Axum API; carries the built page. `lib.rs` is the server, `main.rs` its command line
lins-web/      (SvelteKit) — draws cards itself, HTML with SVG links: in lanes, and around one node (built 2026-10-12 for views with an arrangement). Sigma draws the free picture, and the whole main page for every other graph
lins-app/      (Rust bin)  — the desktop app: lins-server and a Tauri 2 window on the page it serves (built 2026-10-12); remembers its last start for the dock (2026-10-14)
lins-render/   (Rust lib)  — the trial's Rust-drawn scene (egui); parked
```

### Set aside: one Rust renderer for everything
The earlier proposal was egui for a native window and for the browser (`lins-render`, `lins-app`, `lins-wasm`), which would have made the Svelte page unnecessary. The trial built one scene of it. The ruling of 2026-10-11 goes the other way: the page draws, and Rust surrounds it.

## FalkorDB Connection

- **Protocol**: Redis/RESP via `redis-rs` (NOT Bolt)
- **Default**: `127.0.0.1:6380`, no password. A password comes from the settings file, the environment (`LINS_FALKORDB_PASSWORD`, then `FALKORDB_PASSWORD`), or for the desktop app the keychain. Never write a real one into a file this public repository tracks.
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

Three layers: auto-discovered defaults → plugin overlays → user config overrides. The `lins.toml` config file is a first-class, version-controllable, shareable artifact. **This repository tracks only `lins.toml.example`**: a person's own `lins.toml` is ignored by git here (since 2026-10-11), and a password belongs in the environment (`LINS_FALKORDB_PASSWORD`), never in a file that is shared.

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
| **Fyr** (`~/Projects/fyr/`) | Personal assistant, Tauri 2 + SvelteKit. Primary early user. The pattern Lins's desktop app follows: a Rust shell around a page drawn by the system's web view. What can be shared is page code (cards, scenes) and the shell's lessons (a stable local signature, settings for a dock launch); `lins-render` is parked, not shared. Fyr's dev bundle is still signed ad hoc; the `Lins Local` certificate would keep its identity stable across builds too. Custom entity types (Task, Deadline, Routine). |
| **Graphiti Fork** (`~/Projects/graphiti-official/`) | Knowledge graph that produces the data Lins visualizes. Fork with reliability fixes. |
| **dotfiles** (`~/dotfiles/`) | Where Graphiti MCP config, FalkorDB setup, and knowledge management lives. |

## Specs

Full specifications at `specs/graph-explorer/`:
- `requirements.md` — 11 user stories, acceptance criteria, version tiers
- `design.md` — architecture, data models, API, config schema
- `tasks.md` — 40 tasks across v1-v3

## Running

```bash
# One binary that carries the page (build the page first; it is embedded at compile time)
(cd lins-web && npm install && npm run build) && cargo build --release -p lins-server
target/release/lins-server serve --config lins.toml --host 127.0.0.1 --open

# While working on the page: the two apart
cargo run --bin lins-server -- serve --port 3334 --config lins.toml
cd lins-web && npm run dev -- --port 5173      # proxies /api to 3334; open http://localhost:5173
```

**After changing the page, `touch lins-server/src/lib.rs` before rebuilding the server**: the compiler does not see that the embedded files changed, and the old page is served with no error. (`lib.rs` holds the embed since 2026-10-12; before that it was `main.rs`, and touching the old file does nothing.)

**The desktop app (2026-10-12): `scripts/build-app.sh --install`** builds the page, the server and `lins-app`, and puts `Lins.app` in `~/Applications` (`LINS_APP_DIR` names another folder). Needs the Tauri command (`cargo install tauri-cli --version "^2"`). How it is made, and why:
- **One process, no second API.** `lins-app` calls `lins_server::bind(config, "127.0.0.1", 0)`, runs the server on the port it was given, and opens a Tauri window on `http://127.0.0.1:<port>/`. The page and its `/api` calls are the ones the browser uses, unchanged. The spec's task T-26 asked for Tauri commands calling `lins-core`; that would have been a second way to ask every question, to be kept in step with the first by hand. So there are no commands, no IPC and no capabilities file.
- **Settings**: `--config`, then `$LINS_CONFIG`, then `~/.config/lins/lins.toml`, then **what the app was last started with**, then defaults; the `LINS_FALKORDB_*` and `FALKORDB_*` variables on top. `--graph <name>` opens on a graph (the page reads `?graph=`).
- **Opened from the dock it is given nothing, so it remembers** (`lins-app/src/remembered.rs`, 2026-10-14, on the owner's word). A start that reaches the database is kept: the settings file's full path, the host, the port and the graph in `last-start.json` in the app's own folder (`~/Library/Application Support/se.fredrikbranstrom.lins/` on macOS), and the password in the system keychain (service `se.fredrikbranstrom.lins`, account `host:port`), never in a file. A start that came from the kept one writes nothing back. Any start whose settings hold no password looks in the keychain for that `host:port`. To forget: delete `last-start.json`, and the keychain item in Keychain Access. Before this, the dock launch looked for the database on this machine (the default) and failed after ten seconds, which read as a backend that had not started. Set aside: the password in a settings file (plain text on disk); a user settings file that lists a project's views again (a second copy that drifts from the one the project keeps); starting a database on this machine (it would be an empty one, not the one the views describe).
- **When the database cannot be reached** (ten seconds of patience) the window shows `lins-app/shell/index.html` with the reason; the app does not exit.
- **`lins-app/gen/` is generated** by the build and ignored. The icon is drawn in `lins-app/icons/icon.svg`; the other sizes are made from it with `cargo tauri icon`.
- **Install outside `target/`** if anything on your machine sweeps build folders: an app left in `target/release/bundle/` goes with them.
- **Signed with a certificate made on this machine when there is one** (2026-10-14): `build-app.sh` signs with the code-signing certificate named `Lins Local` (or `$LINS_SIGNING_IDENTITY`) if the keychain holds it, else ad hoc as `tauri.conf.json` says. With the certificate the app's identity is the same across builds (identifier plus the certificate's hash), so the keychain keeps trusting it. Ad hoc, every build is a new app to macOS and asks again. The certificate is made in Keychain Access (Certificate Assistant, *Create a Certificate*, type *Code Signing*), counts on this machine only, and `codesign` asks once for the login keychain's password unless *Always Allow* was chosen. Not checked yet: whether the local network question stops asking after the next rebuild.
- It is a workspace member outside `default-members`, like `lins-render`: an ordinary `cargo build` does not compile Tauri.

**The server has no sign-in and shows every graph in the database.** It listens on this machine only (`127.0.0.1`) unless a config file or `--host 0.0.0.0` says otherwise (the default was `0.0.0.0` until 2026-10-11).

**A read query the server builds itself goes through `query_built`, never `query`**: `query` refuses any text holding a write keyword, so it refused a search for the word *create*. A reader's words enter a built query only through `view::cypher_str`.

Note: port 3000 is taken by brf-auto Rails app. Use 3334 or configure in lins.toml.
