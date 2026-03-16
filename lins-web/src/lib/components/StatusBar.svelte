<script lang="ts">
  import { currentGraph, graphData, isLoading } from '$lib/stores/graph';

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
      <span class="status-text">{nodeCount} nodes, {edgeCount} edges</span>
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
  }

  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: 6px;
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
