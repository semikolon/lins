<script lang="ts">
  import { currentGraph, availableGraphs, schema, graphData, isLoading, error, selectedNode, selectedEdge } from '$lib/stores/graph';
  import { fetchSchema, fetchGraphData } from '$lib/api';
  import type { GraphInfo } from '$lib/types';

  let isOpen = $state(false);
  let current = $state<string | null>(null);
  let graphs = $state<GraphInfo[]>([]);

  currentGraph.subscribe((g) => { current = g; });
  availableGraphs.subscribe((g) => { graphs = g; });

  function currentInfo(): GraphInfo | undefined {
    return graphs.find((g) => g.name === current);
  }

  async function selectGraph(name: string) {
    isOpen = false;
    if (name === current) return;

    currentGraph.set(name);
    selectedNode.set(null);
    selectedEdge.set(null);
    isLoading.set(true);
    error.set(null);

    try {
      const [s, d] = await Promise.all([
        fetchSchema(name),
        fetchGraphData(name),
      ]);
      schema.set(s);
      graphData.set(d);
    } catch (err) {
      error.set(err instanceof Error ? err.message : 'Failed to load graph');
    } finally {
      isLoading.set(false);
    }
  }

  function toggle() {
    isOpen = !isOpen;
  }

  function onBlur() {
    setTimeout(() => { isOpen = false; }, 200);
  }
</script>

<div class="graph-selector">
  <button class="selector-button" onclick={toggle} onblur={onBlur}>
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
      <circle cx="12" cy="12" r="3" />
      <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
    </svg>
    <span class="selector-name">{current || 'Select graph'}</span>
    {#if currentInfo()}
      <span class="selector-counts">
        {currentInfo()?.node_count}n / {currentInfo()?.edge_count}e
      </span>
    {/if}
    <svg class="chevron" class:open={isOpen} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
      <polyline points="6 9 12 15 18 9" />
    </svg>
  </button>

  {#if isOpen}
    <div class="dropdown">
      {#each graphs as g}
        <button
          class="option"
          class:active={g.name === current}
          onmousedown={() => selectGraph(g.name)}
        >
          <span class="option-name">{g.name}</span>
          <span class="option-counts">{g.node_count}n / {g.edge_count}e</span>
        </button>
      {/each}
      {#if graphs.length === 0}
        <div class="option empty">No graphs available</div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .graph-selector {
    position: relative;
  }

  .selector-button {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text);
    transition: border-color var(--transition-fast);
    white-space: nowrap;
  }

  .selector-button:hover {
    border-color: var(--primary-light);
  }

  .selector-name {
    font-weight: 600;
    font-size: 14px;
  }

  .selector-counts {
    font-size: 12px;
    color: var(--text-dim);
    font-variant-numeric: tabular-nums;
  }

  .chevron {
    transition: transform var(--transition-fast);
    color: var(--text-dim);
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  .dropdown {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    min-width: 200px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    z-index: 100;
    animation: fadeIn var(--transition-fast);
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 8px 12px;
    text-align: left;
    color: var(--text);
    transition: background var(--transition-fast);
  }

  .option:hover {
    background: var(--surface-hover);
  }

  .option.active {
    background: var(--surface-hover);
    color: var(--primary-light);
  }

  .option-name {
    font-weight: 500;
  }

  .option-counts {
    font-size: 12px;
    color: var(--text-dim);
    font-variant-numeric: tabular-nums;
  }

  .option.empty {
    color: var(--text-dim);
    font-style: italic;
    cursor: default;
  }
</style>
