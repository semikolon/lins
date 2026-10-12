<!--
  Cards in lanes: one of the view's arrangements.

  Where a card sits says something: its lane is the node it belongs to, or the value it carries,
  and its place in the lane is its rank by one number. So belonging needs no line, and no line is
  drawn until a card is pointed at; then its ties to the other cards on show are drawn, and the
  cards it is not tied to step back. A press on a card opens it with all its ties around it.

  The server decides which lanes there are, their order, and which cards each shows
  (lins-core, `lanes`); this draws what it is handed.
-->
<script lang="ts">
  import { untrack } from 'svelte';
  import { board, schema, view } from '$lib/stores/graph';
  import { LANE_STEP, arrangementsOf, showInLane, showScene } from '$lib/explore';
  import { CARD_VARS } from '$lib/cardstyle';
  import { fillOf, fillPercent, labelColor, nodeFullName, relColor, relWords } from '$lib/view';
  import type { GraphEdge, GraphNode, Lane } from '$lib/types';

  const b = $derived($board);
  const v = $derived($view);
  const labels = $derived($schema?.labels ?? []);
  const perLane = $derived((b ? arrangementsOf(v)[b.index]?.per_lane : null) ?? LANE_STEP);

  const colorOf = (node: GraphNode) => labelColor(node.labels[0] ?? '', v, labels);

  function laneName(lane: Lane): string {
    if (lane.node) return nodeFullName(lane.node, v);
    if (lane.value != null) return lane.value;
    return `no ${(b?.property ?? 'value').replaceAll('_', ' ')}`;
  }

  // ── The ties of the card pointed at ────────────────────────────────────────────────────

  /** The links among the cards on show, by each card they touch. */
  const tiesOf = $derived.by(() => {
    const map = new Map<number, GraphEdge[]>();
    for (const link of b?.links ?? []) {
      for (const end of [link.source_id, link.target_id]) {
        const list = map.get(end) ?? [];
        list.push(link);
        map.set(end, list);
      }
    }
    return map;
  });

  let sheet = $state<HTMLDivElement | null>(null);
  let pointed = $state<number | null>(null);
  let lines = $state.raw<{ d: string; color: string }[]>([]);

  /** The card pointed at and the cards it is tied to; nothing when no card is pointed at. */
  const lit = $derived.by(() => {
    if (pointed === null) return null;
    const ids = new Set<number>([pointed]);
    for (const link of tiesOf.get(pointed) ?? []) {
      ids.add(link.source_id);
      ids.add(link.target_id);
    }
    return ids;
  });

  /** What the pointed card's ties are, in words, for the line under the lanes. */
  const said = $derived.by(() => {
    if (pointed === null || !b) return '';
    const links = tiesOf.get(pointed) ?? [];
    if (links.length === 0) return 'No ties to the other cards on show. Press it to see everything it is tied to.';
    const kinds = new Map<string, number>();
    for (const link of links) kinds.set(link.relationship_type, (kinds.get(link.relationship_type) ?? 0) + 1);
    const parts = [...kinds].sort((x, y) => y[1] - x[1]).map(([kind, n]) => `${relWords(kind, v)} ${n}`);
    return `Tied to ${lit ? lit.size - 1 : 0} of the cards on show: ${parts.join(' · ')}. Press it to see everything it is tied to.`;
  });

  function point(id: number | null) {
    pointed = id;
    const el = sheet;
    if (id === null || !el) {
      lines = [];
      return;
    }
    const origin = el.getBoundingClientRect();
    const box = (card: number) => {
      const found = el.querySelector<HTMLElement>(`[data-card="${card}"]`);
      if (!found) return null;
      const r = found.getBoundingClientRect();
      return { left: r.left - origin.left, right: r.right - origin.left, mid: r.top - origin.top + r.height / 2 };
    };
    const from = box(id);
    if (!from) {
      lines = [];
      return;
    }
    const seen = new Set<string>();
    const drawn: { d: string; color: string }[] = [];
    for (const link of tiesOf.get(id) ?? []) {
      const other: number = link.source_id === id ? link.target_id : link.source_id;
      const key = `${other}:${link.relationship_type}`;
      if (other === id || seen.has(key)) continue;
      seen.add(key);
      const to = box(other);
      if (!to) continue;
      const color = relColor(link.relationship_type, v);
      if (Math.abs(to.left - from.left) < 4) {
        // The same lane: an arc out to the right of it and back.
        const out = Math.min(56, 18 + Math.abs(to.mid - from.mid) * 0.12);
        drawn.push({ d: `M${from.right} ${from.mid}C${from.right + out} ${from.mid} ${to.right + out} ${to.mid} ${to.right} ${to.mid}`, color });
      } else {
        const rightward = to.left > from.left;
        const [x1, x2] = rightward ? [from.right, to.left] : [from.left, to.right];
        const bend = (x2 - x1) / 2;
        drawn.push({ d: `M${x1} ${from.mid}C${x1 + bend} ${from.mid} ${x2 - bend} ${to.mid} ${x2} ${to.mid}`, color });
      }
    }
    lines = drawn;
  }

  // A new set of lanes has none of the old cards under the pointer.
  $effect(() => {
    void b;
    untrack(() => point(null));
  });
