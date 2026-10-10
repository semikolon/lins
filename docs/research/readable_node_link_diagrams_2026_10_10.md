<!--
Kept word for word. Research report: Readable node-link diagrams, and exploring a large graph without overwhelming the viewer.
Date: 2026-10-10.
What this is: the report of a research helper (a language model with web access and no access
to this repository), kept exactly as it was returned. Nobody has checked it against its sources
a second time. Each claim carries the helper's own mark for how it was read; trust the marks, and
check a number against its source before quoting it elsewhere.
Why it was asked: the first view built for a real graph of 7,780 nodes was judged barely legible
by the project's owner. The proposal that draws on this report is docs/display_proposal_2026_10_10.md.
The question the helper was given follows the report, also word for word.
-->

# Research report: readable node-link diagrams and exploring a large graph without overwhelming the viewer

Prepared for the Lins redesign (argument graph: about 2,300 claims, 35 documents, 100 concepts, typed directed links).

## How to read this report

Every finding carries a URL and a mark for how I read it:

- **[FULL]**: I read the full text, or the relevant sections, of the source itself.
- **[EXCERPT]**: I read substantial passages of the real page or paper as returned by a search or fetch tool. These are partial but are source text.
- **[EXTRACT]**: a fetch tool returned a model-written extraction of the page. I did not see the page text. Treat the numbers as plausible but spot-check them before quoting.
- **[SNIPPET]**: I saw only a search-result abstract or summary.
- **[MEMORY]**: background knowledge that I did not fetch this session.

Strength tags used in the rules at the end:

- **S**: controlled study.
- **V**: vendor or practitioner guidance.
- **I**: my own inference or arithmetic.

I did not read or edit any project file. I used Bash/python only to grep web documents that the fetch tools had saved to their tool-results folder.

**Reference point for the Lins graph.** The current first view has 150 nodes and 470 links. That is 3.1 links per node, an average degree of about 6.3, and a density of 2.1% by the survey's formula |E|/(|V|(|V|−1)). Keep these figures in mind when reading the thresholds below.

---

## 1. How much to show

**Yoghourdjian, Archambault, Diehl, Dwyer, Klein, Purchase, Wu (2018), "Exploring the limits of complexity: a survey of empirical studies on graph visualisation", Visual Informatics 2(4).**
URL: https://arxiv.org/pdf/1809.00270 (also https://doi.org/10.1016/j.visinf.2018.12.006). I read the full text [FULL] by grepping it.

- The survey covers 152 graph-visualisation user studies.
- Table II gives the sizes used.

  | Measure | Median | Lower quartile | Upper quartile |
  |---|---|---|---|
  | Nodes | 49.5 | 18.5 | 116.5 |
  | Edges | 73 | 23 | 242.5 |
  | Linear density (edges/nodes) | 1.5 | 1.0 | 2.4 |
  | Density (%) | 5 | 1.7 | 10 |

- The current Lins view (150 nodes, 470 links, 3.1 links per node) is above the upper quartile on all three of nodes, edges and linear density.
- Only 56 of 152 studies use graphs with more than 100 nodes. Many of those show only a sub-part of the network or use abstraction or aggregation.
- Study sizes cluster at 20, 50 and 100 nodes. The authors say these round numbers were chosen arbitrarily and were not found empirically. They also say no controlled study of ceiling and floor effects exists.
- The survey proposes size bands, with clear cuts at 20, 50 and 200 nodes:
  - Small: up to 20 nodes.
  - Medium: 21 to 50.
  - Large: 51 to 200.
  - Very large: over 200.
- The authors' explicit threshold is "200 nodes and 10% density". They say it is respected by studies whose tasks need detailed analysis. They call it expert intuition rather than empirical research and say it needs a controlled study.
- The survey notes it is "reasonable" to use node-link diagrams for path finding when the graph has fewer than 200 nodes (citing Bae & Watson and Ghoniem).
- Studies with no interaction mostly use at most 100 nodes (70 of 80, 88%) and at most 500 edges (71 of 80, 89%).
  - Interaction types used on graphs above 100 nodes and 500 edges are Abstract/Elaborate, Connect, Explore, Reconfigure and Select.
  - Graphs below that size mostly used Encode, Create and Filter.
- The survey relays that Huang et al. found errors rising when users face even 25 nodes and 98 links.
- It also relays that Okoe et al. used 900 nodes and 2,500 edges and saw error rates above 50%.
- Caveat: these studies mostly use unlabelled or short-label abstract graphs and tasks such as path finding or counting. None measures reading of 3 to 7 word headlines.

**Ghoniem, Fekete & Castagliola (2005), "On the readability of graphs using node-link and matrix-based representations", Information Visualization 4(2):114–135.**
URL: http://www-sop.inria.fr/orion/COGC/teams/INSITUghoniem-fivj05-final.pdf. Read the abstract, setup, discussion and conclusion [FULL].

- Setup: random undirected graphs with 20, 50 and 100 vertices at link densities 0.2, 0.4 and 0.6, with seven tasks. Node-link diagrams were drawn with Graphviz neato.
- Result: above about 20 vertices, matrices beat node-link diagrams on most tasks. Path finding stays in favour of node-link.
- For small graphs, node-link diagrams were always more readable.
- For larger graphs, node-link performance "deteriorates quickly" while matrices keep roughly a 30% lead in correct answers.
- On dense node-link graphs, users appeared to guess. Answer times fell while error rates rose.
- Path finding is hard on both representations when the shortest path is long. The authors recommend interaction for it, such as selecting a node and showing paths from it.
- Their density is d = links/n². Their sparsest condition is 0.2. Lins is about 0.02 by this formula, roughly ten times sparser. So "fails above 20 vertices" does not transfer directly to Lins. It does show that density, not only count, drives failure.

**Cognitive load.** Huang, Eades & Hong (2009), "Measuring effectiveness of graph visualizations: a cognitive load perspective", Information Visualization 8(3).
URL: https://journals.sagepub.com/doi/abs/10.1057/ivs.2009.10. [SNIPPET] for the paper itself.

- I only have the abstract of the paper. The survey above gives a relayed account of its model [FULL on the survey].
- Cambridge Intelligence cites the "seven things" short-term-memory limit as the reason big graphs overload viewers (see section 3). The survey also names Miller's 7±2 as a commonly cited rule of thumb, but not as an established graph limit.

**Vendor and tool numbers**

