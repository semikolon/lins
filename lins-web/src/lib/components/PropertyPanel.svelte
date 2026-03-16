<script lang="ts">
  import { get } from 'svelte/store';
  import { selectedNode, selectedEdge, graphData } from '$lib/stores/graph';
  import type { GraphNode, GraphEdge, PropertyValue, QueryResult } from '$lib/types';

  const HIDDEN_SUFFIXES = ['_embedding'];
  const HIDDEN_KEYS = new Set(['group_id', 'uuid']);

  const LABEL_COLORS: Record<string, string> = {};
  const ALL_COLORS = [
    '#2d6a4f', '#7c3aed', '#b45309', '#0e7490', '#be185d',
    '#4338ca', '#059669', '#d97706', '#7c2d12', '#1d4ed8',
  ];

  let node = $state<GraphNode | null>(null);
  let edge = $state<GraphEdge | null>(null);
  let visible = $derived(node !== null || edge !== null);
  let connections = $state<{ direction: string; type: string; targetName: string; targetId: number }[]>([]);
  let expandedKeys = $state<Set<string>>(new Set());

  selectedNode.subscribe((n) => {
    node = n;
    if (n) computeConnections(n);
  });
  selectedEdge.subscribe((e) => { edge = e; });

  function computeConnections(n: GraphNode) {
    const conns: typeof connections = [];
    const data: QueryResult | null = get(graphData);
    if (!data) { connections = []; return; }

    const nodeMap = new Map(data.nodes.map((nd: GraphNode) => [nd.id, nd]));
    for (const e of data.edges) {
      if (e.source_id === n.id) {
        const target = nodeMap.get(e.target_id);
        conns.push({
          direction: 'out',
          type: e.relationship_type,
          targetName: target ? getDisplayName(target) : `Node ${e.target_id}`,
          targetId: e.target_id,
        });
      } else if (e.target_id === n.id) {
        const source = nodeMap.get(e.source_id);
        conns.push({
          direction: 'in',
          type: e.relationship_type,
          targetName: source ? getDisplayName(source) : `Node ${e.source_id}`,
          targetId: e.source_id,
        });
      }
    }
    connections = conns;
  }

  function getDisplayName(n: GraphNode): string {
    const name = n.properties['name'];
    if (typeof name === 'string') return name;
    return n.labels[0] || `Node ${n.id}`;
  }

  function shouldShow(key: string): boolean {
    if (HIDDEN_KEYS.has(key)) return false;
    for (const suffix of HIDDEN_SUFFIXES) {
      if (key.endsWith(suffix)) return false;
    }
    return true;
  }

  function formatValue(val: PropertyValue): string {
    if (val === null) return 'null';
    if (typeof val === 'boolean') return val ? 'true' : 'false';
    if (typeof val === 'number') return String(val);
    if (typeof val === 'string') {
      if (/^\d{4}-\d{2}-\d{2}T/.test(val)) {
        try {
          const d = new Date(val);
          return d.toLocaleString('sv-SE', { dateStyle: 'medium', timeStyle: 'short' });
        } catch { return val; }
      }
      return val;
    }
    if (Array.isArray(val)) return JSON.stringify(val);
    return JSON.stringify(val, null, 2);
  }

  function isLongValue(val: PropertyValue): boolean {
    const s = formatValue(val);
    return s.length > 120;
  }

  function toggleExpand(key: string) {
    const next = new Set(expandedKeys);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    expandedKeys = next;
  }

  function close() {
    selectedNode.set(null);
    selectedEdge.set(null);
  }

  function getLabelColor(label: string): string {
    if (!LABEL_COLORS[label]) {
      const idx = Object.keys(LABEL_COLORS).length;
      LABEL_COLORS[label] = ALL_COLORS[idx % ALL_COLORS.length];
    }
    return LABEL_COLORS[label];
  }

  function navigateToNode(nodeId: number) {
    const data: QueryResult | null = get(graphData);
    if (!data) return;
    const target = data.nodes.find((n) => n.id === nodeId);
    if (target) {
      selectedNode.set(target);
      selectedEdge.set(null);
    }
  }

  function getProperties(obj: GraphNode | GraphEdge): [string, PropertyValue][] {
    return Object.entries(obj.properties).filter(([k]) => shouldShow(k));
  }
</script>

