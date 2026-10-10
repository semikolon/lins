<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import Graph from 'graphology';
  import Sigma from 'sigma';
  import forceAtlas2 from 'graphology-layout-forceatlas2';
  import FA2Layout from 'graphology-layout-forceatlas2/worker';
  import { graphData, selectedNode, selectedEdge, schema, view, anchorNode } from '$lib/stores/graph';
  import { labelColor, nodeName, relColor } from '$lib/view';
  import { expandAround } from '$lib/explore';
  import type { GraphNode, GraphEdge, GraphView } from '$lib/types';

  let container: HTMLDivElement;
  let sigmaInstance: Sigma | null = null;
  let graph: Graph | null = null;
  let layout: FA2Layout | null = null;
  let layoutTimer: ReturnType<typeof setTimeout> | null = null;
  let hoveredNode: string | null = null;
  let hoveredEdge: string | null = null;
  let resizeObserver: ResizeObserver | null = null;

  const SURFACE = '#1c2130';
  const TEXT = '#f1f5f9';
  const RING = '#a79df0';

  type Drawn = { x: number; y: number; size: number; label?: string | null; color?: string };
  type TextSettings = { labelSize: number; labelFont: string; labelWeight: string };

  /** A node's name beside it, with a dark edge around the letters so that a link running
   *  under the text does not cut through it. */
  function drawLabel(context: CanvasRenderingContext2D, data: Drawn, settings: TextSettings) {
    if (!data.label) return;
    context.font = `${settings.labelWeight} ${settings.labelSize}px ${settings.labelFont}`;
    const x = data.x + data.size + 4;
    const y = data.y + settings.labelSize / 3;
    context.lineJoin = 'round';
    context.lineWidth = 4;
    context.strokeStyle = 'rgba(15, 18, 26, 0.9)';
    context.strokeText(data.label, x, y);
    context.fillStyle = TEXT;
    context.fillText(data.label, x, y);
  }

  /** The node under the pointer: a ring, and its name on a dark plate. (Sigma's own draws a
   *  white plate and keeps the label colour, which here is light text on white.) */
  function drawHover(context: CanvasRenderingContext2D, data: Drawn, settings: TextSettings) {
    context.beginPath();
    context.arc(data.x, data.y, data.size + 3, 0, Math.PI * 2);
    context.closePath();
    context.lineWidth = 2;
    context.strokeStyle = RING;
    context.stroke();
    if (!data.label) return;
    context.font = `600 ${settings.labelSize}px ${settings.labelFont}`;
    const pad = 6;
    const width = context.measureText(data.label).width + pad * 2;
    const height = settings.labelSize + pad * 2;
    const x = data.x + data.size + 8;
    const y = data.y - height / 2;
    context.beginPath();
    context.roundRect(x, y, width, height, 5);
    context.closePath();
    context.fillStyle = SURFACE;
    context.fill();
    context.lineWidth = 1;
    context.strokeStyle = 'rgba(167, 157, 240, 0.4)';
    context.stroke();
    context.fillStyle = TEXT;
    context.fillText(data.label, x + pad, data.y + settings.labelSize / 3);
  }

  /** Sigma refuses a container with no size, which is what a hidden tab or pane gives it. */
  function hasRoom(): boolean {
    return !!container && container.clientWidth > 0 && container.clientHeight > 0;
  }

  function render() {
    const data = get(graphData);
    if (data && hasRoom()) buildGraph(data, getLabels(), get(view));
  }

  /** Where each node was, so that widening a view moves nothing a person has already found. */
  function positions(): Map<string, { x: number; y: number }> {
    const kept = new Map<string, { x: number; y: number }>();
    graph?.forEachNode((key, attrs) => kept.set(key, { x: attrs.x, y: attrs.y }));
    return kept;
  }

  function buildGraph(data: { nodes: GraphNode[]; edges: GraphEdge[] }, labels: string[], v: GraphView | null) {
    stopLayout();
    hoveredNode = null;
    hoveredEdge = null;
    const before = positions();
    const anchorId = get(anchorNode);
    const anchor = anchorId !== null ? before.get(String(anchorId)) : undefined;
    // A widened view keeps its camera; a new one starts from the whole picture.
    const camera = anchor && sigmaInstance ? sigmaInstance.getCamera().getState() : null;
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
      const key = String(node.id);
      if (graph.hasNode(key)) continue;
      const was = anchor ? before.get(key) : undefined;
      // A new node starts beside the node it was reached from, so the layout has little to undo.
      const spread = anchor ? 6 : 100;
      const origin = anchor ?? { x: 0, y: 0 };
      graph.addNode(key, {
        label: nodeName(node, v),
        color: labelColor(node.labels[0] || 'Unknown', v, labels),
        size: Math.min(16, Math.max(5, Math.sqrt(degree + 1) * 2.2)),
        x: was ? was.x : origin.x + (Math.random() - 0.5) * spread,
        y: was ? was.y : origin.y + (Math.random() - 0.5) * spread,
        nodeData: node,
      });
    }

    // In a small view a link is thick enough to click; in a large one thin enough to see through.
    const linkSize = data.nodes.length <= 80 ? 3 : 1.5;
    for (const edge of data.edges) {
      const src = String(edge.source_id);
      const tgt = String(edge.target_id);
      if (!graph.hasNode(src) || !graph.hasNode(tgt)) continue;
      const color = relColor(edge.relationship_type, v);
      graph.addEdgeWithKey(`e${edge.id}`, src, tgt, {
        label: edge.relationship_type,
        color,
        baseColor: color,
        size: linkSize,
        type: 'arrow',
        edgeData: edge,
      });
    }

    if (!container) return;

    sigmaInstance = new Sigma(graph, container, {
      renderLabels: true,
      renderEdgeLabels: true,
      enableEdgeEvents: true,
      // A small view names every node; a large one names the well-connected and the rest on zoom.
      labelRenderedSizeThreshold: graph.order <= 80 ? 0 : 7,
      defaultNodeColor: '#2d6a4f',
      defaultEdgeColor: '#4a5568',
      defaultEdgeType: 'arrow',
      labelColor: { color: '#e2e8f0' },
      edgeLabelColor: { color: '#94a3b8' },
      edgeLabelSize: 10,
      labelFont: 'Manrope',
      edgeLabelFont: 'Manrope',
      labelSize: 13,
      defaultDrawNodeLabel: drawLabel,
      defaultDrawNodeHover: drawHover,
      nodeReducer(node, data) {
        const res = { ...data };
        if (hoveredNode) {
          if (node === hoveredNode) {
            res.highlighted = true;
          } else if (graph!.hasEdge(hoveredNode, node) || graph!.hasEdge(node, hoveredNode)) {
            // A neighbour keeps its colour and shows its name. Only the node under the
            // pointer gets the plate: a plate on every neighbour of a hub covers the picture.
            res.forceLabel = true;
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
            res.size = Math.max(2.5, linkSize);
          }
        }
        // A link's name is shown for the link under the pointer only. On every link, or on
        // every link of a hovered hub, the names cover each other.
        if (edge === hoveredEdge) {
          res.size = Math.max(3, Number(res.size) || 0);
          res.forceLabel = true;
        } else {
          res.label = '';
        }
        return res;
      },
    });

    if (camera) sigmaInstance.getCamera().setState(camera);

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

    sigmaInstance.on('enterEdge', ({ edge }) => {
      hoveredEdge = edge;
      sigmaInstance?.refresh();
      if (container) container.style.cursor = 'pointer';
    });

    sigmaInstance.on('leaveEdge', () => {
      hoveredEdge = null;
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

    // A double click widens the view by that node's neighbours, in place of zooming.
    sigmaInstance.on('doubleClickNode', (event) => {
      event.preventSigmaDefault();
      const attrs = graph!.getNodeAttributes(event.node);
      const nd = attrs.nodeData as GraphNode | undefined;
      if (nd) expandAround(nd.id);
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

    startLayout(anchor ? 2000 : 4000);
  }

  function startLayout(ms: number) {
    if (!graph || graph.order === 0) return;
    // Settings fitted to the graph's size, then opened up: nodes keep clear of each other
    // (adjustSizes), and weak gravity lets a cluster spread instead of closing into a blob.
    layout = new FA2Layout(graph, {
      settings: {
        ...forceAtlas2.inferSettings(graph),
        gravity: 0.4,
        scalingRatio: 14,
        adjustSizes: true,
        barnesHutOptimize: graph.order > 400,
        slowDown: 4,
      },
    });
    layout.start();

    layoutTimer = setTimeout(() => {
      stopLayout();
    }, ms);
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
    // With no room yet, the observer below draws it when there is.
    render();
  });

  onMount(() => {
    resizeObserver = new ResizeObserver(() => {
      if (!sigmaInstance) render();
      else sigmaInstance.refresh();
    });
    resizeObserver.observe(container);
    render();
  });

  onDestroy(() => {
    resizeObserver?.disconnect();
    resizeObserver = null;
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
