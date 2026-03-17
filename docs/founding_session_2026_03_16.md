# Lins Founding Session — March 16-17, 2026

Full journey from concept to working prototype, including all decisions made, deferred, and paths explored.

## 1. Origin

User had a prior agent summarize the situation: FalkorDB has no good visualization tool. Neo4j Bloom requires Bolt protocol (FalkorDB uses Redis/RESP). The competitive landscape is wide open.

Prior research documents:
- `~/dotfiles/docs/research/knowledge-graph/graph-database-visualization-tools-research.md` (Sep 2025, 300+ lines — NOTE: incorrectly claims FalkorDB supports Bolt)
- `~/dotfiles/docs/knowledge_graph_as_documentation_source_of_truth.md` (Jan 2026, 695 lines — Living Docs architecture, converged with Lins vision Mar 16)
- `~/dotfiles/docs/graphiti_upstream_review_2026_03_15.md` (FalkorDB internals, fork reliability fixes)

## 2. Naming

**Chosen: Lins** (Swedish: *lens* — "a lens on knowledge")

> "Lins is a good name btw :)" — User

Name availability checked:
- **crates.io**: Available
- **GitHub**: No dominant project (small Nim linter, nothing major)
- **npm**: Squatted 10 years ago at 0.0.1, dead (irrelevant for Rust project)

## 3. /spec Process — 10 Socratic Questions

### Q1: Explorer vs Living Docs priority?
**Answer**: Both equally important. Neither is primary.
> "Both should be built. :)" — User

### Q2: Generic FalkorDB vs Graphiti-specific?
**Answer**: Generic with Graphiti as first plugin.
> "I feel like it should be quite generic and I don't mind the extra upfront work on the schema introspection layer. But it should be aware of the possibility that there are custom entities or custom profiles, for docs views/lenses on the graph etc." — User
> "It should be user configurable but intelligently auto derived for styling and templates. There could be some default templates that one could fall back on, but largely I think that should be user defined and maybe there could be plugins or something like that, that I could bundle with the personal assistant app that I'm building or whatever uses this and integrates with it." — User

Embeddings can still be leveraged generically — detect vector-typed properties on nodes, configure an embedding endpoint.

### Q3: Where does the intelligence live?
**Answer**: Option C — config-as-contract with plugin overlays.
> "Option C sounds great. It should be... there should be the possibility for plugins to contribute in a way that's kind of like an overlay. A plugin can specify styling rules, template generation, semantic search, specific output document rendering, file names even for the documents, whatever we want." — User

Three layers: auto-discovered defaults → plugin overlays → user config overrides. Config file is version-controllable and shareable.

### Q4: Form factor (Tauri vs web vs both)?
User initially deferred. Shelved the kiosk idea.
> "Eh. Not sure I want it displayed for everyone in the household to see. It wouldn't be explorable on the kiosk anyway since it doesn't have keyb/mouse plugged in atm. Leave the graph-on-kiosk idea for now..." — User

Later chose both (Q9).

### Q5: Who reads Living Docs, and where?
**Answer**: Both file writing AND web serving.
> "It should be able to do both! :)" — User
> "Could it sense when a markdown file path is being read, and sort of serve the content live from the graph? Would be cool." — User

Virtual files (File Provider / FUSE) noted as v3 stretch goal. Pragmatic middle ground: files on disk as cache, daemon keeps fresh, web API for live queries.

### Q6: Multi-graph handling?
**Answer**: Cross-graph workspaces with merged results.
> "I would like to enable cross-graph search and queries and visualizations. Obviously, that's not a FalkorDB feature, so we would have to query the different graphs and merge results." — User
> "It could very well be quite useful to allow the user to define different workspaces. And to allow the user to define styling overlays, living docs templates. Plugable and shareable. Either as a combined plugin or as separate plugins. Whatever the user wants." — User
> "The Graphiti plugin that auto-discovers all Graphiti managed graphs and the custom entities that are defined in the Graphiti MCP server that is running. That will be quite useful as well for me and for other developers." — User

### Q7: What does the first thing you'd use look like?
**Answer**: A (visual exploration) and B (search) first.
> "A and B first." — User

Build order: renderer → search → daemon.

### Q8: LLM-enhanced search architecture?
**Answer**: Layered — vocabulary instant + LLM async.
> "Yeah, that's great. But it could also autocomplete or suggest autocompletions as I'm typing a natural language sentence, right?" — User

Two-tier typeahead: instant vocabulary matches (0ms, from schema cache) + async LLM semantic completions (300ms debounce). LLM search degrades gracefully when not configured.

