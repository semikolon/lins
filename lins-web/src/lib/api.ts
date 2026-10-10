import type { GraphInfo, GraphSchema, GraphView, QueryResult, Suggestion, ViewResult } from './types';

const API_BASE = '/api';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  });
  if (!res.ok) {
    const body = await res.text().catch(() => '');
    throw new Error(`API error ${res.status}: ${body || res.statusText}`);
  }
  return res.json() as Promise<T>;
}

export async function fetchGraphs(): Promise<GraphInfo[]> {
  return request<GraphInfo[]>('/graphs');
}

export async function fetchSchema(name: string): Promise<GraphSchema> {
  return request<GraphSchema>(`/graphs/${encodeURIComponent(name)}/schema`);
}

export async function fetchDefaultGraph(): Promise<string | null> {
  const config = await request<{ default_graph: string | null }>('/config');
  return config.default_graph;
}

export async function fetchView(name: string): Promise<GraphView> {
  return request<GraphView>(`/graphs/${encodeURIComponent(name)}/view`);
}

/** What a person should see first. `all` brings in the kinds the view hides. */
export async function fetchGraphData(name: string, all = false): Promise<ViewResult> {
  return request<ViewResult>(`/graphs/${encodeURIComponent(name)}/data${all ? '?all=true' : ''}`);
}

/** One node, its neighbours, and the links among them. */
export async function fetchAround(name: string, id: number, all = false): Promise<ViewResult> {
  return request<ViewResult>(
    `/graphs/${encodeURIComponent(name)}/around/${id}${all ? '?all=true' : ''}`,
  );
}

/** Nodes whose name holds the words, or whose id is exactly them. */
export async function findNodes(name: string, words: string, all = false): Promise<QueryResult> {
  const params = new URLSearchParams({ q: words });
  if (all) params.set('all', 'true');
  return request<QueryResult>(`/graphs/${encodeURIComponent(name)}/find?${params}`);
}

export async function executeQuery(name: string, cypher: string): Promise<QueryResult> {
  return request<QueryResult>(`/graphs/${encodeURIComponent(name)}/query`, {
    method: 'POST',
    body: JSON.stringify({ cypher }),
  });
}

export async function searchVocabulary(
  query: string,
  graph?: string,
): Promise<Suggestion[]> {
  return request<Suggestion[]>('/search/vocabulary', {
    method: 'POST',
    body: JSON.stringify({ query, graph }),
  });
}
