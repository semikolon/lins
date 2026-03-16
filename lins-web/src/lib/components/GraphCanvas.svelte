<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import Graph from 'graphology';
  import Sigma from 'sigma';
  import FA2Layout from 'graphology-layout-forceatlas2/worker';
  import { graphData, selectedNode, selectedEdge, schema } from '$lib/stores/graph';
  import type { GraphNode, GraphEdge } from '$lib/types';

  const LABEL_COLORS = [
    '#2d6a4f', '#7c3aed', '#b45309', '#0e7490', '#be185d',
    '#4338ca', '#059669', '#d97706', '#7c2d12', '#1d4ed8',
    '#9333ea', '#dc2626', '#0284c7', '#65a30d', '#c026d3',
  ];

  let container: HTMLDivElement;
  let sigmaInstance: Sigma | null = null;
  let graph: Graph | null = null;
  let layout: FA2Layout | null = null;
  let layoutTimer: ReturnType<typeof setTimeout> | null = null;
  let hoveredNode: string | null = null;

  function getLabelColor(label: string, allLabels: string[]): string {
    const idx = allLabels.indexOf(label);
    return LABEL_COLORS[idx >= 0 ? idx % LABEL_COLORS.length : 0];
  }

  function getNodeLabel(node: GraphNode): string {
    const name = node.properties['name'];
    if (typeof name === 'string') return name;
    if (node.labels.length > 0) return node.labels[0];
    return `Node ${node.id}`;
  }

  function buildGraph(data: { nodes: GraphNode[]; edges: GraphEdge[] }, labels: string[]) {
    stopLayout();
    if (sigmaInstance) {
      sigmaInstance.kill();
      sigmaInstance = null;
    }

    graph = new Graph({ multi: true, type: 'directed' });

    const degreeMap = new Map<number, number>();
    for (const edge of data.edges) {
      degreeMap.set(edge.source_id, (degreeMap.get(edge.source_id) || 0) + 1);
      degreeMap.set(edge.target_id, (degreeMap.get(edge.target_id) || 0) + 1);
    }

    for (const node of data.nodes) {
      const degree = degreeMap.get(node.id) || 0;
      const primaryLabel = node.labels[0] || 'Unknown';
      const color = getLabelColor(primaryLabel, labels);
      const key = String(node.id);

      if (!graph.hasNode(key)) {
        graph.addNode(key, {
          label: getNodeLabel(node),
          color,
          size: Math.max(4, Math.sqrt(degree + 1) * 3),
          x: Math.random() * 100,
          y: Math.random() * 100,
          nodeData: node,
        });
      }
    }

    const nodeSet = new Set(graph.nodes());
    for (const edge of data.edges) {
      const src = String(edge.source_id);
      const tgt = String(edge.target_id);
      if (nodeSet.has(src) && nodeSet.has(tgt)) {
        try {
          graph.addEdge(src, tgt, {
            label: edge.relationship_type,
            color: '#4a5568',
            size: 1,
            edgeData: edge,
          });
        } catch {
          // skip duplicate edges in multi-graph
        }
      }
    }

    if (!container) return;

    sigmaInstance = new Sigma(graph, container, {
      renderLabels: true,
      labelRenderedSizeThreshold: 8,
      defaultNodeColor: '#2d6a4f',
      defaultEdgeColor: '#4a5568',
      labelColor: { color: '#e2e8f0' },
      labelFont: 'Manrope',
      labelSize: 12,
      nodeReducer(node, data) {
        const res = { ...data };
        if (hoveredNode) {
          if (node === hoveredNode || graph!.hasEdge(hoveredNode, node) || graph!.hasEdge(node, hoveredNode)) {
            res.highlighted = true;
          } else {
            res.color = '#2d3748';
            res.label = '';
          }
        }
        return res;
      },
      edgeReducer(edge, data) {
        const res = { ...data };
        if (hoveredNode) {
          const src = graph!.source(edge);
          const tgt = graph!.target(edge);
          if (src !== hoveredNode && tgt !== hoveredNode) {
            res.hidden = true;
          } else {
            res.color = '#94a3b8';
            res.size = 2;
          }
        }
        return res;
      },
    });

    sigmaInstance.on('enterNode', ({ node }) => {
      hoveredNode = node;
      sigmaInstance?.refresh();
      if (container) container.style.cursor = 'pointer';
    });

    sigmaInstance.on('leaveNode', () => {
      hoveredNode = null;
      sigmaInstance?.refresh();
      if (container) container.style.cursor = 'default';
    });

    sigmaInstance.on('clickNode', ({ node }) => {
      const attrs = graph!.getNodeAttributes(node);
      const nd = attrs.nodeData as GraphNode | undefined;
      if (nd) {
        selectedNode.set(nd);
        selectedEdge.set(null);
      }
    });

    sigmaInstance.on('clickEdge', ({ edge }) => {
      const attrs = graph!.getEdgeAttributes(edge);
      const ed = attrs.edgeData as GraphEdge | undefined;
      if (ed) {
        selectedEdge.set(ed);
        selectedNode.set(null);
      }
    });

    sigmaInstance.on('clickStage', () => {
      selectedNode.set(null);
      selectedEdge.set(null);
    });

    startLayout();
  }

  function startLayout() {
    if (!graph || graph.order === 0) return;
    layout = new FA2Layout(graph, {
      settings: {
        gravity: 1,
        scalingRatio: 2,
        barnesHutOptimize: true,
        slowDown: 5,
      },
    });
    layout.start();

    layoutTimer = setTimeout(() => {
      stopLayout();
    }, 3000);
  }

  function stopLayout() {
    if (layout) {
      layout.kill();
      layout = null;
    }
    if (layoutTimer) {
      clearTimeout(layoutTimer);
      layoutTimer = null;
    }
  }

  function getLabels(): string[] {
    const s = get(schema);
    if (s && s.labels.length > 0) return s.labels;
    const data = get(graphData);
    if (!data) return [];
    const labelSet = new Set<string>();
    for (const n of data.nodes) {
      for (const l of n.labels) labelSet.add(l);
    }
    return Array.from(labelSet);
  }

  const unsubData = graphData.subscribe((data) => {
    if (!data || !container) return;
    buildGraph(data, getLabels());
  });

  onMount(() => {
    const current = get(graphData);
    if (current) {
      buildGraph(current, getLabels());
    }
  });

  onDestroy(() => {
    unsubData();
    stopLayout();
    if (sigmaInstance) {
      sigmaInstance.kill();
      sigmaInstance = null;
    }
  });
</script>

<div bind:this={container} class="graph-canvas"></div>

<style>
  .graph-canvas {
    width: 100%;
    height: 100%;
    position: relative;
    background: var(--bg);
  }
</style>
