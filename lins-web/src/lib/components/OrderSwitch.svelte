<!--
  The switch between the ways a graph is laid out: each of the view's arrangements (cards in
  lanes), one node with its ties around it, and the free picture. Shown only for a graph whose
  view has an arrangement; any other graph has the free picture alone, as before.
-->
<script lang="ts">
  import { mode, sceneData, view } from '$lib/stores/graph';
  import { arrangementsOf, showFree, showLanes, showScene } from '$lib/explore';

  const orders = $derived(arrangementsOf($view));
  const centre = $derived($sceneData?.center);
</script>

{#if orders.length > 0}
  <div class="switch" role="group" aria-label="How the graph is laid out">
    {#each orders as order, i (order.name)}
      <button
        type="button"
        class:on={$mode.kind === 'lanes' && $mode.index === i}
        aria-pressed={$mode.kind === 'lanes' && $mode.index === i}
        title="Cards in lanes"
        onclick={() => showLanes(i)}>{order.name}</button
      >
    {/each}
    <button
      type="button"
      class:on={$mode.kind === 'scene'}
      aria-pressed={$mode.kind === 'scene'}
      disabled={centre === undefined}
      title={centre === undefined ? 'Press a card or find a node first: this shows one thing with what it is tied to' : 'The thing last opened, with what it is tied to in columns around it'}
      onclick={() => centre !== undefined && showScene(centre)}>One thing and its ties</button
    >
    <button
      type="button"
      class:on={$mode.kind === 'free'}
      aria-pressed={$mode.kind === 'free'}
      title="What is loaded, as dots and lines that place themselves"
      onclick={showFree}>Free</button
    >
  </div>
{/if}

<style>
  .switch {
    display: inline-flex;
    flex-shrink: 0;
    font-size: 12.5px;
  }
  button {
    padding: 5px 11px;
    color: var(--text-muted);
    background: var(--bg);
    border: 1px solid var(--border);
    white-space: nowrap;
    transition: background var(--transition-fast), color var(--transition-fast);
  }
  button + button {
    border-left: 0;
  }
  button:first-child {
    border-radius: var(--radius-md) 0 0 var(--radius-md);
  }
  button:last-child {
    border-radius: 0 var(--radius-md) var(--radius-md) 0;
  }
  button:hover:not(:disabled):not(.on) {
    color: var(--text);
    background: var(--surface-hover);
  }
  button.on {
    color: #fff;
    background: var(--primary);
    border-color: var(--primary);
  }
  button:disabled {
    opacity: 0.45;
    cursor: default;
  }
</style>
