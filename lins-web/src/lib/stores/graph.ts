import { writable } from 'svelte/store';
import type { GraphNode, GraphEdge, QueryResult, GraphSchema, GraphInfo } from '$lib/types';

export const currentGraph = writable<string | null>(null);
export const graphData = writable<QueryResult | null>(null);
export const schema = writable<GraphSchema | null>(null);
export const selectedNode = writable<GraphNode | null>(null);
export const selectedEdge = writable<GraphEdge | null>(null);
export const availableGraphs = writable<GraphInfo[]>([]);
export const isLoading = writable<boolean>(false);
export const error = writable<string | null>(null);
