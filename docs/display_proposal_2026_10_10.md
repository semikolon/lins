# How Lins should show a graph: a proposal

**Date:** 2026-10-10
**Standing:** a proposal. Nothing in it is built, and nothing in it is decided. How Lins looks and
which library draws it are the owner's open decisions (`founding_session_2026_03_16.md` § 9), and
this note is written for him to rule on, one decision at a time (§ 10).
**What it rests on:** his own founding notes, his verdict on the first real view, three research
reports kept word for word in `docs/research/` (2026-10-10), and counts taken from the graph the
view was tried on.

---

## 1. In one page

The first view built for a real graph showed 150 nodes and 470 links as small dots under one
force layout. The owner's verdict: barely legible, far too cramped, the clusters do not read, and
it shows too much at once. All four follow from one choice: the view tried to show a sample of
the *graph*. This proposal shows a **scene** instead: a few dozen things the user asked for, each
large enough to read, placed where its position means something.

Six moves, each argued below:

1. **A scene, never the graph.** Lins opens empty, with a search field and a small map of the
   *kinds* of thing the graph holds. A scene aims at 40 nodes and stops at 100.
2. **Widening is ranked and counted.** Opening a node brings in its 8 most relevant neighbours;
   the rest wait behind a counted stub ("+43 more"), one per kind of link.
3. **A node holds its name.** A node with a sentence for a name is a card with the whole name on
   it, wrapped. A node with a short name is a pill. Nothing is a dot.
4. **Position means something.** Around one node: what points at it on the left, what it points
   at on the right, grouped by kind of link. Free scenes use a force layout that knows how big
   the cards are and never moves what is already placed.
