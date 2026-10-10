# Lins TODO

## Next
- [ ] **Rendering library decision**: Sigma.js (current, working) vs egui+wgpu (Rust-native GPU, WASM for browser). See CLAUDE.md § "CRITICAL: Rendering library NOT committed." **A recommendation with the options compared is in `docs/display_proposal_2026_10_10.md` § 7 (2026-10-10, unruled)**: draw the scene with the browser itself, HTML cards and SVG links.
- [ ] **UI/UX design**: User explicitly deferred — "super conscious decisions" needed before committing. **A proposal is written and waits for his rulings, one at a time: `docs/display_proposal_2026_10_10.md` § 10** (first: cards that hold the whole name, in place of dots). Nothing in it is built.

## Exploratory
- [ ] **Context graph visualization features** (decision trace replay, temporal slider, agent-colored nodes): See `~/dotfiles/docs/context_graphs_analysis_2026_03_17.md` — gap analysis + Lins-specific opportunities in Parts 4-5.

## Measured against a real graph of 7,780 nodes (2026-10-10)

Four gaps were seen in one run and closed the same day; two older faults were found on the way.

- [x] **The first view was the first thousand nodes in storage order**, of whose five thousand fetched links 1,512 joined two fetched nodes. Now: a large graph opens on its best-connected nodes and the links among them, a search result opens one neighbourhood, a double click widens it (`/data`, `/around/{id}`, `/find`).
- [x] **Every node was named by its label**, since only `name` was read. Now the `caption` setting is read, as one key or several tried in order.
- [x] **Vectors were hidden by the suffix `_embedding`.** Now by what the value is. Found on the way: FalkorDB returns a vector as text when a whole node is returned, so the old check by type never fired.
- [x] **Nothing could leave a kind of node or link out.** Now `[[display.nodes.hide]]` and `[display.relationships] hide`, with a switch in the status bar that brings them back.
- [x] **A label's properties were read wrongly** (found on the way): the sampler took a node's own three reply fields, `id`, `labels` and `properties`, for its properties, so every label had exactly those three and the vocabulary search worked from them.
- [x] **Plugin overlays load** (`plugins = [...]`), per graph, with the config's own rules on top.
- [x] **The server carries the page** (`rust-embed`), and `--open` opens the browser.

**The owner's verdict on seeing it, the same evening**: barely legible, needs to be spaced out much more, the clustering needs work, and it shows too much at once. Fixed at once, as plain faults: the hover plate was white with pale text and was drawn on every neighbour of a hovered hub; a link's name was drawn on every link of a hovered node; the status bar wrapped in a narrow pane. **Not fixed, because it is the design question he reserved** (`CLAUDE.md`: UI/UX design NOT decided): node size and text, spacing, what a cluster is, and how much a view may hold. Research and a proposal, both written that night: `docs/research/` (three reports, word for word) and `docs/display_proposal_2026_10_10.md`.

Still open from that run:

- [ ] **No test drives the page.** The server's logic has unit tests (24 in `lins-core`); the routes and the page were checked by hand against the live graph.
- [ ] **The overview is chosen by degree alone.** It reads up to 400,000 pairs of ids to rank nodes, which is fine at this size and would not be at a hundred times it.
- [ ] **Link names are shown only on hover.** Whether a small view should show them always is a UI choice, left alone.
- [ ] **Six linter warnings predate this work** (`cargo clippy`: redundant closures in `connection.rs`, a derivable `Default`, a collapsible `if` in `config.rs`).
- [ ] **A desktop shell** is on the roadmap (v1.5) and not started. Until then `--open` gives a browser tab.
