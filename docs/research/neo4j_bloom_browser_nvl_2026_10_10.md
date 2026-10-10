<!--
Kept word for word. Research report: How Neo4j's tools draw and explore graphs (Bloom/Explore, Browser, NVL).
Date: 2026-10-10.
What this is: the report of a research helper (a language model with web access and no access
to this repository), kept exactly as it was returned. Nobody has checked it against its sources
a second time. Each claim carries the helper's own mark for how it was read; trust the marks, and
check a number against its source before quoting it elsewhere.
Why it was asked: the first view built for a real graph of 7,780 nodes was judged barely legible
by the project's owner. The proposal that draws on this report is docs/display_proposal_2026_10_10.md.
The question the helper was given follows the report, also word for word.
-->

# How Neo4j's tools keep complex graphs readable (Bloom/Explore, Browser, NVL)

Research date: 2026-10-10. Current versions: Bloom 2.37.0 (23 Sep 2026, runs on NVL 2.0.0), @neo4j-nvl/base 2.0.0 (11 Sep 2026), Neo4j Browser repo 2025.8.0 (still pushed 2026-10-09).

## How to read this report

Every claim is followed by a bracketed key. The full URL for each key is in the Source index at the end. Confidence tags on the keys:

- **[R]** = I fetched the page or file and read the text myself. This covers docs pages, GitHub raw source, npm-package files, and Discourse and GitHub JSON.
- **[S]** = I saw only a search-result snippet, not the page.
- **[B]** = I derived the fact from the minified NVL npm bundle. It is not documented, so confidence is medium.
- **[C]** = I computed the number from constants I read. The arithmetic is mine.

Things I could not verify, so you know where the holes are:

- Medium.com blocks all my fetches (403 / Cloudflare), so Medium posts are snippet-only or substituted with neo4j.com copies.
- Bloom's real pixel node size, force parameters and background colour are not documented anywhere I could reach.
- Bloom's behaviour when nodes are added is not documented.
- The NVL docs index page (docs/nvl/current) is only a stub. The real documentation is the TypeDoc API plus the type declarations in the npm package.

---

## 1. What does the user see FIRST, and what caps exist?

### Bloom / Explore

**Empty scene, search-first.** A new Perspective with no data shows an empty scene with two starter options: "Show a graph snippet" (a sample of the data) or "Search the graph" (jumps to the search bar). [bloom/scene-interactions] [R], [aura/scene-interactions] [R]

- The Bloom overview says the scene contains only the parts of the graph you found through search or exploration. [bloom/overview] [R]
- A Neo4j consulting engineer's walkthrough confirms "When you open Bloom, you will see a blank scene." [blog/scooby] [R]
- Typing "Show me a graph" in the search bar gives a random subset. It is a Search phrase stored under Saved Cypher in the Perspective. [blog/autumn-2022] [R]
- Release notes show "Show me a graph" being repaired in 2.28 and 2.31 (the 2.31 fix is for graphs with no relationships). [rn/2.28] [R], [rn/2.31] [R]

**Explicit design statement from a Neo4j product person (Nov 2025).** Explore was "not designed to load an entire graph by default". The aim is to let an analyst find a part of the graph and start there. The same reply says the node limit in Settings is 1000 by default. [canny/cant-see-nodes] [R]

- The user in that thread saw 38 of 236 nodes and was confused. There is no obvious "you are seeing X of Y" cue. The staff answer was to search per type, select all, then Reveal relationships.

**Caps:**

- The Settings drawer has a Node query limit adjustable from 100 to 10000. [bloom/settings] [R] The default is 1000 (staff statement above).
- Search results are capped at 10,000 records. This is a stated safety limit against hanging. [bloom/search-phrases] [R]
- An engineering post says Bloom displays between 100 and 10,000 unique nodes, with a 10,000-row response cap. In their example, 26,000 rows were needed to reach 10,000 unique nodes. [blog/reactive] [R]
- A Neo4j staffer says the 10,000 limit is per query. To show more you must re-query or expand, for example "select all, expand, select all, expand". [comm/45670] [R]
- Expansion returns at most the Node query limit, unless you set a per-expansion limit in the context menu, which overrides it. [bloom/scene-interactions] [R]
- Shortest path is limited to 20 hops and shows the first shortest path found. [bloom/scene-interactions] [R]
- The Settings drawer also has a Search timeout, case-insensitive search, "Search results highlight" (auto-selects the newly added nodes) and "Property shown on hover". [bloom/settings] [R]
- A search-result snippet says Bloom warns when the node limit is exceeded. I could not confirm the wording or placement of that warning. [snippet: comm/59859] [S]

**A schema-level reduction happens before any data is drawn.**

- The first time Bloom meets a database it auto-generates a Perspective. It finds the smallest set of labels that categorises all nodes, picks caption properties and sets default styles. [bloom/perspective-creation] [R]
- A full scan is slow above about 10M nodes plus relationships, so a quick scan is offered. [bloom/perspectives] [R]
- The Perspective can hide relationship types and properties, so noise is removed at the schema level. [bloom/perspective-creation] [R]
- Search suggestions need indexes. Properties on labels with fewer than 1000 nodes count as indexed. A property whose value repeats on more than 10% of nodes is not searchable. [bloom/pattern-search] [R]

### Neo4j Browser (graph result frame)

**Browser draws the query result at once, capped.**

- The setting `initialNodeDisplay` defaults to 300. `maxNeighbours` defaults to 100. `maxRows` is 1000. [src/settingsDuck] [R], [browser/settings-legacy] [R]
- Inside the UI the labels are "Initial Node Display" and "Max neighbours from vis interaction". [src/UserSettings] [R]
- Docs discrepancy: the current manual lists `maxVizNodes`=1000 and `maxNewNeighbours`=100. [browser/settings-manual] [R] The GitHub master source still has 300/100. [src/settingsDuck] [R]
  - My inference: the manual now describes the newer NVL-based Browser inside the Aura console. Neo4j says that version has "an improved visualization component". [blog/15-tools] [R]