- **Neo4j Browser** (https://neo4j.com/docs/browser/operations/browser-settings/, https://neo4j.com/developer/kb/how-do-i-override-browser-configuration-settings/) [EXCERPT]:
  - Defaults are `maxVizNodes` 1000 and `maxNewNeighbours` 100 per expansion.
  - An older config shows `initialNodeDisplay` 300 and `maxNeighbours` 100.
  - These are tool safety limits, not comfort levels.
- **Linkurious**: a supernode threshold set by the administrator (the manual's example is 1000) shows an approximate count badge. Source: https://doc.linkurious.com/user-manual/latest/page.html [SNIPPET].
- **Cambridge Intelligence** (https://cambridge-intelligence.com/blog/visualize-large-networks/) [EXTRACT]:
  - No numeric node limit is given.
  - Hairball density grows quickly with node count.
  - The post argues for designing around the user's workflow, not raw data.
- **Concept maps**: Novak & Cañas say 15 to 25 concepts usually suffice per map. They expect 50 to 60 only when sub-maps are linked. URL: https://cmap.ihmc.us/publications/ResearchPapers/TheoryUnderlyingConceptMapshq.pdf [EXCERPT].
- **Saket et al. (2014)** (https://cns.iu.edu/docs/publications/2014-saket-ieee.pdf) [EXCERPT]:
  - A pilot sized graphs so that a simple task took 5 to 30 seconds. They found 7.3 s at 50 nodes and 40.2 s at 350 nodes.
  - They chose 50, 100 and 200 nodes as the tested sizes (24.3 s at 200 nodes).
  - Quadrupling density (links per node from 1× to 4×) raised node-based and network-task time by about 22% to 33%.
  - Quadrupling size cut network-task accuracy by 12% in node-link-group diagrams.

**Aesthetics and cognitive cost (for how much crossing clutter is tolerable)**

- **Ware, Purchase, Colpoys & McGill (2002), "Cognitive measurements of graph aesthetics"**, Information Visualization 1(2). URL: https://vislab-ccom.unh.edu/pdfs/GraphAesthetics.pdf [EXCERPT]:
  - 137 spring-layout drawings; shortest-path task.
  - Response time was predicted by path length, path "bendiness" (continuity), crossings on the path itself, and the number of side branches off path nodes.
  - 100 degrees of bend cost about 1.7 s, one crossing about 0.65 s, and one extra branch about 0.3 s.
  - One crossing is roughly equal in cost to 38 degrees of bend.
  - The total number of crossings in the whole drawing was not significant. Only crossings on the path mattered.
  - Caveat: spring layouts only, and small graphs.
- **Purchase** (summarised in Burch et al., https://opus.lib.uts.edu.au/bitstream/10453/144947/2/FINAL%20Article.pdf) [EXCERPT]:
  - Fewer crossings was by far the most important aesthetic.
  - Fewer bends and more symmetry mattered less.
  - Maximising minimum angle and grid-fixing had little effect.
  - Purchase, Carrington & Allder (2002) found crossings the top-priority aesthetic for UML diagrams (preference, not performance). URL: https://webdoc.sub.gwdg.de/edoc/e/EMIS/journals/JGAA/accepted/02/PurchaseAllderCarrington02.6.3.pdf [EXCERPT].
- **Huang, Hong & Eades**, crossing angles (https://dl.acm.org/doi/10.1016/j.jvlc.2014.03.001 and https://arxiv.org/pdf/0810.4431) [EXCERPT]:
  - Two studies (22 and 37 participants): larger crossing angles are easier to read, and the benefit levels off near 90 degrees.
  - Eye tracking shows crossings barely matter for node-finding. They matter for path tracing, and more so on larger graphs and longer paths.
  - Small angles cause confusion and extra eye movement.

**Where sources disagree.** The survey's threshold of 200 nodes and 10% density is expert intuition. Ghoniem shows failure from 20 vertices, but at much higher density. The Cambridge Intelligence "seven things" argument is a cognitive-psychology rule of thumb, not a graph study. No source gives an empirically validated "above N, do X" rule.

---

## 2. Exploration models that avoid showing the whole graph

**van Ham & Perer (2009), "Search, Show Context, Expand on Demand", IEEE TVCG 15(6):953–960.**
URL: https://perer.org/papers/adamPerer-DOIGraphs-InfoVis2009.pdf. I read sections 1 to 3.5 and the start of 3.4 [FULL]. I did not read the case study or discussion.

- Thesis: for many tasks a global overview is neither relevant nor practical. They propose "search, show context, expand on demand" as an alternative to "overview, zoom, details on demand".
- Degree of interest is a linear mix of three things:
  - a priori importance (API), such as degree or an attribute;
  - user interest (UI) from search terms or facets;
  - graph distance from the focus.
- API and UI interest is diffused to neighbours (factor δ between 0 and 1), because the interest function has many local maxima in graphs.
- Edge-specific "disinterest" values let some link types count as longer than others.
- The context subgraph is a connected subgraph of size at most S around the focus, found by greedy best-first search on DOI in O(S log S).
- Expansion: clicking a node brings in only the top N most interesting hidden neighbours. A further click brings N more. The authors warn that a naive expand can add hundreds of nodes and give an unreadable graph.
- They show the neighbour count on nodes. They highlight the top n hidden directions, with n small ("n<5") to avoid overloading the user with choices. Fading edges show that more connections exist.
- Search results list the size of each node's surrounding subgraph, with a cap. This stops users picking singleton nodes.
- Node size encodes DOI. Matching nodes get a halo. Layout is stress majorization, chosen for stability over plain gradient descent.
- Related work notes that contextual views "struggle with very high degree nodes". TreePlus is cited as the "plant a seed and watch it grow" model.
- The evidence is a demonstration on 300,000 legal-case nodes. I saw no controlled study of the top-N rule.

**Furnas (1986), "Generalized fisheye views".**
URL: https://cspages.ucalgary.ca/~saul/581/exer.eps/4furnas86.pdf [SNIPPET]. The DOI formula is interest = a priori importance minus distance. The same formula is restated in the van Ham & Perer text I read [FULL].

**TreePlus (Lee, Parr, Plaisant, Bederson et al., 2006, IEEE TVCG 12(6)).**
URL: https://www.cs.umd.edu/hcil/trs/2006-04/2006-04.htm [EXTRACT]:

- Start from a node and expand it. A single click previews adjacent nodes before expansion. Transitions are animated in steps.
- Controlled study with 28 participants and six tasks: the advantage over a traditional graph interface grew as density grew. 26 of 28 preferred TreePlus.
- I did not verify the dataset sizes in the extraction.

**Vizster (Heer & boyd, InfoVis 2005).**
URL: http://vis.stanford.edu/jheer/projects/vizster/ [SNIPPET]:

- Ego networks of one person and friends. Users expand by selecting nodes. Public and controlled studies support usability.

**Shneiderman's mantra and its critics.**

- Craft & Cairns (2005), "Beyond guidelines: what can we learn from the visual information seeking mantra?" [SNIPPET].
  - URL: https://ieeexplore.ieee.org/document/1509067/ and https://www.semanticscholar.org/paper/Beyond-guidelines:-what-can-we-learn-from-the-Craft-Cairns/d5021de6058689983e68f474cd06f58753d9d83a.
  - Their points: the mantra is widely cited but not empirically validated, and it is used prescriptively although Shneiderman called it descriptive.
- For graphs specifically, the van Ham & Perer text above is the critic [FULL].

**Focus plus context.** Cockburn, Karlson & Bederson (2008), ACM Computing Surveys 41(1).
URL: https://faculty.cc.gatech.edu/~stasko/7450/Papers/cockburn-surveys08.pdf [EXCERPT, partial]:

- It names four approaches: overview+detail, zooming, focus+context and cue-based (highlight or suppress).
- Fisheye distortion risks misreading the data and makes targets harder to hit.
- I did not read the empirical-comparison section.

**Ego-network layouts.** Ehlers et al. (2024), "Me! Me! Me! Me! A study and comparison of ego network representations", Computers & Graphics 125.
URL: https://www.cg.tuwien.ac.at/research/publications/2024/ehlers-2024-mmm/ehlers-2024-mmm-paper.pdf [EXCERPT]:

- Crowd-sourced study of straight-line, radial and layered node-link diagrams and matrices.
- Matrices were worse than all node-link variants.
- The three node-link variants showed no statistically detectable performance difference. The authors still concluded that layered is "most preferable" on experience and performance together.

**Which neighbours when a node has hundreds**

- **van Ham & Perer**: rank by DOI, top N per click, "+N more" on repeat click, degree badge, at most about 5 highlighted directions (above) [FULL].
- **Linkurious** [SNIPPET, https://doc.linkurious.com/user-manual/latest/page.html]: expanding a supernode opens a safeguard. The user picks edge types, neighbour categories and a maximum number of neighbours. The manual warns too many neighbours is unreadable.
- **Cambridge Intelligence**, "Graph visualization UX" (https://cambridge-intelligence.com/blog/designing-intuitive-data-experiences-with-graph-visualizations/) [EXTRACT]:
  - A hub that takes over the view is a "starburst".
  - Remedies: redesign the data model, limit expansion, group less important nodes, or remove the central node and show connections as glyphs.
- **Neo4j Browser**: cap of 100 new neighbours per expansion (above) [EXCERPT].

**History and breadcrumbs.**
Heer, Mackinlay, Stolte & Agrawala (2008), "Graphical histories for visualization", TVCG. URL: https://idl.cs.washington.edu/files/2008-GraphicalHistories-InfoVis.pdf [EXCERPT of abstract and conclusion]. It is a design-space analysis plus a Tableau design study with usage-log analysis, not a controlled study of graph navigation. I found no controlled evidence on breadcrumbs for graph exploration.

---

## 3. Reducing clutter

**Filtering and progressive display (vendors)**

- Cambridge Intelligence [EXTRACT, https://cambridge-intelligence.com/blog/visualize-large-networks/, https://cambridge-intelligence.com/blog/hairball-effect-in-graph-visualization/]:
  - Start with an empty or filtered chart and let users add data.
  - Remodel the data for the workflow.
  - Use centrality measures to pick what to show.
  - Use combos (groups that open, close and nest) that merge their links.
  - Adapt styling to zoom level.
- Tom Sawyer [SNIPPET, https://blog.tomsawyer.com/large-scale-graph-visualization]: reduce density by relevance, timestamp or weight; cluster and abstract; use levels of detail.
- Linkurious [SNIPPET]: filters, hide, and collapse a group into one node.
- yFiles docs [EXCERPT, https://docs.yfiles.com/yfiles-html/dguide/large_graph_performance/]:
  - Before optimising for large graphs, ask whether all elements must be shown at once.
  - Fold parts that are not the focus, group by shared properties, or hide subtrees.
  - Run a full layout rarely and prefer incremental layouts that move only a chosen subset. Both HierarchicalLayout and OrganicLayout can do this.
- yFiles folding [EXCERPT, https://docs.yfiles.com/yfiles-html/dguide/folding/]: a merging converter produces at most one edge between each pair of collapsed endpoints.

**Motif simplification.** Dunne & Shneiderman (2013), CHI. URL: https://www.cs.umd.edu/hcil/trs/2012-29/2012-29.pdf [EXCERPT]:

- Fans (a head node with leaf-only neighbours), connectors and cliques are replaced by compact glyphs.
- The authors say fans can dominate a diagram.
- The controlled study with 36 participants "demonstrates the effectiveness" of the technique. I did not see effect sizes.
- Symmetric glyph shapes work better than crescents when many edges connect.

**Edge bundling and when it hurts**

- McGee & Dingliana (2012), AVI. URL: https://dl.acm.org/doi/10.1145/2254556.2254670 [EXCERPT of abstract]. Bundling hurts path tracing in accuracy and time. It gives no clear accuracy gain for cluster connectivity but speeds it up.
- Bach et al., as reported in the 2025 CHI paper (https://dl.acm.org/doi/10.1145/3706598.3713444) [EXCERPT]: bundling did not hurt accuracy but slowed path tasks on small directed graphs.
- The CHI 2025 study found people follow false connections in tight bundles, and routing changes which nodes they assign to clusters.
- A Telea et al. user report (https://webspace.science.uu.nl/~telea001/uploads/PAPERS/Dagstuhl09/paper.pdf) [EXCERPT] found hierarchical edge bundles far better than node-link diagrams for compound call graphs of hundreds of nodes. That is a small study with five users and hierarchical data.
- Sources disagree on benefit, but all agree bundling hurts path tracing. Argument chains are path-like.

**Hiding weak links and label thinning**

- Venturini, Jacomy & Jensen (2019), "What do we see when we look at networks" [EXCERPT]: size nodes and their labels by degree and delete the smallest labels, because inspecting hundreds of nodes is "clearly not an option". URL: https://hal.science/hal-03432404v1/file/VenturiniJacomyJensen_WhatDoWeSee_PREPRINT.pdf.

**Evidence on grouping into super-nodes versus alternatives:** see section 4 (Saket versus Jianu).

---

## 4. Clusters and cluster-aware layout

**Community detection.** Traag, Waltman & van Eck (2019), "From Louvain to Leiden", Scientific Reports. URLs: https://www.nature.com/articles/s41598-019-41695-z and https://arxiv.org/html/1810.08473v3 [EXCERPT]:

- Louvain can return badly connected or even disconnected communities. In their tests up to 25% were badly connected and up to 16% disconnected.
- Leiden guarantees connected communities, is faster, and finds better partitions. The authors conclude it is "strongly preferable".
- Resolution parameter: higher means more, smaller communities in this paper's convention.
- Convention trap: the Gephi tutorial says the opposite for its UI ("lower to get more communities"). Source: https://jveerbeek.gitlab.io/gephi/docs/community.html [EXCERPT, third-party tutorial]. Check which convention your library uses.

**Colour versus enclosure.**

- Jianu, Rusu, Hu & Taggart (2014), TVCG 20(11) [EXCERPT of abstract and methods]. URL: https://city-ref-test.eprints-hosting.org/id/eprint/15379/1/paper_TVCG2014_groupinfo.pdf.
  - About 800 online subjects, 850-book network, 26 groups, four techniques: node colouring, GMap, BubbleSets, LineSets.
  - BubbleSets was best for group-membership tasks.
  - Encoding groups over a basic node-link diagram cost about 25% accuracy on network tasks.
  - GMap's prominent labels improved memorability.
- Saket, Simonetto, Kobourov & Börner (2014), TVCG [EXCERPT]. URL: https://cns.iu.edu/docs/publications/2014-saket-ieee.pdf.
  - Node, node-link and node-link-group (contiguous map regions) diagrams, 50 to 200 nodes, 4 to 7 groups.
  - Group regions did not hurt network tasks and helped group tasks. Group-task accuracy was about 95%. Overall accuracy was about 8% higher than node-link.
  - The authors attribute the conflict with Jianu to fragmented regions, colour choice and group labels in Jianu's setup. They stress guaranteed contiguity of regions.
- Verdict: enclosing clusters helps group-level questions if regions are contiguous. Colouring alone, without enclosure, is simpler but gave a penalty when it displaced other cues. The studies disagree; the main moderator is contiguity.

**Group-in-a-box.** Rodrigues, Milic-Frayling, Smith, Shneiderman & Hansen (2011). URL: https://www.cs.umd.edu/~ben/papers/Rodrigues2011Group.pdf [EXCERPT]:

- Treemap boxes sized by cluster size hold each cluster's sub-layout. The result gives well-separated sub-graphs.
- It is demonstrated on real and synthetic networks, not tested with users. The authors say readability criteria are still to come from observing users.

**ForceAtlas2 (Jacomy, Venturini, Heymann & Bastian 2014, PLoS ONE 9(6):e98679).**
URL: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4051631/ and https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0098679. I read the settings sections [FULL by grep].

- Designed for Gephi users' "scale-free, 10 to 10,000 nodes" networks. The authors call it engineering and usability, not theory.
- **LinLog mode**: makes clusters tighter, and the resulting placement corresponds to Newman's modularity. It converges slowly in some cases, and "switching from regular mode to LinLog mode needs a readjustment of the scaling parameter".
- **Scaling**: higher scaling gives a larger, more spread graph. They chose a single parameter on purpose.
- **Gravity**: stops disconnected components drifting away. Strong gravity makes a very compact, possibly biased layout.
- **Dissuade hubs**: divides attraction by degree. It pushes hubs outward and keeps authorities (high in-degree) central.
- **Prevent overlap**: gives "a more readable and aesthetically pleasing image" but adds considerable friction, so it converges slowly.
- **Edge weight influence**: 0 ignores weights; 1 is proportional; above 1 exaggerates.
- **Tolerance defaults** in Gephi: 0.1 under 5,000 nodes.
- The paper does not give numeric "make clusters visible" recipes beyond these descriptions, and it reports no human cluster-reading study.

**ForceAtlas2 in practice**

- Venturini, Jacomy & Jensen (2019) [EXCERPT]:
  - Clusters in a jazz network were less visible in Fruchterman–Reingold, more visible in default ForceAtlas2, and most visible with LinLog on and gravity set to 0.
  - They say a trial-and-error adjustment of settings is the key, not a catch-all configuration.
  - Layout-based clusters and Louvain modularity clusters correspond well on clear-cut graphs and poorly on ambiguous ones. They frame force layouts as conserving ambiguity.
- graphology-layout-forceatlas2 docs and source [EXCERPT, https://graphology.github.io/standard-library/layout-forceatlas2.html and https://raw.githubusercontent.com/graphology/graphology/master/src/layout-forceatlas2/index.js]:
  - Library defaults are `scalingRatio` 1, `gravity` 1, `linLogMode` false and `adjustSizes` false.
  - `inferSettings(graph)` returns `scalingRatio` 10, `gravity` 0.05, `strongGravityMode` true, `slowDown` 1 + ln(order), and Barnes–Hut on above 2,000 nodes.
- Gephi manual by J. Veerbeek [EXCERPT, third-party tutorial, https://jveerbeek.gitlab.io/gephi/docs/force_directed.html]:
  - Strong gravity is usually paired with a low gravity value such as 0.1; otherwise gravity is 0.01 to 0.5.
  - Switch on prevent-overlap after the layout has converged.
  - Changing settings can distort interpretation, so record them.

**How to space clusters apart (evidence ranked)**

1. LinLog with low or zero gravity, and re-tune scaling afterwards. Authors' guidance; no user study.
2. Higher scaling.
3. Lay out per cluster in boxes. Group-in-a-box gives guaranteed separation, with no user study.
4. Draw contiguous regions. Saket supports this.

**Cambridge Intelligence's own layouts.** Its combos are the vendor analogue of folding. See section 3.

---

## 5. Layout choice by graph kind

**Evidence that force-directed is right**

- Pohl, Schmitt & Diehl (2009), "Comparing the readability of graph layouts using eyetracking and task-oriented analysis". URL: https://www.st.uni-trier.de/diehl/pubs/cae09.pdf [EXCERPT]:
  - 36 subjects, random graphs of 10, 15 and 20 nodes with average degree 3.
  - Compared Fruchterman–Reingold, Sugiyama layered, and orthogonal layouts.
  - Force-directed was best for the path, group and subgraph tasks (tasks 2 to 4).
  - Layered was worst for "is there a path?": 58% correct against 93% for force-directed and 91% for orthogonal.
  - Edge crossings were confusing in the layered drawings.
  - Caveat: tiny random graphs, not DAGs or argument maps. The authors expect layered to be better for finding parents or following hierarchy, which they did not test.

**Evidence that layered is right**

- Ehlers et al. (2024), above: layered "most preferable" for ego networks, though no detectable performance difference among node-link variants [EXCERPT].
- Archambault et al. on cascade visualisation. URL: https://journals.sagepub.com/doi/10.1177/1473871615576758 [SNIPPET of abstract]. Hierarchical layouts were faster than force-directed for both animation and small multiples, and more accurate under animation. This is a spread-over-edges task, not argument reading.
- Gd14 poster, "Compared drawing styles for directed graphs". URL: https://mozart.diei.unipg.it/gd/gd2014/data/uploads/posters/gd14-poster-1.pdf [EXCERPT]:
  - 21 students; graphs of 77 to 122 vertices with density 2.5 to 3.5 (comparable to Lins).
  - Hierarchical drawings were among the fastest; overall error 0.199.
  - Overloaded orthogonal drawings were the most accurate (0.119 error).
  - Orthogonal was worst (0.369).
  - Matrices were slowest.
  - There was no force-directed condition, and it is a poster.
- Purchase (UML preference study): readers prefer arrows flowing in one direction, and information flow ranked third among preferences [EXCERPT, JGAA link above].
- Pohl and Ehlers disagree. The disagreement may stem from graph kind and task. Neither studied argument-shaped graphs.

**Practice**

- Argdown [EXCERPT]:
  - Its maps are directed, non-hierarchical graphs with boxes and arrows.
  - It recommends Graphviz layout (a layered layout) first, and treats Dagre as experimental.
  - For yEd it recommends hierarchical layout with polyline routing.
  - Its Dagre example uses `rankSep` 20 and `nodeSep` 10 for a compact map.
  - URL: https://argdown.org/guide/changing-the-graph-layout.html.
- dagre defaults [EXCERPT of source]: `ranksep` 50, `nodesep` 50, `edgesep` 20. URL: https://github.com/dagrejs/dagre/blob/master/lib/layout.ts. Dagre halves `ranksep` and doubles `minlen` when edge labels are present.
- ELK layered has separate node-node and between-layers spacing options. I did not retrieve its default numbers. [SNIPPET, https://github.com/kieler/elkjs/issues/19]
- yFiles: hierarchical and organic layouts both support incremental updates [EXCERPT].
- van Ham & Perer chose stress majorization for layout stability [FULL]. I found no study comparing stress majorization and ForceAtlas2 for readability.

**Aesthetics for path-following (relevant because argument chains are paths)**

- Ware et al.: keep paths straight; a crossing costs about as much as 38 degrees of bend [EXCERPT].
- Huang et al.: near-90-degree crossings [EXCERPT].
- Purchase: crossings first [EXCERPT].
- Tapered edges beat arrowheads for reading direction. Holten & van Wijk (2009), CHI. URL: https://dl.acm.org/doi/10.1145/1518701.1519054 [EXCERPT]:
  - Standard arrowheads performed poorly, probably from occlusion at high-degree vertices.
  - Tapered edges gave the best speed and accuracy among six single-cue representations.
  - A secondary compilation, GuidelineExplorer (https://arxiv.org/html/2406.05558v1), says tapering struggles on denser graphs [EXTRACT].

**Radial or concentric for one node's neighbourhood.** Ehlers et al. found radial about as good as straight-line and layered [EXCERPT]. Kialo uses a sunburst as a whole-discussion overview (section 8). No study I found shows radial beating layered for argument-shaped ego networks.

---

## 6. Nodes and labels

**Text size**

- Legge & Bigelow (2011), "Does print size matter for reading?", Journal of Vision 11(5):8. URL: https://jov.arvojournals.org/article.aspx?articleid=2191906 [EXCERPT]:
  - Reading is fastest above a critical print size of about 0.2 degrees x-height (range 0.15 to 0.3), and speed falls sharply below it.
  - At 40 cm that is about 9 pt for Times New Roman.
  - The fluent range runs to about 2 degrees.
- My arithmetic [I]: at laptop distance (40 cm) this means about 11–12 CSS px fonts. At desktop distance (60 cm) it means about 15–17 px for fluent reading. Short labels are scanned, not read fluently, so a floor of 12 px and a default of 13–14 px is a sensible compromise. This is not a published graph figure.
- The survey, Saket and Ware et al. say nothing about label size.

**Light versus dark backgrounds**

- Piepenbrock, Mayr & Buchner (2014), Human Factors. URL: https://pubmed.ncbi.nlm.nih.gov/25141597/ [EXCERPT of abstract]:
  - Dark text on a light background was read better than the reverse.
  - The advantage grows as characters get smaller (tested at 8, 10, 12 and 14 pt).
  - The authors say to avoid negative polarity especially for small fonts.
- This is about proofreading running text, not graph labels, and not mid-dark backgrounds. I found no study of mid-dark backgrounds.
- WCAG 2.2 [EXCERPT, https://www.w3.org/TR/WCAG22/]:
  - Text needs 4.5:1 contrast (3:1 for large text).
  - Graphical objects needed to understand the content need 3:1 against adjacent colours.
  - This is an accessibility standard, not a graph-readability study. It can conflict with the common practice of faint edges in dense graphs.

**Where to put text**

- Cambridge Intelligence [EXTRACT]: use smart truncation with tooltips, and avoid repeating words across nodes.
- Venturini et al.: size labels with node importance and drop the smallest [EXCERPT].
- Wong et al. (GreenArrow) [EXCERPT, https://perso.telecom-paristech.fr/eagan/media/papers/wong05.pdf]: draw a label as a line when text would be illegible, with a hand-tuned conservative threshold. Their study showed traditional layouts hard to beat when graphs and labels were small. It had six subjects in its third study.
- Schulze et al. (edge labels in layered drawings) [EXCERPT, https://exa.ai/library/publication/bg13t8vzbjr]:
  - Placing a label on its edge gives more compact diagrams.
  - Labels on edges need a solid background or fade.
  - Direction arrows on labels helped users navigate large diagrams.
- Khartabil et al. (2021) [EXCERPT, section 8 below]: experts want text in place; novices did fine with pop-ups.
- I found no study comparing text inside a node with text beside it, nor truncation versus wrapping.

**Level of detail**

- Sigma.js defaults [EXCERPT, https://www.sigmajs.org/reference/settings/]: `labelRenderedSizeThreshold` 6 px, `labelDensity` 1, `labelGridCellSize` 100 px. A GitHub issue shows the maintainers tuned these by eye: a cell size of 60 gave too many labels on zoom, and a large cell size distributed labels badly. See https://github.com/jacomyal/sigma.js/issues/1172.
- Cytoscape.js [EXCERPT, https://github.com/cytoscape/cytoscape.js/blob/master/documentation/md/performance.md]:
  - `min-zoomed-font-size` hides labels that would be too small to read.
  - `text-outline-width` and `text-background-opacity` add a halo or background. Backgrounds and outlines make labels costlier to render.
- Cambridge Intelligence: adaptive styling by zoom [EXTRACT].

**Node size and link style**

- No study gives a node-size range or a minimum gap between nodes.
- Targets: WCAG 2.5.8 asks for 24 by 24 CSS px pointer targets, or enough spacing. It exempts dense interactive data visualisations. URL: https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html [EXCERPT].
- Link thickness and opacity: I found no controlled evidence beyond Holten & van Wijk's tapered result.

**Colour**

- Okabe & Ito's colour-blind-safe palette (8 colours) is documented via package pages [SNIPPET, https://easystats.github.io/see/reference/palette_okabeito.html]. I did not retrieve hex values; I know them from memory and you should verify them.
- Distinguishable categorical colours: a viewer reliably separates only "six to twelve" categorical colour bins. The statement comes from Brandes et al.'s chapter and a lecture deck citing Munzner [SNIPPET, https://visvar.github.io/assets/pdf/brandes2019netvis.pdf].
- Small marks need larger colour differences. Szafir (2018), "Modeling color difference for visualization design", TVCG [EXCERPT]: perceived colour difference falls with mark size. A just-noticeable difference for a 2-degree scatterplot point was about three times larger than controlled-lab models predicted. Long thin marks are more discriminable than dots. URL: https://unc-visualab.org/papers/colordiff_vis2017.pdf.
- Cambridge Intelligence advises pairing colour with icons or text and testing with ColorBrewer colour-blind-safe palettes [EXTRACT].

---

## 7. Keeping the mental map

- **Archambault & Purchase (2012)**, "Mental map preservation helps user orientation in dynamic graphs", GD 2012. URL: https://scispace.com/pdf/mental-map-preservation-helps-user-orientation-in-dynamic-3kb9uymfsf.pdf [EXCERPT]:
  - Within-participant study with animation and small multiples.
  - Preserved mental map was significantly faster and had fewer errors for locating nodes and following paths (p<0.001).
  - Animation did not differ significantly from small multiples, but looked slightly better.
  - They note many earlier studies found no benefit, so benefit depends on the task.
- **Archambault, Purchase & Pinaud (2011)**, TVCG. URL: https://exa.ai/library/publication/0p8dq3dlzlg [EXCERPT of abstract]:
  - Small multiples were faster than animation overall.
  - Animation made fewer errors on added and removed element tasks.
  - Mental map preservation "had little influence" on errors and time.
  - This disagrees with the 2012 result; the tasks differ.
- **Bederson & Boltman (1999)**, InfoVis. URL: https://www.cs.umd.edu/projects/hcil/jazz/learn/papers/CS-TR-3964.pdf [EXCERPT]:
  - Animated viewpoint change (one-second transitions) helped users reconstruct the spatial layout, with no time penalty.
  - It did not help in the first (exploration) task, and participants reported no preference.
  - The authors say a full second may not be necessary, and they did not test duration.
- **Misue, Eades, Lai & Sugiyama (1995)** introduced the term. I did not read it [MEMORY, cited in https://www.sciencedirect.com/science/article/abs/pii/S107158191300102X].
- TreePlus used multi-step animated transitions [EXTRACT]. Telea's users said hiding and showing nodes disturbs bundled layouts less than node-link layouts [EXCERPT].
- Pinning: no study; practice only. yFiles recommends incremental layouts that move only selected nodes [EXCERPT].
- The survey gives a general review of mental-map work (Archambault & Purchase) [FULL].

---

## 8. Domain prior: how argument-mapping tools draw arguments

**Kialo**

- Help page [EXCERPT, https://support.kialo-edu.com/en/hc/discussion-minimap/]: thesis is blue, pro claims green, con claims red.
- Two minimap modes:
  - Tree Navigation is the default. It is recommended for new users because it limits visible claims to those directly related to the selected claim, and it highlights the path to the thesis.
  - Sunburst shows all claims and can confuse newer users in deep discussions.
- Release note [EXCERPT, https://releases.kialo-edu.com/2023/09/22/sunburst-navigator/]: the tree cannot show all claims, so a sunburst toggle shows everything. A black frame marks the visible region, nearby claims are drawn larger, and shading reflects votes.
- Kialo Edu user research [EXCERPT, https://blog.kialo-edu.com/wp-content/uploads/sites/3/2023/07/Kialo-Edu-Design-Review-and-User-Research.pdf]: a qualitative classroom review. Participants found the map intuitive. Some disliked the pro/con language.

**Argdown** [EXCERPT, https://argdown.org/guide/elements-of-an-argument-map.html]:

- Boxes and arrows. Statements are round boxes with a thick coloured border; arguments are coloured boxes; groups are grey boxes.
- Edge colours: green for support or entail, red for attack, contrary or contradiction, purple for undercut.
- Layered layouts (Graphviz) are recommended.

**Compendium and IBIS** (Conklin)

- Dialog Mapping chapter [EXCERPT, http://www.cognexus.org/ConklinCaseStudyChapter.pdf]:
  - IBIS has questions, ideas, and arguments (pro or con), with a grammar.
  - Maps grow left to right and have no size limit.
  - A map can contain sub-maps, so information is "chunked", usually at a new major question.
  - Groups without a moderator "end up with very large and poorly structured maps", so a "View Keeper" role was recommended.
- Field guide [EXCERPT, https://www.cognexus.org/IBIS_FieldguideVer12012010.pdf]: labels should be succinct but complete and make sense alone. Pros are green, cons red.
- Epstein note [EXCERPT, http://www.cognexus.org/IBIS-A_Tool_for_All_Reasons.pdf]: sentence case is easiest to read in large maps. A 20-node map was organised under four questions.

**Large-scale maps**

- Klein et al. (MIT Deliberatorium) [EXCERPT of OCR-garbled slides, https://icaart.scitevents.org/Documents/Previous_Invited_Speakers/2011/ICAART2011_Klein.pdf]: even moderate communities rapidly generate very large maps, and the open challenge is helping people "see the forest for the trees".
- Iandoli, Klein & Zollo [SNIPPET, https://www.kiphub.com/paper/61e50ad23c814a0c0cf6371d]: a 160-member community built about 5,000 posts in three weeks; users could explore comprehensively; moderation was needed.

**Study of argument visualisations.** Khartabil, Collins, Wells, Bach & Kennedy (2021), "Design and evaluation of visualization techniques to facilitate argument exploration", Computer Graphics Forum 40(6). URL: https://vialab.ca/wp-content/publication_media/pdf/dan2021.pdf [EXCERPT]:

- Compared sunburst with pop-up text, stacked boxes, and a focus+context icicle, in a controlled study with 21 participants and three tasks.
- Sunburst with pop-up was the best trade-off: highest accuracy, easiest overview. The icicle had the highest error (up to 71% on one task).
- Experts preferred stacked boxes because they need to read the text in place.
- The tool studies show hierarchical argument structure, not general graphs.

**Reviews.** A 2025 review of 45 argument-visualisation studies found Rationale (box-and-arrow) the most used tool, mostly small studies of learners, and no comparative evidence on layout. URL: https://exa.ai/library/publication/1d1kjj4jk58 [EXCERPT].

**Not retrieved.** I found no design statements or readability studies from DebateGraph, Argunet or OVA beyond descriptions of what they offer. OVA puts the source text beside the map and uses coloured boxes (https://sv-journal.org/2024-3/11/en.pdf [EXCERPT]).

**Pattern across the domain**

- Boxes with full sentences.
- Green/red colour for support and attack.
- Layered tree or left-to-right layouts.
- Sub-maps or a tree minimap to handle size.
- Tree-first for newcomers.
- Nobody shows thousands of claims in a force-directed hairball.

---

## Design rules these sources support

Strength tags: S = controlled study, V = vendor or practice, I = my inference.

**How much to show**

1. Do not open on the whole graph. Start from a chosen or searched claim and show its neighbourhood (S/V: van Ham & Perer demonstration, TreePlus study, Cambridge, Linkurious).
2. Target 30 to 50 nodes in the default view and cap at 100. 150 nodes is above the upper quartile of every size the surveyed studies used (S for the survey medians; the cap itself is I).
3. Keep links to about 2 per node in view, since the surveyed upper quartile is 2.4 and the current view is 3.1 (S/I).
4. Show an overview as 6 to 12 cluster blobs, not claims (V/I; colour-bin limit of 6 to 12).
5. Treat 200 nodes at 10% density as the outer edge of reasonable node-link use for detailed tasks. It is expert intuition, not a study result (S, weak).

**Exploration**

6. Rank hidden neighbours by degree of interest: importance plus relevance to the current search plus distance, with edge-type weights (S/V: van Ham & Perer).
7. Bring in 8 neighbours per click (range 5 to 10), repeating on each click, grouped by link type with counts and a "+k more" badge. No source gives N; this is I anchored in van Ham & Perer's top-N and Linkurious's safeguards.
8. Highlight at most 4 suggested directions at a time (S/V: "n<5").
9. Show the degree or hidden-neighbour count on every node (V: van Ham & Perer, Linkurious).
10. Offer an outline or tree list of a claim's neighbours alongside the graph (V: Kialo, Klein).

**Reducing clutter**

11. Hide `extracted-from` and `uses-concept` links by default and show document and concept membership as chips, colour, or enclosing regions (I from the fan-motif and starburst sources).
12. Provide type-filter toggles with counts. Support and attack are on by default (V).
13. Do not bundle edges, because bundling hurts path tracing (S).
14. Collapse into super-nodes with merged edges and a count (V: yFiles, Cambridge combos).

**Clusters**

15. Use Leiden, not Louvain, and tune resolution so 6 to 12 communities appear (S/I).
16. For the cluster overview, run ForceAtlas2 with LinLog and low gravity, and re-tune scaling after switching (V/authors: Jacomy 2014, Venturini 2019).
    - Start from graphology `inferSettings`: scaling 10, gravity 0.05.
    - Add an overlap pass last, because it slows convergence.
17. Enclose clusters with contiguous regions where group questions matter (S: Saket). Remember Jianu found a penalty when group encoding was fragmented or labelled heavily.

**Layout**

18. Use a layered layout (dagre or ELK, top-down) for the focused argument neighbourhood. Argdown, IBIS and Kialo converge on it and Ehlers favours layered for ego nets (S weak/V). Pohl found force-directed better on tiny random graphs, so evidence is mixed.
19. Keep force-directed for the cluster map, where direction matters less (V).
20. Minimise crossings on paths, prefer near-right-angle crossings and straight chains, because one crossing costs about as much as 38 degrees of bend (S: Ware, Huang).
21. Start spacing at dagre's defaults, ranksep 50 and nodesep 50 px, then adjust for label width (V). The compact Argdown example is 20 and 10.

**Nodes and labels**

22. Label font floor of 12 CSS px, default 13 to 14 px (I from Legge & Bigelow). Add a 2 to 3 px halo in the background colour (V: Cytoscape outline).
23. Show no more than 25 to 40 labels at the default zoom. Four labels per zoom level threshold is not evidence-based; reveal more with zoom using grid culling (V: sigma, Cambridge). This is arithmetic: at about 30 characters and 12 px, 150 labels cover roughly a third of a 1400 by 900 viewport, while 35 cover about 9% (I).
24. Wrap short headlines instead of truncating, and put full text in a side panel on selection (I from Cambridge's "smart truncation" and IBIS "succinct but complete").
25. Give nodes a hit target of at least 24 CSS px or the spacing exception (V: WCAG 2.5.8).
26. Keep label contrast at 4.5:1 or better and node and typed-edge contrast at 3:1 or better against the background (V: WCAG). Because positive polarity helps more as text shrinks, a mid-dark background argues for larger or bolder labels (S for text, I for graphs).
27. Use at most 6 to 8 hues, encode node kind (claim, document, concept) by shape or outline as well as colour, and use green, red and neutral lines for support, attack and other links (V: Kialo, Argdown, IBIS). Use larger colour differences for small marks (S: Szafir). Cambridge Intelligence suggests pairing colour with icons or text.
28. Avoid large arrowheads on hubs. Consider tapered edges or small heads, with layer direction carrying flow in the layered view (S: Holten & van Wijk; weaker on dense graphs).

**Mental map**

29. When expanding, keep existing nodes where they are and place only the new ones, and re-run a global layout only on user request (V: yFiles incremental layout; S: Archambault & Purchase 2012 for orientation tasks, with 2011 finding little effect on other tasks).
30. Animate transitions (S: Bederson & Boltman, a one-second transition helped spatial recall). No source gives a best duration. Keep a clickable trail of recent focus claims (breadcrumb); this rests on design practice only.

---

## What I could not find evidence for

- Any controlled threshold for **labelled** node-link diagrams, such as how many headlines can be read at once. Every size threshold above comes from tasks on abstract or short-label graphs.
- An empirical minimum gap between nodes or label boxes, a recommended node-size range, or an optimum for link thickness and opacity.
- A study of text inside a node versus beside it, of truncation versus wrapping, or of halos behind labels.
- Dark versus light background studies for graph labels, or anything about a mid-dark background. The polarity studies are for running text.
- A user study of ForceAtlas2 or LinLog showing that people see clusters better. The case is mathematical plus authors' judgement.
- A user study of Group-in-a-Box.
- A controlled test of the "top N per click" expansion rule, or any evidence for a best N.
- Controlled evidence on breadcrumbs or history for graph exploration, and on best animation duration.
- Readability studies for argument maps at scale (hundreds of claims). I found qualitative statements from Conklin, Klein and Kialo, plus Khartabil's 21-participant study of hierarchical arguments.
- Content I could not read in full: Huang et al.'s cognitive-load paper (abstract only), Craft & Cairns, Vizster, Furnas 1986, Holten 2011, the Linkurious and Tom Sawyer pages (summaries only), Misue et al., and Cowan's working-memory limit. DebateGraph, Rationale and Argunet have no design or readability statements I could retrieve.
- Items from memory to verify: the Okabe–Ito hex values, and the exact ELK default spacings.

---

## Appendix: the question as it was put to the helper

> Research task (web only; do not read or edit any local files). I am redesigning the display of "Lins", a small open-source explorer for graph databases that draws node-link diagrams in a browser. The graph it is being tuned on is an ARGUMENT GRAPH: about 2,300 claims taken from texts (each with a short headline of 3 to 7 words and a longer text), 35 source documents, 100 concepts, joined by typed directed links (supports, attacks, qualifies, is-a-part-of, uses-concept, extracted-from). Today its first view shows ~150 nodes and ~470 links as small dots with small labels, laid out by ForceAtlas2, and the owner's verdict is: barely legible, needs to be spaced out much more, the clustering needs work, and it shows too much at once and is overwhelming. He wants the user experience handled carefully.
>
> Your job: find what RESEARCH AND PROFESSIONAL PRACTICE say about making node-link diagrams readable and about exploring a large graph without overwhelming the viewer, with sources. I need findings concrete enough to turn into design rules and default numbers.
>
> Cover these, each with sources (papers, vendor guides, well-regarded practitioner posts):
>
> 1. HOW MUCH TO SHOW. Empirical limits of node-link readability: Yoghourdjian et al. 2018 "Exploring the limits of complexity: a survey of empirical studies on graph visualisation"; Ghoniem, Fekete & Castagliola (node-link vs matrix, sizes and densities where node-link fails); Huang/Eades/Purchase work on crossings and cognitive load; any guidance of the form "above N nodes or density D, do X instead". What numbers of nodes and links do studies and vendors treat as comfortable for one view?
>
> 2. EXPLORATION MODELS that avoid showing the whole graph: van Ham & Perer 2009 "Search, Show Context, Expand on Demand" (degree-of-interest); Furnas' degree of interest / fisheye; Shneiderman's mantra and its critics for graphs; "start with what you know, then grow" (Heer & boyd Vizster; Lee et al. TreePlus); ego-network / focus-plus-context views; breadcrumb/history; how tools decide WHICH neighbours to show when a node has hundreds (ranking, grouping by type, "show 10 more"). 
>
> 3. REDUCING CLUTTER: filtering by type; aggregation into super-nodes / "combos" / meta-nodes (Cambridge Intelligence's KeyLines/ReGraph guidance, yFiles, Linkurious, Tom Sawyer); edge bundling and when it hurts; collapsing leaves; hiding weak links; motif simplification (Dunne & Shneiderman).
>
> 4. CLUSTERS: community detection (Louvain, Leiden) followed by cluster-aware layout; group-in-a-box layouts (Rodrigues et al.); convex hulls / bubble sets; whether colouring by cluster or enclosing clusters works better; ForceAtlas2's own paper (Jacomy et al. 2014) on which settings make clusters separate visibly (LinLog mode, scaling, gravity, "dissuade hubs", prevent overlap) and what the authors recommend; how to space clusters apart.
>
> 5. LAYOUT CHOICE BY GRAPH KIND: when a force-directed layout is right and when a layered (Sugiyama / dagre / ELK) layout is better for directed, mostly tree-like structures such as argument maps; radial / concentric layouts for one node's neighbourhood; stress majorization; evidence on which layouts people read best for path-following and for seeing groups (Purchase's aesthetics studies: crossings, bends, edge length uniformity, symmetry, orthogonality).
>
> 6. NODES AND LABELS: evidence and practice on label legibility in graphs: minimum text size, text inside the node vs beside it, truncation vs wrapping, label collision avoidance, halos/backgrounds behind text, showing labels by importance and revealing more on zoom (level of detail / semantic zoom), node size ranges, minimum spacing between nodes, link thickness and arrowheads, contrast on dark vs light backgrounds (and on a mid-dark background specifically), colour palettes for categorical node types that are distinguishable (including for colour-blind viewers).
>
> 7. KEEPING THE MENTAL MAP when the view changes: incremental layouts that do not move what the viewer has already placed, pinning, animated transitions and their measured benefit (Archambault, Purchase et al. on mental map preservation).
>
> 8. DOMAIN PRIOR: how argument-mapping tools draw arguments (Kialo's tree and sunburst, Argdown's maps, DebateGraph, Rationale, Compendium/IBIS maps, Argunet, OVA/AIFdb): boxes with full sentences, pro/con colour, layered top-down layouts, and what their designers or studies say about readability and overload.
>
> Output: a structured report with one section per topic. Every claim carries the URL (or the paper's full citation and where you read it) and a mark of whether you read the source itself or a summary/snippet. Paraphrase; quote at most one short phrase per source. Where sources disagree, say so. End with a section "Design rules these sources support", each rule one sentence with a default number where one can be justified (for example nodes per view, minimum label size, minimum gap between nodes, neighbours brought in per expansion), and a section "What I could not find evidence for". Be thorough and honest about the strength of each finding: a vendor's rule of thumb is not a study.
