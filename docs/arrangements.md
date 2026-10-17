# Cards in lanes: a graph laid out by a rule

**Date:** 2026-10-12
**Standing:** built, for the graphs whose view asks for it. Every other graph opens as before.
**What it is for:** a picture that gives a reader order. Where a card sits says something, so the
reader can hold the picture in mind, and one graph can be looked at in several orders.

---

## 1. Why

The first picture Lins drew of a small graph of work in progress was 45 dots placed by a
simulation of springs. The owner, looking at it: *"It's good that this exists but it's not helping
me in creating mental order by way of the visualization. What do you think could help ordering and
helping me shift between different perspectives on this data?"*

Read off that picture:

- **Where a dot sat meant nothing.** Springs placed it, and placed it elsewhere on the next load.
- **Most lines said only that one thing belongs to another**, which draws a star around every
  parent. The rest crossed through the stars.
- **Names were cut and lay on top of each other.**
- **Every dot looked the same**, though what the reader wanted to tell apart was how much of each
  thing was done.

The display proposal (`display_proposal_2026_10_10.md`) had already said what answers each: a
card that holds its name (§ 5.3), a place that means something (§ 5.4), and belonging shown without
a line (§ 5.5). What it did not have is **more than one order for the same graph**. That is the
addition here.

## 2. What was built

**An arrangement** is one way of laying a graph out as cards in lanes. A view can define several,
and the page offers a switch between them.

- **A lane is a node or a value.** With `lanes` and `member`, the lanes are the nodes the rules
  name, and a lane's cards are the nodes that hold a link of the kind `member` to it. With `cards`
  and `lane_property`, the cards are the nodes the rules name and there is a lane for each value
  of the property, with a last lane for the cards that lack it.
- **Belonging is the lane.** The link that puts a card in its lane is never drawn.
- **Order inside a lane is one number** (`sort`), largest first unless `ascending`. Cards without
  the number come last. Lanes that are nodes are ordered by `lane_sort`; lanes that are values by
  the value itself.
- **A lane shows its first cards and counts the rest** (`per_lane`, 8 unless set). A stub brings in
  8 more of that lane.
- **A card can be drawn part full** (`[display.fill]`): a faint wash of its own colour from the
  left, as wide as the share that is done. The share is `done` out of `of`, or what is not `left`
  out of `of`, each a property that holds a number. A lane's heading carries the same as a thin
  bar: its own numbers when the lane is a node that has them, else its cards' summed.
- **Lines appear for one card at a time.** Pointing at a card draws its links to the other cards
  on show and lets the cards it is not tied to step back. Nothing else is drawn.
- **A card says what it is when the pointer rests on it** (2026-10-15): the properties its label's
  `tooltip` setting names, on a plate beside the card, a moment after the pointer stops. Each line of a
  property is a part of its own, cut at a few lines; the plate sits on whichever side of the card has
  more room and fades where it is cut, and the whole of every property is in the side panel. The same
  plate shows around a node. The free picture does not show it.
- **A press on a card opens it with its ties**: the card in the middle, what points at it in
  columns on the left, what it points at on the right, each column grouped by kind of link and
  counted, 8 of each kind with a stub for the rest. This is the scene the trial page drew
  (`display_proposal_2026_10_10.md` § 8a), now a part of the main page.
- **The switch** lists the view's arrangements, then *One thing and its ties*, then *Free*, the
  picture of dots and lines that Lins has always drawn. *Free* shows what was last loaded, so
  going there from a card shows that card's neighbourhood.

A graph whose view defines no arrangement has no switch and opens on the free picture, exactly as
before.

## 3. The settings

```toml
# How full a card is drawn.
[display.fill]
left = "open_count"        # or: done = "settled_count"
of = "total_count"

# Lanes that are nodes: a lane for each Project, its Tasks as cards, most open first.
[[display.arrangements]]
name = "By project"
lanes = [{ label = "Project" }]
member = "PART_OF"         # the kind of link from a card to its lane
sort = "open_count"
lane_sort = "open_count"

# Lanes that are values: a lane for each week, the earliest on the left.
[[display.arrangements]]
name = "By week"
cards = [{ label = "Task" }]
lane_property = "week"
lane_ascending = true
per_lane = 12
```

A rule is written as elsewhere in a view: a label, and optionally a key and the values it may
hold. The first arrangement is the one a graph opens on. An overlay's arrangements come first, in
its order; the user's own file replaces one of the same name and adds any other after them.