<div class="panel" class:visible>
  {#if node}
    <div class="panel-header">
      <div class="header-info">
        <span class="label-dot" style:background-color={getLabelColor(node.labels[0] || 'Unknown')}></span>
        <div>
          <div class="header-label">{node.labels.join(', ')}</div>
          <div class="header-name">{getDisplayName(node)}</div>
        </div>
      </div>
      <button class="close-btn" onclick={close} aria-label="Close panel">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18">
          <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <div class="panel-content">
      <section class="properties">
        <h3 class="section-title">Properties</h3>
        {#each getProperties(node) as [key, val]}
          <div class="prop-row">
            <span class="prop-key">{key}</span>
            <div class="prop-value" class:truncated={isLongValue(val) && !expandedKeys.has(key)}>
              {#if isLongValue(val) && !expandedKeys.has(key)}
                {formatValue(val).slice(0, 120)}...
                <button class="expand-btn" onclick={() => toggleExpand(key)}>show more</button>
              {:else}
                {formatValue(val)}
                {#if isLongValue(val)}
                  <button class="expand-btn" onclick={() => toggleExpand(key)}>show less</button>
                {/if}
              {/if}
            </div>
          </div>
        {/each}
      </section>

      {#if connections.length > 0}
        <section class="relationships">
          <h3 class="section-title">Relationships ({connections.length})</h3>
          {#each connections.slice(0, 20) as conn}
            <button class="rel-row" onclick={() => navigateToNode(conn.targetId)}>
              <span class="rel-direction">{conn.direction === 'out' ? '\u2192' : '\u2190'}</span>
              <span class="rel-type">{conn.type}</span>
              <span class="rel-target">{conn.targetName}</span>
            </button>
          {/each}
          {#if connections.length > 20}
            <div class="more-indicator">...and {connections.length - 20} more</div>
          {/if}
        </section>
      {/if}
    </div>
  {:else if edge}
    <div class="panel-header">
      <div class="header-info">
        <span class="label-dot" style:background-color="#4a5568"></span>
        <div>
          <div class="header-label">Relationship</div>
          <div class="header-name">{edge.relationship_type}</div>
        </div>
      </div>
      <button class="close-btn" onclick={close} aria-label="Close panel">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18">
          <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <div class="panel-content">
      <section class="properties">
        <h3 class="section-title">Properties</h3>
        {#each getProperties(edge) as [key, val]}
          <div class="prop-row">
            <span class="prop-key">{key}</span>
            <div class="prop-value">{formatValue(val)}</div>
          </div>
        {/each}
        {#if getProperties(edge).length === 0}
          <div class="empty-state">No properties</div>
        {/if}
      </section>
    </div>
  {/if}
</div>

<style>
  .panel {
    width: 320px;
    min-width: 320px;
    background: var(--surface);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    transform: translateX(100%);
    transition: transform var(--transition-normal);
    overflow: hidden;
  }

  .panel.visible {
    transform: translateX(0);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .header-info {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .label-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .header-label {
    font-size: 11px;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .header-name {
    font-weight: 600;
    font-size: 15px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 220px;
  }

  .close-btn {
    color: var(--text-dim);
    padding: 4px;
    border-radius: var(--radius-sm);
    transition: color var(--transition-fast), background var(--transition-fast);
  }

  .close-btn:hover {
    color: var(--text);
    background: var(--surface-hover);
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: 12px 0;
  }

  .section-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-dim);
    padding: 4px 16px 8px;
  }

  .properties,
  .relationships {
    margin-bottom: 16px;
  }

  .prop-row {
    padding: 4px 16px;
  }

  .prop-key {
    display: block;
    font-size: 11px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .prop-value {
    font-size: 13px;
    color: var(--text);
    word-break: break-word;
    line-height: 1.4;
  }

  .prop-value.truncated {
    display: -webkit-box;
  }

  .expand-btn {
    font-size: 11px;
    color: var(--primary-light);
    padding: 0;
    margin-left: 4px;
  }

  .expand-btn:hover {
    text-decoration: underline;
  }

  .rel-row {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 5px 16px;
    text-align: left;
    font-size: 13px;
    transition: background var(--transition-fast);
  }

  .rel-row:hover {
    background: var(--surface-hover);
  }

  .rel-direction {
    color: var(--text-dim);
    flex-shrink: 0;
    width: 16px;
    text-align: center;
  }

  .rel-type {
    color: var(--secondary);
    font-family: var(--font-mono);
    font-size: 11px;
    flex-shrink: 0;
  }

  .rel-target {
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .more-indicator {
    padding: 4px 16px;
    font-size: 12px;
    color: var(--text-dim);
  }

  .empty-state {
    padding: 8px 16px;
    font-size: 13px;
    color: var(--text-dim);
    font-style: italic;
  }
</style>