5. **Fewer lines.** A link that only says "belongs to" (a claim's source, a tag) is drawn as a
   chip on the card. In the graph measured, that removes 52% of the lines.
6. **The legend is the control panel.** Every kind of node and link is listed with its colour and
   its count in the scene and in the graph; it selects, hides and brings back. What is left out
   is always said, with the reason.

The consequence that needs a ruling of its own: **cards with wrapped text are drawn best by the
browser itself** (HTML for nodes, SVG for links), which means Sigma.js is retired from the default
path (§ 7).

---

## 2. What went wrong, measured

Counts taken 2026-10-10 from the argument graph the view was tried on (7,780 nodes; 2,436 shown
once question scaffolding is left out).

| What was seen | What caused it | The number |
|---|---|---|
| "Barely legible" | Names beside dots 5 to 16 px wide, thinned by Sigma's grid | A claim's name is 35 characters at the median, 60 at most. Neo4j Browser fits about 19 in a 50 px circle. |
| "Spaced out much more" | 150 nodes and 470 links in one pane | 3.1 links per node. In 152 published studies the median is 49.5 nodes, 73 links, 1.5 links per node; the upper quartile is 116 nodes and 2.4. |
| "Clustering needs work" | The view was the 150 *best-connected* nodes | The best-connected nodes are the hubs: a source has 51 neighbours at the median and 173 at most. A view of hubs is a view of starbursts. |
| "Too much at any one time" | The first view was chosen by the program, not asked for | A claim has 2 neighbours at the median and 6 at the ninetieth percentile; 96% of claims have 8 or fewer. What a person asks about is small. |
| Half the lines say nothing | Membership drawn as a line | Of 5,438 links among shown nodes (resemblance links left out), 2,192 say "taken from this source" and 654 "uses this concept": 52%. |

Neo4j Browser has the same fault in its own code: it fits the whole result to the pane, so 150
nodes open at about a fifth of full size, each node about 10 px wide
(`docs/research/neo4j_bloom_browser_nvl_2026_10_10.md` § 3). The first Lins view reproduced
Browser's weakness, not Bloom's strength.

---

## 3. What the owner has already said

From the founding notes (March 2026), which this proposal treats as the brief:

- "I wanna at least replicate Bloom's features. But also build on it, make it even more intuitive
  and we should have great UX and great defaults for styling and auto-adjustments of node spacing,
  font sizes, padding, distances etc to maximize readability and make it really smooth on the eyes
  and easy to understand relationships."
- The default look "should have subtle contrast and feel super intuitive instead of like a
  terminal from the Matrix or something a sysadmin would endure looking at".
- A theme in the middle: "Mid-to-dark grey or dark blue background, white or bright text, accent
  color." Today's canvas is `#0f1419`, which is near black.
- Bloom's quality is its design: large nodes, names that can be read, warm distinct colours, a
  clear ring on the selection, a legend, generous spacing. And its information architecture:
  curated views, search that builds a scene, actions on a selection.
- "Development effort is a non-issue." Choices are made on what a thing can do and how it runs.
- "I'm not at all decided yet on the UI/UX design, will need some deep thought, I want to make
  super conscious decisions about it."

From 2026-10-10, on the first real view: barely legible; space it out much more; the clustering
needs work; do not show too much of the graph at any one time; take the UI with care.

---

## 4. What the research says, in the lines that carry weight

Full reports, with a source and a reading mark on every claim, are in `docs/research/`. The
strength of each line is given, because most of this field is practice and little of it is
experiment.

**How much to show.**
- No study gives a threshold for diagrams whose nodes carry names. The nearest facts: across 152
  studies the median graph has about 50 nodes and 73 links; 56 of the 152 use more than 100 nodes,
  many of them with part of the graph hidden or merged; "200 nodes at 10% density" is offered as the outer edge
  for detailed work and is the authors' judgement. *(a survey of studies; the edge is opinion)*
- Reading time grows fast with size: one simple task took 7 s at 50 nodes and 40 s at 350.
  *(one controlled study)*
- A concept map is advised to hold 15 to 25 concepts. *(practice)*

**How to explore.**
- "Search, show context, expand on demand" was proposed as the replacement for "overview first"
  on large graphs: start from a found node, show a connected piece of bounded size chosen by
  *degree of interest*, bring in a few of the most interesting hidden neighbours per click, and
  show on each node how much more there is. *(a demonstrated system, 300,000 nodes; no
  controlled test of the number per click)*
- An interface that grows a view from one node and lists the rest was preferred by 26 of 28
  people over a plain graph view, and its advantage grew with density. *(controlled study)*
- Bloom opens on an empty scene and a search field, by design. Its cap is 1,000 nodes, raised by
  the user. Its staff say it "was not designed to load an entire graph by default". Its weak
  points: what it cuts is "the first N", nothing says why 38 of 236 are shown, and there is no
  map of a large graph before the first search. *(vendor documentation, a staff reply, user
  reports)*

**How to draw.**
- What slows a reader following a path: bends in the path, crossings on the path, and branches
  off it. Crossings elsewhere in the picture did not matter. *(controlled study)*
- Bundling links together makes paths harder to follow and invites reading a link that is not
  there. *(two controlled studies)*
- A view that keeps its nodes where they were is read faster and with fewer errors when the task
  is finding a node again or following a path. *(controlled study; a second found little effect
  on other tasks)*
- Layered against force-directed layout: the studies disagree, and none used an argument graph.
  Practice does not disagree: every argument-mapping tool looked at draws boxes holding whole
  sentences, in layers or as a tree, with green for support and red for attack. *(mixed studies;
  uniform practice)*
- Text is read fastest above a critical size, about 12 px on a laptop and 15 to 17 px on a
  desktop screen at arm's length. Dark text on a light ground reads better than the reverse, and
  the gap grows as text shrinks; no study covers a mid-dark ground or graph labels. *(controlled
  studies of running text; the pixel figures are arithmetic from them)*
- A viewer tells six to twelve colours apart reliably, fewer when the marks are small.
  *(studies)*

**Clusters.**
- Drawing a region around a group helps questions about groups when the region is in one piece,
  and cost accuracy on other tasks in one study where regions were fragmented. *(two controlled
  studies that disagree; the difference is whether a region is contiguous)*
- Neo4j's new drawing library pulls clusters apart by letting links *across* a cluster boundary
  exert no pull at all, and draws an outline in the cluster's colour. Bloom itself documents only
  manual grouping. *(vendor source and documentation)*
- Nobody has tested whether a force layout tuned for clusters makes people *see* them better.
  *(a gap)*

**What cannot be had.** Neo4j's library may be used only with Neo4j's own databases, by its
licence. No open library avoids collisions between name boxes; only the layout engines that take
box sizes as input (ELK, and the commercial yFiles) place names as part of the layout.

---

## 5. The proposal

### 5.1 A scene, never the graph

**Lins opens on an empty canvas with three ways in.**

1. **Search** (as today): words in any order, or an exact id. A hit opens a scene around it.
2. **The kinds map.** One large node for each kind of thing in the graph, with its count, and one
   arrow for each pair of kinds that are linked, listing the kinds of link and their counts. Lins
   already reads this from the database. For the graph measured it is three nodes and three
   arrows carrying ten kinds of link; for any graph it is small. Clicking
   a kind lists its nodes in the panel, ranked, and one of them is opened from the list. This is
   the map before the first search that Bloom lacks.
3. **Where you were.** The last scenes, by the node each was opened from.

**A scene has a budget.** It aims at 40 nodes and stops at 100. Both are settings; the starting
values come from § 4 and should be tuned by eye, since no study covers named nodes. At the stop,
a widening still happens, and the nodes farthest from the path the user took are folded back
into counted stubs to make room, with a line saying so and an undo.

**What is shown is always stated**: "23 of 2,436 shown · opened from *Conscious-access workspace
not brain-specific*", and in the legend what is left out and why (§ 5.6).

### 5.2 Widening is ranked and counted

Opening a node brings in **up to 8 neighbours**, the rest wait behind **one stub per kind of
link**: "+12 supports", "+3 attacks". A stub is clicked to bring the next 8 of that kind. Every
node carries a small count of the neighbours not yet shown, so the user sees where more lies
before clicking.

**Which 8** is decided by an interest score, computed by the server, stated in the interface, and
the same on every run:

- links to nodes already in the scene (a neighbour that ties two shown nodes together says the
  most);
- a match with the words of the current search;
- the kind of link, by a weight per kind from the config (an argument link before a membership
  link);
- its own number of links, counted on a slow scale so that hubs do not always win.

Neo4j Browser takes "the lowest ids first", which is arbitrary; Bloom takes whatever the query
returned. Ranking, and saying what the ranking was, is the first place to be better than both.

Eight is a starting value. In the graph measured it shows 96% of claims whole at the first
click; stubs matter for the 100 claims with more than 8 neighbours and for nearly every source.

### 5.3 A node holds its name

**A card** for a kind whose names are sentences: a rounded rectangle 168 px wide, text at 13 to
14 px, wrapped over up to three lines, with a coloured edge for its kind. At that width the
median name in the graph measured (35 characters) takes two lines and the longest (60) takes
three. Below the name, small chips for what the node belongs to (§ 5.5).

**A pill** for a kind whose names are short (concepts: 14 characters at the median): the name
inside, on one line.

**Which shape a kind gets is worked out from its names**, at the median length, and can be set in
the config. This is the automatic adjustment the founding notes ask for, and it is where a
text-heavy graph is served better than by Bloom, whose users report that even at the largest
size a circle cannot hold five to fifteen words.

**Zoomed far out**, a card keeps its colour and drops its text, except the cards on the path the
user took. A scene of 40 cards at full size is meant to fit a laptop pane, so this is the exception.

### 5.4 Position means something

**Around one node (the default when a scene is opened from a node): columns.** The node in the
middle; on the left what points at it, on the right what it points at; within a column, one group
per kind of link in a fixed order, each group in rank order. Opening a neighbour adds a column
further out. Arrows run one way, and a place on the page answers a question: left bears on the
node, right is borne on by it.

An overlay may assign kinds of link to sides instead of using direction. An argument graph would
put support on the left and attack on the right, as its own reading surface does.

Starting distances: 20 px between cards in a column, 96 px between columns, enough for a link's
name. Both scale with one spacing control.

**Free (when a scene holds several things found by search, or by choice): forces that know the
cards.**

- Cards collide as the rectangles they are, with a margin.
- A link's rest length is the two cards' half-widths plus 90 px.
- The layout is computed before the first frame is shown.
- **What is placed stays placed.** New nodes start beside the node they came from and settle;
  nothing else moves. A node the user has dragged is pinned.
- One control, compact to airy, in the panel. Neo4j exposes no spacing control at all, and its
  users ask for one.

Switching between the two arrangements animates the cards to their new places.

### 5.5 Fewer lines

**Membership is a chip, not a line.** A kind of link can be marked in the config as membership.
Such a link is then drawn as a small chip on the card ("source: …", "concept: safeguards"), and
the thing it points to appears as a node only when the user opens it. This removes about half the
lines in the graph measured, and it removes the starbursts, because the hubs were sources.

**The links that remain** are 2 px, coloured by kind, with a small arrowhead. The kind's name
runs along the line when the scene has 30 links or fewer, and on hover or selection otherwise,
on a dark plate. Several links between the same two nodes are drawn as separate arcs up to
three, and as one counted line beyond that. Links are never bundled.

### 5.6 Groups that open, and a legend that controls

**A group** is a set of nodes sharing something named in the config: a label by default, a
property, or the node at the far end of a membership link (for a claim, its source).

- **Group map**: above the level of single nodes, groups are drawn as blobs sized by their
  members, with counted lines between them. Opening a blob opens its 8 most relevant members and
  a stub. This is the second level of the kinds map, and it holds 6 to 40 blobs.
- **In a scene**: when three or more members of a group are present and lie together, a soft
  outline in the group's colour is drawn around them.
- **Later**: groups found by a community algorithm on the server (Leiden, which unlike Louvain
  guarantees that each group is connected). One caution from the graph measured: its own notes
  record that a cut by topic misdescribes an argument, since 35% of its links cross such a cut.
  A group is a way in, never the authority on where a node sits in a neighbourhood.

**The legend** lists every kind of node and link with its colour and two counts, in the scene and
in the graph. Clicking a kind selects its nodes; an eye beside it hides or shows the kind; the
hidden kinds are listed with the reason and the number ("unanswered question scaffolding: 4,116,
hidden by the view"). This replaces today's single "show hidden" switch.

**The panel** also holds the properties of the selection (as today, every stored field of a node
or a link), the path the user took (each step returns to it), undo and redo, and clear.

### 5.7 The look

**Canvas** `#2a3040`: a mid-dark blue-grey, far from today's near black.

**Cards**, two candidates, with contrast measured by the WCAG formula:

| | Text on card | Card on canvas | Fits |
|---|---|---|---|
| Raised dark card `#3b445a`, bright text `#eef1f6`, coloured edge | 8.6 : 1 | 1.35 : 1 (the coloured edge carries the outline: 3.5 : 1 or better against the card, 4.8 : 1 or better against the canvas) | His words: "white or bright text", "subtle contrast" |
| Light card `#e8e6f2`, dark text `#1f2430` | 12.6 : 1 | 10.7 : 1 | The reading research: dark on light reads better at small sizes |

The proposal shows the first, at 14 px, and keeps the second as a setting.

**Colours by kind**: at most eight, muted and warm, each at 4.8 : 1 or better against the canvas
(violet `#a99be8`, amber `#e0b062`, teal `#5cc0b8`, green `#6cc48a`, red `#ee7a72`, gold
`#e3b758`, pink `#dc8cc0`, grey-blue `#9aa8c2`). Kinds differ by shape as well as colour, so
colour is never the only cue.

**Selection**: a 3 px ring in a colour used for nothing else, with a 2 px gap. **Hover**: a
lighter edge; neighbours keep full strength and the rest fade to 35%.

**Type**: the founding notes ask for a face like Söhne. Söhne is a paid face and cannot ship in
an MIT repository, so the proposal is Inter, which is free to ship and drawn for small sizes on
screens, at 14 px on cards, 12 px for chips, never smaller.

### 5.8 What this adds to the config

All of it follows the existing contract: worked out from the graph by default, changed by an
overlay, overruled by the user's own file.

```toml
[display.scene]
target = 40            # what a scene aims at
ceiling = 100          # where it folds the farthest nodes into stubs
step = 8               # neighbours brought in by one widening
arrangement = "columns"   # or "free"
spacing = 1.0          # the one spacing control

[styling.labels.Claim]
shape = "card"         # worked out from name length when absent: card, pill

[display.relationships]
as_chip = ["EXTRACTED_FROM", "USES_CONCEPT"]   # membership: a chip, not a line
left = ["SUPPORTS"]    # optional: sides by kind instead of by direction
right = ["ATTACKS"]

[display.groups]
by = "EXTRACTED_FROM"  # or a property name; a label by default

[display.interest]     # weights for the ranking in § 5.2
in_scene = 3.0
search = 2.0
degree = 0.5
```

---

## 6. Where this is better than Bloom, point by point

| Bloom or Browser today | This proposal |
|---|---|
| Opens empty; a random "graph snippet" is the only other way in | Opens empty with a map of kinds, then groups, before any search |
| Cuts to "the first N"; nothing says why 38 of 236 are shown | Ranks by a stated score; "N of M" always shown; stubs count the rest |
| A name inside a circle, cut at about 19 characters; no way to enlarge text | A card holding the whole name, wrapped; shape chosen from the names |
| No spacing control; "dense" or "spread out" by picking one of two layouts | Spacing follows card size, with one control |
| Each update restarts the layout; new nodes fly in from the origin | What is placed stays placed; new nodes start beside their parent |
| One force layout for every scene | Columns around a node, where a place has a meaning; forces for free scenes |
| Membership drawn as lines; hubs become starbursts | Membership as chips |
| Grouping by hand | Groups from the config, opened like nodes; outlines in a scene |
| Filters are lost when the scene is cleared | The legend's choices are the view, kept in the config |
| Needs a GPU and WebGL | Drawn by the browser's own text and vector engines |
| Light canvas assumed by the defaults | Tuned for a mid-dark canvas, contrast measured |

What Bloom has and this proposal keeps for later: saved searches with parameters, actions on a
selection, a range filter over time with playback, shortest path between two nodes, running
graph algorithms and styling by the result. None is needed to make a scene readable.

---

## 7. What draws it

The proposal sets the requirements. A scene is at most 100 nodes and a few hundred links. Its
nodes are boxes sized to wrapped text, with chips and counts inside them. Its layouts need the
boxes' sizes. Text must be sharp at 12 to 14 px.

| Option | What it can do for this design | What it cannot |
|---|---|---|
| **The browser itself: HTML cards, SVG links, in Svelte**, positions from a layout engine | Everything above. Text is the browser's own: wrapped, any face, sharp at every size, selectable, found by page search, read by screen readers. Hit-testing and transitions come free. | More than a few hundred nodes on screen, which the design never asks for |
| **Cytoscape.js** (canvas, MIT, released 2026-09) | Wrapped text inside nodes sized to it, rings, typed curved links with rotated names, layered and constrained force layouts, groups as rectangles | Chips and counts inside a node (one text block per node; its HTML overlay add-on was last released in 2021); collisions between names; its two best force layouts were last released in 2022 and 2023 |
| **Sigma.js** (today; version 4 came out on 2026-10-08) | Tens of thousands of dots, fast | No wrapped text, no layered layout, no groups; name thinning by grid cell with no collision test; version 4 is two days old and draws names on the GPU, so the name and hover drawing Lins has today would be written again for it |
| **egui and wgpu, compiled to WebAssembly** (the owner's own interest) | One code base for a native window and the web; every pixel under control | The graph crate for it says it is not in active development; its default node is a dot of 5 px radius and its layouts ignore node size, so the renderer and the text layout would be written from nothing; the web build is 2.5 MiB, cannot be searched by the page, and is weak on accessibility and text input |
| **G6** | Wrapped names, groups, many layouts | Set aside in March on how it runs (idle CPU at 500 nodes, faults when adding data); today 334 open issues and a 1.4 MB bundle |
| **Neo4j's own library** | The look Bloom has | Its licence allows use only with Neo4j's databases |

**Recommendation: the browser itself.** For a scene of this size the limit on quality is text,
and nothing draws text as well as the browser. The speed that Sigma and a GPU renderer buy is
speed for tens of thousands of nodes on screen, which this design rules out on purpose.

Three things follow, and each is said plainly because the owner kept this decision open:

- **Sigma leaves the default path.** It could return for one purpose: a far view of a whole graph
  as dots, if that is ever wanted.
- **The Rust renderer is not taken up now.** Its one benefit that a web page lacks, a native
  window, is on the roadmap as a desktop shell around the same page (v1.5). Writing a text
  engine in Rust would be work without a gain a user can see at this scene size.
- **Layout can still be Rust.** The server could compute positions (crates exist for force and
  layered layouts) and send them with the scene. That is a later choice and a small one.

Within the recommendation, two parts are named but not chosen here: Svelte Flow (MIT, built for
Svelte 5) as the shell that gives panning, zooming, dragging, selection and a minimap; and dagre
or ELK for the columns (ELK also places link names, at 1.6 MB and under the Eclipse licence).

---

## 8. What only a trial can show

Before the renderer is built in full, one small trial page should answer these, on the owner's
main machine and on an older laptop:

1. 100 cards and 250 links: do panning, zooming and dragging stay smooth? (Counted in frames.)
2. Cards are measured before they are placed: is there a visible jump on first paint?
3. Forces with rectangle collision: does a scene of 100 settle before first paint, and does
   adding 8 leave every placed card where it was?
4. Columns on a neighbourhood with cycles in it: does the layered layout stay readable?
5. Link names along the line: readable at 12 px on short links between large cards?
6. Bright text on raised dark cards at 14 px against dark text on light cards: which does the
   owner read faster on his own screen?

---

## 9. Build order, once ruled

Each step is useful alone and can be checked against the real graph.

1. **Scene and ranking on the server**: the interest score, `step`, stubs with counts, "N of M".
   Independent of what draws it; today's page benefits at once.
2. **The trial page** of § 8.
3. **The scene renderer**: cards, pills, links, both arrangements, stable placement.
4. **Membership as chips; the legend as control; path, undo, clear.**
5. **The kinds map and the group map.**
6. **Theme and type.**
7. Later: groups by algorithm; ranking by meaning from vectors already stored on nodes (the
   server sees them and strips them today); saved searches and actions on a selection.

---

## 10. Decisions for the owner, in the order they depend on each other

1. **Cards that hold the whole name, drawn by the browser, in place of dots drawn by Sigma.**
   Everything else rests on this.
2. The budget: aim at 40, stop at 100, bring in 8 at a time.
3. Columns around a node as the default arrangement, forces for free scenes.
4. Membership as chips.
5. Bright text on raised dark cards, or dark text on light cards.
6. Inter as the face.

---

## 11. What was not checked

- Nothing here was built or tried. Every distance, size and count in § 5 is a starting value.
- No study covers how many *named* nodes can be read at once, a mid-dark canvas, text inside
  against text beside a node, or the best number of neighbours per click. Where the proposal
  gives a number for these, it is judgement from the nearest evidence.
- The three research reports were written by a model reading the web and were not checked a
  second time against their sources. Figures read from Neo4j's minified library code are marked
  as such in the report.
- That Svelte Flow can draw links that attach anywhere on a card's edge was not verified.
- That Söhne's licence bars shipping it comes from general knowledge, not from reading the
  licence.
- The counts in § 2 are from one graph, an argument graph. A graph of short-named entities (a
  memory graph, for instance) would get pills where this one gets cards, and its membership
  links would be different ones.