**A value lane is one exact value.** Dates are not gathered into weeks and numbers are not
gathered into ranges: the graph is expected to carry the value it wants lanes for. Values that are
all numbers are ordered as numbers, anything else as text.

The address can name how a graph opens: `?graph=<name>&order=<n>` for the arrangement in place
`n` (0 is the first), `order=free` for the free picture, and `focus=<id>` for one node with its
ties.

## 4. Where it lives

| Part | Where | What it does |
|---|---|---|
| The settings | `lins-core/src/config.rs`: `Arrangement`, `FillRule`, `LaneRule` | Parsed and layered like the rest of a view. `Arrangement::lane_rule` says what the lanes are, or that the settings name half of it |
| The queries and the order | `lins-core/src/lanes.rs` | Query text for lanes, counts, sums and cards; the order of lanes; a fill from two numbers. No database call, so all of it has unit tests |
| The route | `lins-server/src/api/graph.rs`: `GET /api/graphs/{name}/lanes/{index}` | Lanes in order, each with its count, its fill and its first cards, and the links among the cards shown. `shown` (a JSON object, lane key to count) raises the number for the lanes it names; `all=true` brings in what the view hides |
| The lanes | `lins-web/src/lib/components/LanesView.svelte` | Draws what the route returns; measures the cards only to draw the pointed card's lines |
| One thing and its ties | `lins-web/src/lib/components/SceneView.svelte` | The browser's half of the trial page, with a count for each kind of link in place of one for the whole scene |
| The switch | `lins-web/src/lib/components/OrderSwitch.svelte` | Shown only when the view has an arrangement |
| A card's note on hover | `lins-web/src/lib/components/CardNote.svelte`; `tooltipLines` in `lins-web/src/lib/view.ts` | The plate, and which lines go on it |
| What is on show | `lins-web/src/lib/stores/graph.ts`: `mode`, `board`, `sceneData`, `laneCounts`; `lins-web/src/lib/explore.ts` | One place that loads each of the three and moves between them |
| A card's look | `lins-web/src/lib/cardstyle.ts` | The scene's sizes and colours as CSS variables, so a card is the same card in a lane and around a node |

Names that are written into a query (a kind of link, a property) are checked to be plain names
twice, in `lane_rule` and again where the query is written. A value a reader's graph holds goes
into a query only as a string literal that cannot end itself.

## 5. What this settles of the display proposal, and what it leaves

The owner's word, to the proposal as sketched for one graph: *"Yes build it."* (2026-10-12). For
a graph whose view defines an arrangement that covers, of the proposal's § 10:

- **Decision 3, how much one view holds**: in part. Eight at a time, the rest behind counted
  stubs, is built, for a lane and for each kind of link around a node. The aim of 40 and the stop
  at 100 for a whole scene are not built: a lane grows as far as it is asked to.
- **Decision 4, columns around a node**: built, as the view of one thing and its ties. Forces that
  know the cards, for a free scene, are not: the free picture is still dots.
- **Decision 5, belonging without a line**: built as lanes. A chip on a card naming what it
  belongs to is not built, so around a node belonging is still a column like any other.

For every other graph all three are as open as before. Decisions 6 and 7 (light or dark cards,
the face) are untouched: the cards use the trial's colours and the page's present font.

## 6. What is not built, and what is not known

- **Which neighbours are loaded around a node is not ranked by the server.** The route for a node
  and its neighbours returns the first 60 it meets; the page then ranks what it was given. A node
  with more than 60 neighbours shows a cut it did not choose, and says so in the line under it.
- **A stub in a lane loads the whole arrangement again** with one number raised. Fine at the sizes
  tried; one lane's next cards alone would be a second route.
- **No test drives the page.** The settings, the queries, the order and the fill have unit tests.
  The route and the three views were checked by hand against a real graph of about 2,300 nodes:
  two arrangements, the lines of a pointed card, a press through to a card's ties, a stub in each
  place, the free picture and back, and a graph with no arrangement.
- **The placing rules for one thing and its ties still exist twice**, in TypeScript and in Rust,
  as long as the trial page and its Rust half stay in the tree. The count for each kind of link is
  in the TypeScript alone, since it decides what a scene holds and not where a box goes.
- **Not known**: how lanes read with dozens of them, or with cards of very different heights; and
  whether a reader wants the page to open on the order they last used rather than the first.

**Queries without a fill rule:** totals still return three fill columns, each with its own alias.
FalkorDB rejects repeated unnamed zero columns; this matters for ordinary cards without progress
counts as well as partially specified fill rules (`lanes::fill_sums`, fixed 2026-10-16).
