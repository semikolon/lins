<script lang="ts">
  import { onMount } from 'svelte';
  import GraphCanvas from '$lib/components/GraphCanvas.svelte';
  import SearchBar from '$lib/components/SearchBar.svelte';
  import PropertyPanel from '$lib/components/PropertyPanel.svelte';
  import GraphSelector from '$lib/components/GraphSelector.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import {
    currentGraph,
    graphData,
    schema,
    availableGraphs,
    isLoading,
    error,
  } from '$lib/stores/graph';
  import { fetchGraphs, fetchSchema, fetchGraphData } from '$lib/api';

  let loading = $state(true);
  let errMsg = $state<string | null>(null);

  isLoading.subscribe((l) => { loading = l; });
  error.subscribe((e) => { errMsg = e; });

  onMount(async () => {
    isLoading.set(true);
    error.set(null);

    try {
      const graphs = await fetchGraphs();
      availableGraphs.set(graphs);

      if (graphs.length > 0) {
        const first = graphs[0].name;
        currentGraph.set(first);

        const [s, d] = await Promise.all([
          fetchSchema(first),
          fetchGraphData(first),
        ]);
        schema.set(s);
        graphData.set(d);
      }
    } catch (err) {
      error.set(err instanceof Error ? err.message : 'Failed to connect to Lins server');
    } finally {
      isLoading.set(false);
    }
  });

  async function retry() {
    error.set(null);
    isLoading.set(true);
    try {
      const graphs = await fetchGraphs();
      availableGraphs.set(graphs);
      if (graphs.length > 0) {
        const first = graphs[0].name;
        currentGraph.set(first);
        const [s, d] = await Promise.all([
          fetchSchema(first),
          fetchGraphData(first),
        ]);
        schema.set(s);
        graphData.set(d);
      }
    } catch (err) {
      error.set(err instanceof Error ? err.message : 'Failed to connect');
    } finally {
      isLoading.set(false);
    }
  }
</script>

<div class="explorer">
  <header class="toolbar">
    <GraphSelector />
    <SearchBar />
  </header>

  <main class="content">
    {#if loading && !errMsg}
      <div class="center-overlay">
        <div class="spinner"></div>
        <p class="loading-text">Connecting to FalkorDB...</p>
      </div>
    {:else if errMsg}
      <div class="center-overlay">
        <div class="error-icon">!</div>
        <p class="error-text">{errMsg}</p>
        <button class="retry-btn" onclick={retry}>Retry</button>
      </div>
    {:else}
      <div class="canvas-area">
        <GraphCanvas />
      </div>
    {/if}
    <PropertyPanel />
  </main>

  <StatusBar />
</div>

<style>
  .explorer {
    display: grid;
    grid-template-rows: 56px 1fr 28px;
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
