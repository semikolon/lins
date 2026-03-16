export interface GraphNode {
  id: number;
  labels: string[];
  properties: Record<string, PropertyValue>;
}

export interface GraphEdge {
  id: number;
  relationship_type: string;
  source_id: number;
  target_id: number;
  properties: Record<string, PropertyValue>;
}

export type PropertyValue =
  | string
  | number
  | boolean
  | null
  | PropertyValue[]
  | { [key: string]: PropertyValue };

export interface QueryResult {
  nodes: GraphNode[];
  edges: GraphEdge[];
  query_time_ms: number;
  source_graph: string;
}

export interface GraphSchema {
  graph_name: string;
  labels: string[];
  relationship_types: string[];
  property_keys: string[];
  node_count: number;
  edge_count: number;
}

export interface GraphInfo {
  name: string;
  node_count: number;
  edge_count: number;
}

export interface Suggestion {
  suggestion_type: 'label' | 'relationship_type' | 'property_value' | 'property_key';
  value: string;
  cypher: string;
  count?: number;
}