### Q9: Where do you open Lins? (form factor revisited)
**Answer**: Both web and desktop (Option C).
> "I kind of like having the dock icon and the separation of apps in my main OS. Just having an easier time switching between them. I can quickly open it whenever I want." — User
> "If we need a separate daemon process for the living docs feature, then anyone using this will have to run an installer anyway, right? So why not then also install a native desktop app." — User
> "I guess we can do option C. How hard would it be?" — User

Answer: ~10-15% more code. Shared core, shared frontend, Tauri wraps same UI.

### Q10: Version tiers?
**Answer**: User adjusted tiers to add v3.
> "I think we can add a version 3 tier and put the file provider virtual files thing in there and maybe the living docs daemon all together and the integration with Fyr." — User

Final tiers:
- v1: Explorer Core (connection, schema, rendering, vocabulary search, config)
- v1.5: Search & UX (LLM search, workspaces, Tauri, plugins, scene building)
- v2: Intelligence (Graphiti plugin, cross-graph, embeddings)
- v3: Living Docs & Integration (daemon, templates, File Provider, Fyr)

## 4. Frontend Framework Decision

**Chosen: SvelteKit** (matching Fyr's Tauri 2 + SvelteKit stack)
> "Let's def use Svelte since I'm really developing this project as a sort of side-project to all of my dotfiles/Ruby and Fyr work etc." — User

This decision may be SUPERSEDED by the egui/Rust-native path (see §8).

## 5. Neo4j Bloom Research

Bloom's tech stack: TypeScript frontend, NVL (Neo4j Visualization Library) — Canvas for small graphs, WebGL for large. Layout in Web Workers.

**Key insight**: Bloom's premium feel comes from INFORMATION ARCHITECTURE, not rendering tech:

1. **Perspectives** — curated views with styling, saved queries, role-based visibility. Maps directly to Lins' config overlay system.
2. **Search-driven exploration** — vocabulary-aware autocomplete, additive scene building. Not LLM — just pattern matching against schema.
3. **Scene Actions** — right-click selected nodes → run contextual Cypher → results animate in.

### Bloom Visual Design (from screenshots)
- **Light/white background** — warm, welcoming, NOT dark
- **Large circular nodes** (~50-70px) with category ICONS inside
- **Labels above nodes** (name text positioned above the circle)
- **Edge labels along edge lines** — rotated, colored per relationship type
- **Warm, distinct colors** — yellow, green, coral, purple. NOT neon, NOT saturated
- **Selection = cyan ring** — thick glowing border
- **Category legend** in right sidebar — color dot + icon + name + count
- **Tokenized search** — query as chips/pills, not raw text
- **Generous spacing** — nodes never overlap, plenty of breathing room

### Rendering Library Landscape

| Library | Renderer | Scale | Best For |
|---------|----------|-------|----------|
| **Sigma.js** | WebGL-native | 100K+ | Focused rendering, custom UX layers |
| **G6/AntV** | Canvas/SVG/WebGL | Medium-large | Batteries-included, Chinese ecosystem |
| **Cosmos.gl** | GPU compute | 1M+ | Massive graphs (CC-BY-NC license — rejected) |
| **Cytoscape.js** | Canvas + WebGL preview | 5-10K | Graph algorithms |
| **NVL** (Bloom) | Canvas + WebGL | 100K+ | Neo4j only (proprietary) |

## 6. Implementation (Mar 16)

Built working v1 prototype in ~2 hours:

### Backend (lins-core + lins-server)
- FalkorDB connection via redis-rs
- Schema introspection (labels, relationships, properties, sampling)
- Verbose mode parsing (switched from compact — compact returns integer IDs, not strings)
- Read-only Cypher validation
- TOML config with auto-generation
- Vocabulary index for instant search
- Axum REST API (graphs, schema, data, query, search)
- Smart query curation (count-first, pagination for large graphs)

### Frontend (lins-web)
- SvelteKit + TypeScript
- Sigma.js v3 + graphology + ForceAtlas2 (web worker)
- GraphCanvas, SearchBar, PropertyPanel, GraphSelector, StatusBar components
- Dark theme with forest green palette
- Vite proxy to backend API

### Verified against live FalkorDB
- 16 graphs discovered with real counts (brf-auto: 234 nodes/574 edges, fyr: 61/165)
- Schema introspection returns actual labels (Entity, Episodic, Community, Decision, Project, etc.)
- Queries execute in <1ms
- Frontend renders graph with force-directed layout

### Git History (7 commits pushed)
1. `20b9bd9` — chore: initialize Rust workspace
2. `206b9dd` — docs: add three-file specification
3. `a1315aa` — feat: implement lins-core library
4. `9655faf` — feat: implement lins-server with Axum REST API
5. `22c2a34` — feat: implement SvelteKit frontend with Sigma.js
6. `461cb4c` — docs: add README, config example
7. `068a577` — fix: update Vite proxy to match server port

## 7. Sigma.js vs G6 Deep Evaluation

### G6 v5 Advantages
- 20 built-in layout algorithms (dagre, radial, fishbone, mindmap)
- `AutoAdaptLabel` behavior (labels show/hide by zoom + collision)
- Built-in plugins (Tooltip, Contextmenu, Minimap, Legend)
- Functional style mappers: `fill: (d) => ...`
- Declarative state system (selected, inactive, hover)
- Icons inside nodes — composite shapes built-in

### G6 v5 Problems (Critical)
- **Performance regression at 500 nodes** (issue #7402, open): continuous CPU computation at idle, memory leaks
- **addData has open bugs** (issue #6658): incremental rendering inconsistent — directly affects additive scene building
- **7.7x bundle size** (7.5 MB vs 970 KB)
- **No Svelte wrapper** (dead v4 wrapper, 6 stars, 2023)
- **Documentation**: machine-translated, SPA-rendered, broken examples reported
- **860ms → 2192ms regression** between minor versions for 5000 nodes

### Decision: Sigma.js as WORKING prototype, but NOT committed
The rendering library decision is open. User wants to evaluate egui/Rust-native path.

## 8. Rendering Library Pivot — Rust GPU Path

### The Revelation: Bloom's Quality Is Design, Not Technology

After studying Bloom screenshots, the core insight: Bloom looks premium because of DESIGN CHOICES, not rendering engine:
- Light background, large nodes with icons, warm colors, generous spacing
- Text inside/above nodes, edge labels along edges
- Category-specific icons and colors

Current Lins dark theme looks like "a terminal from the Matrix" per user.

> "I wanna at least replicate Bloom's features. But also build on it, make it even more intuitive and we should have great UX and great defaults for styling and auto-adjustments of node spacing, font sizes, padding, distances etc to maximize readability and make it really smooth on the eyes and easy to understand relationships." — User
> "The default color scheme should have subtle contrast and feel super intuitive instead of like a terminal from the Matrix or something a sysadmin would endure looking at — the user just wants to make sense of their life instead of spending half an hour customizing styling, aint nobody got time for that." — User
> "We should also make maximal smooth use of AI where applicable." — User

### Bloom Feature → Library Capability Mapping

| Bloom Feature | Sigma.js | G6 | Rust GPU (wgpu/egui) |
|---|---|---|---|
| Large circles + icon inside | Hard (custom GLSL) | Easy (built-in) | Full control |
| Labels above/inside nodes | Built-in | Built-in | Full control |
| Edge labels along edges | Built-in | Built-in | Custom (hard) |
| Color per category | nodeReducer | Functional mappers | Full control |
| Selection ring | Custom | Built-in states | Full control |
| Auto-adapt labels | Threshold-based | Built-in behavior | Custom LOD |
| Smooth animations | Basic | Rich system | GPU interpolation |

### The egui + WASM Insight

**egui compiles to WASM** — same Rust code runs as native desktop app (via eframe/wgpu) AND in browser (via WASM). One codebase, same visual quality everywhere.

> "Full control but lots of work sounds fun :D" — User
> "Development effort is a non-issue, remember?" — User

Proposed architecture:
```
lins-core/     — FalkorDB, config, queries (EXISTS, KEEP)
lins-server/   — Axum API + serves WASM build (EXISTS, KEEP)
lins-render/   — Graph renderer + all UI (egui, NEW)
lins-app/      — Native desktop via eframe/wgpu (NEW)
lins-wasm/     — Same code → WASM for browser (NEW)
```

User is researching egui independently before committing.

### SDF Text Rendering
Signed Distance Fields — technique for resolution-independent text rendering on GPU. Text stays perfectly crisp at any zoom level. Used by Valve, Google Maps, game engines. egui doesn't use SDF by default (re-rasterizes) but crates exist. Cross this bridge if zoomed text looks soft.

### Windows Support
Tauri 2 / eframe support Windows natively. `.msi` builds. Web mode works in any browser. No issues.
> User mentioned wanting it to run on "my dad's Windows laptop."

## 9. Open Decisions (as of Mar 17, 2026)

### OPEN: Rendering Technology
- **Sigma.js**: Working prototype, fast, good for web. Limited visual control for Bloom-quality.
- **egui + wgpu**: Full Rust, native + WASM, full pixel control. User researching.
- **Canvas 2D**: Could be a pragmatic middle ground for Bloom-quality visuals at 500 nodes.
- **Decision**: User is evaluating. Not committed.

### OPEN: UI/UX Design
> "I'm not at all decided yet on the UI/UX design, will need some deep thought, I want to make super conscious decisions about it and haven't gotten to that yet." — User

Key principles established:
- Bloom-inspired but better
- **Mid-to-dark theme** — NOT pure dark, NOT light. Mid-to-dark grey or dark blue background, white or bright text, accent color. User explicitly said they prefer "neither" dark nor light — "somewhere in the middle of the spectrum."
- **Font**: Söhne-style (the font Claude's UI uses, by Klim Type Foundry). Or similar clean, modern sans-serif.
- Great defaults (zero-config beauty)
- Smooth auto-adjustments (spacing, font sizes, padding)
- Subtle contrast, intuitive colors
- NOT "sysadmin terminal" aesthetic
- Maximal smooth use of AI

### OPEN: SvelteKit vs egui for UI
If Rust-native path chosen, SvelteKit (`lins-web/`) becomes unnecessary. All UI moves to egui.

### DECIDED: Core Architecture
- Generic FalkorDB explorer with plugin system
- Config-as-contract with overlays
- Layered search (vocabulary + LLM)
- Cross-graph workspaces
- Version tiers (v1→v1.5→v2→v3)
- Build order: visual exploration → search → living docs daemon

### DECIDED: Name
Lins (Swedish: lens). Available on crates.io.

### DECIDED: License
MIT. User confirmed Mar 17.

### DECIDED: Config Format
TOML (`lins.toml`). Standard for Rust ecosystem. User confirmed Mar 17.

### DECIDED: Template Engine (v3, Living Docs)
Tera (Rust-native Jinja2) is the current plan. User open to exploring alternatives later.

### DECIDED: FalkorDB Connection
Redis/RESP via redis-rs. Verbose mode. Read-only validation. Schema introspection.

## 10. Reference Documents & URLs

### Project Documents
| Document | Location |
|----------|----------|
| Spec: requirements | `specs/graph-explorer/requirements.md` |
| Spec: design | `specs/graph-explorer/design.md` |
| Spec: tasks | `specs/graph-explorer/tasks.md` |
| Graph viz research (Sep 2025) | `~/dotfiles/docs/research/knowledge-graph/graph-database-visualization-tools-research.md` |
| Living Docs research (Jan 2026) | `~/dotfiles/docs/knowledge_graph_as_documentation_source_of_truth.md` |
| Graphiti upstream review | `~/dotfiles/docs/graphiti_upstream_review_2026_03_15.md` |
| Local embeddings plan | `~/dotfiles/docs/local_embeddings_plan_2026_03_16.md` |
| FalkorDB config | `~/.graphiti/redis.conf` |
| Fyr project | `~/Projects/fyr/` |
| ANAMNESIS graph viz section | `~/dotfiles/ANAMNESIS.md` (lines 526-554) |

### Bloom UX Reference (Neo4j)
- **Bloom User Guide**: https://neo4j.com/docs/bloom-user-guide/current
- **Bloom Prerequisites (WebGL req)**: https://neo4j.com/docs/bloom-user-guide/current/bloom-installation/bloom-prerequisites
- **Bloom Scene Interactions**: https://neo4j.com/docs/bloom-user-guide/current/bloom-visual-tour/bloom-scene-interactions/
- **Bloom Scene Actions**: https://neo4j.com/docs/bloom-user-guide/current/bloom-tutorial/scene-actions/
- **Bloom Graph Pattern Search**: https://neo4j.com/docs/bloom-user-guide/current/bloom-tutorial/graph-pattern-search
- **Bloom Search Phrases**: https://neo4j.com/docs/bloom-user-guide/current/bloom-tutorial/search-phrases-advanced
- **Bloom Deployment Modes**: https://neo4j.com/docs/bloom-user-guide/current/bloom-installation/bloom-deployment-modes
- **ScoobiGraph blog post (screenshots)**: https://neo4j.com/blog/developer/scoobygraph-3/
- **Bloom screenshots studied in session**:
  - `https://cdn-images-1.medium.com/max/1024/1*oDxfy6bUdF9jxUDs6402rA.png` (Scooby Doo 46 nodes, category icons, light background)
  - `https://cdn-images-1.medium.com/max/1024/1*UxyZi-9PXCmSq8ibnIMQkg.png` (Scene Actions config — Saved Cypher + $nodes parameter)
  - `https://cdn-images-1.medium.com/max/1024/1*I-jMt_B9kCPhGR49GNGgbA.png` (4 nodes zoomed, tokenized search, colored edge labels)
  - `https://cdn-images-1.medium.com/max/1024/1*OKjn0De4_nubAkRUyZ0HFg.png` (Scooby radial, Perspective button, category legend sidebar)

### NVL (Bloom's Rendering Engine)
- **NVL Documentation**: https://neo4j.com/docs/nvl/current/
- **NVL on npm**: https://www.npmjs.com/package/@neo4j-nvl/base
- **NVL YouTube deep dive**: https://www.youtube.com/watch?v=uVxhYgWsHZw

### Rendering Libraries Evaluated
- **Sigma.js** (front-runner, working prototype): https://www.sigmajs.org / https://www.npmjs.com/package/sigma
- **graphology** (graph data model): https://graphology.github.io
- **G6/AntV** (batteries-included, rejected for performance issues): https://g6.antv.antgroup.com/en/manual/introduction
  - Performance issue #7402: https://github.com/antvis/G6/issues/7402
  - addData bug #6658: https://github.com/antvis/G6/issues/6658
  - Rendering regression #6137: https://github.com/antvis/G6/issues/6137
- **Cosmos.gl** (rejected — CC-BY-NC license): https://openjsf.org/blog/introducing-cosmos-gl
- **Cytoscape.js** (WebGL preview, not evaluated deeply): https://blog.js.cytoscape.org/2025/01/13/webgl-preview/
- **Orb/Memgraph** (Canvas, d3-force in WebWorker): https://github.com/memgraph/orb
  - Architecture blog: https://memgraph.com/blog/how-to-build-a-graph-visualization-engine-and-why-you-shouldnt
- **GraphGPU** (WebGPU, experimental): https://github.com/dragonman225/GraphGPU
- **GraphWaGu** (WebGPU academic): https://github.com/harp-lab/GraphWaGu

### Rust GPU / Native Rendering (under evaluation)
- **wgpu** (Rust WebGPU/Vulkan/Metal): https://wgpu.rs
- **egui** (immediate-mode GUI, compiles to native + WASM): https://github.com/emilk/egui
- **egui_graphs** (graph visualization on egui): https://github.com/blitzarx1/egui_graphs
- **eframe** (egui native framework): https://docs.rs/eframe
- **lyon** (2D tessellation for wgpu): https://github.com/nical/lyon
- **cosmic-text** (Rust text rendering): https://github.com/nickelc/cosmic-text
- **SDF text rendering** (Valve's seminal paper): https://steamcdn-a.akamaihd.net/apps/valve/2007/SIGGRAPH2007_AlphaTestedMagnification.pdf

### Comparison Resources
- **Graph library comparison 2026**: https://www.pkgpulse.com/blog/cytoscape-vs-vis-network-vs-sigma-graph-visualization-javascript-2026
- **Neo4j graph visualization tools list**: https://neo4j.com/blog/developer/neo4j-graph-visualization-tools/
- **Graphistry GPU architecture**: https://graphistry-admin-docs.readthedocs.io/en/latest/planning/architecture.html
- **Graphistry keynote (2B edges)**: https://www.graphistry.com/blog/gtp2025-graphistry-keynote-2b-edge-graphs-in-2-seconds-unlock-gpu-power-with-gfql

### FalkorDB Ecosystem
- **FalkorDB**: https://www.falkordb.com
- **FalkorDB Browser** (47 stars, Docker-only): https://github.com/FalkorDB/falkordb-browser
- **G.V()** (commercial, recently added FalkorDB support): https://gdotv.com

## 11. User Preferences (Collected)

- **Generic over specific**: prefers building general-purpose tools that serve personal needs via plugins
- **Programmer happiness matters**: Ruby ideally, Python if needed, Rust fine with AI writing code
- **Performance is non-negotiable**: "stellar performance and stability"
- **Development effort is not a factor**: AI writes all code
- **Zero-config beauty**: defaults should be beautiful, intuitive, and readable without customization
- **Plugin/overlay architecture**: composable, shareable, version-controllable configs
- **AI integration where applicable**: LLM-enhanced search, smart auto-adjustments
- **Native desktop feel**: dock icon, window separation, snappy interaction
- **Cross-platform**: macOS primary, Windows support desired (dad's laptop), MBP access
- **Open-source**: MIT license, but adoption is secondary to personal use quality
- **Fyr integration**: shared components/crates, both Rust-based
