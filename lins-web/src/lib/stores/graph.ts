import { writable } from 'svelte/store';
import type { Board, GraphNode, GraphEdge, QueryResult, GraphSchema, GraphInfo, GraphView, Mode, ViewInfo, ViewResult } from '$lib/types';

export const currentGraph = writable<string | null>(null);
export const graphData = writable<QueryResult | null>(null);
export const schema = writable<GraphSchema | null>(null);
export const selectedNode = writable<GraphNode | null>(null);
export const selectedEdge = writable<GraphEdge | null>(null);
export const availableGraphs = writable<GraphInfo[]>([]);
export const isLoading = writable<boolean>(false);
export const error = writable<string | null>(null);

/** How the current graph is shown: names, colours, what is left out. */
export const view = writable<GraphView | null>(null);
/** True when the kinds the view hides have been asked for. */
export const showAll = writable<boolean>(false);
/** What the canvas holds against what the graph holds. */
export const viewInfo = writable<ViewInfo | null>(null);
/** The node new nodes are placed around when a view is widened. */
export const anchorNode = writable<number | null>(null);

/** What the middle of the page shows. A graph whose view has no arrangements stays on the free picture. */
export const mode = writable<Mode>({ kind: 'free' });
/** The arrangement on show: its lanes and their cards. */
export const board = writable<Board | null>(null);
/** The node last opened with its ties, for the view around one thing. */
export const sceneData = writable<ViewResult | null>(null);
/** How many cards a lane shows where a stub was opened: by arrangement, then by the lane's key. */
export const laneCounts = writable<Record<number, Record<string, number>>>({});