</script>

<div class="lanes" style={CARD_VARS}>
  {#if b && b.lanes.length > 0}
    <div class="scroll">
      <div class="sheet" bind:this={sheet}>
        <svg class="ties" aria-hidden="true">
          {#each lines as line, i (i)}
            <path d={line.d} stroke={line.color} />
          {/each}
        </svg>
        {#each b.lanes as lane (lane.key)}
          {@const share = fillPercent(lane.fill)}
          {@const rest = lane.total - lane.cards.length}
          <section class="lane" aria-label={laneName(lane)}>
            <header class="head">
              {#if lane.node}
                <button type="button" class="lane-name" title="Open this with everything it is tied to" onclick={() => lane.node && showScene(lane.node.id)}>{laneName(lane)}</button>
              {:else}
                <span class="lane-name">{laneName(lane)}</span>
              {/if}
              <span class="count">
                {rest > 0 ? `${lane.cards.length} of ${lane.total.toLocaleString('en')} shown` : `${lane.total.toLocaleString('en')} in all`}
              </span>
              {#if share !== null && lane.fill}
                <span class="bar" title="{lane.fill.done.toLocaleString('en')} of {lane.fill.of.toLocaleString('en')}"><i style="width:{share}%"></i></span>
              {/if}
            </header>
            {#each lane.cards as card (card.id)}
              {@const cardShare = fillPercent(fillOf(card, v))}
              <button
                type="button"
                class="card"
                class:filled={cardShare !== null}
                class:dim={lit !== null && !lit.has(card.id)}
                class:pointed={pointed === card.id}
                data-card={card.id}
                style="--kind:{colorOf(card)};{cardShare !== null ? `--share:${cardShare}%` : ''}"
                onmouseenter={() => point(card.id)}
                onmouseleave={() => point(null)}
                onfocus={() => point(card.id)}
                onblur={() => point(null)}
                onclick={() => showScene(card.id)}
              >
                <span class="body"><span class="name">{nodeFullName(card, v)}</span></span>
              </button>
            {/each}
            {#if rest > 0}
              <button type="button" class="stub" onclick={() => showInLane(lane.key, lane.cards.length + LANE_STEP)}>
                +{rest.toLocaleString('en')} more
              </button>
            {/if}
            {#if lane.cards.length > perLane}
              <button type="button" class="fewer" onclick={() => showInLane(lane.key, perLane)}>show the first {perLane} only</button>
            {/if}
            {#if lane.total === 0}
              <p class="none">nothing here</p>
            {/if}
          </section>
        {/each}
      </div>
    </div>
    <footer>
      <p>
        {b.shown_count.toLocaleString('en')} of {b.card_count.toLocaleString('en')} cards shown in {b.lanes.length}
        {b.lanes.length === 1 ? 'lane' : 'lanes'}{b.lanes_cut ? ' (more lanes than are drawn)' : ''} ·
        the graph holds {b.total_node_count.toLocaleString('en')} nodes
      </p>
      <p class="quiet">{pointed !== null ? said : 'Point at a card to see what it is tied to among the cards on show. Press one to open it with all its ties.'}</p>
    </footer>
  {:else if b}
    <p class="say">This arrangement has no lanes: nothing in the graph matches what it names.</p>
  {:else}
    <p class="say">Loading…</p>
  {/if}
</div>

<style>
  .lanes {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: var(--canvas);
    color: var(--ink);
    font-family: var(--font-ui);
  }
  .scroll {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  .sheet {
    position: relative;
    display: flex;
    align-items: flex-start;
    gap: 28px;
    width: max-content;
    min-width: 100%;
    padding: 22px 28px 40px;
  }
  .ties {
    position: absolute;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;
    overflow: visible;
    pointer-events: none;
    z-index: 2;
  }
  .ties path {
    fill: none;
    stroke-width: 2;
    stroke-opacity: 0.9;
  }

  .lane {
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: 232px;
    flex-shrink: 0;
  }
  .head {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 0 2px 8px;
    border-bottom: 1px solid rgba(169, 179, 198, 0.22);
    margin-bottom: 2px;
  }
  .lane-name {
    font: 600 14px/19px var(--font-ui);
    color: var(--ink);
    text-align: left;
    text-wrap: balance;
    overflow-wrap: anywhere;
  }
  button.lane-name:hover {
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .count {
    font-size: 11.5px;
    color: var(--ink-muted);
  }
  .bar {
    display: block;
    height: 3px;
    margin-top: 3px;
    border-radius: 2px;
    background: rgba(169, 179, 198, 0.2);
    overflow: hidden;
  }
  .bar i {
    display: block;
    height: 100%;
    border-radius: 2px;
    background: var(--ink-muted);
  }

  .card {
    display: block;
    width: 100%;
    padding: 0;
    text-align: left;
    background: var(--kind);
    border-radius: var(--radius);
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.28);
    transition: opacity 0.12s, box-shadow 0.1s;
  }
  .card .body {
    display: block;
    margin-left: var(--stripe);
    padding: var(--pad-y) var(--pad-x);
    background: var(--card);
    border-radius: 0 var(--radius) var(--radius) 0;
  }
  /* The done share as a faint wash of the card's own colour, from the left. */
  .card.filled .body {
    background: linear-gradient(
      90deg,
      color-mix(in srgb, var(--kind) 30%, var(--card)) 0 var(--share),
      var(--card) var(--share) 100%
    );
  }
  .card .name {
    display: block;
    overflow-wrap: anywhere;
    font: var(--name);
    color: var(--ink);
  }
  .card:hover,
  .card.pointed {
    box-shadow: 0 2px 14px color-mix(in srgb, var(--kind) 45%, transparent);
  }
  .card.dim {
    opacity: 0.35;
  }

  .stub,
  .fewer {
    width: 100%;
    color: var(--ink-muted);
    font: 500 var(--small) / 1 var(--font-ui);
    text-align: center;
  }
  .stub {
    height: var(--stub-h);
    border: 1px solid rgba(169, 179, 198, 0.4);
    border-radius: var(--radius);
    background: rgba(169, 179, 198, 0.07);
  }
  .stub:hover {
    color: var(--ink);
    background: rgba(169, 179, 198, 0.16);
  }
  .fewer {
    padding: 2px 0;
  }
  .fewer:hover {
    color: var(--ink);
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .none {
    font-size: var(--small);
    color: var(--ink-muted);
  }

  footer {
    padding: 7px 14px 9px;
    border-top: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-muted);
    font-size: 12.5px;
    line-height: 1.45;
  }
  footer p {
    margin: 0;
  }
  .quiet {
    color: var(--text-dim);
  }
  .say {
    flex: 1;
    display: grid;
    place-items: center;
    padding: 24px;
    text-align: center;
    color: var(--ink-muted);
  }
</style>
