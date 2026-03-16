import type { GraphInfo, GraphSchema, QueryResult, Suggestion } from './types';

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

export async function fetchGraphData(name: string): Promise<QueryResult> {
  return request<QueryResult>(`/graphs/${encodeURIComponent(name)}/data`);
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
