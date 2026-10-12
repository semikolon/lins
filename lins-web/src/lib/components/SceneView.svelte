<!--
  One thing and its ties: a node in the middle, what points at it in columns on the left, what
  it points at on the right, each column grouped by kind of link and counted.

  The scene and where its boxes go come from lib/trial/scene.ts, the rules the trial page drew
  twice; this is the browser's half of that page as a part of the main one. A press on a card
  opens the scene from that card. A press on a stub brings in more of what is waiting.
-->
<script lang="ts">
  import { tick, untrack } from 'svelte';
  import { schema, sceneData, view } from '$lib/stores/graph';
  import { showScene } from '$lib/explore';
  import { CARD_VARS } from '$lib/cardstyle';
  import { fillOf, fillPercent } from '$lib/view';
  import { STYLE, buildScene, layout, type Heights, type Layout, type Scene, type SceneGroup } from '$lib/trial/scene';
  import type { GraphNode } from '$lib/types';

  const ZOOM_MIN = 0.2;
  const ZOOM_MAX = 3;
  /** A scene never opens with its text smaller than this share of full size. */
  const HOME_ZOOM_MIN = 0.75;
  /** How many neighbours of each kind of link a scene opens with, and how many more a stub brings in. */
  const STEP = 8;
  const clamp = (value: number, low: number, high: number) => Math.min(high, Math.max(low, value));

  interface Camera {
    x: number;
    y: number;
    zoom: number;
  }

  const result = $derived($sceneData);
  const v = $derived($view);

  // How many neighbours each group shows where a stub was pressed, by the group's key (its side
  // and its kind of link). A new middle node starts from the first few of each again.
  let wanted = $state.raw<Record<string, number>>({});
  let wantedFor: number | undefined;
  $effect(() => {
    const centre = result?.center;
    if (centre !== wantedFor) {
      wantedFor = centre;
      wanted = {};
    }
  });

  /** Among neighbours equally tied to the others, the one with most left to do comes first. */
  function left(node: GraphNode): number {
    const fill = fillOf(node, v);
    return fill ? fill.of - fill.done : 0;
  }

  const built = $derived.by(() => {
    if (!result) return null;
    try {
      return buildScene(result, v, STEP, { perGroup: wanted, weight: left });
    } catch (e) {
      return { error: e instanceof Error ? e.message : String(e) };
    }
  });
  const scene = $derived(built && 'scene' in built ? built.scene : null);
  const facts = $derived(built && 'facts' in built ? built.facts : null);
  const sceneError = $derived(built && 'error' in built ? built.error : null);

  /** How much of each node on show is done, by its id as the scene writes it. */
  const shares = $derived.by(() => {
    const map = new Map<string, number>();
    for (const node of result?.nodes ?? []) {
      const share = fillPercent(fillOf(node, v));
      if (share !== null) map.set(String(node.id), share);
    }
    return map;
  });

  // ── Measuring, then placing ────────────────────────────────────────────────────────────

  let viewport = $state<HTMLDivElement | null>(null);
  let ruler = $state<HTMLDivElement | null>(null);
  let measured = $state.raw<{ scene: Scene; heights: Heights } | null>(null);
  let camera = $state.raw<Camera>({ x: 0, y: 0, zoom: 1 });
  let grabbing = $state(false);

  // Cards are measured before they are placed: the scene's cards are set once in a hidden
  // ruler at their real width, with the fonts loaded, and their heights are read from it.
  $effect(() => {
    const now = scene;
    const from = ruler;
    if (!now || !from) return;
    let live = true;
    void (async () => {
      const names = [now.focus, ...now.left.flatMap((g) => g.cards), ...now.right.flatMap((g) => g.cards)].map((n) => n.name).join(' ');
      const family = getComputedStyle(from).fontFamily;
      await Promise.all([
        document.fonts.load(`500 ${STYLE.font}px ${family}`, names),
        document.fonts.load(`600 ${STYLE.focus_font}px ${family}`, names),
      ]).catch(() => undefined);
      await tick();
      if (!live) return;
      const tall = (key: string) => from.querySelector<HTMLElement>(`[data-m="${key}"]`)?.offsetHeight ?? 0;
      const side = (name: 'left' | 'right') => now[name].map((g, gi) => g.cards.map((_, ci) => tall(`${name}:${gi}:${ci}`)));
      measured = { scene: now, heights: { focus: tall('focus'), left: side('left'), right: side('right') } };
    })();
    return () => {
      live = false;
    };
  });

  const placed = $derived(scene && measured && measured.scene === scene ? layout(scene, measured.heights) : null);

  function fitted(to: Layout, el: HTMLElement): Camera {
    const [left, top, right, bottom] = to.bounds;
    const zoom = clamp(Math.min(el.clientWidth / (right - left), el.clientHeight / (bottom - top)), ZOOM_MIN, 1);
    return { x: el.clientWidth / 2 - (zoom * (left + right)) / 2, y: el.clientHeight / 2 - (zoom * (top + bottom)) / 2, zoom };
  }

  /**
   * Where a scene opens: as wide as the pane when that keeps the text at three quarters of its
   * size or more, with the middle node half way down. The top and bottom of a tall side are
   * then out of view, a drag away.
   */
  function home(to: Layout, el: HTMLElement): Camera {
    const [left, , right] = to.bounds;
    const zoom = clamp(el.clientWidth / (right - left), HOME_ZOOM_MIN, 1);
    return { x: el.clientWidth / 2 - (zoom * (left + right)) / 2, y: el.clientHeight / 2, zoom };
  }

  // A scene opened from another node starts at home. One that only grew by a stub stays put.
  let homedFor: number | undefined;
  /** True once the person has moved or zoomed this scene: from then on it is theirs to place. */
  let moved = false;
  $effect(() => {
    const [to, el, centre] = [placed, viewport, result?.center];
    if (to && el && centre !== homedFor) {
      homedFor = centre;
      moved = false;
      untrack(() => (camera = home(to, el)));
    }
  });

  // The pane changes size when the panel beside it opens or the window is resized. A scene nobody
  // has moved is set at home again in the new size. The observer lives as long as the pane does.
  $effect(() => {
    const el = viewport;
    if (!el) return;
    const observer = new ResizeObserver(() => {
      const to = untrack(() => placed);
      if (to && !moved) camera = home(to, el);
    });
    observer.observe(el);
    return () => observer.disconnect();
  });

  function about(from: Camera, px: number, py: number, zoom: number): Camera {
    const to = clamp(zoom, ZOOM_MIN, ZOOM_MAX);
    const by = to / from.zoom;
    // The point of the scene under (px, py) stays there.
    return { x: px - by * (px - from.x), y: py - by * (py - from.y), zoom: to };
  }

  function onWheel(e: WheelEvent) {
    e.preventDefault();
    moved = true;
    const box = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const [px, py] = [e.clientX - box.left, e.clientY - box.top];
    if (e.ctrlKey || e.metaKey) {
      // A pinch arrives as a wheel with Ctrl held; ⌘ and the wheel zooms too.
      camera = about(camera, px, py, camera.zoom * Math.exp(-e.deltaY * (e.ctrlKey ? 0.01 : 0.005)));
    } else {
      camera = { ...camera, x: camera.x - e.deltaX, y: camera.y - e.deltaY };
    }
  }

  // The listener lives as long as the pane it is on: the effect takes it off when the pane goes.
  $effect(() => {
    const el = viewport;
    if (!el) return;
    el.addEventListener('wheel', onWheel, { passive: false });
    return () => el.removeEventListener('wheel', onWheel);
  });

  let drag: { pointer: number; x: number; y: number; moved: boolean } | null = null;
  let wasDragged = false;
  function onDown(e: PointerEvent) {
    if (e.button !== 0) return;
    drag = { pointer: e.pointerId, x: e.clientX, y: e.clientY, moved: false };
    wasDragged = false;
  }
  function onMove(e: PointerEvent) {
    if (!drag || e.pointerId !== drag.pointer) return;
    const [dx, dy] = [e.clientX - drag.x, e.clientY - drag.y];
    if (!drag.moved) {
      if (Math.hypot(dx, dy) < 4) return;
      drag.moved = true;
      moved = true;
      wasDragged = true;
      grabbing = true;
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    }
    drag.x = e.clientX;
    drag.y = e.clientY;
    camera = { ...camera, x: camera.x + dx, y: camera.y + dy };
  }
  function onUp() {
    drag = null;
    grabbing = false;
  }
  function pick(id: string) {
    if (!wasDragged) void showScene(Number(id));
  }
  /** Bring in more of one group: the stub's side and kind of link name it. */
  function more(side: number, kind: string) {
    if (wasDragged) return;
    const key = `${side}:${kind}`;
    wanted = { ...wanted, [key]: (wanted[key] ?? STEP) + STEP };
  }
  function place(to: Camera) {
    moved = true;
    camera = to;
  }

  const path = (p: [number, number][]) => `M${p[0][0]} ${p[0][1]}C${p[1][0]} ${p[1][1]} ${p[2][0]} ${p[2][1]} ${p[3][0]} ${p[3][1]}`;
  const head = (tip: [number, number], dir: number) => {
    const back = tip[0] - dir * STYLE.arrow_len;
    return `${tip[0]},${tip[1]} ${back},${tip[1] - STYLE.arrow_half} ${back},${tip[1] + STYLE.arrow_half}`;
  };
  const percent = (zoom: number) => `${Math.round(zoom * 100)}%`;
