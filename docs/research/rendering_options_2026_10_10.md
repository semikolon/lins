<!--
Kept word for word. Research report: Rendering options for Lins.
Date: 2026-10-10.
What this is: the report of a research helper (a language model with web access and no access
to this repository), kept exactly as it was returned. Nobody has checked it against its sources
a second time. Each claim carries the helper's own mark for how it was read; trust the marks, and
check a number against its source before quoting it elsewhere.
Why it was asked: the first view built for a real graph of 7,780 nodes was judged barely legible
by the project's owner. The proposal that draws on this report is docs/display_proposal_2026_10_10.md.
The question the helper was given follows the report, also word for word.
-->

# Graph rendering options for Lins: capability research (as of 2026-10-10)

## 0. Legend, method, and four things to know first

Marks after each claim:
- **[raw]**: I read the primary file or API response verbatim (raw GitHub source, npm/crates.io/GitHub API JSON, a LICENSE file).
- **[page]**: I fetched the web page. The fetch tool condenses long pages, so detail may be lost.
- **[snip]**: I saw only a search-result snippet.
- **[bg]**: my background knowledge, not re-verified here.
- **[inference]**: my own deduction from sources I did read.

Date format is ISO. Bundle sizes are minified, uncompressed UMD from jsDelivr, measured by me with HEAD requests. I could not run any code.

**Housekeeping:** I used `curl` through Bash for web APIs. I accidentally wrote one scratch file, `/tmp/egui_cl.txt` (a copy of the egui changelog). It is unrelated to your project, and I left it untouched because deleting is outside my read-only brief. Delete it if you like.

**Things to know first:**
1. **Sigma 4.0.0 was published two days ago, on 2026-10-08.** Your pinned 3.0.2 dates from 2025-05-27. The last v3 release is 3.0.3 (2026-04-30). The sigmajs.org docs now describe only v4, and old v3 URLs redirect. [raw] https://api.github.com/repos/jacomyal/sigma.js/releases?per_page=12 and https://registry.npmjs.org/sigma
2. **v4 removes much of the v3 API that your question names.**
   - Gone: `nodeProgramClasses`, custom `NodeProgram` classes, `@sigma/edge-curve`, `@sigma/node-square`.
   - Replaced by: declarative `primitives` and `styles`.
   - Labels move from a 2D canvas to GPU signed-distance-field (SDF) text. [page] https://github.com/jacomyal/sigma.js/releases/tag/sigma%404.0.0 and https://www.sigmajs.org/how-to/technical/migration-v3-v4/
3. **Neo4j NVL cannot be used with FalkorDB.** The license text bars it (section 2.9 below).
4. **No candidate has real label-box collision avoidance for node labels.** Sigma uses a per-grid-cell heuristic. Cytoscape, G6 and vis-network have none. Only ELK and yFiles place edge or node labels as part of layout.

---

## 1. Sigma.js in depth

### 1.1 Status

| Item | Value | Source |
|---|---|---|
| Latest stable | 4.0.0, 2026-10-08. Betas ran 2026-08-14 to 2026-10-05. | [raw] https://api.github.com/repos/jacomyal/sigma.js/releases?per_page=12 |
| Previous | 3.0.3 (2026-04-30), 3.0.2 (2025-05-27), 3.0.1 (2025-01-16) | same |
| Repo | MIT, 12,184 stars, 7 open issues/PRs, last push 2026-10-09 | [raw] https://api.github.com/repos/jacomyal/sigma.js |
| Bundle | 4.0.0: 391 KB. 3.0.2: 187 KB. | [raw] https://cdn.jsdelivr.net/npm/sigma@4.0.0/dist/sigma.min.js |
| Companion packages at 4.0.0 | `@sigma/node-border`, `@sigma/node-image`, `@sigma/utils` (all 4.0.0, 2026-10-08) | [raw] https://registry.npmjs.org/@sigma/node-border |
| Companion package left at v3 | `@sigma/edge-curve` 3.1.0 (2024-12-13) | [raw] https://registry.npmjs.org/@sigma/edge-curve |

The v4 alpha thread says the rewrite was driven by depth and opacity problems, canvas labels that could not take events, and WebGL context overhead. A commenter described the code base as beta-mature by mid-2026. [page] https://github.com/jacomyal/sigma.js/discussions/1539

### 1.2 Capability table against the wanted look

| Wanted | v3.0.x | v4.0.0 |
|---|---|---|
| Large nodes (40–70 px) | Yes. Sizes are screen pixels (`itemSizesReference: "screen"`). | Yes. Sizes are in layout units by default (`"positions"`), so they scale with the layout. |
| Name inside node | No built-in. A custom `defaultDrawNodeLabel` can draw centred text on the canvas label layer. | Built-in: `labelPosition: "over"`, one line, GPU SDF text. |
| Wrapped or multi-line | Only in your own canvas drawer, using `measureText`. | No wrap option found. Canvas or SVG "label attachments" are the documented way to get arbitrary multi-line content. |
| Label collision | Per-cell top-N by node size. No box test. | Same heuristic. |
| Hover box | `defaultDrawNodeHover` is replaceable. | White backdrop is just a default style, fully overridable. |
| Selection ring | Border program plus reducers. | `layerBorder()` or backdrop, driven by state. |
| Edge colour by type | Edge `color` attribute. | Style rules, including node-referenced colours. |
| Arrowheads | Arrow or curve programs. | `extremityArrow()` and others, per edge. |
| Curved and parallel edges | `@sigma/edge-curve` | Built in (`pathCurved` etc.), native parallel and self-loop handling. |
| Edge labels | Canvas, off by default. | GPU, off by default. |
| Edge hit-testing | GPU picking, `enableEdgeEvents` off by default | Same, plus `edgePickingPadding` default 4. |
| Drag | Wire up yourself | Built-in `enableNodeDrag` |
| Zero-size container | Throws unless `allowInvalidContainer` | Setting still exists |

### 1.3 Findings: how it renders and what you can configure

**v3 render stack**
- v3 builds seven layers in this order: `edges` (WebGL), `edgeLabels` (2D), `nodes` (WebGL), `labels` (2D), `hovers` (2D), `hoverNodes` (WebGL), `mouse` (2D). [page] https://raw.githubusercontent.com/jacomyal/sigma.js/sigma%403.0.2/packages/sigma/src/sigma.ts
- Node labels are therefore native canvas text drawn above all nodes. Text inside a node always sits over neighbouring nodes. 2D canvases are scaled by the pixel ratio, so text is crisp on HiDPI. [page] same URL
- Defaults in 3.0.2: `labelFont` "Arial", `labelSize` 14, `labelWeight` "normal", `labelColor` #000, `stagePadding` 30, `zoomToSizeRatioFunction` `Math.sqrt`, `itemSizesReference` "screen", `autoRescale` true, `labelDensity` 1, `labelGridCellSize` 100, `labelRenderedSizeThreshold` 6, `minEdgeThickness` 1.7. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/sigma%403.0.2/packages/sigma/src/settings.ts

**v3 label and hover drawing**
- The default label drawer sets the font from settings and draws text at `x + size + 3`, i.e. to the right of the node. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/sigma%403.0.2/packages/sigma/src/rendering/node-labels.ts
- The default hover drawer paints a white rounded box with an 8 px black shadow, then calls the label drawer. That is your "white box, pale text" complaint. [page] https://raw.githubusercontent.com/jacomyal/sigma.js/sigma%403.0.2/packages/sigma/src/rendering/node-hover.ts
- Lookup order is `nodeProgram?.drawLabel || defaultDrawNodeLabel`, and the same for hover. [page] sigma.ts URL above
- Both functions receive the canvas context plus node display data (viewport `x`, `y`, `size`). A drawer can therefore centre multi-line text inside the disc.
- A maintainer-level answer confirms this: override `defaultDrawNodeLabel` and `defaultDrawNodeHover` to get multi-line "callout boxes". [page] https://github.com/jacomyal/sigma.js/discussions/1506