- When the cap is hit, the UI shows one info line: not all returned nodes are displayed due to the Initial Node Display setting, and only the first N are shown. [src/GraphVisualizer] [R]
  - Nodes are de-duplicated and truncated in result order. After truncation, relationships are filtered to the kept nodes. [src/VisualizationView] [R]
- "Connect result nodes" (`autoComplete`, default true) runs an extra query to add relationships between returned nodes. That adds links, not nodes. [browser/settings-legacy] [R], [comm/browser-pick-nodes] [R]

---

## 2. How a view is widened and narrowed

### Bloom

**Select**

- Click selects. Cmd/Ctrl-click adds to the selection.
- Left-drag on empty background pans. "Marquee tools" select several elements. [bloom/scene-interactions] [R]
- Clicking a category in the Legend selects all nodes of that category. [bloom/legend] [R]
- The docs say "marquee tools" and do not say lasso versus box. NVL ships both a Lasso and a BoxSelect interaction handler. [nvl/api-nav] [R]

**Inspect**

- Double-click opens the Inspector, not an expand. [bloom/scene-interactions] [R]
- The Card list shows properties of nodes in the scene, filterable by selection and by text. Cards support Expand and Dismiss. [bloom/card-list] [R]
- Hover shows the label and chosen properties. Nodes are often too small to fit their caption, and tooltips exist for that case. [blog/bloom-1.9] [R]

**Widen**

- Right-click a node (or a selection) and choose Expand. This brings in all immediate neighbours, up to the node limit. [bloom/scene-interactions] [R]
- The context menu can expand along one relationship type and direction. [bloom/scene-interactions] [R]
- "Advanced Expansion" expands along several paths and/or to specific neighbour node types, with its own result limit. [bloom/scene-interactions] [R]
- Expand is also reachable from the Inspector's neighbour cards. [bloom/scene-interactions] [R]
- Keyboard shortcut is Ctrl/Cmd+E. [bloom/shortcuts] [R]
- Reveal relationships adds links between nodes already in the scene without adding nodes. If only one node is selected it works only for self-loops. [bloom/scene-interactions] [R]
- Shortest path between two selected nodes runs from the context menu. [bloom/scene-interactions] [R]
- Search always adds to the scene rather than replacing it. [canny/cant-see-nodes] [R]

**Narrow**

- Dismiss removes the selection (Ctrl/Cmd+H). Dismiss others removes everything not selected (Ctrl/Cmd+Shift+H). [bloom/shortcuts] [R]
- "Dismiss single nodes" removes nodes with no relationship to anything visible. It is available from background right-click, with no selection needed (Ctrl/Cmd+Shift+S). [bloom/scene-interactions] [R], [bloom/shortcuts] [R]
- "Select related nodes" (Ctrl/Cmd+Shift+R) selects a node's neighbours. The docs pair it with "dismiss others" to isolate a neighbourhood. [bloom/scene-interactions] [R]
- Clear Scene (Ctrl/Cmd+Backspace) empties the scene. Undo and Redo are search-bar actions. Other search-bar actions are Invert selection, Fit to selection and Jump to node. [bloom/search-bar] [R], [bloom/shortcuts] [R]
- Group nodes: two or more selected nodes become one visually distinct larger node. Group relationships: parallel relationships between the same pair become one arrow with a count. [bloom/scene-interactions] [R]

**Filter**

- Filters grey out non-matching elements. They stay visible but are not interactive. "Dismiss filtered elements" removes them. [bloom/scene-interactions] [R]
- A numeric filter shows a histogram with sliders. Supported types are integer, float, boolean, string and the five temporal types. Time zones can be normalised. [bloom/scene-interactions] [R]
- Filters are not saved. They vanish when the scene is cleared. [bloom/scene-interactions] [R]

**Slicer**

- It is a bottom panel with range filters over numeric and temporal properties (dateTime, date, time, localTime, localDateTime, duration, integer, float). [bloom/slicer] [R]
- Hovering a bar on the timeline shows how many visible nodes have that value. [bloom/slicer] [R]
- Up to five ranges can be combined. [bloom/slicer] [R]
- Playback has three modes: Slide range to end, Start of range to end, and Within range. [bloom/slicer] [R]
- While the Slicer is open you can only select and deselect. "Keep Scene and Close" keeps the sliced result. The X restores the scene as it was. [bloom/slicer] [R]

**Navigation aids**

- A minimap sits bottom-right with a zoom percentage; clicking the number resets zoom to 100%. [bloom/scene-interactions] [R]
- Fit-to-screen and Presentation mode are also there, and the minimap is toggleable. [bloom/scene-interactions] [R]

### Browser

- Double-click a node expands it. Double-click again collapses it. [browser/result-frames] [R]
  - Collapse is recursive. The code removes nodes added by that expansion, and nodes those added in turn. [src/Graph.ts] [R]
- Right-click a node for the context menu: expand/collapse, dismiss, unpin, "explore with Bloom", show all relationships. Undo is available for menu actions. Right-clicking a relationship offers dismiss. [browser/result-frames] [R]
- Expansion runs `MATCH path=(a)--(o) ... ORDER BY id(o) LIMIT (maxNeighbours − already-visible neighbours)`. [src/VisualizationView] [R]
  - The subset is therefore deterministic but arbitrary: lowest ids first.
  - When capped, the UI says rendering was limited to N of the node's total M neighbours. [src/GraphVisualizer] [R]
  - If the budget is already used up, it only fetches the count. [src/VisualizationView] [R]
