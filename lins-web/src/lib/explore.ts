import { get } from 'svelte/store';
import { fetchAround, fetchGraphData, fetchSchema, fetchView } from './api';
import {
  anchorNode,
  currentGraph,
  error,
  graphData,
  isLoading,
  schema,
  selectedEdge,
  selectedNode,
  showAll,
  view,
  viewInfo,
} from './stores/graph';
import type { QueryResult, ViewResult } from './types';

function info(result: ViewResult) {
  return {
    mode: result.mode,
    total: result.total_node_count,
    hidden: result.hidden_node_count,
    truncated: result.truncated,
  };
}

/** Add a result to what the canvas holds, without repeating a node or a link. */
export function merged(existing: QueryResult | null, added: QueryResult): QueryResult {
  if (!existing) return added;
  const nodeIds = new Set(existing.nodes.map((n) => n.id));
  const edgeIds = new Set(existing.edges.map((e) => e.id));
  return {
    nodes: [...existing.nodes, ...added.nodes.filter((n) => !nodeIds.has(n.id))],
    edges: [...existing.edges, ...added.edges.filter((e) => !edgeIds.has(e.id))],
    query_time_ms: added.query_time_ms,
    source_graph: added.source_graph,
  };
}

async function run(work: () => Promise<void>, failure: string) {
  isLoading.set(true);
  error.set(null);
  try {
    await work();
  } catch (err) {
    error.set(err instanceof Error ? err.message : failure);
  } finally {
    isLoading.set(false);
  }
}

/** Open a graph on what a person should see first. */
export async function loadGraph(name: string) {
  currentGraph.set(name);
  selectedNode.set(null);
  selectedEdge.set(null);
  anchorNode.set(null);
  await run(async () => {
    const [s, v, d] = await Promise.all([fetchSchema(name), fetchView(name), fetchGraphData(name, get(showAll))]);
    schema.set(s);
    view.set(v);
    viewInfo.set(info(d));
    graphData.set(d);
  }, 'Failed to load graph');
}

/** Replace the canvas with one node and its neighbours, and select that node. */
export async function openAround(id: number) {
  const name = get(currentGraph);
  if (!name) return;
  await run(async () => {
    const d = await fetchAround(name, id, get(showAll));
    anchorNode.set(null);
    viewInfo.set(info(d));
    graphData.set(d);
    selectedEdge.set(null);
    selectedNode.set(d.nodes.find((n) => n.id === id) ?? null);
  }, 'Failed to open the node');
}

/** Widen the canvas by one node's neighbours, keeping what is there. */
export async function expandAround(id: number) {
  const name = get(currentGraph);
  if (!name) return;
  await run(async () => {
    const d = await fetchAround(name, id, get(showAll));
    anchorNode.set(id);
    const current = get(viewInfo);
    viewInfo.set({
      mode: current?.mode ?? 'around',
      total: d.total_node_count,
      hidden: current?.hidden ?? 0,
      truncated: true,
    });
    graphData.update((existing) => merged(existing, d));
  }, 'Failed to widen the view');
}

/** Show or stop showing the kinds the view hides, and load the graph again. */
export async function setShowAll(all: boolean) {
  showAll.set(all);
  const name = get(currentGraph);
  if (name) await loadGraph(name);
}