**v3 label selection (hence overlaps)**
- A label is shown if the grid selects it and its size is at least `labelRenderedSizeThreshold`. Labels are culled outside the viewport margins. [page] sigma.ts URL
- The v4 grid code, which I read in full, is the same idea. Each 100 px cell keeps the top `ceil(density / ratio²)` candidates, sorted by node size. There is no bounding-box collision test. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/main/packages/sigma/src/core/labels.ts
- Setting `labelVisibility: "visible"` bypasses culling entirely, so then nothing prevents overlap.
- Issue #1295, "Experiment with label anticollision setting" (opened 2022-10-20), is still open and labelled enhancement. [page] https://api.github.com/repos/jacomyal/sigma.js/issues?state=open&per_page=30

**Sizing gotchas for 40–70 px nodes** (the first two are [inference] from the docs; the third is documented)
- v3 uses pixel node sizes but auto-rescaled positions. The pixel gap between nodes then depends on graph bounding box and viewport, and large nodes overlap unless the layout works at pixel scale or rescaling is off.
- v4's `"positions"` mode puts sizes and positions in one unit. [page] https://www.sigmajs.org/concepts/sizes/
- The default zoom function is `Math.sqrt`, so nodes shrink more slowly than distances when you zoom out, and overlaps can appear. The official label-styles example sets an identity function for this reason. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/main/packages/website/src/examples/styling/label-styles.astro
- `autoRescale: true` refits on content change. Adding one node can therefore shift the whole view [inference]. v4 adds `autoRescale: "once"`, and the drag guide says to use it to freeze coordinates. [page] https://www.sigmajs.org/how-to/interactivity/drag-drop/
- v4 `autoRescaleContent` can be "positions" (default), "nodes" or "labels". The last is described as computationally expensive. [page] https://www.sigmajs.org/concepts/sizes/

