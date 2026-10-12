import { get } from 'svelte/store';
import { fetchAround, fetchBoard, fetchGraphData, fetchSchema, fetchView } from './api';
import {
  anchorNode,
  board,
  currentGraph,
  error,
  graphData,
  isLoading,
  laneCounts,
  mode,
  sceneData,
  schema,
  selectedEdge,
  selectedNode,
  showAll,
  view,
  viewInfo,
} from './stores/graph';
import type { Arrangement, GraphView, QueryResult, ViewResult } from './types';

/** How many more cards one press on a lane's stub brings in. */
export const LANE_STEP = 8;

function info(result: ViewResult) {
  return {
    mode: result.mode,
    total: result.total_node_count,
    hidden: result.hidden_node_count,
    truncated: result.truncated,
  };
}

/** The view's arrangements: the ways it lays the graph out as cards in lanes. */
export function arrangementsOf(v: GraphView | null): Arrangement[] {
  return v?.display.arrangements ?? [];
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

async function lanesNow(name: string, index: number) {
  const b = await fetchBoard(name, index, get(showAll), get(laneCounts)[index] ?? {});
  board.set(b);
  mode.set({ kind: 'lanes', index });
}

async function freeNow(name: string) {
  const d = await fetchGraphData(name, get(showAll));
  anchorNode.set(null);
  viewInfo.set(info(d));
  graphData.set(d);
  mode.set({ kind: 'free' });
}

async function sceneNow(name: string, id: number) {
  const d = await fetchAround(name, id, get(showAll));
  anchorNode.set(null);
  viewInfo.set(info(d));
  sceneData.set(d);
  // The free picture and the panel's list of links read the same nodes, so that changing to
  // the free picture from here shows this neighbourhood and nothing has to be asked for twice.
  graphData.set(d);
  selectedEdge.set(null);
  selectedNode.set(d.nodes.find((n) => n.id === id) ?? null);
  mode.set({ kind: 'scene' });
}

/**
 * Open a graph on what a person should see first: the first of its arrangements when its view
 * has any, else the free picture. `first` asks for another opening: an arrangement by its place,
 * the free picture, or one node and its ties.
 */
export async function loadGraph(name: string, first?: { order?: number | 'free'; focus?: number }) {
  currentGraph.set(name);
  selectedNode.set(null);
  selectedEdge.set(null);
  anchorNode.set(null);
  laneCounts.set({});
  await run(async () => {
    const [s, v] = await Promise.all([fetchSchema(name), fetchView(name)]);
    schema.set(s);
    view.set(v);
    board.set(null);
    sceneData.set(null);
    const orders = arrangementsOf(v);
    if (orders.length === 0 || first?.order === 'free') {
      await freeNow(name);
    } else {
      // Nothing of the graph before is left for the free picture to show by mistake.
      graphData.set(null);
      const index = typeof first?.order === 'number' && first.order < orders.length ? first.order : 0;
      await lanesNow(name, index);
    }
    if (first?.focus !== undefined) await sceneNow(name, first.focus);
  }, 'Failed to load graph');
}

/** Show one of the view's arrangements. */
export async function showLanes(index: number) {
  const name = get(currentGraph);
  if (!name) return;
  await run(() => lanesNow(name, index), 'Failed to load the lanes');
}

/** Show more of one lane's cards, or fewer: `count` is how many the lane is to show. */
export async function showInLane(laneKey: string, count: number) {
  const name = get(currentGraph);
  const m = get(mode);
  if (!name || m.kind !== 'lanes') return;
  laneCounts.update((all) => ({ ...all, [m.index]: { ...(all[m.index] ?? {}), [laneKey]: Math.max(0, count) } }));
  await run(() => lanesNow(name, m.index), 'Failed to load the lane');
}

/** Show one node in the middle with what it is tied to in columns around it, and select it. */
export async function showScene(id: number) {
  const name = get(currentGraph);
  if (!name) return;
  await run(() => sceneNow(name, id), 'Failed to open the node');
}

/** Show the free picture: what was last loaded, or the graph's opening view when nothing was. */
export async function showFree() {
  const name = get(currentGraph);
  if (!name) return;
  if (get(graphData)) {
    mode.set({ kind: 'free' });
    return;
  }
  await run(() => freeNow(name), 'Failed to load graph');
}

/** Open a node the way the page is showing things: its ties in columns, or its neighbourhood in the free picture. */
export async function openNode(id: number) {
  if (get(mode).kind === 'free') await openAround(id);
  else await showScene(id);
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
    mode.set({ kind: 'free' });
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

/** Show or stop showing the kinds the view hides, and load again what is on show. */
export async function setShowAll(all: boolean) {
  showAll.set(all);
  const name = get(currentGraph);
  if (!name) return;
  const m = get(mode);
  const centre = get(sceneData)?.center;
  await run(async () => {
    if (m.kind === 'lanes') await lanesNow(name, m.index);
    else if (m.kind === 'scene' && centre !== undefined) await sceneNow(name, centre);
    else await freeNow(name);
  }, 'Failed to load graph');
}