- Hovering a node temporarily fixes it. Dragging a node pins it (`fx/fy`). Unpin is in the menu. [src/mouseEventHandlers] [R], [src/GraphEventHandlerModel] [R]
- A GitHub issue (Aug 2025) asks for a configurable relationship limit on expansion, because expanding highly connected nodes can freeze the UI. [gh/browser#2013] [R]

---

## 3. How nodes and relationships are drawn

### Browser (SVG, d3)

**Node size and text**

- Default node diameter is 50px, with a 2px border. Font is 10px, sans-serif. [src/GraphStyle] [R]
- Size choices in the UI are 10, 20, 50, 65 and 80px. [src/GraphStyle] [R]
- The caption goes INSIDE the circle and is wrapped to the circle's chord widths. [src/GraphGeometryModel] [R]
- Maximum caption length is floor(pi * r^2 / fontSize^2). At defaults that is about 19 characters [C]. Longer text is cut hard, wrapped over up to about 5 lines, and the last word gets an ellipsis. [src/GraphGeometryModel] [R]
- Default caption is chosen by property-name priority: name, title, label, anything ending in "name", description, then the first property, else `<id>`. [src/GraphStyle] [R]
- There is no level-of-detail logic. The font is fixed and the whole SVG is scaled, so labels shrink with the zoom.
- Initial zoom is a fitted formula in node count. [src/Visualization] [R] I computed it [C]:

| Nodes | Initial scale | Node on screen |
|---|---|---|
| 100 | ~0.28 | ~14px |
| 150 | ~0.20 | ~10px |
| 300 | ~0.11 | ~6px |

  - This matches Lins's complaint exactly (about 150 nodes as tiny dots).
- Zoom range is 0.1 to 2, with the minimum lowered to fit big graphs. Fit-to-screen padding is 5%. [src/constants] [R]
- A community post complains that names are cut off with "..." and that nodes can only be circles. [comm/17054] [R]

**Relationships**

- Shaft width defaults to 1px. Width choices are 1, 2, 3, 5, 8, 13, 25 and 38px. [src/GraphStyle] [R]
- Caption defaults to the relationship type, at 8px. [src/GraphStyle] [R]
- The caption sits at the shaft midpoint, rotated to follow the line and flipped to stay upright. It is shortened with an ellipsis to fit the arrow length. [src/renderers/init, PairwiseArcsRelationshipRouting] [R]
- Parallel relationships between one pair are routed as separate arcs. [src/PairwiseArcsRelationshipRouting] [R]

### NVL / Bloom

**Defaults** (from the npm package's type declarations, `utils/constants.d.ts`) [R] [nvl/types-constants]

- Node size is 25 (a radius, so a 50px diameter).
- Relationship width is 1, with a maximum "flat" width of 20.
- Arrowhead is 7 by 9. The gap between node and arrow is 4. A selected relationship is 1.5x wider.
- A curved or tapered relationship has a 30 unit curve length. A self-loop is 25 wide by 35 high.
- Shadows are disabled above 500 arrows.
- Icons are skipped when smaller than 10 on-screen px.
- Captions are Open Sans, bold.

**What the API exposes**

- A node can have `caption` or styled `captions[]` (with bold/italic styles), `captionAlign` (center/top/bottom), `captionSize`, `icon` (a square black icon, inverted automatically on dark fills), `overlayIcon`, `size`, `color`, and `pinned`. [nvl/Node] [R]
- A relationship has `caption(s)`, `captionAlign`, `captionSize`, `width`, `color`, and an experimental HTML caption. [nvl/Relationship] [R]

**Renderers**

- `canvas` is the default and supports captions but is slower. [nvl/NvlOptions] [R], [nvl/types-state] [R]
- `webgl` (WebGL2 since 2.0) is faster and, from 2.0, draws icons and arrowheads, but does NOT draw captions. [nvl/CHANGELOG] [R], [nvl/types-state] [R]
- `svg` is experimental. Per-element docs say captions are visible only in canvas; NvlOptions says canvas or svg. [nvl/NvlOptions] [R]
- Bloom 2.28 added "always use WebGL renderer for small graphs". I could not find what that means in practice. [rn/2.28] [R]

**Level of detail** (read from the minified bundle, not documented [B]):

- On-screen node area is measured as a percent of a 1600x1200 reference viewport: area% = pi * (r * zoom)^2 / 19,200.
- Below 0.04% the node is at info-level 1 and the caption is NOT drawn (text opacity animates to 0). That is r*zoom below about 15.6px [C]. At the default radius of 25 that is zoom below about 0.63 [C].
- Caption font steps with area: below 0.8% gives level 1.1, below 3% gives 1.6, below 8% gives 2.5 (and above 8% stays 2.5).
- Max caption lines are 2, 3 and 4 respectively. The font size is radius / (3.5, 2.75 or 2 by captionSize, plus 1 if there is an icon) / that level.
- So as you zoom in, text gets relatively smaller and more words fit. This matches the Bloom 1.9 comment that nodes are often too small for their caption.
- Zoom limits: default zoom 0.75, min 0.075, max 10. `allowDynamicMinZoom` lets fit zoom out further. [nvl/NvlOptions] [R], [nvl/types-constants] [R]

**Bloom user controls on this**

- Text size is "dynamic in relation to the size of the node". [bloom/scene-interactions] [R]
- Per category you can set caption property(ies), font size, bold/italic/underline, placement and an icon. For relationships you can set the caption property, size and thickness. [bloom/legend] [R], [bloom/perspective-creation] [R]
- A 2020 user said even at the "4x" size they could not show 5-15 words, and asked for text resizing. [comm/22669] [R]
- A 2021 user said zoomed-out relationship edges were so long that text was unreadable. A staffer's workaround was to style by colour or thickness instead of reading labels. [comm/40732] [R]

---

## 4. Layouts, default, spacing, behaviour on adding nodes

### Bloom

- Default is force-based. Five layouts are offered: Force-based 1, Force-based 2, Hierarchical, Coordinate and Circular. [bloom/scene-interactions] [R]
  - Force 1 is GPU-accelerated, fast, and gives "a quite dense and compact graph".
  - Force 2 is not GPU-accelerated, slower and "more spread-out". The difference matters most on larger graphs. [bloom/scene-interactions] [R]
  - Bloom 2.28 added another new force option. Circular arrived in 2.29. [rn/2.28] [R], [rn/2.29] [R]
- Hierarchical orders nodes by relationship direction, with an orientation control. [bloom/scene-interactions] [R]
- Coordinate pins nodes at numeric or point properties. [bloom/scene-interactions] [R]
  - x/y properties are auto-found in the order point, latitude, lat, x (and longitude, long, y).
  - Nodes without values are parked on one side. Mercator projection is used for geographic points. Axes are scalable.
- Circular layout is meant for spotting a node more connected than its peers in a category. [bloom/scene-interactions] [R]
- No gravity, repulsion or link-distance parameters are documented for Bloom, and none are exposed in the UI. The only spacing lever is choosing layout 1 versus 2.
- Evidence the force layout is not always satisfying: a 2023 user with about 4,000 nodes and 3,500 relationships saw it jiggle for seconds and then stop in a non-optimised state. [comm/61920] [R]
- 2.24 fixed "force layout drifting indefinitely" and sped up stabilisation. 2.23 fixed a layout re-animating when not intended. [rn/2.24] [R], [rn/2.23] [R]
- Whether adding nodes moves the existing picture is not documented for Bloom. Users can click-drag nodes into place. [bloom/overview] [R]

### Browser (d3-force)

All values below are from the source. [src/constants, ForceSimulation, circularLayout, Visualization] [R]

- Charge is -400.
- Link distance is r1 + r2 + 90 px (LINK_DISTANCE 45, doubled). With default 50px nodes that is 140px between centres.
- Collision radius is node radius + 25.
- Centring force on x and y is 0.03.
- Velocity decay is 0.4. alphaMin is 0.05. Zoom is 0.1 to 2.
- Only one relationship per node pair is used in the simulation.
- Up to 300 ticks (or 250 ms) are pre-computed before the first paint, then the layout animates.
- Initial seeding: nodes without a position are placed on a circle of radius N*45/(2*pi) around the origin.
- Every update restarts the simulation at alpha 1. Existing nodes keep their coordinates but get re-shuffled. New nodes (including expansions) start on the origin circle and are pulled to their neighbours, which can look like an explosion.
- Dragging a node raises alpha to 0.8, targets 0.09, and pins the node.

### NVL

**Layouts** (type `Layout`): `forceDirected`, `hierarchical`, `grid`, `free`, `d3Force`, `circular` (experimental). [nvl/Layout] [R], [nvl/layout-vars] [R]

- `free` positions nodes only via `setNodePositions`.
- `hierarchical` uses dagre with the "tight-tree" ranker, direction up/down/left/right, packing `bin` or `stack`, and a sub-graph spacing of 100. [nvl/HierarchicalOptions] [R], [nvl/types-hier-constants] [R]
- `d3Force` is the plain d3-force layout. Its constants in the package are LINK_DISTANCE 45, Barnes-Hut theta 1.2, centre 0.03 and velocity decay 0.4. [nvl/types-d3-constants] [R]

**`forceDirected` (the main one)**

- It is a GPU multilevel physics engine (WebGL2 since 2.0). The type declarations show "solar merger" graph coarsening, GLSL shaders and convergence checks. [nvl/types-PhysLayout] [R], [nvl/CHANGELOG] [R]
- Seeding is `'box'` (default) or `'circle'`. [nvl/ForceDirectedOptions] [R]
- `ForceDirectedOptions` has ONLY `clustering` and `seedingMethod`. Spacing, gravity and repulsion are NOT user-configurable. The internal constants are MinRelLength 100, Gravity 25 and stop velocity 0.25. [nvl/ForceDirectedOptions] [R], [nvl/types-constants] [R]
- FPS budget: layout steps are capped at `maxFps` 30 and `minFps` 3. [nvl/NvlOptions] [R]
- The API has `restart(options, retainPositions)`, `pinned` nodes, `fit(nodeIds, zoomOptions)` and a minimap container. [nvl/NVL-class] [R]

---

## 5. Clusters and communities

- **No automatic cluster view in Bloom docs.** Bloom's own handling is manual:
  - Group nodes into one larger node, and group relationships into a counted arrow. [bloom/scene-interactions] [R]
  - Colour or size by a property.
- **GDS integration.** Bloom runs algorithms on the nodes in the scene, or on the whole graph as a projection. [bloom/gds] [R]
  - Centrality: Betweenness, Degree, Eigenvector, PageRank. Community: Louvain, Label propagation, Weakly connected components.
  - Scene-only runs add temporary scores. Whole-graph runs write a property back to the database.
  - Results become visible through rule-based styling in the GDS drawer. Centrality maps to size scale or colour gradient. Community uses "unique values", one colour per community.
- **Rule-based styling.** [bloom/legend] [R]
  - Three modes: single (condition, with a histogram), range (colour or size spectrum, for example green to red or small to big), and unique values (a distinct colour per value).
  - The first matching rule wins per attribute.
  - Works on string, numeric, boolean and temporal properties. Rules override the default style.
- **NVL 2.0.0 adds real clustering** (11 Sep 2026). [nvl/CHANGELOG] [R]
  - `clusterBy(nodeId) -> id | undefined` groups nodes. Nodes that return `undefined` stay independent. [nvl/ClusteringOptions] [R], [nvl/ClusterByFn] [R]
  - It is a physical cluster in the force layout, with optional `drawOutline` (a convex hull in the cluster's colour) and `drawHalo` (a soft glow). Both default to false.
  - If member colours differ, the outline falls back to `overlayLineColor` and the halo is not shown. [nvl/ClusteringOptions] [R]
  - `crossClusterRelationshipMultiplier` (experimental) scales spring pull on edges that cross a cluster boundary, from 0 to 1. The default of 0 means cross-cluster edges exert no pull, so communities separate. Near 1 in dense graphs it can destabilise the layout. [nvl/ClusteringOptions] [R]
  - There is a cluster hit-testing and interaction API (`HitTargetCluster`). A cluster has at least two nodes. [nvl/Cluster] [R]
  - `HierarchicalOptions.clusterBy` (experimental) uses dagre compound sub-graphs. [nvl/HierarchicalOptions] [R]
  - I found no evidence that Bloom 2.37 surfaces these in its UI. The Bloom 2.37 note only says NVL was upgraded with performance improvements. [rn/2.37] [R]
- **No aggregation of many nodes into a counted super-node** exists beyond manual Group. The Neo4j super-nodes post treats hairballs as a data-model problem to fix, not a visual one. [comm/supernodes] [R]

---

## 6. Colours and theme

**Browser** [src/GraphStyle, styled.tsx, themes.tsx] [R]

- Default node fill is grey #A5ABB6, border #9AA1AC. Panel background is #F9FCFF (very light).
- Label colours come from a fixed 12-colour list, in order: #604A0E, #C990C0, #F79767, #57C7E3, #F16667, #D9C8AE, #8DCC93, #ECB5C9, #4C8EDA, #FFC454, #DA7194, #569480. Each has a darker border colour.
- Caption text colour is chosen per fill, white or #2A2C34. The code also has an optional word-hash colour generator (`calculateDefaultNodeColors`); how often it is used, I did not verify.
- Selection: a ring of r+4 with 8px stroke, yellow #fdcc59 at 30% opacity when selected, blue #6ac6ff when hovered. Relationships use the same overlay colours.
- Styling can be edited in place or via `.grass` stylesheets. [browser/styling] [R]

**NVL defaults** [nvl/types-constants] [R]

- Node #FFDF81, relationship #818790, text #1A1B1D / #FFFFFF.
- Selection: outer ring #8FE3E8, inner ring #F5F6F6, shadow #CFD1D4.
- Disabled items #EDEDED, with font #DDDDDD.
- `styling` options let you override these. [nvl/NvlOptions] [R]
- The overlay line colour default #1A1B1D is documented as "use a dark colour on light backgrounds", so NVL defaults assume a light canvas. [nvl/NvlOptions] [R]

**Neo4j design system tokens** [needle/tokens] [R]

- Same 12-colour "graph" palette (#FFDF81, #C990C0, #F79767, #56C7E4, #F16767, #D8C7AE, #8DCC93, #ECB4C9, #4D8DDA, #FFC354, #DA7294, #579380). A separate 12-colour "categorical" chart palette exists.
- Canvas default is #1A1B1D in dark and #F5F6F6 in light. This is the app canvas token, not proven to be Bloom's scene background.

**Bloom**

- Categories get a default colour you can change in the Legend. A category can also have an icon from a library. [bloom/perspective-creation] [R]
- The Legend panel lists categories and relationship types with counts of how many are in the scene. It can be filtered to show only types present. [bloom/legend] [R]
- A 2020 GitHub request asked for a user-selectable background colour. The Settings drawer today lists no theme or background option. [gh/bloom#16] [R], [bloom/settings] [R]
- Dark mode shipped in Aura Workspace in 2025. [canny/dark-mode] [R]

---

## 7. Criticisms and known weaknesses

- **Neo4j itself says Browser has "limited visualizations and styling choices"**. [blog/15-tools] [R]
- **Browser readability**: names cut off with "...", circles only. [comm/17054] [R] Workaround is a hand-edited `.grass` file.
- **Browser starts big graphs tiny**: see the zoom formula in section 3. [src/Visualization] [R]
- **Bloom text**: cannot resize caption text, and long captions are unreadable even at maximum node size. [comm/22669] [R] Edges too long to read when zoomed out. [comm/40732] [R] A request for human-friendly label formatting was closed. [gh/bloom#14] [R]
- **Bloom hairball workflow**: a user asked to filter a large query and then show relationships only among the survivors. The answer was Reveal relationships on a selection. [gh/bloom#78] [R] The user said it did not work on Bloom 2.11.
- **Bloom hides what was cut**: 38 of 236 nodes with no clear explanation. [canny/cant-see-nodes] [R] A 10,000 cap with no override. [comm/45670] [R]
- **Layout stability**: stops moving after seconds on 4,000 nodes [comm/61920] [R]; drifting indefinitely and re-animating bugs [rn/2.24] [R], [rn/2.23] [R]; flicker and out-of-focus nodes after expanding, on a weak GPU [comm/20948] [R].
- **GPU dependency**: Bloom needs WebGL and a GPU and "will not function otherwise". A discrete GPU is preferred. A compatibility mode exists but some functions may break. [bloom/prereq] [R]
- **Reasons data gets messy**: the super-node post says super nodes put the hair into the hairball, and with millions of relationships the picture is just a black mess. [comm/supernodes] [R]
- **Licensing friction for Bloom**: Desktop Bloom is not receiving new releases, and remote databases need a licence. [gh/bloom#78] [R]
- **Third-party opinion** (a vendor, so biased): hairballs are best avoided by not exposing the raw model to users. It lists filtering, grouping, aggregation and pruning as the standard remedies. [ci/hairball] [R]
- I did not find usable Reddit or G2 text. Searches returned only generic aggregator pages, which I did not use. Treat the "community" evidence above as forum posts and GitHub issues.

---

## What Lins could copy as-is

1. **Empty-first, search-first start with two ways in**: "random snippet" and "search". Never auto-draw the whole graph. Say "Showing X of Y" whenever a cap bites. [bloom/scene-interactions] [canny/cant-see-nodes]
2. **A user-adjustable node cap**: default 1000, range 100 to 10,000, plus a per-expansion override. [bloom/settings] [bloom/scene-interactions]
3. **Search ADDS to the scene**, new results get selected/highlighted, plus a one-click Clear scene. [bloom/settings] [bloom/search-bar]
4. **Schema-level curation before data**: auto-generate "categories" as the smallest label set, choose caption properties, allow hiding relationship types and properties. [bloom/perspective-creation]
5. **The widen/narrow vocabulary**, with Bloom's shortcuts:
   - Expand all / by relationship type and direction / by neighbour label.
   - Dismiss, Dismiss others, Dismiss single nodes.
   - Select related, Reveal relationships, Path (up to 20 hops).
   - Undo, Redo, Clear, Fit to selection, Jump to node.
   - Map and Presentation mode. [bloom/scene-interactions] [bloom/shortcuts]
6. **Browser's expansion budget**: cap = maxNeighbours minus neighbours already shown, deterministic order, and a visible message "limited to N of M". Double-click toggles expand and collapse (recursive undo). [src/VisualizationView] [src/GraphEventHandlerModel]
7. **Legend as control panel**: counts per type, click to select all of a type, filter to "in scene", per-type colour, size, caption and icon. [bloom/legend]
8. **Hover tooltip and Inspector/Card list for detail**, so the canvas can stay sparse. [blog/bloom-1.9] [bloom/card-list]
9. **Filter greys out, Dismiss removes**, plus histogram sliders. A Slicer-style range filter with hover counts. [bloom/scene-interactions] [bloom/slicer]
10. **Rule-based styling with single, range and unique modes**, first rule wins per attribute. Use "unique" for a community id and range for size or colour by degree. [bloom/legend] [bloom/gds]
11. **Group nodes and group relationships** into a larger node and a counted arrow. [bloom/scene-interactions]
12. **NVL 2.0 cluster idea (reimplement it, do not copy code)**: physical clusters by a key, convex-hull outline and halo in the cluster colour, cross-cluster spring multiplier default 0. [nvl/ClusteringOptions]
13. **Level-of-detail rules**:
    - Hide captions when the on-screen node radius is under about 15.6px.
    - Skip icons under 10px.
    - Turn off arrow shadows above 500 arrows.
    - Wrap captions into the circle, with more lines as you zoom in.
    [nvl/bundle derived] [nvl/types-constants]
14. **Numbers worth starting from**:
    - Node 50px diameter, relationship 1px, arrowhead 7x9, node gap 4.
    - Force: charge -400, link distance r1+r2+90, collide r+25, centre 0.03, velocity decay 0.4.
    - Pre-compute up to 300 ticks (or 250 ms) before first paint.
    - Zoom 0.075 to 10, default 0.75, fit padding 5%.
    [src/constants] [nvl/types-constants]
15. **Two force presets, compact and spread-out**, as Bloom does with layouts 1 and 2. Offer the same choice, but explain it. [bloom/scene-interactions]
16. **Selection and colour styling**:
    - 12-colour categorical palette.
    - Pick text colour per fill (white or #2A2C34).
    - Selection ring with a light inner ring.
    - Hover ring in a different colour.
    - A visibly "disabled" grey for filtered-out items.
    [src/GraphStyle] [nvl/types-constants] [needle/tokens]

## Where the sources suggest Neo4j's choice is weak

1. **Browser's fit-to-everything start**: 150 nodes land at about 0.2x zoom, so nodes are about 10px and labels are unreadable. There is no level of detail. This is Lins's complaint reproduced. [src/Visualization] [C]
2. **Spacing is not user-tunable.**
   - NVL's `ForceDirectedOptions` exposes only seeding and clustering.
   - Bloom's only lever is "dense" layout 1 versus "spread-out" layout 2, which is slower.
   - Users ask for longer or shorter edges and cannot get either. [nvl/ForceDirectedOptions] [bloom/scene-interactions] [comm/40732]
3. **Text lives inside a circle.**
   - It is cut to roughly 19 characters in Browser. In Bloom, long captions fail even at the largest size, and there is no way to resize text.
   - Tooltips are the escape hatch, which proves labels don't fit. [src/GraphGeometryModel] [comm/22669] [blog/bloom-1.9]
4. **Hard caps with weak feedback.**
   - Truncation is "first N" or "lowest ids first", not ranked by importance.
   - Users report not understanding why they see 38 of 236.
   - There is no "show more" paging for neighbours. A requested relationship cap is open. [src/VisualizationView] [canny/cant-see-nodes] [gh/browser#2013]
5. **Every update re-shuffles the picture.**
   - Browser restarts the simulation at alpha 1 and seeds new nodes on a circle around the origin.
   - Bloom has history of re-animation, drift and early-stop bugs.
   - Nothing documented protects the user's mental map except manual pinning. [src/ForceSimulation] [rn/2.23] [rn/2.24] [comm/61920]
6. **Clustering is manual, or arrives late and experimental.**
   - Bloom documents only manual Group, plus colouring after running GDS.
   - NVL 2.0 clustering is new, parts are marked experimental, and I found no sign Bloom shows it. [bloom/scene-interactions] [bloom/gds] [nvl/CHANGELOG] [rn/2.37]
7. **The start requires knowing what to ask.**
   - "Graph snippet" is a random sample, and the results are repaired repeatedly in release notes.
   - There is no schema or overview map of a big graph before you search.
   - Search quality depends on indexes. Properties repeating on over 10% of nodes cannot be searched. [bloom/pattern-search] [rn/2.28] [rn/2.31]
8. **Hard platform requirements**: GPU plus WebGL, with a compatibility mode that breaks features. A web page for FalkorDB cannot assume this. [bloom/prereq]
9. **Filters are ephemeral.** They are lost on Clear. The Slicer blocks all other scene interaction while open. [bloom/scene-interactions] [bloom/slicer]
10. **Light-canvas assumptions.** NVL defaults (overlay #1A1B1D, pastel fills) target light backgrounds. Bloom has no documented background setting. A near-black canvas needs deliberately re-tuned contrast. [nvl/NvlOptions] [gh/bloom#16]
11. **Licensing, which matters for how Lins copies things.**
    - NVL may be used only with Aura or Neo4j's commercial database products under their agreements. By my reading that rules it out for FalkorDB (not legal advice). The licence text is the same in the earliest package I could read (0.3.1) and in 2.0.0. [nvl/license] [nvl/LICENSE.txt]
    - The npm package also bundles a telemetry client. It can be switched off with `disableTelemetry`. [npm/nvl-base] [nvl/NvlOptions]
    - Neo4j Browser source (GPL-3.0) and the Needle token package (GPL-3.0) can be read for ideas and numbers, but copying code would pull Lins under the GPL. [src/package.json] [needle/npm]

---

## Source index

**Bloom user guide (all [R])**

- bloom/scene-interactions = https://neo4j.com/docs/bloom-user-guide/current/bloom-visual-tour/bloom-scene-interactions/
- bloom/overview = https://neo4j.com/docs/bloom-user-guide/current/bloom-visual-tour/bloom-overview/
- bloom/legend = https://neo4j.com/docs/bloom-user-guide/current/bloom-visual-tour/legend-panel/
- bloom/search-bar = https://neo4j.com/docs/bloom-user-guide/current/bloom-visual-tour/search-bar/
- bloom/settings = https://neo4j.com/docs/bloom-user-guide/current/bloom-visual-tour/settings-drawer/
- bloom/card-list = https://neo4j.com/docs/bloom-user-guide/current/bloom-visual-tour/card-list/
- bloom/shortcuts = https://neo4j.com/docs/bloom-user-guide/current/bloom-appendix/bloom-appendix/
- bloom/pattern-search = https://neo4j.com/docs/bloom-user-guide/current/bloom-tutorial/graph-pattern-search/
- bloom/slicer = https://neo4j.com/docs/bloom-user-guide/current/bloom-tutorial/slicer/
- bloom/gds = https://neo4j.com/docs/bloom-user-guide/current/bloom-tutorial/gds-integration/
- bloom/search-phrases = https://neo4j.com/docs/bloom-user-guide/current/bloom-tutorial/search-phrases-advanced/
- bloom/perspective-creation = https://neo4j.com/docs/bloom-user-guide/current/bloom-perspectives/perspective-creation/
- bloom/perspectives = https://neo4j.com/docs/bloom-user-guide/current/bloom-perspectives/bloom-perspectives/
- bloom/prereq = https://neo4j.com/docs/bloom-user-guide/current/bloom-installation/bloom-prerequisites/
- aura/scene-interactions = https://neo4j.com/docs/aura/explore/explore-visual-tour/scene-interactions/

**Bloom feedback and release notes**

- canny/cant-see-nodes = https://neo4j-aura.canny.io/explore/p/i-cant-see-all-my-nodes [R]
- canny/dark-mode = https://neo4j-aura.canny.io/workspace/p/dark-mode [R]
- rn/2.23 = https://neo4j.com/release-notes/bloom/bloom-2-23-0/ [R]
- rn/2.24 = https://neo4j.com/release-notes/bloom/bloom-2-24-0/ [R]
- rn/2.28 = https://neo4j.com/release-notes/bloom/bloom-2-28-0/ [R]
- rn/2.29 = https://neo4j.com/release-notes/bloom/bloom-2290/ [R]
- rn/2.31 = https://neo4j.com/release-notes/bloom/bloom-2310/ [R]
- rn/2.37 = https://neo4j.com/release-notes/bloom/bloom-2370/ [R]

**Neo4j blogs**

- blog/bloom-1.9 = https://neo4j.com/blog/graph-visualization/simplify-your-visualization-workflow-with-neo4j-bloom-1-9/ [R]
- blog/scooby = https://neo4j.com/blog/developer/scoobygraph-3/ [R]
- blog/autumn-2022 = https://neo4j.com/blog/developer/autumn-is-bloom-ing-with-new-visualization-features/ [R]
- blog/reactive = https://neo4j.com/blog/developer/fetching-large-amount-data-neo4j-reactive-driver-the-bloom-case/ [R]
- blog/15-tools = https://neo4j.com/blog/graph-visualization/neo4j-graph-visualization-tools/ [R]

**Community and GitHub**

- comm/45670 = https://community.neo4j.com/t/bypass-10000-node-limit-in-bloom-neo4j-desktop/45670 [R]
- comm/59859 = https://community.neo4j.com/t/neo4j-bloom-node-limit-on-my-own/59859 [R for thread; the warning-wording claim is [S]]
- comm/61920 = https://community.neo4j.com/t/how-to-make-force-based-layout-continue-running-on-large-graph/61920 [R]
- comm/22669 = https://community.neo4j.com/t/allow-resizing-of-node-caption-text-in-neo4j-bloom/22669 [R]
- comm/17054 = https://community.neo4j.com/t/readability-of-a-graph/17054 [R]
- comm/40732 = https://community.neo4j.com/t/can-you-resize-the-length-of-the-relationships-edges/40732 [R]
- comm/20948 = https://community.neo4j.com/t/bloom-nodes-and-relationship-flickering-and-out-of-focus/20948 [R]
- comm/browser-pick-nodes = https://community.neo4j.com/t/how-does-neo4j-browser-pick-the-nodes-to-visualize/10744 [R]
- comm/supernodes = https://community.neo4j.com/t/graph-modeling-all-about-super-nodes/27464 [R]
- gh/bloom#14 = https://github.com/neo4j-devtools/neo4j-bloom/issues/14 [R]
- gh/bloom#16 = https://github.com/neo4j-devtools/neo4j-bloom/issues/16 [R]
- gh/bloom#78 = https://github.com/neo4j-devtools/neo4j-bloom/issues/78 [R]
- gh/browser#2013 = https://github.com/neo4j/neo4j-browser/issues/2013 [R]

**Browser (all [R])**

- browser/settings-manual = https://neo4j.com/docs/browser-manual/current/operations/browser-settings/
- browser/settings-legacy = https://neo4j.com/docs/browser/legacy/operations/browser-settings/
- browser/result-frames = https://neo4j.com/docs/browser-manual/current/operations/result-frames/
- browser/styling = https://neo4j.com/docs/browser-manual/current/operations/browser-styling/
- src/* = files under https://raw.githubusercontent.com/neo4j/neo4j-browser/master/src/ :
  - shared/modules/settings/settingsDuck.ts
  - browser/modules/Sidebar/UserSettings.tsx
  - browser/modules/Stream/CypherFrame/VisualizationView/VisualizationView.tsx
  - neo4j-arc/graph-visualization/constants.ts
  - neo4j-arc/graph-visualization/models/GraphStyle.ts and models/Graph.ts
  - neo4j-arc/graph-visualization/GraphVisualizer/GraphVisualizer.tsx
  - neo4j-arc/graph-visualization/GraphVisualizer/Graph/GraphEventHandlerModel.ts and Graph/styled.tsx
  - neo4j-arc/graph-visualization/GraphVisualizer/Graph/visualization/ (ForceSimulation.ts, Visualization.ts, GraphGeometryModel.ts, mouseEventHandlers.ts, renderers/init.ts, utils/circularLayout.ts, utils/PairwiseArcsRelationshipRouting.ts)
  - neo4j-arc/common/styles/themes.tsx
- src/package.json = https://raw.githubusercontent.com/neo4j/neo4j-browser/master/package.json [R]

**NVL**

- nvl/license = https://neo4j.com/docs/reference/license/nvl [R]
- nvl/LICENSE.txt = https://unpkg.com/@neo4j-nvl/base@2.0.0/LICENSE.txt [R]
- nvl/CHANGELOG = https://cdn.jsdelivr.net/npm/@neo4j-nvl/base@2.0.0/CHANGELOG.md [R]
- nvl/types-constants, types-state, types-PhysLayout, types-d3-constants, types-hier-constants = declaration files under https://cdn.jsdelivr.net/npm/@neo4j-nvl/base@2.0.0/dist/types/ [R]:
  - utils/constants.d.ts
  - modules/state/types.d.ts and modules/state/constants.d.ts
  - layouts/forcedirectedlayout/PhysLayout.d.ts
  - layouts/d3forcelayout/constants.d.ts
  - layouts/hierarchicallayout/constants.d.ts
  - renderers/domrenderer/shared/arrows/constants.d.ts
- nvl/bundle = https://cdn.jsdelivr.net/npm/@neo4j-nvl/base@2.0.0/dist/base.mjs [R, minified, hence [B]]
- nvl/api-nav and the typedoc pages under https://neo4j.com/docs/api/nvl/current/ [R]:
  - interfaces/_neo4j_nvl_base.NvlOptions.html (nvl/NvlOptions)
  - ...ForceDirectedOptions.html
  - ...ClusteringOptions.html
  - ...HierarchicalOptions.html
  - ...Node.html
  - ...Relationship.html
  - types/_neo4j_nvl_base.Layout.html (nvl/Layout), ...Cluster.html, ...ClusterByFn.html
  - variables/_neo4j_nvl_base.{ForceDirectedLayoutType, HierarchicalLayoutType, GridLayoutType, FreeLayoutType, d3ForceLayoutType, CircularLayoutType, CanvasRendererType, WebGLRendererType}.html (nvl/layout-vars)
  - classes/_neo4j_nvl_base.NVL.html (nvl/NVL-class)
- npm/nvl-base = https://registry.npmjs.org/@neo4j-nvl/base [R]
- needle/tokens = https://cdn.jsdelivr.net/npm/@neo4j-ndl/base@4.23.7/lib/tokens/css/tokens.css [R]
- needle/npm = https://registry.npmjs.org/@neo4j-ndl/base [R]

**Third party**

- ci/hairball = https://cambridge-intelligence.com/blog/hairball-effect-in-graph-visualization/ [R, vendor opinion]

---

## Appendix: the question as it was put to the helper

> Research task (web only; do not read or edit any local files). I am improving "Lins", a small open-source explorer for FalkorDB graph databases (a Rust server plus a web page that draws a node-link diagram). Its owner says it was inspired by Neo4j Bloom and wants it to be "Bloom-inspired but better". Today its first view of a 7,780-node graph shows ~150 nodes and ~470 links as tiny dots with small labels on a near-black background, and the owner's verdict is: barely legible, needs to be spaced out much more, the clustering needs work, and it shows too much at once and is overwhelming.
>
> Your job: find out, from primary sources, HOW NEO4J'S OWN TOOLS VISUALIZE COMPLEX GRAPHS AND KEEP THEM READABLE, in enough concrete detail that their choices could be copied or deliberately improved on. Cover all three products:
>
> A. Neo4j Bloom (now also called "Explore" in Aura / Workspace). Start from the Bloom user guide: https://neo4j.com/docs/bloom-user-guide/current (visual tour, scene interactions, scene actions, search bar / graph pattern search / search phrases, perspectives, rule-based styling, legend panel, filter and slicer, card list, layouts). 
> B. Neo4j Browser's graph view (result frame): its limits on how many nodes are shown initially and how many neighbours one expansion brings in (settings like initialNodeDisplay, maxNeighbours), captions, how double-click expand and dismiss work.
> C. NVL, the Neo4j Visualization Library (https://neo4j.com/docs/nvl/current/): layouts offered (force-directed, hierarchical, grid, free/coordinate, d3-force), renderers (canvas vs WebGL and what each can draw, e.g. captions/labels only in canvas), options that control spacing, zoom-level behaviour, node sizes, captions inside nodes, and its licence (can it be used without a Neo4j database?).
>
> Questions to answer concretely, with numbers where they exist:
> 1. What does a user see FIRST when opening a big graph? (Bloom starts with an empty scene and a search bar; confirm and describe.) How much is shown at once by default, and what caps exist (node/relationship limits, warnings)?
> 2. How is a view WIDENED and NARROWED: expand (all / by relationship type / by neighbour label), dismiss, dismiss other nodes, reveal relationships, shortest path, undo/redo, clearing the scene, filtering and the slicer (range filters on properties/time). Exact interactions (double-click, right-click menu, lasso).
> 3. How are NODES drawn: size in pixels at default zoom, text inside vs beside the node, how long captions are truncated or wrapped, icons, how caption text scales with zoom, which captions are hidden when zoomed out. How are RELATIONSHIPS drawn: thickness, arrowheads, curved multi-edges, labels along the line, when labels appear.
> 4. Which LAYOUTS exist, which is the default, how much spacing the force layout gives (any described parameters: gravity, repulsion, link distance), and how the hierarchical and coordinate layouts work. How does the layout behave when new nodes are added (does the existing picture move)?
> 5. How do they handle CLUSTERS or communities: rule-based colouring/sizing by a property (e.g. a community id from Graph Data Science), any grouping/compound nodes, any aggregation of many nodes into one?
> 6. Colours and theme: background colour, palette, selection ring, how categories get colours, the legend.
> 7. Criticisms and known weaknesses of Bloom/Browser visualization from users or reviewers (forum posts, blog reviews), especially about hairballs, legibility and performance.
>
> Output: a structured report with a section per question, each claim followed by the URL it came from, and a clear mark of whether you read the page itself or only a search snippet. Paraphrase; quote at most one short phrase per source. End with a list titled "What Lins could copy as-is" and one titled "Where the sources suggest Neo4j's choice is weak". Be thorough: this is the main reference the redesign will be judged against. Aim for depth over breadth; if a page cannot be fetched, say so and try another source for the same fact.
