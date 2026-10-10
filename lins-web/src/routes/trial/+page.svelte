<!--
  A trial: one scene, drawn two ways, side by side.

  The same node and neighbours, placed by the same rules with the same sizes and colours, drawn
  once by the browser (HTML cards, SVG links) and once by Rust (egui on a canvas, compiled to
  WebAssembly: lins-render). It exists to settle one decision of the display proposal by eye:
  what draws the cards. See docs/display_proposal_2026_10_10.md, § 7a and § 8.

  /trial                 the best-connected node of the default graph
  /trial?focus=<id>      this node
  /trial?graph=<name>    another graph

  The Rust half is built by scripts/build-render.sh. Without it the page still shows the
  browser's half and says what is missing.
-->
<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
  import { goto } from '$app/navigation';
  import { base } from '$app/paths';
  import { page } from '$app/state';
  import { fetchAround, fetchDefaultGraph, fetchGraphData, fetchGraphs, fetchView } from '$lib/api';
  import type { GraphView, ViewResult } from '$lib/types';
  import { STYLE, buildScene, largestDifference, layout, type Heights, type Layout, type Scene, type SceneGroup } from '$lib/trial/scene';

  const ZOOM_MIN = 0.2;
  const ZOOM_MAX = 3;
  /** A scene never opens with its text smaller than this share of full size. */
  const HOME_ZOOM_MIN = 0.75;
  const clamp = (v: number, low: number, high: number) => Math.min(high, Math.max(low, v));

  interface View {
    x: number;
    y: number;
    zoom: number;
  }

  // ── What is shown ──────────────────────────────────────────────────────────────────────

  let graph = $state<string | null>(null);
  let graphView = $state.raw<GraphView | null>(null);
  let result = $state.raw<ViewResult | null>(null);
  let failure = $state<string | null>(null);
  let budget = $state(8);

  const focusParam = $derived(page.url.searchParams.get('focus'));
  const graphParam = $derived(page.url.searchParams.get('graph'));

  let asked = 0;
  async function load(graphName: string | null, focus: string | null) {
    const mine = ++asked;
    try {
      const name = graphName ?? (await fetchDefaultGraph()) ?? (await fetchGraphs())[0]?.name;
      if (!name) throw new Error('The server has no graph to show.');
      const view = await fetchView(name);
      let id = focus === null ? NaN : Number(focus);
      if (!Number.isFinite(id)) {
        // No node named: open the best-connected node of what the graph shows first.
        const overview = await fetchGraphData(name);
        const links = new Map<number, number>();
        for (const edge of overview.edges) {
          links.set(edge.source_id, (links.get(edge.source_id) ?? 0) + 1);
          links.set(edge.target_id, (links.get(edge.target_id) ?? 0) + 1);
        }
        const best = [...links].sort((a, b) => b[1] - a[1] || a[0] - b[0])[0]?.[0] ?? overview.nodes[0]?.id;
        if (best === undefined) throw new Error('The graph is empty.');
        id = best;
      }
      const around = await fetchAround(name, id);
      if (mine !== asked) return;
      graph = name;
      graphView = view;
      result = around;
      failure = null;
    } catch (e) {
      if (mine === asked) failure = e instanceof Error ? e.message : String(e);
    }
  }

  $effect(() => {
    void load(graphParam, focusParam);
  });

  const built = $derived.by(() => {
    if (!result) return null;
    try {
      return buildScene(result, graphView, budget);
    } catch (e) {
      return { error: e instanceof Error ? e.message : String(e) };
    }
  });
  const scene = $derived(built && 'scene' in built ? built.scene : null);
  const facts = $derived(built && 'facts' in built ? built.facts : null);
  const sceneError = $derived(built && 'error' in built ? built.error : null);

  function open(id: string) {
    const params = new URLSearchParams({ focus: id });
    if (graphParam) params.set('graph', graphParam);
    void goto(`?${params}`, { keepFocus: true, noScroll: true });
  }

  // ── Drawn by the browser ───────────────────────────────────────────────────────────────

  let viewport = $state<HTMLDivElement | null>(null);
  let ruler = $state<HTMLDivElement | null>(null);
  let measured = $state.raw<{ scene: Scene; heights: Heights } | null>(null);
  let bview = $state.raw<View>({ x: 0, y: 0, zoom: 1 });
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

  function fitted(to: Layout, el: HTMLElement): View {
    const [left, top, right, bottom] = to.bounds;
    const zoom = clamp(Math.min(el.clientWidth / (right - left), el.clientHeight / (bottom - top)), ZOOM_MIN, 1);
    return { x: el.clientWidth / 2 - (zoom * (left + right)) / 2, y: el.clientHeight / 2 - (zoom * (top + bottom)) / 2, zoom };
  }

  /**
   * Where a scene opens: as wide as the pane when that keeps the text at three quarters of its
   * size or more, with the middle node half way down. The top and bottom of a tall side are
   * then out of view, a drag away. The same rule as the Rust drawing's.
   */
  function home(to: Layout, el: HTMLElement): View {
    const [left, , right] = to.bounds;
    const zoom = clamp(el.clientWidth / (right - left), HOME_ZOOM_MIN, 1);
    return { x: el.clientWidth / 2 - (zoom * (left + right)) / 2, y: el.clientHeight / 2, zoom };
  }

  let homedFor: Layout | null = null;
  $effect(() => {
    const [to, el] = [placed, viewport];
    if (to && el && to !== homedFor) {
      homedFor = to;
      untrack(() => moveBrowser(home(to, el)));
    }
  });

  function about(view: View, px: number, py: number, zoom: number): View {
    const to = clamp(zoom, ZOOM_MIN, ZOOM_MAX);
    const by = to / view.zoom;
    // The point of the scene under (px, py) stays there.
    return { x: px - by * (px - view.x), y: py - by * (py - view.y), zoom: to };
  }

  /** The browser's view was moved by hand: it leads, and the other drawing follows. */
  let browserLedAt = -Infinity;
  function moveBrowser(to: View) {
    bview = to;
    browserLedAt = performance.now();
    if (linked && rust) rust.set_view(to.x, to.y, to.zoom);
  }

  function onWheel(e: WheelEvent) {
    e.preventDefault();
    const box = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const [px, py] = [e.clientX - box.left, e.clientY - box.top];
    if (e.ctrlKey || e.metaKey) {
      // A pinch arrives as a wheel with Ctrl held; ⌘ and the wheel zooms too. The same two
      // rates as the Rust drawing uses.
      moveBrowser(about(bview, px, py, bview.zoom * Math.exp(-e.deltaY * (e.ctrlKey ? 0.01 : 0.005))));
    } else {
      moveBrowser({ ...bview, x: bview.x - e.deltaX, y: bview.y - e.deltaY });
    }
  }

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
      wasDragged = true;
      grabbing = true;
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    }
    drag.x = e.clientX;
    drag.y = e.clientY;
    moveBrowser({ ...bview, x: bview.x + dx, y: bview.y + dy });
  }
  function onUp() {
    drag = null;
    grabbing = false;
  }
  function pick(id: string) {
    if (!wasDragged) open(id);
  }

  const path = (p: [number, number][]) => `M${p[0][0]} ${p[0][1]}C${p[1][0]} ${p[1][1]} ${p[2][0]} ${p[2][1]} ${p[3][0]} ${p[3][1]}`;
  const head = (tip: [number, number], dir: number) => {
    const back = tip[0] - dir * STYLE.arrow_len;
    return `${tip[0]},${tip[1]} ${back},${tip[1] - STYLE.arrow_half} ${back},${tip[1] + STYLE.arrow_half}`;
  };

  // ── Drawn by Rust ──────────────────────────────────────────────────────────────────────

  interface RustStats {
    frames: number;
    frame_ms: number;
    zoom: number;
    text_zoom: number;
    cut: number;
    own_fonts: boolean;
  }
  /* eslint-disable @typescript-eslint/no-explicit-any */
  let canvas = $state<HTMLCanvasElement | null>(null);
  let rust = $state.raw<any>(null);
  let rustModule = $state.raw<any>(null);
  let rustState = $state<'loading' | 'ready' | 'absent' | 'failed'>('loading');
  let rustNote = $state('');
  let rustStats = $state.raw<RustStats | null>(null);
  let rustView = $state.raw<View>({ x: 0, y: 0, zoom: 1 });
  let linked = $state(true);
  let textMode = $state(1);
  let timing = $state(false);

  onMount(() => {
    let handle: any = null;
    let gone = false;
    let frame = 0;
    let count = 0;
    let last: View = { x: NaN, y: NaN, zoom: NaN };

    void (async () => {
      let mod: any;
      try {
        mod = await import(/* @vite-ignore */ `${base}/render/lins_render.js`);
      } catch {
        rustState = 'absent';
        return;
      }
      try {
        await mod.default();
        handle = new mod.WebHandle();
        await handle.start(canvas);
        if (gone) return handle.destroy();
        handle.on_pick((id: string) => open(id));
        rustModule = mod;
        rust = handle;
        rustState = 'ready';
        // A canvas holds no text a test can read. So what each half is showing is put where
        // a test, or anyone at the console, can ask for it as data.
        Object.assign(window, {
          linsTrial: { rust: () => JSON.parse(handle.placed()), browser: () => placed, stats: () => JSON.parse(handle.stats()) },
        });
      } catch (e) {
        rustState = 'failed';
        rustNote = e instanceof Error ? e.message : String(e);
      }
    })();

    const each = () => {
      frame = requestAnimationFrame(each);
      if (!rust) return;
      if (rust.has_panicked()) {
        rustState = 'failed';
        rustNote = rust.panic_message() ?? 'The Rust drawing stopped.';
        rust = null;
        return;
      }
      const [x, y, zoom] = rust.view() as Float32Array;
      if (x !== last.x || y !== last.y || zoom !== last.zoom) {
        last = { x, y, zoom };
        rustView = last;
        // The Rust drawing was moved by hand: the browser's follows, unless it is the one
        // being moved, in which case this is only the echo of its own lead.
        if (linked && !timing && performance.now() - browserLedAt > 250) bview = last;
      }
      if (count++ % 10 === 0) rustStats = JSON.parse(rust.stats());
    };
    frame = requestAnimationFrame(each);

    return () => {
      gone = true;
      cancelAnimationFrame(frame);
      handle?.destroy();
      delete (window as any).linsTrial;
    };
  });

  $effect(() => {
    if (!rust || !scene) return;
    try {
      rust.set_scene(JSON.stringify(scene));
    } catch (e) {
      rustNote = `The Rust drawing did not take the scene: ${e}`;
    }
  });
  $effect(() => {
    rust?.set_text_mode(textMode);
  });

  // The two drawings place by the same rules. Checked, not assumed: the Rust rules are asked
  // to place this scene with the heights the browser measured, and the answers are compared.
  const rules = $derived.by(() => {
    if (!rustModule || !scene || !measured || measured.scene !== scene || !placed) return null;
    try {
      const theirs = JSON.parse(rustModule.layout_with_heights(JSON.stringify(scene), JSON.stringify(measured.heights))) as Layout;
      return largestDifference(placed, theirs);
    } catch (e) {
      return { px: Infinity, where: String(e) };
    }
  });

  // ── Both ───────────────────────────────────────────────────────────────────────────────

  function zoomTo(zoom: number) {
    if (!viewport) return;
    const [cx, cy] = [viewport.clientWidth / 2, viewport.clientHeight / 2];
    moveBrowser(about(bview, cx, cy, zoom));
    if (!linked && rust) {
      const to = about(rustView, cx, cy, zoom);
      rust.set_view(to.x, to.y, to.zoom);
    }
  }
  function fitBoth() {
    if (placed && viewport) moveBrowser(fitted(placed, viewport));
    if (!linked) rust?.fit();
  }

  interface Timing {
    who: string;
    frames: number;
    median: number;
    p95: number;
    worst: number;
    late: number;
    cpu: number | null;
    note: string;
  }
  let timings = $state.raw<Timing[]>([]);

  /** The next frame's start time, or null when the page is not being shown and no frame comes. */
  const nextFrame = () =>
    new Promise<number | null>((done) => {
      const giveUp = setTimeout(() => done(null), 600);
      requestAnimationFrame((t) => {
        clearTimeout(giveUp);
        done(t);
      });
    });
  const at = (sorted: number[], share: number) => sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * share))] ?? 0;

  /**
   * Zoom out, in and back along one fixed path, a step a frame, and write down how long each
   * frame took to come. One drawing at a time, so neither pays for the other.
   */
  async function timeAZoom() {
    if (!placed || !viewport || timing) return;
    timing = true;
    timings = [];
    const el = viewport;
    const viewBefore = bview;
    const rest = fitted(placed, el);
    const [left, top, right, bottom] = placed.bounds;
    const [mx, my] = [(left + right) / 2, (top + bottom) / 2];
    const steps = 180;
    const along = (i: number): View => {
      const u = i / steps;
      const zoom = clamp(rest.zoom * Math.exp(Math.sin(u * 2 * Math.PI) * Math.log(2.5)), ZOOM_MIN, ZOOM_MAX);
      return { x: el.clientWidth / 2 - zoom * mx + Math.sin(u * 4 * Math.PI) * 40, y: el.clientHeight / 2 - zoom * my, zoom };
    };
    const modeBefore = textMode;
    const runs: { who: string; mode: number | null }[] = [{ who: 'The browser', mode: null }];
    if (rust) {
      runs.push({ who: 'Rust, text stretched', mode: 0 }, { who: 'Rust, sharp when it rests', mode: 1 }, { who: 'Rust, sharp at every frame', mode: 2 });
    }
    const done: Timing[] = [];
    for (const run of runs) {
      const apply = (v: View) => (run.mode === null ? (bview = v) : rust.set_view(v.x, v.y, v.zoom));
      if (run.mode !== null) rust.set_text_mode(run.mode);
      apply(along(0));
      await nextFrame();
      let before = await nextFrame();
      const gaps: number[] = [];
      const cpu: number[] = [];
      for (let i = 1; i <= steps && before !== null; i++) {
        apply(along(i));
        const now = await nextFrame();
        if (now === null) break;
        gaps.push(now - before);
        before = now;
        if (run.mode !== null) cpu.push((JSON.parse(rust.stats()) as RustStats).frame_ms);
      }
      const sorted = [...gaps].sort((a, b) => a - b);
      done.push({
        who: run.who,
        frames: gaps.length,
        median: at(sorted, 0.5),
        p95: at(sorted, 0.95),
        worst: sorted[sorted.length - 1] ?? 0,
        late: gaps.filter((g) => g > 1.5 * at(sorted, 0.5)).length,
        cpu: cpu.length > 0 ? at([...cpu].sort((a, b) => a - b), 0.5) : null,
        note: gaps.length < steps ? 'stopped: no frames came, the page was not being shown' : '',
      });
      timings = [...done];
      apply(viewBefore);
    }
    rust?.set_text_mode(modeBefore);
    timing = false;
    moveBrowser(viewBefore);
    if (!linked) rust?.set_view(rustView.x, rustView.y, rustView.zoom);
  }

  const vars = [
    `--canvas:${STYLE.canvas}`,
    `--card:${STYLE.card}`,
    `--card-hover:${STYLE.card_hover}`,
    `--card-focus:${STYLE.card_focus}`,
    `--ink:${STYLE.text}`,
    `--ink-muted:${STYLE.muted}`,
    `--card-w:${STYLE.card_w}px`,
    `--focus-w:${STYLE.focus_w}px`,
    `--pad-x:${STYLE.pad_x}px`,
    `--pad-y:${STYLE.pad_y}px`,
    `--radius:${STYLE.radius}px`,
    `--stripe:${STYLE.stripe}px`,
    `--name:500 ${STYLE.font}px/${STYLE.line}px var(--font-ui)`,
    `--focus-name:600 ${STYLE.focus_font}px/${STYLE.focus_line}px var(--font-ui)`,
    `--small:${STYLE.small_font}px`,
    `--max-lines:${STYLE.max_lines}`,
    `--focus-max-lines:${STYLE.max_lines + 2}`,
    `--header-h:${STYLE.header_h}px`,
    `--stub-h:${STYLE.stub_h}px`,
  ].join(';');

  const percent = (zoom: number) => `${Math.round(zoom * 100)}%`;
  const ms = (v: number) => v.toFixed(1);
  const budgets: [string, number][] = [
    ['8', 8],
    ['24', 24],
    ['40', 40],
    ['all', 10000],
  ];
  const zooms: [string, number][] = [
    ['50%', 0.5],
    ['100%', 1],
    ['200%', 2],
  ];
