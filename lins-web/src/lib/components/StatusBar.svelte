<script lang="ts">
  import { currentGraph, graphData, isLoading, showAll, view, viewInfo } from '$lib/stores/graph';
  import { setShowAll } from '$lib/explore';
  import { hiddenSummary } from '$lib/view';
  import type { GraphView, ViewInfo } from '$lib/types';

  let info = $state<ViewInfo | null>(null);
  let all = $state(false);
  let currentView = $state<GraphView | null>(null);
  viewInfo.subscribe((v) => { info = v; });
  showAll.subscribe((a) => { all = a; });
  view.subscribe((v) => { currentView = v; });

  const MODE_WORDS: Record<string, string> = {
    all: 'the whole graph',
    overview: 'its best-connected nodes',
    first: 'where this view opens',
    around: 'one neighbourhood',
    query: 'a query',
  };

  function shown(): string {
    if (!info) return `${nodeCount} nodes, ${edgeCount} edges`;
    const of = info.truncated || info.hidden > 0 ? ` of ${info.total.toLocaleString('en')}` : '';
    return `${nodeCount.toLocaleString('en')}${of} nodes, ${edgeCount.toLocaleString('en')} links · ${MODE_WORDS[info.mode] ?? info.mode}`;
  }

  let graph = $state<string | null>(null);
  let nodeCount = $state(0);
  let edgeCount = $state(0);
  let queryTime = $state(0);
  let loading = $state(false);

  currentGraph.subscribe((g) => { graph = g; });
  graphData.subscribe((d) => {
    if (d) {
      nodeCount = d.nodes.length;
      edgeCount = d.edges.length;
      queryTime = d.query_time_ms;
    } else {
      nodeCount = 0;
      edgeCount = 0;
      queryTime = 0;
    }
  });
  isLoading.subscribe((l) => { loading = l; });
</script>

<div class="status-bar">
  <div class="status-left">
    <span class="status-indicator" class:connected={graph !== null}></span>
    <span class="status-text">{graph ? 'Connected' : 'Disconnected'}</span>
    {#if graph}
      <span class="divider">|</span>
      <span class="status-text">Graph: <strong>{graph}</strong></span>
      <span class="divider">|</span>
      <span class="status-text counts" title={shown()}>{shown()}</span>
      {#if hiddenSummary(currentView) !== ''}
        <span class="divider">|</span>
        <label class="hidden-toggle" title={`The view leaves out: ${hiddenSummary(currentView)}`}>
          <input type="checkbox" checked={all} onchange={(e) => setShowAll(e.currentTarget.checked)} />
          show hidden{info && !all && info.hidden > 0 ? ` (${info.hidden.toLocaleString('en')})` : ''}
        </label>
      {/if}
    {/if}
  </div>
  <div class="status-right">
    {#if loading}
      <span class="status-loading">Loading...</span>
    {:else if queryTime > 0}
      <span class="status-text">Last query: {queryTime.toFixed(0)}ms</span>
    {/if}
  </div>
</div>

<style>
  .hidden-toggle {
    white-space: nowrap;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-dim, #94a3b8);
    cursor: pointer;
  }

  .hidden-toggle input {
    accent-color: var(--accent, #7c3aed);
  }

  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    height: 28px;
    background: var(--surface);
    border-top: 1px solid var(--border);
    font-size: 12px;
    color: var(--text-dim);
    flex-shrink: 0;
    user-select: none;
    gap: 12px;
    min-width: 0;
    overflow: hidden;
  }

  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  /* In a narrow window the left side gives way and is cut with an ellipsis; the switch that
     brings hidden kinds back, and the right side, keep their place. */
  .status-left {
    min-width: 0;
    flex: 1 1 auto;
    overflow: hidden;
  }

  .status-left > * {
    flex-shrink: 0;
  }

  .status-left > .status-text.counts {
    flex-shrink: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .status-right {
    flex-shrink: 0;
  }

  .status-indicator {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--danger);
    flex-shrink: 0;
  }

  .status-indicator.connected {
    background: var(--success);
  }

  .status-text {
    white-space: nowrap;
  }

  .status-text strong {
    color: var(--text-muted);
    font-weight: 600;
  }

  .divider {
    color: var(--border);
  }

  .status-loading {
    color: var(--primary-light);
    animation: pulse 1.5s ease infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.6; }
    50% { opacity: 1; }
  }
</style>