**v4 labels**
- Labels are GPU SDF text: glyphs are rasterised once into an atlas and drawn as textured quads. The docs say they stay crisp at any zoom. [page] https://www.sigmajs.org/concepts/rendering/
- The atlas source defaults to a 64 px base font, 8 px buffer, 24 radius and 2048 px maximum texture. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/main/packages/sigma/src/core/sdf-atlas.ts
- Style properties: `labelPosition` (right, left, above, below, over), `labelAngle`, `labelVisibility` (auto, visible, hidden), `labelDepth`, `labelSize`, `labelFont`, `labelColor`, `labelBackgroundColor`, `labelBackgroundPadding`. Backgrounds are flat rectangles with no border or radius, and they are the hit area for label events. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/main/packages/website/src/content/docs/how-to/labels/styles-positioning.mdx
- `LabelOptions` has only `margin` (default 5), `font`, `color` and `zoomToLabelSizeRatioFunction`. [page] https://www.sigmajs.org/api/sigma/rendering/interfaces/labeloptions/
- I grepped the node-label generator, factory, base, label-renderer and atlas sources for wrap, newline, maxWidth, ellipsis and line-height. I found no wrapping or multi-line support (a negative result from grep; the atlas packer only matched "wrap" in a GL texture-parameter sense). [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/main/packages/sigma/src/rendering/nodes/labels/generator.ts
- Label attachments draw a callback's SVG or Canvas into a texture, rendered by WebGL. An `"html"` type exists but is blocked on Chromium and Safari because it relies on `foreignObject`. Placement options are below, above, left, right (not "centre on node"). [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/main/packages/website/src/content/docs/how-to/labels/attachments.mdx and https://raw.githubusercontent.com/jacomyal/sigma.js/main/packages/sigma/src/types/styles.ts
- Edge labels (v4): position over, above, below or auto; fixed or scaled font size; `textBorder`; backgrounds. [page] https://www.sigmajs.org/how-to/labels/styles-positioning/
- Atlas overflow bug #1552 corrupted glyphs once roughly 410–440 unique code points had accumulated at devicePixelRatio ~1.5. It was fixed 2026-09-14 in the beta line. [page] https://github.com/jacomyal/sigma.js/issues/1552
- `hideLabelsOnMove` initially failed to hide node labels in v4 (#1556). It was fixed. [page] https://github.com/jacomyal/sigma.js/issues/1556
- Node shapes: `sdfCircle`, `sdfSquare`, `sdfTriangle`, `sdfDiamond`, `sdfRectangle`. The rectangle takes `aspectRatio` (fixed or from a node attribute) and `cornerRadius`, so a pill sized from measured text width is possible [inference]. [page] https://www.sigmajs.org/how-to/nodes/shapes/

**Hover and highlight (v4)**
- The default node style shows a white backdrop (shadow blur 12, padding 6) only when hovered. Every `backdrop*` property is overridable: colour, border, radius, shadow, area. The docs call backdrops expensive and meant for a handful of elements. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/main/packages/sigma/src/types/styles.ts and https://raw.githubusercontent.com/jacomyal/sigma.js/main/packages/website/src/content/docs/how-to/nodes/backdrops.mdx
- Neighbourhood highlighting uses `renderer.on("enterNode")` to set an active subgraph state, plus style rules that grey out inactive items and lift active ones into a higher depth layer. [page] https://www.sigmajs.org/how-to/interactivity/hover-search/
- v3 `nodeReducer`/`edgeReducer` still exist in v4 as escape hatches. [page] migration URL above

**Edges**
- v4 paths: `pathLine`, `pathCurved`, `pathCurvedS`, `pathStep`, `pathStepCurved`, `pathLoop`. Colours can be constants, attribute references, or the source or target node colour. Dashed and gradient layers exist. [page] https://www.sigmajs.org/how-to/edges/types-colors/
- v3 `@sigma/edge-curve` offers quadratic-Bezier curves, a curvature attribute, `indexParallelEdgesIndex` for parallels, and a pre-built curved arrow program. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/sigma%403.0.2/packages/edge-curve/README.md
- Its factory exposes arrowhead length and wideness as uniforms named as ratios of edge thickness, and ships `createDrawCurvedEdgeLabel`. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/sigma%403.0.2/packages/edge-curve/src/factory.ts. So arrow size tracks line thickness [inference].
- The v3 default straight-edge label drawer returns without drawing when the centre distance is less than the sum of the node radii. With large nodes, short edges lose their labels. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/sigma%403.0.2/packages/sigma/src/rendering/edge-labels.ts
- v3 `@sigma/node-border`: variable or fixed border thickness and colour, multiple borders, default 10 % of radius. Its README says nothing about labels. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/sigma%403.0.2/packages/node-border/README.md

**Interaction**
- v4 has built-in dragging: `enableNodeDrag`, node state `isDragged`, `getDraggedNodes` for multi-drag, `dragPositionToAttributes` for snapping. A `ForceSupervisor` can pin dragged nodes via `isNodeFixed`. [page] https://www.sigmajs.org/how-to/interactivity/drag-drop/
- Camera: `animate()` returns a Promise, `reset()`, `zoomIn()`/`zoomOut()`. `fitViewportToNodes(renderer, nodes, {animate: true})` from `@sigma/utils`. Renamed from v3's `animatedReset` and `animatedZoom`. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/main/packages/website/src/content/docs/how-to/interactivity/camera-viewport.mdx and the migration page
- Position tweening for layouts uses `animateNodes(graph, positions, {duration, easing})` from `sigma/utils`. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/main/packages/website/src/examples/cookbook/layouts.astro
- Lasso: I found no lasso or selection package in the repo tree. [raw] https://api.github.com/repos/jacomyal/sigma.js/git/trees/main?recursive=1
- Combo/grouping: not supported. Issue #1515 asks for it and is open. [page] issues URL above
- v4 adds a GPU ForceAtlas2 package. Adding or dropping nodes stops its run, so you must restart it. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/main/packages/layout-fa2-gpu/README.md. For 150 nodes the CPU worker is enough.

**Other v4 details**
- WebGL 2 is required (WebGL 1 dropped). [page] release notes URL above
- HiDPI: v3 scales canvases by pixel ratio (above). The v4 label shader takes a pixel-ratio uniform and a pixel-snapping option. [raw] generator.ts URL above
- Zero-size container: v3 throws unless `allowInvalidContainer` is set. v4 retains the setting. [page] sigma.ts URL; [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/main/packages/sigma/src/settings.ts
- Performance of the label layer: the docs claim smooth behaviour with tens of thousands of items and give no label-specific figure. [page] https://www.sigmajs.org/concepts/rendering/. A few hundred labels is not a concern [inference].
- README note: since v4, development uses LLM assistance for mechanical code under human architecture. [page] https://github.com/jacomyal/sigma.js

### 1.4 Layout companions (graphology)

Package status:
- graphology 0.26.0 (2025-01-26).
- layout-forceatlas2 0.10.1 (2022-10-17), with 0.11.0-rc1 (2024-11-08).
- layout-noverlap 0.4.2 (2022-02-08).
- layout-force 0.2.4 (2022-01-05).
- layout 0.6.1 (2022-09-20).
- communities-louvain 2.0.2 (2024-12-17).

All are stable but mostly quiet. [raw] https://registry.npmjs.org/graphology-layout-forceatlas2 (and the same path for the other package names)

**ForceAtlas2 settings.** Defaults and API are from the docs [page] https://graphology.github.io/standard-library/layout-forceatlas2.html and defaults.js [raw] https://raw.githubusercontent.com/graphology/graphology/master/src/layout-forceatlas2/defaults.js. The effect column is my background on the algorithm [bg].

| Setting (default) | Effect on spacing and clusters |
|---|---|
| `scalingRatio` (1) | Repulsion coefficient. Raise it for more space between nodes. This is the main spacing knob. |
| `gravity` (1) | Pull to the centre. Higher means more compact. |
| `strongGravityMode` (false) | Gravity does not weaken with distance. This keeps disconnected pieces close. |
| `linLogMode` (false) | Noack LinLog: tighter clusters with larger gaps between them. |
| `outboundAttractionDistribution` (false) | Divides attraction by degree, so hubs stay central and their leaves spread. |
| `adjustSizes` (false) | Uses node `size` in repulsion. The source comment calls it anti-collision. Nodes need a `size` before the run. |
| `edgeWeightInfluence` (1) | Needs `weighted: true`. |
| `slowDown` (1) | Damps motion. Higher is slower and smoother. |
| `barnesHutOptimize` (false), `barnesHutTheta` (0.5) | Approximation for large graphs, unnecessary at 150 nodes. |

- `inferSettings(graph)` returns `strongGravityMode: true, gravity: 0.05, scalingRatio: 10, slowDown: 1 + ln(order)`, and `barnesHutOptimize` only when order > 2000. [raw] https://raw.githubusercontent.com/graphology/graphology/master/src/layout-forceatlas2/index.js
- The layout reads a node's `size` and a `fixed` attribute (1 = pinned), so pinning existing nodes while new ones relax is possible. [raw] https://raw.githubusercontent.com/graphology/graphology/master/src/layout-forceatlas2/helpers.js
- Caveat: `size` there is a layout unit. With Sigma v3's pixel sizes the units mix [inference].
- The worker listens for node and edge add or drop events and re-sends its matrices. [raw] https://raw.githubusercontent.com/graphology/graphology/master/src/layout-forceatlas2/worker.js
- Nodes must have non-zero initial x/y. All-zero start fails. [page] FA2 docs URL above

**Other layouts**
- **noverlap.** `gridSize` 20, `margin` 5, `expansion` 1.1, `ratio` 1, `speed` 3, `maxIterations` 500. Iterative, "might not converge easily". It has a worker variant. [page] https://graphology.github.io/standard-library/layout-noverlap.html
- **layout-force.** `attraction` 0.0005, `repulsion` 0.1, `gravity` 0.0001, `inertia` 0.6, `maxMove` 200, with `isNodeFixed`. It runs on `requestAnimationFrame`. The docs call it "organic" and a good fit for dragging. It has no size awareness. [page] https://graphology.github.io/standard-library/layout-force.html
- **layout.** `circular`, `random`, `circlepack` (bubble layout by `hierarchyAttributes`), plus rotation and position-collection helpers. [page] https://graphology.github.io/standard-library/layout.html
- **louvain.** `assign`, `detailed`, `resolution`, `rng`, `getEdgeWeight`. It works on undirected, directed and multigraphs. [page] https://graphology.github.io/standard-library/communities-louvain.html
- **Layered (Sugiyama) layouts: none.** The layout package lists nothing hierarchical. The only DAG package (`graphology-dag`) offers topological and cycle utilities. [page] https://graphology.github.io/standard-library/ I found no npm package bridging graphology to dagre or ELK. [raw] https://registry.npmjs.org/-/v1/search?text=graphology+dagre. You would call dagre or ELK directly and write x/y back.

**Official examples for cluster labels and hulls**
- `cluster-labels` computes cluster centroids and positions HTML elements through `graphToViewport()` on `afterRender`. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/main/packages/website/src/content/docs/how-to/layers/sync-html-svg.mdx
- `highlight-group` runs louvain (seeded) and draws a contour overlay with `@sigma/layer-webgl`'s `createContoursProgram`. [raw] https://raw.githubusercontent.com/jacomyal/sigma.js/main/packages/website/src/examples/webgl-layers/highlight-group.astro

---

## 2. Alternatives

### 2.1 Cytoscape.js 3.34.3 (published 2026-09-07), MIT, canvas renderer

| Criterion | Assessment |
|---|---|
| Text inside node, wrapping | Yes. Label alignment inside (`text-valign: center`, `*-inside`), `text-wrap: wrap/ellipsis`, `text-max-width`. Font size does not shrink to fit. |
| Label collision | None built in. `min-zoomed-font-size` only hides tiny text. |
| Layouts | Force with constraints (fcose), cola (`avoidOverlap`), dagre, ELK, and Cytoscape's built-ins. |
| Incremental | fcose `randomize: false`, `fixedNodeConstraint`. Cola `randomize: false` default. |
| Expand/collapse, grouping | Compound nodes (rectangles only). The expand-collapse extension is unmaintained. |
| Animation | Layout `animate`, element and viewport animations. |
| Edge hit-testing | Geometry on canvas. |
| HiDPI | `pixelRatio: 'auto'` |
| Performance | Fine for 150 nodes. WebGL preview exists, not needed here. |
| Svelte 5 | No official wrapper. Create in `onMount`/`$effect`. |

Findings:
- Label properties: `text-wrap` (none, wrap, ellipsis), `text-max-width`, `text-overflow-wrap`, `text-background-shape` (rectangle, round-rectangle, circle), `text-outline-width`, `text-halign`/`text-valign` including `*-inside`, `min-zoomed-font-size`, `text-events`. [raw] https://raw.githubusercontent.com/cytoscape/cytoscape.js/unstable/documentation/md/style.md
- Node `width`/`height` accept the keyword `label`. I confirmed it in the source (nodeSize enum), not in the docs. [raw] https://raw.githubusercontent.com/cytoscape/cytoscape.js/unstable/src/style/properties.mjs
- Selection ring: `outline-width/color/style/offset` and `overlay-*`/`underlay-*` exist (a ring outside a 60 px disc is expressible). [raw] style.md URL above
- Compound nodes support only rectangle shapes. [raw] style.md URL above
- Edges: `curve-style` haystack (default, no arrows), straight, bezier (bundled parallels, `control-point-step-size`), taxi and others. Arrowheads need non-haystack styles. `edge-text-rotation: autorotate`. [raw] style.md URL above
- Layout dimensions: `nodeDimensionsIncludeLabels` is documented for `layoutDimensions()`. fcose honours it in "proof" quality. [raw] https://raw.githubusercontent.com/cytoscape/cytoscape.js/unstable/documentation/md/collection/layoutDimensions.md and the fcose README below
- fcose 2.2.0 (2023-01-17), MIT, last push 2026-04-17. [raw] https://registry.npmjs.org/cytoscape-fcose and https://api.github.com/repos/iVis-at-Bilkent/cytoscape.js-fcose
  - README options: `quality` draft/default/proof, `randomize`, `animate`, `nodeDimensionsIncludeLabels`, `packComponents`, `nodeRepulsion`, `idealEdgeLength`, `nodeSeparation` (75), `initialEnergyOnIncremental`, `fixedNodeConstraint`, alignment and relative-placement constraints. The README says `randomize: false` requires `quality: "proof"`. [raw] https://raw.githubusercontent.com/iVis-at-Bilkent/cytoscape.js-fcose/master/README.md
- cola 2.5.1 (2022-02-23): `avoidOverlap` true, `nodeSpacing`, `handleDisconnected`, `flow`, `alignment`, `randomize: false`. [raw] https://raw.githubusercontent.com/cytoscape/cytoscape.js-cola/master/README.md
- dagre extension 4.0.1 (2026-08-28), elk extension 2.3.0 (2024-11-26), cose-bilkent 4.1.0 (2019). [raw] https://registry.npmjs.org/cytoscape-dagre and the sibling paths
- Expand-collapse 4.1.1 (2024-08-28): README states the repo is no longer maintained. It collapses compound nodes and parallel edges, with animation and cue icons. [page] https://github.com/iVis-at-Bilkent/cytoscape.js-expand-collapse. Expanding a node's neighbours is plain `cy.add` and needs none of this [bg].
- I found no built-in "rich text" or HTML labels in 3.31–3.34. The release pages carried no details. [page] https://blog.js.cytoscape.org/ The HTML overlay extension `cytoscape-node-html-label` last released 1.2.2 on 2021-01-27. [raw] https://registry.npmjs.org/cytoscape-node-html-label
- `pixelRatio: 'auto'` is the default. [raw] https://raw.githubusercontent.com/cytoscape/cytoscape.js/unstable/documentation/md/core/init.md
- WebGL preview (3.31.0, 2025-01-13), enabled with `renderer: {name: 'canvas', webgl: true}`:
  - It supports node styles and labels, straight, haystack and bezier edges, and triangle arrows only. It does not support dashed lines, edge overlays or underlays, source and target edge labels, other arrow shapes, or gradients. [page] https://blog.js.cytoscape.org/2025/01/13/webgl-preview/
  - Published figures: a 1,200-node graph goes from 20 to over 100 FPS, a 3,200-node graph from 3 to 10 FPS. It builds sprite sheets at load. It was still labelled experimental. [page] https://github.com/cytoscape/cytoscape.js/issues/3305
  - Canvas is the right choice at 150 nodes [inference].
- Repo: 11,236 stars, 25 open issues/PRs, last push 2026-10-06. [raw] https://api.github.com/repos/cytoscape/cytoscape.js. Bundle 436 KB. [raw] https://cdn.jsdelivr.net/npm/cytoscape@3.34.3/dist/cytoscape.min.js
- Layout engines ignoring labels is a long-running complaint (issues #1013, #1626, #2872). [snip] https://github.com/cytoscape/cytoscape.js/issues/1626

### 2.2 AntV G6 5.1.1 (latest dist-tag; published 2026-05-08), MIT

| Criterion | Assessment |
|---|---|
| Text inside, wrapping | `center` placement exists. `labelWordWrap`, `labelMaxWidth`, ellipsis. |
| Collision | No node-label collision handling found. |
| Layouts | d3-force, ForceAtlas2, Fruchterman, dagre, antv-dagre, circular, concentric, radial, grid, MDS, combo-combined. |
| Incremental | Docs do not describe it. Unverified. |
| Grouping | Combos (circle and rect) with collapse-expand. |
| Rendering | Canvas default, optional SVG or WebGL. |
| Heavy | 1.38 MB bundle |
| Svelte 5 | Framework-agnostic core. A community `svelte-g6` wrapper exists (age unknown). |

Findings:
- Label placement type includes `'center'`. Node defaults are `labelPlacement: 'bottom'`, `labelWordWrap: false`, `labelMaxWidth: '200%'`. The shared label shape defaults to `maxLines: 1`, `wordWrapWidth: 128` and `textOverflow: '...'`. Halo, badge, icon and port shapes exist. [raw] https://raw.githubusercontent.com/antvis/G6/v5/packages/g6/src/elements/nodes/base-node.ts, https://raw.githubusercontent.com/antvis/G6/v5/packages/g6/src/types/placement.ts and https://raw.githubusercontent.com/antvis/G6/v5/packages/g6/src/elements/shapes/label.ts
- 20 built-in layouts. Web Worker via `enableWorker` for all but tree layouts. WASM versions for Fruchterman, ForceAtlas and Force, plus Dagre. GPU versions for Fruchterman and GForce. [raw] https://raw.githubusercontent.com/antvis/G6/v5/packages/site/docs/manual/layout/overview.en.md
- `@antv/layout-wasm` is described as a WASM binding of a Rust implementation. [raw] https://registry.npmjs.org/@antv/layout-wasm/latest. Versions 1.5.2 and 1.6.2 were published 2026-05-19 while the `latest` tag still points at 1.4.2. The same tagging oddity applies to G6 (5.2.1 and 5.3.1 published 2026-05-19, `latest` is 5.1.1). Check which version you would actually install. [raw] https://registry.npmjs.org/@antv/g6
- `@antv/layout` 2.0.0 (2026-02-11), 2.0.1 (2026-06-05). [raw] https://registry.npmjs.org/@antv/layout. 5.1 renamed shared layout fields to match it. [snip] https://g6.antv.vision/en/manual/whats-new/upgrade-to-5-1/
- Renderers: Canvas default; SVG via `@antv/g-svg`; WebGL via `@antv/g-webgl`; per-layer renderer choice. [raw] https://raw.githubusercontent.com/antvis/G6/v5/packages/site/docs/manual/further-reading/renderer.en.md
- Combos: circle combo wraps children in a circular boundary; collapse-expand keeps positions of expanded or collapsed nodes. [snip] https://g6.antv.antgroup.com/en/manual/element/combo/circle-combo
- The layout overview page does not mention incremental layout. [page] https://g6.antv.antgroup.com/en/manual/layout/overview. A third-party R wrapper preserves positions only with a special option and animation disabled. [snip] https://www.cynkra.com/blog/2026-01-20-g6R-0.5.0/ (this suggests incremental placement needs care)
- Repo: 12,324 stars, 334 open issues/PRs, last push 2026-10-10. [raw] https://api.github.com/repos/antvis/G6. Bundle 1.38 MB. [raw] https://cdn.jsdelivr.net/npm/@antv/g6@5.1.1/dist/g6.min.js

### 2.3 Svelte Flow (@xyflow/svelte 1.7.0, 2026-09-24, MIT) with ELK.js or dagre

| Criterion | Assessment |
|---|---|
| Text inside, wrapping | Full: nodes are DOM, so CSS typography, wrapping and pill sizing all work. |
| Collision | None. Layout engines avoid node overlap when sizes are measured. |
| Layouts | None built in. Docs recommend dagre, d3-hierarchy, d3-force, elkjs. |
| Incremental | You write it. ELK has interactive options. |
| Grouping | Parent/sub-flow nodes. |
| Animation | Svelte transitions; connected edges follow animated nodes. |
| Edge hit-testing | SVG path plus a wide invisible hit path. |
| HiDPI | Browser-native text. |
| Svelte 5 | Native (requires Svelte ^5.25.0). |

Findings:
- Svelte 1.0 shipped 2025-05-14, built on Svelte 5 runes. [snip] https://svelteflow.dev/whats-new/2025-05-14. 1.7.0's peer dependency is svelte ^5.25.0. [raw] https://registry.npmjs.org/@xyflow/svelte/latest. Repo: 38,628 stars, 148 open issues/PRs, pushed 2026-10-09. [raw] https://api.github.com/repos/xyflow/xyflow
- Layout guide (updated 2026-09-09): dagre is the simple choice and handles dynamic node sizes. d3-hierarchy gives all nodes the same size. d3-force is physics but needs care for large graphs. The docs say they rarely recommend elkjs because of its complexity. [page] https://svelteflow.dev/learn/layouting/layouting
- Edge labels: `<EdgeLabel>` renders an HTML div positioned in the viewport coordinate system and selects the edge on click. Edge paths come from `getBezierPath`/`getSmoothStepPath`/`getStraightPath` plus `BaseEdge`. Arrow markers Arrow and ArrowClosed are available. [page] https://svelteflow.dev/api-reference/components/edge-label, https://svelteflow.dev/learn/customization/custom-edges and https://svelteflow.dev/api-reference/svelte-flow
- `onlyRenderVisibleElements` defaults to false. `minZoom` 0.5 and `maxZoom` 2 are defaults. `fitView` options include duration and node subsets. [page] https://svelteflow.dev/api-reference/svelte-flow
- Performance guidance (React Flow doc, same core): trouble comes from re-renders and heavy CSS (animations, shadows, gradients). [page] https://reactflow.dev/learn/advanced-use/performance. Hundreds of nodes are fine in practice [bg].
- Older issue: the viewport transform was not rounded, so nodes looked blurry at some resolutions (issue #3282, date unknown). [snip] https://github.com/wbkd/react-flow/issues/3282
- It is a node-editor model. Organic force-directed exploration, parallel edges and curved typed edges all need custom edge components [inference].

### 2.4 ELK.js 0.12.0 (2026-07-17) as a layout engine under any renderer

| Criterion | Assessment |
|---|---|
| Algorithms | layered, stress, force, radial, mrtree, box, disco, rectpacking, sporeOverlap, sporeCompaction, fixed, vertiflex, random. |
| Node/edge labels | Part of the layout (placement options). |
| Incremental | `org.eclipse.elk.interactive`, stress "fixed position" per node. |
| License | EPL-2.0 OR GPL-3.0-or-later. |
| Size | 1.6 MB bundled file, 8 MB unpacked package. |

Findings:
- Algorithm list from the reference; interactive mode is advertised for Box, DisCo, Fixed, Force, Layered, Mr. Tree, Radial, Rectangle Packing, Stress and others. "Interpretation varies by algorithm." [page] https://eclipse.dev/elk/reference/algorithms.html and https://eclipse.dev/elk/reference/options/org-eclipse-elk-interactive.html
- Stress: stress majorization, desired edge length 100 by default, per-node fixed position, interactive mode. It does not remove node overlap by itself. [page] https://eclipse.dev/elk/reference/algorithms/org-eclipse-elk-stress.html
- Layered: spacing options (node-node 20, between layers 20 by default), orthogonal, polyline or spline routing, edge label placement (centre, `SMART_DOWN` side selection), ports, cycle breaking (greedy default). Layering strategies include INTERACTIVE (the docs say nothing about how it works). [page] https://eclipse.dev/elk/reference/algorithms/org-eclipse-elk-layered.html and https://eclipse.dev/elk/reference/options/org-eclipse-elk-layered-layering-strategy.html
- elkjs exposes async `layout()`. An optional Web Worker is configured with `workerUrl`. The Java-to-JS transpile may cost performance. [page] https://github.com/kieler/elkjs
- Licence per npm: `EPL-2.0 OR GPL-3.0-or-later`. The GitHub API reports NOASSERTION. [raw] https://registry.npmjs.org/elkjs/latest and https://api.github.com/repos/kieler/elkjs. Repo: 2,811 stars, 97 open issues/PRs, pushed 2026-10-06.
- Sizes: bundled file 1,609,707 bytes; unpacked package 8,046,232 bytes. [raw] https://cdn.jsdelivr.net/npm/elkjs@0.12.0/lib/elk.bundled.js and https://registry.npmjs.org/elkjs/latest
- I found no runtime benchmark for ELK at ~150 nodes. It must be measured.

### 2.5 d3-force or d3-dag with Canvas or SVG, and force-graph

| Criterion | Assessment |
|---|---|
| Text inside, wrapping | Whatever you draw: measure text, fit font, wrap, pill shapes. |
| Collision | `forceCollide` handles circles only. Label-box collision is yours to build. |
| Layouts | `forceLink` distance, `forceX/Y/Radial` clustering. d3-dag Sugiyama for DAGs. |
| Incremental | Reheat with `alpha` or `alphaTarget`; pin with `fx`/`fy`. |
| Expand/group | Yours to build. |
| Animation | You control it. |
| Edge hit-testing | Yours to build (or use force-graph's shadow canvas). |
| HiDPI | Yours (force-graph handles it). |
| Svelte 5 | Natural for SVG, since Svelte can render the markup and d3 computes positions [bg]. |

Findings:
- d3-force 3.0.0 (2021-06-05), ISC, repo last pushed 2023-12-30: stable and effectively finished. 8 KB. [raw] https://registry.npmjs.org/d3-force and https://api.github.com/repos/d3/d3-force
- `forceCollide`: circles only, soft constraint, `radius`/`strength`/`iterations`. [page] https://d3js.org/d3-force/collide
- Simulation: default `alphaDecay` ≈ 300 ticks, `velocityDecay` 0.4. Add nodes with `nodes(array)`, `alpha(1).restart()`. `fx`/`fy` pin nodes. [page] https://d3js.org/d3-force/simulation
- `forceLink`: default distance 30, strength inverse to min degree. `forceX/Y` default strength 0.1 with a documented per-group clustering pattern. [page] https://d3js.org/d3-force/link and https://d3js.org/d3-force/position
- d3-dag 1.2.2 (2026-07-05), MIT: Sugiyama (default), Zherebko, Grid. Custom node sizes, quality presets, edge route points. DAG only. No compound graphs. [page] https://github.com/erikbrinkman/d3-dag. Repo last pushed 2026-10-01. [raw] https://api.github.com/repos/erikbrinkman/d3-dag
- **force-graph 1.52.0** (2026-09-28), MIT, 173 KB, canvas on d3-force-3d:
  - Custom drawing: `nodeCanvasObject`, `linkCanvasObject`, `nodePointerAreaPaint`, `linkPointerAreaPaint`.
  - Built-ins: directional arrows, `linkCurvature`, `warmupTicks`/`cooldownTicks`, `d3Force('collide', …)`, `zoomToFit` and `centerAt`/`zoom` with transition milliseconds, `autoPauseRedraw`.
  - `graphData()` updates keep existing positions. [page] https://github.com/vasturiano/force-graph
  - Source: uses `window.devicePixelRatio` for both the display canvas and a hit-testing shadow canvas, and drags set `fx`/`fy` and raise `alphaTarget` to 0.3. [raw] https://raw.githubusercontent.com/vasturiano/force-graph/master/src/force-graph.js
  - Repo: 2,141 stars, 151 open issues/PRs. [raw] https://api.github.com/repos/vasturiano/force-graph. `3d-force-graph` 1.80.1 is the 3D sibling and irrelevant to this view.

### 2.6 vis-network 10.1.2 (2026-08-19), Apache-2.0 OR MIT

| Criterion | Assessment |
|---|---|
| Text inside, wrapping | Label is inside ellipse, circle, database, box and text shapes. Outside for dot, square, image and others. `widthConstraint` wraps text. |
| Layouts | Physics solvers (barnesHut, forceAtlas2Based, repulsion, hierarchicalRepulsion) with `avoidOverlap`, plus a hierarchical layout. |
| Edge labels | `font.align`, multi-font markup (`multi: 'html'|'markdown'`). |
| Maintenance | 351 open issues/PRs. |

Findings:
- Label-inside shapes (ellipse, circle, database, box, text) versus outside (image, circularImage, diamond, dot, star, triangle, hexagon, square, icon). `widthConstraint` breaks text at spaces, `font.multi` enables markup, `borderWidthSelected` defaults to 2, and `chosen` takes callbacks for selected/hover styling. [page] https://visjs.github.io/vis-network/docs/network/nodes.html
- A "circle" shape sizes itself to the label [bg]. A fixed 60 px disc needs a different approach.
- Physics: `avoidOverlap` 0–1 uses node radius. Docs do not describe adding nodes at runtime. [page] https://visjs.github.io/vis-network/docs/network/physics.html
- Edge arrows (`to`, `from`, `middle`, `scaleFactor`, types), `color.inherit`, `smooth` types (dynamic, curvedCW, cubicBezier, …), `selfReference`, `hoverWidth`, `selectionWidth`. [page] https://visjs.github.io/vis-network/docs/network/edges.html
- Repo: 3,631 stars, 351 open issues/PRs, pushed 2026-10-08. [raw] https://api.github.com/repos/visjs/vis-network. Bundle 652 KB. [raw] https://cdn.jsdelivr.net/npm/vis-network@10.1.2/standalone/umd/vis-network.min.js

### 2.7 Reagraph, Cosmograph/cosmos.gl

- **Reagraph 4.32.0** (2026-06-25), Apache-2.0. React-only (peer `react >=16`) and built on three.js and react-three-fiber. Layouts include force, radial, hierarchical, no-overlap, ForceAtlas2, circular and concentric. It has lasso selection, clustering and edge bundling. [raw] https://registry.npmjs.org/reagraph/latest, [page] https://github.com/reaviz/reagraph. It cannot be used from Svelte without a React island.
- **cosmos.gl 3.5.1** (2026-10-08), MIT: GPU layout and rendering for hundreds of thousands of points. The README I fetched did not mention text labels; I saw no built-in node label API. Cosmograph's separate label code is in other repos. [page] https://github.com/cosmosgl/graph, [raw] https://registry.npmjs.org/@cosmos.gl/graph/latest. Scale-oriented, so poor fit for richly labelled small graphs [inference]. The older `@cosmograph/cosmos` and `@cosmograph/cosmograph` packages are CC-BY-NC-4.0 (non-commercial). [raw] https://registry.npmjs.org/@cosmograph/cosmos

### 2.8 Neo4j NVL (@neo4j-nvl/base 2.0.0, 2026-09-15)

The npm package ships a LICENSE.txt. It says NVL may only be used with (i) Neo4j Aura or (ii) Neo4j's proprietary commercial graph database products. [raw] https://unpkg.com/@neo4j-nvl/base@2.0.0/LICENSE.txt and https://registry.npmjs.org/@neo4j-nvl/base. **It cannot be used with FalkorDB.** Whether an MIT project could ship it at all is also a problem. I did not review what else NVL does, since the licence settles it.

### 2.9 Commercial, for reference only
- **yFiles for HTML.** Hierarchical, organic, orthogonal, tree, radial, circular layouts and edge routing. Label placement (`GenericLabeling`). Hierarchical layout from an existing sketch (`fromSketchMode`, `incrementalNodes`) adds elements while preserving the mental map. Yearly term licence, per developer or project. [page] https://www.yfiles.com/the-yfiles-sdk and [snip] https://docs.yfiles.com/yfiles-html/api/HierarchicalLayout/
- **Ogma 6.1.0 (Linkurious).** WebGL-first renderer, force, hierarchical, radial, concentric and grid layouts, grouping. Commercial. [page] https://doc.linkurious.com/ogma/latest/
- **KeyLines/ReGraph.** GPU rendering, organic, hierarchy, radial, structural, sequential and lens layouts, combos. Per-application licence. ReGraph is the React edition. [page] https://cambridge-intelligence.com/keylines/
- What they add over the open options: label placement that avoids collisions, incremental layouts that preserve positions, and combos.

---

## 3. Rust-native: egui + wgpu/eframe to WASM

| Criterion | Assessment |
|---|---|
| Text inside, wrapping | Possible with custom node drawing. The default node draws a small circle with a monospace label above it. |
| Collision | None. |
| Layouts | Random, a simple level-based hierarchical layout, Fruchterman-Reingold. None is node-size aware. |
| Incremental | Layout "extras" and state; not verified for incremental use. |
| HiDPI text | Improved: skrifa with hinting since 0.34. Browser text features are absent. |
| Web limits | No page search, weak accessibility and mobile text input, WASM size. |

Findings:
- egui_graphs 0.32.0 (2026-08-31), MIT, 707 stars. Its README says the project is "not in active development", although releases continue. Features are random, hierarchical and Fruchterman-Reingold layouts with composable extras, custom `DisplayNode`/`DisplayEdge`, and a WASM demo. [page] https://github.com/blitzarx1/egui_graphs, [raw] https://crates.io/api/v1/crates/egui_graphs. Releases and targeted egui versions: v0.30.0 for egui 0.34.1, v0.29.0 for egui 0.33. [page] https://github.com/blitzarx1/egui_graphs/releases
- From source: the default node is a `CircleShape` of radius 5. Its label is drawn in monospace, font size equal to the radius, no wrapping, above the node, and only when labels are forced or the node is selected, dragged or hovered. The hierarchical layout has fixed `row_dist`/`col_dist` steps and no ranking by node size. The Fruchterman-Reingold code has no node-size handling that I could find. [raw] https://raw.githubusercontent.com/blitzarx1/egui_graphs/main/crates/egui_graphs/src/draw/displays_default/node.rs, https://raw.githubusercontent.com/blitzarx1/egui_graphs/main/crates/egui_graphs/src/layouts/hierarchical/layout.rs and https://raw.githubusercontent.com/blitzarx1/egui_graphs/main/crates/egui_graphs/src/layouts/force_directed/implementations/fruchterman_reingold/core.rs. Everything beyond that (in-node wrapped text, ring, typed edge labels) means writing your own `DisplayNode` and `DisplayEdge`.
- egui 0.36.2 (2026-09-08). Recent text history:
  - 0.33.0 (2025-10-09): improved kerning.
  - 0.34.0 (2026-03-26): font backend moved from `ab_glyph` to `skrifa` + `vello_cpu`, with hinting.
  - 0.35.0 (2026-06-25): `harfrust` for better kerning and ligatures.
  - 0.36.0 (2026-08-05): mobile IME fixes.
  - Unreleased 0.37: colored emoji and system glyphs through `GlyphRasterizer`/`FontProvider`.
  [raw] https://raw.githubusercontent.com/emilk/egui/main/CHANGELOG.md
- The eframe README lists web limitations: pixel-perfect rendering is hard, pages cannot be searched, mobile text editing is weaker, there is no integration with browser colour or font settings, and accessibility is experimental. It suggests eframe for apps where performance and responsiveness matter more than those. [raw] https://raw.githubusercontent.com/emilk/egui/main/crates/eframe/README.md. It renders through WebGPU when available, otherwise WebGL2, and defaults to wgpu.
- WASM size: issue #8328 reports a jump from 572 KiB (egui 0.33) to 2.52 MiB (0.35), blamed on `vello_cpu`, `skrifa` and `harfrust`. It was only partly mitigated: monochrome emoji fonts became opt-in on 2026-09-05. [page] https://github.com/emilk/egui/issues/8328
- Open web/text issues include mobile keyboard and focus (#4500, #4572, #4571), a canvas resize loop (#4699) and UI flicker on fast resize (#903). [page] https://github.com/emilk/egui/issues?q=is%3Aissue+is%3Aopen+label%3Aweb+text
- Other Rust crates (crates.io API):
  - `forceatlas2` 0.8.0 (2025-10-12).
  - `fdg-sim` 0.9.1 (2022-12-17), stale; the plain `fdg` crate is not on crates.io.
  - `rust-sugiyama` 0.4.0 (2025-09-21).
  - `layout-rs` 0.1.3 (2025-04-24).
  - `egui-snarl` 0.12.0 (2026-08-26), a node-editor widget.
  - `egui_xyflow` 0.4.3 (2026-05-06).
  - `glyphon` 0.12.0 (2026-07-09), for wgpu text.
  [raw] https://crates.io/api/v1/crates/forceatlas2 (same path pattern per crate)
- Since the Lins server is Rust, the layout could also be computed server-side with the `forceatlas2` or `rust-sugiyama` crates and sent to any JS renderer. This is a hybrid I did not research further.

---

## (a) What Sigma.js can be made to do, and what it cannot

**Can do (with v4.0.0 or with v3 plus custom drawers):**
- Draw large circular or rounded nodes (40–70 px) with a fixed look: fill, border layers (`@sigma/node-border`), backdrops.
- Show the name inside a node: v4 with `labelPosition: "over"` (single line); v3 with a custom `defaultDrawNodeLabel` that centres text on the label canvas.
- Wrap names inside nodes in v3 through your own canvas drawer. In v4, a Canvas label attachment is the documented route, untested for centred wrapped text (attachment placements are only below/above/left/right).
- Replace the white hover box: v3 by replacing `defaultDrawNodeHover`; v4 by restyling the default backdrop.
- Colour edges by type, with arrowheads, curved and parallel edges, and edge labels (v4 native; v3 through `@sigma/edge-curve`).
- Drag nodes, animate the camera and position changes, fit to a node set, highlight neighbourhoods, label clusters with an HTML overlay, and draw community contours.
- Keep nodes from overlapping, if layout units and node sizes are aligned (ForceAtlas2 `adjustSizes` plus a noverlap pass, with v4 `"positions"` sizing and an identity `zoomToSizeRatioFunction`).
- Run incrementally: graphology FA2 supports a `fixed` attribute, and `animateNodes` tweens the result.

**Cannot, or not out of the box:**
- Real label-box collision avoidance. It uses a per-cell top-N heuristic, and forcing labels visible disables even that.
- Native text wrapping or multi-line node labels in v4 labels.
- Layered (Sugiyama) layouts or compound groups. The graphology ecosystem has none, so dagre or ELK would be bridged by hand. Combos are an open request (#1515).
- A built-in lasso.
- Guaranteed no overlap across zoom levels without custom size and zoom settings.
- Keeping the view stable when nodes are added, unless you use v4's `autoRescale: "once"`.
- Any proof yet that v4's SDF text matches native canvas text at 11–14 px. It is two days into its stable life, and the ecosystem (wrappers, examples) has not caught up.

## (b) Ranked shortlist for at most ~150 labelled nodes

1. **Cytoscape.js (canvas).** Closest to the wanted look with the fewest custom parts. Fixed discs with in-node, wrapped and ellipsised text, outline rings, typed bezier edges with arrowheads and rotated labels with backgrounds, fcose or cola layouts with incremental and constraint options, animated layouts, `pixelRatio` auto, geometry hit-testing, MIT, and release 3.34.3 on 2026-09-07. Gaps: font does not shrink to fit, no label collision, rectangular-only compounds, and fcose and cola have not been released since 2023 and 2022. Runtime at 150 nodes on canvas is not a concern.
2. **Canvas 2D on d3-force, ideally force-graph 1.52.0 as the shell.** Highest ceiling: you can fit text to the disc, shape pills, add real collision against label boxes, and write your own edge-label placement. force-graph already provides pointer-area hit-testing, devicePixelRatio handling, zoom transitions, and position-preserving graph updates. The cost is that each missing feature (rectangular collision, edge label avoidance, arrowheads sized to large nodes) is yours to build. Runtime is fine at this scale.
3. **AntV G6 5.1.x.** Similar capability to Cytoscape: centred labels, word wrap, max lines, ellipsis, combos with circular boundaries, d3-force with collision, dagre, and Rust-based WASM layouts. Held back by a 1.38 MB bundle, 334 open issues, the `latest` tag lagging newer published versions, and no documented incremental-layout behaviour.
4. **Sigma.js (v4.0.0, or v3.0.3 with custom drawers).** Strong GPU rendering, but nothing in this view needs that scale. Gaps for this brief: no native wrapping, heuristic label culling, external layouts, no groups, and a v4 line that is two days old. It moves up if prototype checks (below) show SDF text is crisp enough and attachments or v3 drawers handle wrapped names well.
5. **Svelte Flow 1.7 + ELK or dagre.** Best typography and layered layouts (ELK also places edge labels), native Svelte 5, and CSS selection rings. Weaker as an organic graph explorer: no force layout with collision, parallel and curved typed edges are custom, and the docs themselves steer away from elkjs. It would rank 2nd if a tidy layered look is acceptable.
6. **vis-network 10.1.2.** Fast to get label-in-circle nodes and physics, but fixed-size discs and fine control of edge-label styling are awkward, and it has 351 open issues.
7. **egui_graphs on egui/wgpu WASM.** Last for capability and runtime: README says the project is not in active development, the default node is a radius-5 dot with a hover-only label, layouts ignore node size, WASM grew to 2.5 MiB, there is no page search, and text input and accessibility are weak. It would amount to writing your own renderer.
- **Excluded:** NVL (licence forbids non-Neo4j use), Reagraph (React-only), cosmos.gl (scale-oriented, no label API found), commercial products (cost).

## (c) Open questions only a prototype can answer

1. **Sigma v4 SDF text:** how sharp is 11–14 px text on 1x and 2x displays, white on warm colours, compared with native canvas text? Does it need an outline for contrast? (Only edge labels list `textBorder`.)
2. **Sigma v4 `"over"` labels:** what happens with names of 15–25 characters inside a 50 px disc? Does anything clip or shrink the text? Can `labelSize` be a function of text length?
3. **Sigma v4 attachments:** can a canvas attachment hold centred, wrapped names for ~150 nodes? What are the texture memory cost and per-frame cost?
4. **Sigma stability on node add:** with `autoRescale: "once"`, do existing nodes stay put? Can opacity be animated for a fade-in? Do `animateNodes` and `fitViewportToNodes` behave in v4?
5. **ForceAtlas2 at 40–70 px nodes:** which `scalingRatio`, `adjustSizes` and noverlap combination gives no overlap including label extents? Does pinning existing nodes with `fixed` while new ones relax look natural?
6. **Cytoscape edge hit area:** how forgiving is hit-testing on thin bezier edges? Do rotated edge labels stay readable on parallel curved edges? Does fcose with `randomize: false` and `quality: "proof"` keep old nodes steady?
7. **G6 incremental layout:** when you add nodes, do existing nodes move? Does the combo layout hold up?
8. **ELK:** run time in a worker for ~150 nodes with node and edge label sizes (layered, stress, force). Does `elk.stress` with fixed positions give a stable incremental result? Is a 1.6 MB bundle acceptable?
9. **Svelte Flow:** the measure-then-layout flicker, ELK layered on cyclic FalkorDB data, and whether HTML edge labels stay readable at zoom levels 0.5–2.
10. **Label collision strategy:** for ~150 nodes and ~300 typed edges, how often do edge labels collide in each option, and is a greedy placement pass (yours, or ELK's) needed?
11. **Lifecycle:** behaviour when the container has zero size at first paint, on resize, and when the window moves between displays with different pixel ratios (Sigma, Cytoscape, force-graph).
12. **egui, if kept alive:** can a custom `DisplayNode` draw wrapped text inside a 60 px disc with acceptable crispness? What is the real compressed WASM size after opting out of the optional fonts?

---

## Appendix: the question as it was put to the helper

> Research task (web only; do not read or edit any local files). I maintain "Lins", a small open-source (MIT) explorer for FalkorDB graph databases: a Rust server and a SvelteKit (Svelte 5, TypeScript, Vite) page that today draws the graph with Sigma.js 3.0.2 + graphology, laid out by graphology-layout-forceatlas2 in a web worker. The owner wants the look and feel of Neo4j Bloom or better: LARGE circular or rounded nodes (roughly 40 to 70 px) with the node's name readable INSIDE or directly beside the node, generous spacing with no overlaps, warm distinct colours on a mid-dark background, a clear selection ring, links coloured by type with arrowheads and a label that can be read, smooth animation when nodes are added, and views that stay small (a few dozen nodes, rarely more than ~150) and grow by expanding one node at a time. Today it renders tiny dots with small labels; on hover Sigma's default hover label draws a white box with pale text; labels overlap. The rendering library is an OPEN decision; the candidates named so far are Sigma.js (current), a Canvas 2D or SVG renderer, and a Rust-native egui+wgpu renderer compiled to WASM.
>
> Your job: establish, from documentation, source repositories and issues, WHAT EACH OPTION CAN ACTUALLY DO for this kind of view, so a choice can be made on capabilities and runtime behaviour (not on ease of development).
>
> Part 1. Sigma.js v3 in depth (https://www.sigmajs.org/, https://github.com/jacomyal/sigma.js, the storybook examples):
> - Can it draw large nodes with text inside? What exists: @sigma/node-border, @sigma/node-image, @sigma/node-piechart, @sigma/node-square, custom node programs, the "node label inside" patterns; how labels are rendered (a 2D canvas layer above WebGL) and what the settings labelRenderedSizeThreshold, labelDensity, labelGridCellSize, labelFont, labelSize, labelWeight, labelColor, zoomToSizeRatioFunction, itemSizesReference, autoRescale do; how to supply custom defaultDrawNodeLabel and defaultDrawNodeHover functions (to replace the white hover box), and whether multi-line or wrapped labels are possible.
> - Edges: @sigma/edge-curve for curved and parallel edges, arrow programs, edge labels (renderEdgeLabels and custom drawers), edge events and hit-testing of thin edges.
> - Interaction: dragging nodes, camera animation (animatedReset, fitting to a set of nodes), highlighting neighbourhoods, lasso.
> - Layout companions in graphology: forceatlas2 settings and what each does to spacing and clusters (linLogMode, outboundAttractionDistribution, adjustSizes, edgeWeightInfluence, scalingRatio, strongGravityMode, gravity, slowDown, barnesHutOptimize, barnesHutTheta; inferSettings), graphology-layout-noverlap, graphology-layout-force (the supervisor variant for live dragging), circular/circlepack/random layouts, graphology-communities-louvain, and the official examples that draw cluster labels or cluster hulls. Is there any layered (Sugiyama-style) layout in the graphology ecosystem, or would dagre/ELK have to be bridged?
> - Known limits and open issues relevant here (text quality, labels inside nodes, hover renderer, performance of the label layer with many labels, HiDPI handling, container with zero size at start).
>
> Part 2. Alternatives suited to views of roughly 20 to 300 richly labelled nodes, each assessed on: node-with-text-inside and text wrapping; label collision handling; layouts available (force with collision, layered/dagre/ELK, radial/concentric, compound/grouped); incremental layout when nodes are added without moving the rest; expand/collapse and grouping support; animation; hit-testing of edges; HiDPI text crispness; measured or documented performance limits; licence; maintenance health (last release, open issues); how it is used from Svelte 5.
>   - Cytoscape.js (with fcose, cola, dagre, elk, the expand-collapse and compound-node features, cytoscape-node-html-label / the newer built-in rich labels, the WebGL renderer preview)
>   - AntV G6 v5 (combos, built-in behaviours, layouts, its Rust/WASM layout options)
>   - Svelte Flow / xyflow with ELK.js or dagre (DOM nodes, so full typographic control)
>   - d3-force with SVG or Canvas (forceCollide, forceLink distance, forceX/Y clustering forces) and d3-dag
>   - vis-network, force-graph / 3d-force-graph (vasturiano), Reagraph, Cosmograph/cosmos.gl
>   - Neo4j's own NVL (@neo4j-nvl/base): licence terms and whether it may be used with a non-Neo4j database
>   - Commercial: yFiles, Ogma (Linkurious), KeyLines/ReGraph: what they do that the open libraries do not (for reference only)
>   - ELK.js itself as a layout engine usable under any renderer (layered, force, stress, radial, "mrtree"), its options for node spacing and for incremental layout.
>
> Part 3. Rust-native: egui + wgpu/eframe compiled to WASM, the crate egui_graphs, and any other Rust graph-drawing crates (e.g. fdg, layout crates); what text rendering quality, layout algorithms and interaction they give today; known egui limits (text shaping, emoji, IME, memory).
>
> Output: a structured report: for each option a short capability table against the wanted look, then findings with a URL after each claim and a mark of whether you read the page/repo itself or only a snippet. Paraphrase; quote at most one short phrase per source. End with (a) "What Sigma.js can be made to do, and what it cannot", (b) a ranked shortlist for a view budget of at most ~150 labelled nodes, with the reason for each rank stated in terms of capability and runtime behaviour, and (c) the open questions that only a small prototype could answer. Be precise about version numbers and dates; things in this field change quickly, so prefer 2025 and 2026 sources and say when a source is older.