</script>

<svelte:head><title>Lins · one scene, drawn two ways</title></svelte:head>

{#snippet card(name: string, color: string, focus: boolean)}
  <span class="card" class:focus style="--kind:{color}">
    <span class="body"><span class="name">{name}</span></span>
  </span>
{/snippet}

{#snippet rulerSide(name: 'left' | 'right', groups: SceneGroup[])}
  {#each groups as group, gi (group.kind)}
    {#each group.cards as node, ci (node.id)}
      <div data-m="{name}:{gi}:{ci}">{@render card(node.name, node.color, false)}</div>
    {/each}
  {/each}
{/snippet}

<div class="trial" style={vars}>
  <header>
    <div class="title">
      <strong>One scene, drawn two ways</strong>
      <a href="{base}/">Lins</a>
    </div>
    <div class="controls">
      <span class="set" role="group" aria-label="Neighbours shown">
        <em>Neighbours</em>
        {#each budgets as [label, n] (n)}
          <button type="button" class:on={budget === n} onclick={() => (budget = n)}>{label}</button>
        {/each}
      </span>
      <span class="set" role="group" aria-label="Zoom">
        <button type="button" onclick={fitBoth}>Fit</button>
        {#each zooms as [label, z] (z)}
          <button type="button" onclick={() => zoomTo(z)}>{label}</button>
        {/each}
      </span>
      <label><input type="checkbox" bind:checked={linked} /> Move together</label>
      <label>
        Rust text while zooming
        <select bind:value={textMode}>
          <option value={0}>stretched</option>
          <option value={1}>sharp when it rests</option>
          <option value={2}>sharp at every frame</option>
        </select>
      </label>
      <button type="button" class="plain" onclick={timeAZoom} disabled={timing || !placed}>{timing ? 'Timing…' : 'Time a zoom'}</button>
    </div>
  </header>

  <main class="arms">
    <section class="arm">
      <div class="tag"><strong>Drawn by the browser</strong> HTML cards, SVG links · {percent(bview.zoom)}</div>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="viewport" class:grabbing bind:this={viewport} onpointerdown={onDown} onpointermove={onMove} onpointerup={onUp} onpointercancel={onUp}>
        {#if placed}
          <div class="world" style="transform:translate({bview.x}px,{bview.y}px) scale({bview.zoom})">
            <svg class="links" width="1" height="1" aria-hidden="true">
              {#each placed.links as link, i (i)}
                <path d={path(link.p)} stroke={link.color} stroke-opacity={link.arc ? STYLE.arc_alpha : STYLE.link_alpha} stroke-width={STYLE.link_w} fill="none" />
                <polygon points={head(link.tip, link.dir)} fill={link.color} fill-opacity={link.arc ? STYLE.arc_alpha : STYLE.link_alpha} />
              {/each}
            </svg>
            {#each placed.items as item, i (i)}
              {#if item.role === 'focus'}
                <div class="at" style="left:{item.x}px;top:{item.y}px">{@render card(item.text, item.color, true)}</div>
              {:else if item.role === 'card'}
                <button type="button" class="at pick" style="left:{item.x}px;top:{item.y}px" onclick={() => pick(item.id)}>{@render card(item.text, item.color, false)}</button>
              {:else if item.role === 'header'}
                <div class="at heading" style="left:{item.x}px;top:{item.y}px;color:{item.color}">{item.text}</div>
              {:else}
                <div class="at stub" style="left:{item.x}px;top:{item.y}px;--kind:{item.color}">{item.text}</div>
              {/if}
            {/each}
          </div>
        {/if}
        {#if failure || sceneError}
          <p class="say">{failure ?? sceneError}</p>
        {:else if !placed}
          <p class="say">Loading…</p>
        {/if}
      </div>
    </section>

    <section class="arm">
      <div class="tag">
        <strong>Drawn by Rust</strong> egui on a canvas, WebAssembly
        {#if rustStats}
          · {percent(rustStats.zoom)} · text set at {percent(rustStats.text_zoom)} · {ms(rustStats.frame_ms)} ms a frame
        {/if}
      </div>
      <div class="viewport">
        <canvas bind:this={canvas}></canvas>
        {#if rustState === 'absent'}
          <p class="say">The Rust drawing is not built. Run <code>scripts/build-render.sh</code>, then build the page again.</p>
        {:else if rustState === 'failed'}
          <p class="say">The Rust drawing stopped: {rustNote}</p>
        {:else if rustState === 'loading'}
          <p class="say">Loading the Rust drawing…</p>
        {/if}
      </div>
    </section>
  </main>

  <footer>
    {#if facts && scene && result}
      <p>
        {facts.shown + 1} of {result.total_node_count.toLocaleString('en')} nodes shown · opened from “{scene.focus.name}”{graph ? ` in ${graph}` : ''} ·
        {facts.shown} of {facts.neighbours} neighbours{facts.truncated ? ' (the server cut the neighbourhood short)' : ''}
        {#if placed && (placed.arcs_left_out > 0 || facts.second_links > 0)}
          · not drawn: {placed.arcs_left_out} {placed.arcs_left_out === 1 ? 'link' : 'links'} between neighbours in different columns, {facts.second_links} second {facts.second_links === 1 ? 'link' : 'links'} to the middle node
        {/if}
      </p>
    {/if}
    <p class="quiet">
      Both halves read one list of sizes and colours and place by one set of rules; each measures its own text.
      {#if rules}
        {rules.px < 0.01 ? 'Checked on this scene: given the same text heights, the two sets of rules place every box within a hundredth of a pixel.' : `The two sets of rules disagree by ${rules.px} px at ${rules.where}.`}
      {/if}
      {#if rustStats && !rustStats.own_fonts}The Rust half was built without the page's font and is drawing with egui's own.{/if}
      {#if rustStats && rustStats.cut > 0}{rustStats.cut} names did not fit in the Rust half and were cut.{/if}
      {#if rustNote && rustState === 'ready'}{rustNote}{/if}
      Drag to move, pinch or ⌘ and scroll to zoom, click a card to open the scene from it.
    </p>
    {#if timings.length > 0}
      <table>
        <thead>
          <tr><th>One zoom out, in and back, 180 steps</th><th>frames</th><th>middle frame</th><th>19 in 20 within</th><th>slowest</th><th>late</th><th>Rust's own work</th></tr>
        </thead>
        <tbody>
          {#each timings as t (t.who)}
            <tr>
              <td>{t.who}{t.note ? ` (${t.note})` : ''}</td>
              <td>{t.frames}</td>
              <td>{ms(t.median)} ms</td>
              <td>{ms(t.p95)} ms</td>
              <td>{ms(t.worst)} ms</td>
              <td>{t.late}</td>
              <td>{t.cpu === null ? '' : `${ms(t.cpu)} ms`}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      <p class="quiet">
        Time between one frame and the next, with {facts ? facts.shown + 1 : 0} cards and {placed ? placed.links.length : 0} links. A frame is late when it
        took more than one and a half middle frames. Counted on the page's own thread, so work the browser does elsewhere does not show; the last column is
        what the Rust drawing itself spent on a frame.
      </p>
    {/if}
  </footer>

  <div class="ruler" bind:this={ruler} aria-hidden="true">
    {#if scene}
      <div data-m="focus">{@render card(scene.focus.name, scene.focus.color, true)}</div>
      {@render rulerSide('left', scene.left)}
      {@render rulerSide('right', scene.right)}
    {/if}
  </div>
</div>

<style>
  .trial {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    position: relative;
    background: var(--bg);
    color: var(--text);
    font-family: var(--font-ui);
    font-size: 12.5px;
  }
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 18px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }
  .title {
    display: flex;
    align-items: baseline;
    gap: 10px;
    font-size: 13.5px;
  }
  .title a {
    color: var(--text-muted);
    font-size: 12px;
  }
  .controls {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 14px;
    color: var(--text-muted);
  }
  .controls label {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .set {
    display: inline-flex;
    align-items: center;
  }
  .set em {
    font-style: normal;
    margin-right: 6px;
  }
  .set button,
  .plain,
  select {
    font: inherit;
    color: var(--text);
    background: var(--bg);
    border: 1px solid var(--border);
    padding: 3px 9px;
    cursor: pointer;
  }
  .set button + button {
    border-left: 0;
  }
  .set button:first-of-type {
    border-radius: var(--radius-sm) 0 0 var(--radius-sm);
  }
  .set button:last-of-type {
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
  }
  .set button.on {
    background: var(--primary);
    border-color: var(--primary);
  }
  .plain,
  select {
    border-radius: var(--radius-sm);
  }
  .plain:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .arms {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-rows: 1fr 1fr;
    gap: 1px;
    background: var(--border);
  }
  @media (min-width: 1100px) {
    .arms {
      grid-template-rows: 1fr;
      grid-template-columns: 1fr 1fr;
    }
  }
  .arm {
    position: relative;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: var(--canvas);
  }
  .tag {
    padding: 4px 12px;
    background: rgba(15, 20, 25, 0.45);
    color: var(--text-muted);
    font-size: 11.5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .tag strong {
    color: var(--text);
    font-weight: 600;
    margin-right: 4px;
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
  canvas {
    display: block;
    width: 100%;
    height: 100%;
    outline: none;
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
  .say code {
    font-family: var(--font-mono);
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
  .card.focus {
    width: var(--focus-w);
    outline: 2px solid var(--kind);
    outline-offset: 2px;
  }
  .card.focus .body {
    background: var(--card-focus);
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
    border: 1px solid color-mix(in srgb, var(--kind) 55%, transparent);
    border-radius: var(--radius);
    background: color-mix(in srgb, var(--kind) 7%, transparent);
    color: var(--ink-muted);
    font: 500 var(--small) / calc(var(--stub-h) - 2px) var(--font-ui);
    text-align: center;
    white-space: nowrap;
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
    padding: 7px 12px 9px;
    border-top: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-muted);
    line-height: 1.45;
  }
  footer p {
    margin: 0;
  }
  footer p + p {
    margin-top: 2px;
  }
  .quiet {
    color: var(--text-dim);
  }
  table {
    margin-top: 8px;
    border-collapse: collapse;
    font-variant-numeric: tabular-nums;
  }
  th,
  td {
    padding: 2px 14px 2px 0;
    text-align: right;
    font-weight: 500;
  }
  th:first-child,
  td:first-child {
    text-align: left;
  }
  th {
    color: var(--text-dim);
    font-weight: 500;
  }
  td {
    color: var(--text);
  }
</style>