</script>

{#snippet card(id: string, name: string, color: string, focus: boolean)}
  {@const share = shares.get(id)}
  <span class="card" class:focus class:filled={share !== undefined} style="--kind:{color};{share !== undefined ? `--share:${share}%` : ''}">
    <span class="body"><span class="name">{name}</span></span>
  </span>
{/snippet}

{#snippet rulerSide(name: 'left' | 'right', groups: SceneGroup[])}
  {#each groups as group, gi (group.kind)}
    {#each group.cards as node, ci (node.id)}
      <div data-m="{name}:{gi}:{ci}">{@render card(node.id, node.name, node.color, false)}</div>
    {/each}
  {/each}
{/snippet}

<div class="scene" style={CARD_VARS}>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="viewport" class:grabbing bind:this={viewport} onpointerdown={onDown} onpointermove={onMove} onpointerup={onUp} onpointercancel={onUp}>
    {#if placed}
      <div class="world" style="transform:translate({camera.x}px,{camera.y}px) scale({camera.zoom})">
        <svg class="links" width="1" height="1" aria-hidden="true">
          {#each placed.links as link, i (i)}
            <path d={path(link.p)} stroke={link.color} stroke-opacity={link.arc ? STYLE.arc_alpha : STYLE.link_alpha} stroke-width={STYLE.link_w} fill="none" />
            <polygon points={head(link.tip, link.dir)} fill={link.color} fill-opacity={link.arc ? STYLE.arc_alpha : STYLE.link_alpha} />
          {/each}
        </svg>
        {#each placed.items as item, i (i)}
          {#if item.role === 'focus'}
            <div class="at" style="left:{item.x}px;top:{item.y}px">{@render card(item.id, item.text, item.color, true)}</div>
          {:else if item.role === 'card'}
            <button type="button" class="at pick" style="left:{item.x}px;top:{item.y}px" onclick={() => pick(item.id)}>{@render card(item.id, item.text, item.color, false)}</button>
          {:else if item.role === 'header'}
            <div class="at heading" style="left:{item.x}px;top:{item.y}px;color:{item.color}">{item.text}</div>
          {:else}
            <button type="button" class="at stub" style="left:{item.x}px;top:{item.y}px;--kind:{item.color}" title="Show {STEP} more of these" onclick={() => more(item.side, item.id)}>{item.text}</button>
          {/if}
        {/each}
      </div>
      <div class="zoom" role="group" aria-label="Zoom">
        <button type="button" title="Show the whole of it" onclick={() => viewport && placed && place(fitted(placed, viewport))}>Fit</button>
        <button type="button" title="Back to full size" onclick={() => viewport && place(about(camera, viewport.clientWidth / 2, viewport.clientHeight / 2, 1))}>{percent(camera.zoom)}</button>
      </div>
    {/if}
    {#if sceneError}
      <p class="say">{sceneError}</p>
    {:else if !placed}
      <p class="say">Loading…</p>
    {/if}
  </div>

  <footer>
    {#if facts && scene && result}
      <p>
        {facts.shown + 1} of {result.total_node_count.toLocaleString('en')} nodes shown · opened from “{scene.focus.name}” ·
        {facts.shown} of {facts.neighbours} neighbours loaded{facts.truncated ? ' (the node has more than were loaded)' : ''}
        {#if placed && (placed.arcs_left_out > 0 || facts.second_links > 0)}
          · not drawn: {placed.arcs_left_out} {placed.arcs_left_out === 1 ? 'link' : 'links'} between neighbours in different columns, {facts.second_links} second {facts.second_links === 1 ? 'link' : 'links'} to the middle node
        {/if}
      </p>
    {/if}
    <p class="quiet">Drag to move, pinch or ⌘ and scroll to zoom. Press a card to open the view from it, a “more” box to bring in more.</p>
  </footer>

  <div class="ruler" bind:this={ruler} aria-hidden="true">
    {#if scene}
      <div data-m="focus">{@render card(scene.focus.id, scene.focus.name, scene.focus.color, true)}</div>
      {@render rulerSide('left', scene.left)}
      {@render rulerSide('right', scene.right)}
    {/if}
  </div>
</div>

<style>
  .scene {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    position: relative;
    background: var(--canvas);
    font-family: var(--font-ui);
  }
  .viewport {
    flex: 1;
    min-height: 0;
    position: relative;
    overflow: hidden;
    background: var(--canvas);
    cursor: default;
    touch-action: none;
    user-select: none;
  }
  .viewport.grabbing {
    cursor: grabbing;
  }
  .say {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    padding: 24px;
    text-align: center;
    color: var(--ink-muted);
    pointer-events: none;
  }

  .world {
    position: absolute;
    left: 0;
    top: 0;
    transform-origin: 0 0;
  }
  .links {
    position: absolute;
    left: 0;
    top: 0;
    overflow: visible;
  }
  .at {
    position: absolute;
  }
  .pick {
    appearance: none;
    display: block;
    margin: 0;
    padding: 0;
    border: 0;
    background: none;
    font: inherit;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }

  .card {
    display: block;
    width: var(--card-w);
    background: var(--kind);
    border-radius: var(--radius);
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.28);
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
    background: linear-gradient(90deg, color-mix(in srgb, var(--kind) 30%, var(--card)) 0 var(--share), var(--card) var(--share) 100%);
  }
  .card .name {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: var(--max-lines);
    line-clamp: var(--max-lines);
    overflow: hidden;
    overflow-wrap: anywhere;
    font: var(--name);
    color: var(--ink);
  }
  .pick:hover .body {
    background: var(--card-hover);
  }
  .pick:hover .card.filled .body {
    background: linear-gradient(90deg, color-mix(in srgb, var(--kind) 36%, var(--card-hover)) 0 var(--share), var(--card-hover) var(--share) 100%);
  }
  .card.focus {
    width: var(--focus-w);
    outline: 2px solid var(--kind);
    outline-offset: 2px;
  }
  .card.focus .body {
    background: var(--card-focus);
  }
  .card.focus.filled .body {
    background: linear-gradient(90deg, color-mix(in srgb, var(--kind) 30%, var(--card-focus)) 0 var(--share), var(--card-focus) var(--share) 100%);
  }
  .card.focus .name {
    -webkit-line-clamp: var(--focus-max-lines);
    line-clamp: var(--focus-max-lines);
    font: var(--focus-name);
  }
  .heading {
    width: var(--card-w);
    height: var(--header-h);
    padding-left: 2px;
    font: 600 var(--small) / var(--header-h) var(--font-ui);
    white-space: nowrap;
  }
  .stub {
    box-sizing: border-box;
    width: var(--card-w);
    height: var(--stub-h);
    padding: 0;
    border: 1px solid color-mix(in srgb, var(--kind) 55%, transparent);
    border-radius: var(--radius);
    background: color-mix(in srgb, var(--kind) 7%, transparent);
    color: var(--ink-muted);
    font: 500 var(--small) / calc(var(--stub-h) - 2px) var(--font-ui);
    text-align: center;
    white-space: nowrap;
    cursor: pointer;
  }
  .stub:hover {
    color: var(--ink);
    background: color-mix(in srgb, var(--kind) 18%, transparent);
  }

  .zoom {
    position: absolute;
    right: 12px;
    top: 12px;
    display: inline-flex;
    font-size: 12px;
  }
  .zoom button {
    padding: 3px 9px;
    color: var(--ink);
    background: rgba(15, 20, 25, 0.5);
    border: 1px solid rgba(169, 179, 198, 0.3);
  }
  .zoom button + button {
    border-left: 0;
  }
  .zoom button:first-child {
    border-radius: var(--radius-sm) 0 0 var(--radius-sm);
  }
  .zoom button:last-child {
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    min-width: 52px;
  }
  .zoom button:hover {
    background: rgba(15, 20, 25, 0.75);
  }

  .ruler {
    position: absolute;
    left: 0;
    top: 0;
    width: 0;
    height: 0;
    overflow: hidden;
    visibility: hidden;
    pointer-events: none;
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
</style>
