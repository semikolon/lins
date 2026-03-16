<script lang="ts">
  import { get } from 'svelte/store';
  import { currentGraph, graphData, isLoading } from '$lib/stores/graph';
  import { searchVocabulary, executeQuery } from '$lib/api';
  import type { Suggestion, QueryResult } from '$lib/types';

  let query = $state('');
  let suggestions = $state<Suggestion[]>([]);
  let isOpen = $state(false);
  let activeIndex = $state(-1);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let inputEl: HTMLInputElement;

  const groupLabels: Record<string, string> = {
    label: 'Labels',
    relationship_type: 'Relationships',
    property_key: 'Properties',
    property_value: 'Values',
  };

  const groupIcons: Record<string, string> = {
    label: 'L',
    relationship_type: 'R',
    property_key: 'P',
    property_value: 'V',
  };

  interface GroupedSuggestion {
    type: string;
    label: string;
    items: Suggestion[];
  }

  function groupSuggestions(items: Suggestion[]): GroupedSuggestion[] {
    const groups = new Map<string, Suggestion[]>();
    for (const item of items) {
      const key = item.suggestion_type;
      if (!groups.has(key)) groups.set(key, []);
      groups.get(key)!.push(item);
    }
    const order = ['label', 'relationship_type', 'property_key', 'property_value'];
    const result: GroupedSuggestion[] = [];
    for (const type of order) {
      const items = groups.get(type);
      if (items && items.length > 0) {
        result.push({
          type,
          label: groupLabels[type] || type,
          items,
        });
      }
    }
    return result;
  }

  function flatSuggestions(): Suggestion[] {
    const grouped = groupSuggestions(suggestions);
    const flat: Suggestion[] = [];
    for (const g of grouped) {
      for (const item of g.items) flat.push(item);
    }
    return flat;
  }

  async function onInput() {
    if (debounceTimer) clearTimeout(debounceTimer);
    if (query.length < 1) {
      suggestions = [];
      isOpen = false;
      return;
    }
    debounceTimer = setTimeout(async () => {
      let graphName: string | null = null;
      graphName = get(currentGraph);
      try {
        suggestions = await searchVocabulary(query, graphName || undefined);
        isOpen = suggestions.length > 0;
        activeIndex = -1;
      } catch {
        suggestions = [];
        isOpen = false;
      }
    }, 150);
  }

  async function selectSuggestion(suggestion: Suggestion) {
    let graphName: string | null = null;
    graphName = get(currentGraph);
    if (!graphName) return;

    isOpen = false;
    query = suggestion.value;
    isLoading.set(true);

    try {
      const result: QueryResult = await executeQuery(graphName, suggestion.cypher);
      // Additive merge: add new nodes/edges to existing data
      graphData.update((existing) => {
        if (!existing) return result;
        const existingNodeIds = new Set(existing.nodes.map((n) => n.id));
        const existingEdgeIds = new Set(existing.edges.map((e) => e.id));
        return {
          nodes: [...existing.nodes, ...result.nodes.filter((n) => !existingNodeIds.has(n.id))],
          edges: [...existing.edges, ...result.edges.filter((e) => !existingEdgeIds.has(e.id))],
          query_time_ms: result.query_time_ms,
          source_graph: result.source_graph,
        };
      });
    } catch (err) {
      console.error('Query failed:', err);
    } finally {
      isLoading.set(false);
    }
  }

  async function onKeydown(e: KeyboardEvent) {
    if (!isOpen) {
      if (e.key === 'Enter' && query.length > 0) {
        e.preventDefault();
        await searchAsProperty();
      }
      return;
    }

    const flat = flatSuggestions();

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      activeIndex = Math.min(activeIndex + 1, flat.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      activeIndex = Math.max(activeIndex - 1, -1);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (activeIndex >= 0 && activeIndex < flat.length) {
        await selectSuggestion(flat[activeIndex]);
      } else {
        await searchAsProperty();
      }
    } else if (e.key === 'Escape') {
      isOpen = false;
      activeIndex = -1;
    }
  }

  async function searchAsProperty() {
    let graphName: string | null = null;
    graphName = get(currentGraph);
    if (!graphName || !query) return;

    isOpen = false;
    isLoading.set(true);

    const cypher = `MATCH (n) WHERE ANY(k IN keys(n) WHERE toString(n[k]) CONTAINS '${query.replace(/'/g, "\\'")}') RETURN n LIMIT 50`;
    try {
      const result = await executeQuery(graphName, cypher);
      graphData.update((existing) => {
        if (!existing) return result;
        const existingNodeIds = new Set(existing.nodes.map((n) => n.id));
        return {
          nodes: [...existing.nodes, ...result.nodes.filter((n) => !existingNodeIds.has(n.id))],
          edges: existing.edges,
          query_time_ms: result.query_time_ms,
          source_graph: result.source_graph,
        };
      });
    } catch (err) {
      console.error('Search failed:', err);
    } finally {
      isLoading.set(false);
    }
  }

  function onBlur() {
    // Delay to allow click on suggestion
    setTimeout(() => { isOpen = false; }, 200);
  }
</script>

<div class="search-bar">
  <div class="search-input-wrapper">
    <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
    </svg>
    <input
      bind:this={inputEl}
      type="text"
      placeholder="Search graph..."
      bind:value={query}
      oninput={onInput}
      onkeydown={onKeydown}
      onblur={onBlur}
      onfocus={() => { if (suggestions.length > 0) isOpen = true; }}
      class="search-input"
    />
  </div>

  {#if isOpen && suggestions.length > 0}
    {@const grouped = groupSuggestions(suggestions)}
    {@const flat = flatSuggestions()}
    <div class="dropdown">
      {#each grouped as group}
        <div class="group-header">{group.label}</div>
        {#each group.items as item}
          {@const globalIndex = flat.indexOf(item)}
          <button
            class="suggestion"
            class:active={globalIndex === activeIndex}
            onmousedown={() => selectSuggestion(item)}
            onmouseenter={() => { activeIndex = globalIndex; }}
          >
            <span class="badge badge-{item.suggestion_type}">{groupIcons[item.suggestion_type]}</span>
            <span class="suggestion-value">{item.value}</span>
            {#if item.count != null}
              <span class="suggestion-count">{item.count}</span>
            {/if}
          </button>
        {/each}
      {/each}
    </div>
  {/if}
</div>

<style>
  .search-bar {
    position: relative;
    width: 100%;
    max-width: 500px;
  }

  .search-input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 10px;
    width: 16px;
    height: 16px;
    color: var(--text-dim);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 8px 12px 8px 34px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text);
    font-size: 14px;
    transition: border-color var(--transition-fast);
  }

  .search-input:focus {
    border-color: var(--primary-light);
    outline: none;
  }

  .dropdown {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    max-height: 320px;
    overflow-y: auto;
    z-index: 100;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    animation: fadeIn var(--transition-fast);
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .group-header {
    padding: 6px 12px 4px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-dim);
  }

  .suggestion {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    text-align: left;
    color: var(--text);
    transition: background var(--transition-fast);
  }

  .suggestion:hover,
  .suggestion.active {
    background: var(--surface-hover);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 700;
    flex-shrink: 0;
  }

  .badge-label { background: var(--primary); color: white; }
  .badge-relationship_type { background: var(--secondary); color: white; }
  .badge-property_key { background: #b45309; color: white; }
  .badge-property_value { background: #0e7490; color: white; }

  .suggestion-value {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .suggestion-count {
    font-size: 12px;
    color: var(--text-dim);
    font-variant-numeric: tabular-nums;
  }
</style>
