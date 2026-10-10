<script lang="ts">
  import { onMount } from 'svelte';
  import GraphCanvas from '$lib/components/GraphCanvas.svelte';
  import SearchBar from '$lib/components/SearchBar.svelte';
  import PropertyPanel from '$lib/components/PropertyPanel.svelte';
  import GraphSelector from '$lib/components/GraphSelector.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import { availableGraphs, graphData, isLoading, error } from '$lib/stores/graph';
  import { fetchGraphs, fetchDefaultGraph } from '$lib/api';
  import { loadGraph } from '$lib/explore';

  let loading = $state(true);
  let errMsg = $state<string | null>(null);

  /** True once a graph has been drawn. From then on the canvas stays mounted while more is
   *  loaded, so that widening a view moves nothing and a failed request loses nothing. */
  let hasData = $state(false);

  isLoading.subscribe((l) => { loading = l; });
  error.subscribe((e) => { errMsg = e; });
  graphData.subscribe((d) => { hasData = d !== null; });

  /** Open the graph the config names, or failing that the first one. */
  async function start() {
    isLoading.set(true);
    error.set(null);
    try {
      const [graphs, preferred] = await Promise.all([fetchGraphs(), fetchDefaultGraph().catch(() => null)]);
      availableGraphs.set(graphs);
      const first = graphs.find((g) => g.name === preferred)?.name ?? graphs[0]?.name;
      if (first) await loadGraph(first);
    } catch (err) {
      error.set(err instanceof Error ? err.message : 'Failed to connect to Lins server');
    } finally {
      isLoading.set(false);
    }
  }

  onMount(start);

  const retry = start;
</script>

<div class="explorer">
  <header class="toolbar">
    <GraphSelector />
    <SearchBar />
  </header>

  <main class="content">
    {#if hasData}
      <div class="canvas-area">
        <GraphCanvas />
        {#if errMsg}
          <div class="notice" role="alert">
            {errMsg}
            <button class="notice-close" onclick={() => error.set(null)} aria-label="Dismiss">×</button>
          </div>
        {/if}
      </div>
    {:else if errMsg}
      <div class="center-overlay">
        <div class="error-icon">!</div>
        <p class="error-text">{errMsg}</p>
        <button class="retry-btn" onclick={retry}>Retry</button>
      </div>
    {:else}
      <div class="center-overlay">
        <div class="spinner"></div>
        <p class="loading-text">{loading ? 'Connecting to FalkorDB...' : 'No graph to show.'}</p>
      </div>
    {/if}
    <PropertyPanel />
  </main>

  <StatusBar />
</div>

<style>
  .notice {
    position: absolute;
    left: 16px;
    bottom: 16px;
    max-width: 60%;
    padding: 8px 12px;
    font-size: 12px;
    color: #f87171;
    background: var(--surface);
    border: 1px solid rgba(248, 113, 113, 0.3);
    border-radius: 6px;
    z-index: 5;
  }

  .notice-close {
    margin-left: 8px;
    color: inherit;
    background: none;
    border: none;
    cursor: pointer;
  }

  .explorer {
    display: grid;
    grid-template-rows: 56px 1fr 28px;
    /* minmax(0, …): a grid column is otherwise as wide as its widest row wants to be, so a long
       line in the status bar made the whole page wider than the window and cut the canvas off
       at the right. */
    grid-template-columns: minmax(0, 1fr);
    height: 100%;
    width: 100%;
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 0 16px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    z-index: 10;
  }

  .content {
    display: flex;
    overflow: hidden;
    position: relative;
  }

  .canvas-area {
    flex: 1;
    position: relative;
    overflow: hidden;
  }

  .center-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    flex: 1;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border);
    border-top-color: var(--primary-light);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .loading-text {
    color: var(--text-dim);
    font-size: 14px;
  }

  .error-icon {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: var(--danger);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 20px;
  }

  .error-text {
    color: var(--text-muted);
    font-size: 14px;
    max-width: 400px;
    text-align: center;
  }

  .retry-btn {
    padding: 8px 20px;
    background: var(--primary);
    color: white;
    border-radius: var(--radius-md);
    font-weight: 600;
    transition: background var(--transition-fast);
  }

  .retry-btn:hover {
    background: var(--primary-light);
  }
</style>
